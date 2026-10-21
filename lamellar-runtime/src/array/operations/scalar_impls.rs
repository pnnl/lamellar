use crate::array::*;
use crate::OneSidedMemoryRegion;
use crate::Remote;

/// Holds a scalar batch-op payload (vals/indices/idx_val) either inline (small
/// payloads, avoids the RDMA-registered pool + extra GET round trip) or via the
/// existing `OneSidedMemoryRegion` RDMA path (large payloads, where the round
/// trip is amortized). See `SCALAR_INLINE_THRESHOLD` in `array/unsafe/operations.rs`.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) enum ScalarValBuf {
    Rdma(OneSidedMemoryRegion<u8>),
    Inline(Vec<u8>),
}

impl ScalarValBuf {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Rdma(r) => r.len(),
            Self::Inline(v) => v.len(),
        }
    }

    pub(crate) async unsafe fn get_bytes(&self) -> Vec<u8> {
        match self {
            Self::Rdma(r) => {
                let n = r.len();
                unsafe { r.get_buffer(0, n).await }
            }
            Self::Inline(v) => v.clone(),
        }
    }

    /// Fetches `num_elems` elements of `T`. For the `Rdma` variant this mirrors
    /// the existing `to_base::<T>().get_buffer(...)` pattern. For `Inline`,
    /// `read_unaligned` is required per-element since a `Vec<u8>`'s allocation
    /// is only guaranteed byte-aligned.
    pub(crate) async unsafe fn get_typed<T: Remote>(&self, num_elems: usize) -> Vec<T> {
        match self {
            Self::Rdma(r) => unsafe { r.clone().to_base::<T>().get_buffer(0, num_elems).await },
            Self::Inline(v) => v
                .chunks_exact(std::mem::size_of::<T>())
                .map(|c| unsafe { std::ptr::read_unaligned(c.as_ptr() as *const T) })
                .collect(),
        }
    }
}

