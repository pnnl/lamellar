//! A symmetric remote target: a byte-addressed [`SharedMemoryRegion`] (the heap, or the registered data segment) plus
//! the `u32` / `u64` views of it that the runtime's atomics operate on. Everything is routed through the runtime's
//! public region API; the local side of a transfer is any C pointer.

use lamellar::memregion::prelude::*;

/// The atomic operations of the OpenSHMEM AMO API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AtomicKind {
    Fetch,
    Set,
    Swap,
    CompareSwap,
    Add,
    And,
    Or,
    Xor,
}

/// Reductions the runtime's allreduce supports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReduceKind {
    Sum,
    Prod,
    Min,
    Max,
    And,
    Or,
    Xor,
}

pub(crate) struct Sym {
    bytes: SharedMemoryRegion<u8>,
    w32: SharedMemoryRegion<u32>,
    w64: SharedMemoryRegion<u64>,
}

macro_rules! atomic_op {
    ($fn:ident, $view:ident, $t:ty) => {
        /// Atomic operation on the `$t` at byte `off` of `pe`'s range; returns the previous value (0 for `Set`).
        pub(crate) unsafe fn $fn(&self, pe: usize, off: usize, op: AtomicKind, a: $t, b: $t) -> $t {
            const SZ: usize = std::mem::size_of::<$t>();
            assert!(off % SZ == 0, "unaligned atomic offset {off}");
            let (v, i) = (&self.$view, off / SZ);
            match op {
                // a fetch-add of 0 is a load on every backend (ucx has no native read)
                AtomicKind::Fetch => v.atomic_fetch_add_blocking(pe, i, 0),
                AtomicKind::Set => {
                    v.atomic_store_blocking(pe, i, a);
                    0
                }
                AtomicKind::Swap => v.atomic_swap_blocking(pe, i, a),
                AtomicKind::CompareSwap => match v.atomic_compare_exchange_blocking(pe, i, a, b) {
                    Ok(old) | Err(old) => old,
                },
                AtomicKind::Add => v.atomic_fetch_add_blocking(pe, i, a),
                AtomicKind::And => v.atomic_fetch_bit_and_blocking(pe, i, a),
                AtomicKind::Or => v.atomic_fetch_bit_or_blocking(pe, i, a),
                AtomicKind::Xor => v.atomic_fetch_bit_xor_blocking(pe, i, a),
            }
        }
    };
}

impl Sym {
    /// Wrap `bytes`, whose start must be aligned for 8 bytes (the heap is allocated as `u64`s; registered memory
    /// starts on a page). A tail that is not a whole word is simply not addressable by atomics.
    pub(crate) unsafe fn new(bytes: SharedMemoryRegion<u8>) -> Result<Self, String> {
        let len = bytes.len();
        let w32 = bytes.sub_region(0..len & !3).try_cast::<u32>().map_err(|e| e.to_string())?;
        let w64 = bytes.sub_region(0..len & !7).try_cast::<u64>().map_err(|e| e.to_string())?;
        Ok(Self { bytes, w32, w64 })
    }

    /// Blocking put of `len` bytes from local `src` to byte `off` on `pe`. On return `src` may be reused.
    pub(crate) unsafe fn put(&self, pe: usize, off: usize, src: *const u8, len: usize) {
        self.bytes.put_ptr_blocking(pe, off, src, len);
    }

    /// Non-blocking put; `src` must stay valid and unmodified until the next `quiet`.
    pub(crate) unsafe fn put_nbi(&self, pe: usize, off: usize, src: *const u8, len: usize) {
        self.bytes.put_ptr_unmanaged(pe, off, src, len);
    }

    /// Blocking get of `len` bytes from byte `off` on `pe` into local `dst`.
    pub(crate) unsafe fn get(&self, pe: usize, off: usize, dst: *mut u8, len: usize) {
        self.bytes.get_into_ptr_blocking(pe, off, dst, len);
    }

    /// Non-blocking get; `dst` must stay valid, and must not be read, until the next `quiet`.
    pub(crate) unsafe fn get_nbi(&self, pe: usize, off: usize, dst: *mut u8, len: usize) {
        self.bytes.get_into_ptr_unmanaged(pe, off, dst, len);
    }

    /// Wait for all outstanding operations issued by this PE.
    pub(crate) fn quiet(&self) {
        self.bytes.wait_all();
    }

    atomic_op!(atomic_u32, w32, u32);
    atomic_op!(atomic_u64, w64, u64);

    /// Collective all-reduce over `len` elements of `T` at byte `off`, across every PE holding the region. `None` if `off`
    /// is not a multiple of `size_of::<T>()` or `len` is 0 (the caller then needs another path). All PEs must call it
    /// with the same arguments. The source elements must lie inside the region and not change during the call.
    pub(crate) unsafe fn allreduce<T: lamellar::Remote>(&self, off: usize, len: usize, op: ReduceKind) -> Option<Vec<T>> {
        let sz = std::mem::size_of::<T>();
        if sz == 0 || off % sz != 0 || len == 0 {
            return None;
        }
        let v = self.bytes.sub_region(off..off + len * sz).try_cast::<T>().ok()?;
        Some(match op {
            ReduceKind::Sum => v.sum_all(0, len).block(),
            ReduceKind::Prod => v.prod_all(0, len).block(),
            ReduceKind::Min => v.min_all(0, len).block(),
            ReduceKind::Max => v.max_all(0, len).block(),
            ReduceKind::And => v.bit_and_all(0, len).block(),
            ReduceKind::Or => v.bit_or_all(0, len).block(),
            ReduceKind::Xor => v.bit_xor_all(0, len).block(),
        })
    }
}