// Op groups used by `impl_ops_scalar_type_match!` / `impl_ops_option_scalar_type_match!`.
//
// `macro_rules!` cannot expand to individual match arms, so each group expands to a
// complete `match $op { <group arms>, $op => $next }` expression whose fallback arm
// (`$next`) is the next group in the chain. `$op` is re-bound (shadowed) by the
// fallback arm, so a type class's op set is simply a chain of groups ending in the
// `invalid op cmd` panic. The first token selects the group, the second whether the
// generated fn returns the fetched values (`true`) or nothing (`false`).
// The bracketed list is `[op array idx_vals self]`: the identifiers are passed in so
// that they share hygiene with the fn that declares them.
macro_rules! scalar_op_group {
    (arith false, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::Add => {$array.mut_local_data().await.local_add($iv);},
            ArrayOpCmd::Sub => {$array.mut_local_data().await.local_sub($iv);},
            ArrayOpCmd::Mul => {$array.mut_local_data().await.local_mul($iv);},
            ArrayOpCmd::Div => {$array.mut_local_data().await.local_div($iv);},
            ArrayOpCmd::Rem => {$array.mut_local_data().await.local_rem($iv);},
            $op => $next,
        }
    };
    (arith true, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::FetchAdd => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_add($iv,true).unwrap()).await},
            ArrayOpCmd::FetchSub => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_sub($iv,true).unwrap()).await},
            ArrayOpCmd::FetchMul => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_mul($iv,true).unwrap()).await},
            ArrayOpCmd::FetchDiv => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_div($iv,true).unwrap()).await},
            ArrayOpCmd::FetchRem => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_rem($iv,true).unwrap()).await},
            $op => $next,
        }
    };
    (bitwise false, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::And => {$array.mut_local_data().await.local_bit_and($iv);},
            ArrayOpCmd::Or => {$array.mut_local_data().await.local_bit_or($iv);},
            ArrayOpCmd::Xor => {$array.mut_local_data().await.local_bit_xor($iv);},
            $op => $next,
        }
    };
    (bitwise true, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::FetchAnd => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_bit_and($iv,true).unwrap()).await},
            ArrayOpCmd::FetchOr => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_bit_or($iv,true).unwrap()).await},
            ArrayOpCmd::FetchXor => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_bit_xor($iv,true).unwrap()).await},
            $op => $next,
        }
    };
    (shift false, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::Shl => {$array.mut_local_data().await.local_shl($iv);},
            ArrayOpCmd::Shr => {$array.mut_local_data().await.local_shr($iv);},
            $op => $next,
        }
    };
    (shift true, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::FetchShl => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_shl($iv,true).unwrap()).await},
            ArrayOpCmd::FetchShr => {$this.vec_to_bytes($array.mut_local_data().await.local_fetch_shr($iv,true).unwrap()).await},
            $op => $next,
        }
    };
    // loads only exist for the fetching variants
    (read false, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => { $next };
    (read true, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::Load => {$this.vec_to_bytes($array.local_data().await.local_load($iv)).await},
            ArrayOpCmd::Get => {$this.vec_to_bytes($array.local_data().await.local_load($iv)).await},
            $op => $next,
        }
    };
    (write false, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::Store => {$array.mut_local_data().await.local_store($iv);},
            ArrayOpCmd::Put => {$array.mut_local_data().await.local_store($iv);},
            $op => $next,
        }
    };
    (write true, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::Swap => {$this.vec_to_bytes($array.mut_local_data().await.local_swap($iv)).await},
            $op => $next,
        }
    };
    (cmpxchg false, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => { $next };
    (cmpxchg true, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::CompareExchange(old) => {$this.vec_to_bytes($array.mut_local_data().await.local_compare_exchange($iv, old)).await},
            $op => $next,
        }
    };
    (cmpxchg_eps false, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => { $next };
    (cmpxchg_eps true, [$op:ident $array:ident $iv:ident $this:ident], $next:expr) => {
        match $op {
            ArrayOpCmd::CompareExchangeEps(old, eps) => {$this.vec_to_bytes($array.mut_local_data().await.local_compare_exchange_epsilon($iv, old, eps)).await},
            $op => $next,
        }
    };
}

/// The op set for each class of scalar type, as a chain of `scalar_op_group!`s.
macro_rules! scalar_op_match {
    (ints $r:tt $ctx:tt) => {
        scalar_op_group!(arith $r, $ctx,
        scalar_op_group!(bitwise $r, $ctx,
        scalar_op_group!(shift $r, $ctx,
        scalar_op_group!(read $r, $ctx,
        scalar_op_group!(write $r, $ctx,
        scalar_op_group!(cmpxchg $r, $ctx,
        scalar_op_group!(cmpxchg_eps $r, $ctx,
        panic!("invalid op cmd"))))))))
    };
    (floats $r:tt $ctx:tt) => {
        scalar_op_group!(arith $r, $ctx,
        scalar_op_group!(read $r, $ctx,
        scalar_op_group!(write $r, $ctx,
        scalar_op_group!(cmpxchg_eps $r, $ctx,
        panic!("invalid op cmd")))))
    };
    (bools $r:tt $ctx:tt) => {
        scalar_op_group!(bitwise $r, $ctx,
        scalar_op_group!(read $r, $ctx,
        scalar_op_group!(write $r, $ctx,
        scalar_op_group!(cmpxchg $r, $ctx,
        panic!("invalid op cmd")))))
    };
    // `Option<T>` elements only support stores/swaps and (for non-floats) compare-exchange
    (option_ints $r:tt $ctx:tt) => {
        scalar_op_group!(write $r, $ctx,
        scalar_op_group!(cmpxchg $r, $ctx,
        panic!("invalid op cmd")))
    };
    (option_floats $r:tt $ctx:tt) => {
        scalar_op_group!(write $r, $ctx,
        panic!("invalid op cmd"))
    };
}

// `false` means the generated fn returns nothing, `true` that it returns a
// `OneSidedMemoryRegion<u8>`.
macro_rules! scalar_op_ret {
    (false) => { () };
    (true) => { OneSidedMemoryRegion<u8> };
}

/// Generates `async fn bare_type(&self, bytes, array)`, dispatching on
/// `self.scalar_type`. `$bytes` is the name of the bytes parameter and `$bytes_expr`
/// the expression passed to `self.idx_vals` (it must refer to that parameter).
macro_rules! impl_ops_scalar_type_match {
    ($bytes:ident : $bytes_type:ty, $bytes_expr:expr, $result:tt) => {
        impl_ops_scalar_type_match! {@gen $bytes, $bytes_type, $bytes_expr, $result,
            ints: [u8, u16, u32, u64, usize, u128, i8, i16, i32, i64, isize, i128],
            floats: [f32, f64],
            bools: [bool]}
    };
    (@gen $bytes:ident, $bytes_type:ty, $bytes_expr:expr, $result:tt,
     ints: [$($int:ident),*], floats: [$($float:ident),*], bools: [$($bool:ident),*]) => {
        #[allow(unused_variables, unused_mut, unreachable_patterns)] // fallback arms / `_` arm may be dead depending on the type class
        async fn bare_type(&self, mut $bytes: $bytes_type, mut array: LamellarByteArray) -> scalar_op_ret!($result) {
            match self.scalar_type {
                $( ScalarType::$int => {
                    let idx_vals = self.idx_vals::<$int>($bytes_expr);
                    let op: ArrayOpCmd<$int> = self.op.clone().into();
                    scalar_op_match!(ints $result [op array idx_vals self])
                } )*
                $( ScalarType::$float => {
                    let idx_vals = self.idx_vals::<$float>($bytes_expr);
                    let op: ArrayOpCmd<$float> = self.op.clone().into();
                    scalar_op_match!(floats $result [op array idx_vals self])
                } )*
                $( ScalarType::$bool => {
                    let idx_vals = self.idx_vals::<$bool>($bytes_expr);
                    let op: ArrayOpCmd<$bool> = self.op.clone().into();
                    scalar_op_match!(bools $result [op array idx_vals self])
                } )*
                _ => panic!("invalid type for bare array"),
            }
        }
    };
}

/// Same as `impl_ops_scalar_type_match!` but for arrays of `Option<T>`; generates
/// `async fn option_type(&self, bytes, array)`.
macro_rules! impl_ops_option_scalar_type_match {
    ($bytes:ident : $bytes_type:ty, $bytes_expr:expr, $result:tt) => {
        impl_ops_option_scalar_type_match! {@gen $bytes, $bytes_type, $bytes_expr, $result,
            ints: [u8, u16, u32, u64, usize, u128, i8, i16, i32, i64, isize, i128, bool],
            floats: [f32, f64]}
    };
    (@gen $bytes:ident, $bytes_type:ty, $bytes_expr:expr, $result:tt,
     ints: [$($int:ident),*], floats: [$($float:ident),*]) => {
        #[allow(unused_variables, unused_mut, unreachable_patterns)]
        async fn option_type(&self, mut $bytes: $bytes_type, mut array: LamellarByteArray) -> scalar_op_ret!($result) {
            match self.scalar_type {
                $( ScalarType::$int => {
                    let idx_vals = self.idx_vals::<Option<$int>>($bytes_expr);
                    let op: ArrayOpCmd<Option<$int>> = self.op.clone().into();
                    scalar_op_match!(option_ints $result [op array idx_vals self])
                } )*
                $( ScalarType::$float => {
                    let idx_vals = self.idx_vals::<Option<$float>>($bytes_expr);
                    let op: ArrayOpCmd<Option<$float>> = self.op.clone().into();
                    scalar_op_match!(option_floats $result [op array idx_vals self])
                } )*
                _ => panic!("invalid type for option array"),
            }
        }
    };
}

macro_rules! impl_vec_to_bytes {
    () => {
        async fn vec_to_bytes<T: Remote>(&self, result: Vec<T>) -> OneSidedMemoryRegion<u8> {
            // fully written by local_copy_from_slice below
            let mut mem_region = unsafe {
                self.array
                    .team()
                    .try_alloc_one_sided_mem_region_uninit::<T>(result.len())
            };
            while let None = mem_region {
                async_std::task::yield_now().await;
                mem_region = unsafe {
                    self.array
                        .team()
                        .try_alloc_one_sided_mem_region_uninit::<T>(result.len())
                };
            }
            let mem_region = mem_region.unwrap();
            unsafe {
                mem_region.local_copy_from_slice(&result);
                mem_region.to_base::<u8>()
            }
        }
    };
}

#[lamellar_impl::AmDataRT(Clone)]
pub(crate) struct PackedIndicies {
    pub(crate) data: Vec<u8>,
    byte_width: usize,
    estimated_elems: usize,
}

impl PackedIndicies {
    // pub(crate) fn new(byte_width: usize) -> PackedIndicies {
    //     PackedIndicies { data: Vec::new(), byte_width , estimated_elems: 0}
    // }
    pub(crate) fn new_with_capacity(byte_width: usize, estimated_elems: usize) -> PackedIndicies {
        let data = Vec::with_capacity(byte_width * estimated_elems);
        PackedIndicies {
            data,
            byte_width,
            estimated_elems,
        }
    }
    pub(crate) fn push<T>(&mut self, value: T) {
        let byte_width = self.byte_width;
        let value_bytes =
            unsafe { std::slice::from_raw_parts((&value as *const T) as *const u8, byte_width) };
        self.data.extend_from_slice(value_bytes);
    }
    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }
    pub(crate) fn clear(&mut self) {
        self.data.clear();
    }
    // pub(crate) fn slice_as_iter<T>(byte_width: usize, data: &[u8]) -> impl Iterator<Item=T> + '_ {
    //     data.chunks_exact(byte_width).map(move |chunk| {
    //         let mut data = vec![0; std::mem::size_of::<T>()];
    //         data[..chunk.len()].copy_from_slice(chunk);
    //         unsafe { std::ptr::read_unaligned(data.as_ptr() as *const T) }
    //     })
    // }
}

#[lamellar_impl::AmDataRT(Clone)]
pub(crate) struct PackedIdxVal {
    data: Vec<u8>,
    idx_byte_width: usize,
}

impl PackedIdxVal {
    // pub(crate) fn new(idx_byte_width: usize) -> PackedIdxVal {
    //     PackedIdxVal { data: Vec::new(), idx_byte_width }
    // }
    pub(crate) fn new_with_capacity<T>(
        idx_byte_width: usize,
        estimated_elems: usize,
    ) -> PackedIdxVal {
        let data = Vec::with_capacity(
            idx_byte_width * estimated_elems + std::mem::size_of::<T>() * estimated_elems,
        );
        PackedIdxVal {
            data,
            idx_byte_width,
        }
    }
    pub(crate) fn push<T>(&mut self, idx: usize, value: T) {
        let idx_byte_width = self.idx_byte_width;
        let idx_bytes = unsafe {
            std::slice::from_raw_parts((&idx as *const usize) as *const u8, idx_byte_width)
        };
        self.data.extend_from_slice(idx_bytes);
        let value_bytes = unsafe {
            std::slice::from_raw_parts((&value as *const T) as *const u8, std::mem::size_of::<T>())
        };
        self.data.extend_from_slice(value_bytes);
    }
    pub(crate) fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }
    pub(crate) fn clear(&mut self) {
        self.data.clear();
    }
    pub(crate) fn slice_as_iter<T>(
        idx_byte_width: usize,
        data: &[u8],
    ) -> impl Iterator<Item = (usize, T)> + '_ {
        data.chunks_exact(idx_byte_width + std::mem::size_of::<T>())
            .map(move |chunk| {
                let mut idx_data = vec![0; std::mem::size_of::<usize>()];
                let mut val_data = vec![0; std::mem::size_of::<T>()];
                idx_data[..idx_byte_width].copy_from_slice(&chunk[..idx_byte_width]);
                val_data.copy_from_slice(&chunk[idx_byte_width..]);
                let idx = unsafe { std::ptr::read_unaligned(idx_data.as_ptr() as *const usize) };
                let val = unsafe { std::ptr::read_unaligned(val_data.as_ptr() as *const T) };
                (idx, val)
            })
    }
}

#[lamellar_impl::AmDataRT]
pub(crate) struct ScalarSingleIdxMultiValAm {
    pub(crate) array: LamellarByteArray,
    pub(crate) index: usize,
    pub(crate) vals: ScalarValBuf,
    pub(crate) scalar_type: ScalarType,
    pub(crate) opt: bool,
    pub(crate) op: ArrayOpCmd<Vec<u8>>,
}

impl ScalarSingleIdxMultiValAm {
    fn idx_vals<'a, T: Dist + Sized>(
        &self,
        bytes: &'a [u8],
    ) -> impl Iterator<Item = (usize, T)> + 'a {
        let vals = bytes
            .chunks_exact(std::mem::size_of::<T>())
            .map(|chunk| unsafe { std::ptr::read_unaligned(chunk.as_ptr() as *const T) });
        std::iter::repeat(self.index).zip(vals)
    }
    impl_ops_option_scalar_type_match!(bytes: Vec<u8>, &bytes, false);
    impl_ops_scalar_type_match!(bytes: Vec<u8>, &bytes, false);
}

#[lamellar_impl::rt_am]
impl LamellarAM for ScalarSingleIdxMultiValAm {
    async fn exec(&self) {
        let bytes = unsafe { self.vals.get_bytes().await };
        let array = self.array.clone();

        if self.opt {
            self.option_type(bytes, array).await;
        } else {
            self.bare_type(bytes, array).await;
        }
    }
}

#[lamellar_impl::AmDataRT]
pub(crate) struct ScalarSingleIdxMultiValAmReturn {
    pub(crate) array: LamellarByteArray,
    pub(crate) index: usize,
    pub(crate) vals: ScalarValBuf,
    pub(crate) scalar_type: ScalarType,
    pub(crate) opt: bool,
    pub(crate) op: ArrayOpCmd<Vec<u8>>,
}

impl ScalarSingleIdxMultiValAmReturn {
    fn idx_vals<'a, T: Dist + Sized>(
        &self,
        bytes: &'a [u8],
    ) -> impl Iterator<Item = (usize, T)> + 'a {
        let vals = bytes
            .chunks_exact(std::mem::size_of::<T>())
            .map(|chunk| unsafe { std::ptr::read_unaligned(chunk.as_ptr() as *const T) });
        std::iter::repeat(self.index).zip(vals)
    }
    impl_vec_to_bytes!();
    impl_ops_option_scalar_type_match!(bytes: Vec<u8>, &bytes, true);
    impl_ops_scalar_type_match!(bytes: Vec<u8>, &bytes, true);
}

#[lamellar_impl::rt_am]
impl LamellarAM for ScalarSingleIdxMultiValAmReturn {
    async fn exec(&self) -> OneSidedMemoryRegion<u8> {
        let bytes = unsafe { self.vals.get_bytes().await };
        let array = self.array.clone();

        if self.opt {
            self.option_type(bytes, array).await
        } else {
            self.bare_type(bytes, array).await
        }
    }
}

#[lamellar_impl::AmDataRT]
pub(crate) struct ScalarMultiIdxSingleValAm {
    pub(crate) array: LamellarByteArray,
    pub(crate) val: Vec<u8>,
    pub(crate) indices: ScalarValBuf,
    pub(crate) idx_size: usize,
    pub(crate) scalar_type: ScalarType,
    pub(crate) opt: bool,
    pub(crate) op: ArrayOpCmd<Vec<u8>>,
}

impl ScalarMultiIdxSingleValAm {
    fn idx_vals<'a, T: Dist + Sized>(
        &self,
        indices: impl Iterator<Item = usize> + 'a,
    ) -> impl Iterator<Item = (usize, T)> + 'a {
        let val = unsafe { std::ptr::read_unaligned(self.val.as_ptr() as *const T) };
        indices.zip(std::iter::repeat(val))
    }
    impl_ops_option_scalar_type_match!(bytes: impl Iterator<Item = usize>, bytes, false);
    impl_ops_scalar_type_match!(bytes: impl Iterator<Item = usize>, bytes, false);
}

#[lamellar_impl::rt_am]
impl LamellarAM for ScalarMultiIdxSingleValAm {
    async fn exec(&self) {
        let num_bytes = self.indices.len();
        let array = self.array.clone();

        let indices: Box<dyn Iterator<Item = usize> + Send + '_> = match self.idx_size {
            1 => Box::new(
                unsafe { self.indices.get_typed::<u8>(num_bytes).await }
                    .into_iter()
                    .map(|b| b as usize),
            ),
            2 => Box::new(
                unsafe { self.indices.get_typed::<u16>(num_bytes / 2).await }
                    .into_iter()
                    .map(|b| b as usize),
            ),
            4 => Box::new(
                unsafe { self.indices.get_typed::<u32>(num_bytes / 4).await }
                    .into_iter()
                    .map(|b| b as usize),
            ),
            _ => Box::new(
                unsafe { self.indices.get_typed::<u64>(num_bytes / 8).await }
                    .into_iter()
                    .map(|b| b as usize),
            ),
        };
        if self.opt {
            self.option_type(indices, array).await;
        } else {
            self.bare_type(indices, array).await;
        }
    }
}

#[lamellar_impl::AmDataRT]
pub(crate) struct ScalarMultiIdxSingleValAmReturn {
    pub(crate) array: LamellarByteArray,
    pub(crate) val: Vec<u8>,
    pub(crate) indices: ScalarValBuf,
    pub(crate) idx_size: usize,
    pub(crate) scalar_type: ScalarType,
    pub(crate) opt: bool,
    pub(crate) op: ArrayOpCmd<Vec<u8>>,
}

impl ScalarMultiIdxSingleValAmReturn {
    fn idx_vals<'a, T: Dist + Sized>(
        &self,
        indices: impl Iterator<Item = usize> + 'a,
    ) -> impl Iterator<Item = (usize, T)> + 'a {
        let val = unsafe { std::ptr::read_unaligned(self.val.as_ptr() as *const T) };
        indices.zip(std::iter::repeat(val))
    }
    impl_vec_to_bytes!();
    impl_ops_option_scalar_type_match!(bytes: impl Iterator<Item = usize>, bytes, true);
    impl_ops_scalar_type_match!(bytes: impl Iterator<Item = usize>, bytes, true);
}

#[lamellar_impl::rt_am]
impl LamellarAM for ScalarMultiIdxSingleValAmReturn {
    async fn exec(&self) -> OneSidedMemoryRegion<u8> {
        let num_bytes = self.indices.len();
        let array = self.array.clone();

        let indices: Box<dyn Iterator<Item = usize> + Send + '_> = match self.idx_size {
            1 => Box::new(
                unsafe { self.indices.get_typed::<u8>(num_bytes).await }
                    .into_iter()
                    .map(|b| b as usize),
            ),
            2 => Box::new(
                unsafe { self.indices.get_typed::<u16>(num_bytes / 2).await }
                    .into_iter()
                    .map(|b| b as usize),
            ),
            4 => Box::new(
                unsafe { self.indices.get_typed::<u32>(num_bytes / 4).await }
                    .into_iter()
                    .map(|b| b as usize),
            ),
            _ => Box::new(
                unsafe { self.indices.get_typed::<u64>(num_bytes / 8).await }
                    .into_iter()
                    .map(|b| b as usize),
            ),
        };
        if self.opt {
            self.option_type(indices, array).await
        } else {
            self.bare_type(indices, array).await
        }
    }
}

#[lamellar_impl::AmDataRT]
pub(crate) struct ScalarMultiIdxMultiValAm {
    pub(crate) array: LamellarByteArray,
    pub(crate) idx_val: ScalarValBuf,
    pub(crate) idx_size: usize,
    pub(crate) scalar_type: ScalarType,
    pub(crate) opt: bool,
    pub(crate) op: ArrayOpCmd<Vec<u8>>,
}

impl ScalarMultiIdxMultiValAm {
    fn idx_vals<'a, T: Dist + Sized>(
        &self,
        bytes: &'a [u8],
    ) -> impl Iterator<Item = (usize, T)> + 'a {
        PackedIdxVal::slice_as_iter::<T>(self.idx_size, bytes)
    }

    impl_ops_option_scalar_type_match!(bytes: Vec<u8>, &bytes, false);
    impl_ops_scalar_type_match!(bytes: Vec<u8>, &bytes, false);
}

#[lamellar_impl::rt_am]
impl LamellarAM for ScalarMultiIdxMultiValAm {
    async fn exec(&self) {
        let bytes = unsafe { self.idx_val.get_bytes().await };
        let array = self.array.clone();

        if self.opt {
            self.option_type(bytes, array).await;
        } else {
            self.bare_type(bytes, array).await;
        }
    }
}

#[lamellar_impl::AmDataRT]
pub(crate) struct ScalarMultiIdxMultiValAmReturn {
    pub(crate) array: LamellarByteArray,
    pub(crate) idx_val: ScalarValBuf,
    pub(crate) idx_size: usize,
    pub(crate) scalar_type: ScalarType,
    pub(crate) opt: bool,
    pub(crate) op: ArrayOpCmd<Vec<u8>>,
}

impl ScalarMultiIdxMultiValAmReturn {
    fn idx_vals<'a, T: Dist + Sized>(
        &self,
        bytes: &'a [u8],
    ) -> impl Iterator<Item = (usize, T)> + 'a {
        PackedIdxVal::slice_as_iter::<T>(self.idx_size, bytes)
    }

    impl_vec_to_bytes!();
    impl_ops_option_scalar_type_match!(bytes: Vec<u8>, &bytes, true);
    impl_ops_scalar_type_match!(bytes: Vec<u8>, &bytes, true);
}

#[lamellar_impl::rt_am]
impl LamellarAM for ScalarMultiIdxMultiValAmReturn {
    async fn exec(&self) -> OneSidedMemoryRegion<u8> {
        let bytes = unsafe { self.idx_val.get_bytes().await };
        let array = self.array.clone();

        if self.opt {
            self.option_type(bytes, array).await
        } else {
            self.bare_type(bytes, array).await
        }
    }
}

#[lamellar_impl::AmDataRT]
pub(crate) struct UsizeSingleValMultiIdxAm {
    pub(crate) array: UnsafeArray<usize>,
    pub(crate) indices: OneSidedMemoryRegion<u8>,
    pub(crate) idx_size: usize,
    pub(crate) val: usize,
}

#[lamellar_impl::rt_am]
impl LamellarAM for UsizeSingleValMultiIdxAm {
    async fn exec(&self) {
        let num_bytes = self.indices.len();
        let indices = unsafe {
            self.indices
                .clone()
                .to_base::<u32>()
                .get_buffer(0, num_bytes / 4)
                .await
        };
        // let bytes = unsafe { self.indices.get_buffer(0, num_bytes).await };
        // // let indices = PackedIndicies::slice_as_iter::<usize>(self.idx_size, bytes);
        // let indices= unsafe {
        //     std::slice::from_raw_parts(bytes.as_ptr() as *const u32, bytes.len() / 4)
        // };
        let local_data = unsafe { self.array.mut_local_data() };
        indices.iter().for_each(|i| {
            local_data[*i as usize] += self.val;
        });
    }
}
