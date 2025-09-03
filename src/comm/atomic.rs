use super::message::extract_raw_addr_and_ctx;
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::cq::ReadCq;
use crate::enums::AtomicFetchMsgOptions;
use crate::enums::AtomicMsgOptions;
use crate::enums::AtomicOp;
use crate::ep::Connected;
use crate::ep::Connectionless;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::ep::EpState;
use crate::eq::ReadEq;
use crate::fid::AsRawTypedFid;
use crate::fid::AsTypedFid;
use crate::fid::EpRawFid;
use crate::infocapsoptions::AtomicCap;
use crate::infocapsoptions::ReadMod;
use crate::infocapsoptions::WriteMod;
use crate::mr::MappedMemoryRegionKey;
use crate::mr::MemoryRegionDesc;
use crate::mr::MemoryRegionSlice;
use crate::mr::MemoryRegionSliceMut;
use crate::trigger::TriggeredContext;
use crate::utils::check_error;
use crate::utils::Either;
use crate::xcontext::RxContextBase;
use crate::xcontext::RxContextImplBase;
use crate::xcontext::TxContextBase;
use crate::xcontext::TxContextImplBase;
use crate::AsFiType;
use crate::Context;
use crate::RemoteMemAddrSlice;
use crate::RemoteMemAddrSliceMut;
use crate::RemoteMemoryAddress;
use crate::FI_ADDR_UNSPEC;

pub(crate) trait AtomicWriteEpImpl: AsTypedFid<EpRawFid> + AtomicValidEp {
    #[allow(clippy::too_many_arguments)]
    fn atomic_impl<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: Option<*mut std::ffi::c_void>,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_atomic(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn atomicv_impl<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: Option<*mut std::ffi::c_void>,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_atomicv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                ioc.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                ioc.len(),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    fn atomicmsg_impl<T: AsFiType>(
        &self,
        msg: Either<&crate::msg::MsgAtomic<T>, &crate::msg::MsgAtomicConnected<T>>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_atomic_msg = match msg {
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_atomicmsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_atomic_msg,
                options.as_raw(),
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn inject_atomic_impl<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = dest_addr {
            addr.raw_addr()
        } else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_inject_atomic(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
            )
        };
        check_error(err)
    }
}

macro_rules! gen_atomic_op_decl_single {
    ($func:ident (< $( $N:ident $(: $b0:ident $(+$b:ident)* )? ),* >),  ($self: ident, $($p: ident : $t: ty),*)) =>
    {
        unsafe fn $func< $( $N $(: $b0 $(+$b)* )? ),* >
        (
            &$self,
            $($p: $t),*
        ) -> Result<(), crate::error::Error>;
    };

    ($func:ident (),  $($p_and_t: tt)*) => {
        gen_atomic_op_decl_single!($func (<>), $($p_and_t),*);
    };
}

macro_rules! gen_atomic_op_decl {
    ($gen: tt, $args: tt, $($func:ident),+) =>
    {
        $(
            gen_atomic_op_decl_single!($func $gen, $args);
        )+
    }
}

macro_rules! gen_atomic_op_def_single {
    ($func:ident (< $( $N:ident $(: $b0:ident $(+$b:ident)* )? ),* >),  ($self: ident, $($p: ident : $t: ty),*), $inner_func:ident ($($vals: expr),*), $op: path) =>
    {
        #[inline]
        unsafe fn $func< $( $N $(: $b0 $(+$b)* )? ),* >
        (
            &$self,
            $($p: $t),*
        ) -> Result<(), crate::error::Error>
        {
            $self.$inner_func($($vals,)* $op)
        }
    };

    ($func:ident (),  $p_and_t: tt, $inner_func:ident $vals: tt, $op: path) => {
        gen_atomic_op_def_single!($func (<>), $p_and_t, $inner_func $vals, $op);
    };
}

macro_rules! gen_atomic_op_def {
    ($gen: tt, $args: tt, $inner_func: ident $vals: tt, $($op: path,)+, $($func:ident),+) =>
    {
        $(
            gen_atomic_op_def_single!($func $gen, $args, $inner_func $vals, $op);
        )+
    }
}



pub trait AtomicWriteEp {
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomic_min_to, atomic_max_to, atomic_sum_to,  atomic_prod_to, atomic_bor_to, atomic_band_to, atomic_bxor_to
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomic_lor_to, atomic_land_to, atomic_lxor_to
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
    atomic_min_to_with_context, atomic_max_to_with_context, atomic_sum_to_with_context,  atomic_prod_to_with_context, atomic_bor_to_with_context, atomic_band_to_with_context, atomic_bxor_to_with_context
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
    atomic_lor_to_with_context, atomic_land_to_with_context, atomic_lxor_to_with_context
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
    atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered,  atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered 
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
    atomic_lor_to_triggered, atomic_land_to_triggered, atomic_lxor_to_triggered
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomicv_min_to, atomicv_max_to, atomicv_sum_to,  atomicv_prod_to, atomicv_bor_to, atomicv_band_to, atomicv_bxor_to
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomicv_lor_to, atomicv_land_to, atomicv_lxor_to
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
    atomicv_min_to_with_context, atomicv_max_to_with_context, atomicv_sum_to_with_context,  atomicv_prod_to_with_context, atomicv_bor_to_with_context, atomicv_band_to_with_context, atomicv_bxor_to_with_context
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
    atomicv_lor_to_with_context, atomicv_land_to_with_context, atomicv_lxor_to_with_context
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ), 
    atomicv_min_to_triggered, atomicv_max_to_triggered, atomicv_sum_to_triggered,  atomicv_prod_to_triggered, atomicv_bor_to_triggered, atomicv_band_to_triggered, atomicv_bxor_to_triggered
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ), 
    atomicv_lor_to_triggered, atomicv_land_to_triggered, atomicv_lxor_to_triggered
    );


    unsafe fn atomicmsg_to<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error>;


    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomic_inject_min_to, atomic_inject_max_to, atomic_inject_sum_to,  atomic_inject_prod_to, atomic_inject_bor_to, atomic_inject_band_to, atomic_inject_bxor_to
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomic_inject_lor_to, atomic_inject_land_to, atomic_inject_lxor_to
    );
}


macro_rules! gen_atomic_mr_op_def_single {
    ($func:ident (< $( $N:ident $(: $b0:ident $(+$b:ident)* )? ),* >),  ($self: ident, $($p: ident : $t: ty),*), ($($vals: expr),*), $base_func: ident) =>
    {
        unsafe fn $func< $( $N $(: $b0 $(+$b)* )? ),* >
        (
            &$self,
            $($p: $t),*
        ) -> Result<(), crate::error::Error>
        {
            $self.$base_func($($vals,)*)
        }
    };

    ($func:ident (),  $p_and_t: tt, $vals: tt, $base_func: ident) => {
        gen_atomic_mr_op_def_single!($func (<>), $p_and_t, $vals, $base_func);
    };
}

macro_rules! gen_atomic_mr_op_def {
    ($gen: tt, $args: tt, $vals: tt, $($base_func: ident,)+, $($func:ident),+) =>
    {
        $(
            gen_atomic_mr_op_def_single!($func $gen, $args, $vals, $base_func);
        )+
    }
}

// TODO: Enable support for boolean operations
pub trait AtomicWriteEpMrSlice: AtomicWriteEp {
    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        (mr_slice.as_slice(), Some(mr_slice.desc()), dest_addr, mem_addr, mapped_key),
        atomic_min_to, atomic_max_to, atomic_sum_to,  atomic_prod_to, atomic_bor_to, atomic_band_to, atomic_bxor_to,, atomic_mr_min_to, atomic_mr_max_to, atomic_mr_sum_to,  atomic_mr_prod_to, atomic_mr_bor_to, atomic_mr_band_to, atomic_mr_bxor_to
    );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
        (mr_slice.as_slice(), Some(mr_slice.desc()), dest_addr, mem_addr, mapped_key, context),
        atomic_min_to_with_context, atomic_max_to_with_context, atomic_sum_to_with_context,  atomic_prod_to_with_context, atomic_bor_to_with_context, atomic_band_to_with_context, atomic_bxor_to_with_context,, atomic_mr_min_to_with_context, atomic_mr_max_to_with_context, atomic_mr_sum_to_with_context,  atomic_mr_prod_to_with_context, atomic_mr_bor_to_with_context, atomic_mr_band_to_with_context, atomic_mr_bxor_to_with_context
    );


    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
        (mr_slice.as_slice(), Some(mr_slice.desc()), dest_addr, mem_addr, mapped_key, context),
        atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered,  atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered,, atomic_mr_min_to_triggered, atomic_mr_max_to_triggered, atomic_mr_sum_to_triggered,  atomic_mr_prod_to_triggered, atomic_mr_bor_to_triggered, atomic_mr_band_to_triggered, atomic_mr_bxor_to_triggered
    );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        (mr_slice.as_slice(), dest_addr, mem_addr, mapped_key),
        atomic_inject_min_to, atomic_inject_max_to, atomic_inject_sum_to,  atomic_inject_prod_to, atomic_inject_bor_to, atomic_inject_band_to, atomic_inject_bxor_to,, atomic_mr_inject_min_to, atomic_mr_inject_max_to, atomic_mr_inject_sum_to,  atomic_mr_inject_prod_to, atomic_mr_inject_bor_to, atomic_mr_inject_band_to, atomic_mr_inject_bxor_to
    );
}

impl<EP: AtomicWriteEp> AtomicWriteEpMrSlice for EP {}

pub trait ConnectedAtomicWriteEp {
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
    atomic_min_with_context, atomic_max_with_context, atomic_sum_with_context, atomic_prod_with_context, atomic_bor_with_context, atomic_band_with_context, atomic_bxor_with_context
    );
    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
    atomic_lor_with_context, atomic_land_with_context, atomic_lxor_with_context
    );
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),atomic_min, atomic_max, atomic_sum, atomic_prod, atomic_bor, atomic_band, atomic_bxor
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomic_lor, atomic_land, atomic_lxor
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ), 
    atomic_min_triggered, atomic_max_triggered, atomic_sum_triggered, atomic_prod_triggered, atomic_bor_triggered, atomic_band_triggered, atomic_bxor_triggered
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ), 
    atomic_lor_triggered, atomic_land_triggered, atomic_lxor_triggered
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomicv_min, atomicv_max, atomicv_sum, atomicv_prod, atomicv_bor, atomicv_band, atomicv_bxor
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomicv_lor, atomicv_land, atomicv_lxor
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
    atomicv_min_with_context, atomicv_max_with_context, atomicv_sum_with_context, atomicv_prod_with_context, atomicv_bor_with_context, atomicv_band_with_context, atomicv_bxor_with_context
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
    atomicv_lor_with_context, atomicv_land_with_context, atomicv_lxor_with_context
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
    atomicv_min_triggered, atomicv_max_triggered, atomicv_sum_triggered, atomicv_prod_triggered, atomicv_bor_triggered, atomicv_band_triggered, atomicv_bxor_triggered
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
    atomicv_lor_triggered, atomicv_land_triggered, atomicv_lxor_triggered
    );

    unsafe fn atomicmsg<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error>;

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomic_inject_min, atomic_inject_max, atomic_inject_sum, atomic_inject_prod, atomic_inject_bor, atomic_inject_band, atomic_inject_bxor
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
    atomic_inject_lor, atomic_inject_land, atomic_inject_lxor
    );
}

pub trait ConnectedAtomicWriteEpMrSlice: ConnectedAtomicWriteEp {
    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        (mr_slice.as_slice(), Some(mr_slice.desc()), mem_addr, mapped_key),
        atomic_min, atomic_max, atomic_sum,  atomic_prod, atomic_bor, atomic_band, atomic_bxor,, atomic_mr_min, atomic_mr_max, atomic_mr_sum,  atomic_mr_prod, atomic_mr_bor, atomic_mr_band, atomic_mr_bxor
    );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
        (mr_slice.as_slice(), Some(mr_slice.desc()), mem_addr, mapped_key, context),
        atomic_min_with_context, atomic_max_with_context, atomic_sum_with_context,  atomic_prod_with_context, atomic_bor_with_context, atomic_band_with_context, atomic_bxor_with_context,, atomic_mr_min_with_context, atomic_mr_max_with_context, atomic_mr_sum_with_context,  atomic_mr_prod_with_context, atomic_mr_bor_with_context, atomic_mr_band_with_context, atomic_mr_bxor_with_context
    );


    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
        (mr_slice.as_slice(), Some(mr_slice.desc()), mem_addr, mapped_key, context),
        atomic_min_triggered, atomic_max_triggered, atomic_sum_triggered,  atomic_prod_triggered, atomic_bor_triggered, atomic_band_triggered, atomic_bxor_triggered,, atomic_mr_min_triggered, atomic_mr_max_triggered, atomic_mr_sum_triggered,  atomic_mr_prod_triggered, atomic_mr_bor_triggered, atomic_mr_band_triggered, atomic_mr_bxor_triggered
    );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        (mr_slice.as_slice(), mem_addr, mapped_key),
        atomic_inject_min, atomic_inject_max, atomic_inject_sum,  atomic_inject_prod, atomic_inject_bor, atomic_inject_band, atomic_inject_bxor,, atomic_mr_inject_min, atomic_mr_inject_max, atomic_mr_inject_sum,  atomic_mr_inject_prod, atomic_mr_inject_bor, atomic_mr_inject_band, atomic_mr_inject_bxor
    );
}

impl<EP: ConnectedAtomicWriteEp> ConnectedAtomicWriteEpMrSlice for EP {}

pub trait AtomicWriteRemoteMemAddrSliceEp: AtomicWriteEp {
    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>
    ),   
        (
            buf,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_min_to, atomic_max_to, atomic_sum_to,  atomic_prod_to, atomic_bor_to, atomic_band_to, atomic_bxor_to,, atomic_mr_slice_min_to, atomic_mr_slice_max_to, atomic_mr_slice_sum_to,  atomic_mr_slice_prod_to, atomic_mr_slice_bor_to, atomic_mr_slice_band_to, atomic_mr_slice_bxor_to
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>
    ),   
        (
            buf,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_lor_to, atomic_land_to, atomic_lxor_to,, atomic_mr_slice_lor_to, atomic_mr_slice_land_to, atomic_mr_slice_lxor_to
    );
    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ),   
        (
            buf,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_min_to_with_context, atomic_max_to_with_context, atomic_sum_to_with_context,  atomic_prod_to_with_context, atomic_bor_to_with_context, atomic_band_to_with_context, atomic_bxor_to_with_context,, atomic_mr_slice_min_to_with_context, atomic_mr_slice_max_to_with_context, atomic_mr_slice_sum_to_with_context,  atomic_mr_slice_prod_to_with_context, atomic_mr_slice_bor_to_with_context, atomic_mr_slice_band_to_with_context, atomic_mr_slice_bxor_to_with_context
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut Context
    ),   
        (
            buf,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_lor_to_with_context, atomic_land_to_with_context, atomic_lxor_to_with_context,, atomic_mr_slice_lor_to_with_context, atomic_mr_slice_land_to_with_context, atomic_mr_slice_lxor_to_with_context
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut TriggeredContext
    ),   
        (
            buf,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered,  atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered,, atomic_mr_slice_min_to_triggered, atomic_mr_slice_max_to_triggered, atomic_mr_slice_sum_to_triggered,  atomic_mr_slice_prod_to_triggered, atomic_mr_slice_bor_to_triggered, atomic_mr_slice_band_to_triggered, atomic_mr_slice_bxor_to_triggered
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut TriggeredContext
    ),   
        (
            buf,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_lor_to_triggered, atomic_land_to_triggered, atomic_lxor_to_triggered,, atomic_mr_slice_lor_to_triggered, atomic_mr_slice_land_to_triggered, atomic_mr_slice_lxor_to_triggered
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>
    ), 
        (
            ioc,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomicv_min_to, atomicv_max_to, atomicv_sum_to,  atomicv_prod_to, atomicv_bor_to, atomicv_band_to, atomicv_bxor_to,, atomicv_mr_slice_min_to, atomicv_mr_slice_max_to, atomicv_mr_slice_sum_to,  atomicv_mr_slice_prod_to, atomicv_mr_slice_bor_to, atomicv_mr_slice_band_to, atomicv_mr_slice_bxor_to
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>
    ), 
        (
            ioc,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomicv_lor_to, atomicv_land_to, atomicv_lxor_to,, atomicv_mr_slice_lor_to, atomicv_mr_slice_land_to, atomicv_mr_slice_lxor_to
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ), 
        (
            ioc,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_min_to_with_context, atomicv_max_to_with_context, atomicv_sum_to_with_context,  atomicv_prod_to_with_context, atomicv_bor_to_with_context, atomicv_band_to_with_context, atomicv_bxor_to_with_context,, atomicv_mr_slice_min_to_with_context, atomicv_mr_slice_max_to_with_context, atomicv_mr_slice_sum_to_with_context,  atomicv_mr_slice_prod_to_with_context, atomicv_mr_slice_bor_to_with_context, atomicv_mr_slice_band_to_with_context, atomicv_mr_slice_bxor_to_with_context
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut Context
    ), 
        (
            ioc,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_lor_to_with_context, atomicv_land_to_with_context, atomicv_lxor_to_with_context,, atomicv_mr_slice_lor_to_with_context, atomicv_mr_slice_land_to_with_context, atomicv_mr_slice_lxor_to_with_context
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut TriggeredContext
    ), 
        (
            ioc,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_min_to_triggered, atomicv_max_to_triggered, atomicv_sum_to_triggered,  atomicv_prod_to_triggered, atomicv_bor_to_triggered, atomicv_band_to_triggered, atomicv_bxor_to_triggered,, atomicv_mr_slice_min_to_triggered, atomicv_mr_slice_max_to_triggered, atomicv_mr_slice_sum_to_triggered,  atomicv_mr_slice_prod_to_triggered, atomicv_mr_slice_bor_to_triggered, atomicv_mr_slice_band_to_triggered, atomicv_mr_slice_bxor_to_triggered
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut TriggeredContext
    ), 
        (
            ioc,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_lor_to_triggered, atomicv_land_to_triggered, atomicv_lxor_to_triggered,, atomicv_mr_slice_lor_to_triggered, atomicv_mr_slice_land_to_triggered, atomicv_mr_slice_lxor_to_triggered
    );

    unsafe fn atomicmsg_slice_to<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        // assert!(msg.slice().mem_len() == std::mem::size_of_val(msg.buf()));
        self.atomicmsg_to(msg, options)
    }

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>
    ), 
        (
            buf,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_inject_min_to, atomic_inject_max_to, atomic_inject_sum_to,  atomic_inject_prod_to, atomic_inject_bor_to, atomic_inject_band_to, atomic_inject_bxor_to,, atomic_inject_mr_slice_min_to, atomic_inject_mr_slice_max_to, atomic_inject_mr_slice_sum_to,  atomic_inject_mr_slice_prod_to, atomic_inject_mr_slice_bor_to, atomic_inject_mr_slice_band_to, atomic_inject_mr_slice_bxor_to
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>
    ), 
        (
            buf,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_inject_lor_to, atomic_inject_land_to, atomic_inject_lxor_to,, atomic_inject_mr_slice_lor_to, atomic_inject_mr_slice_land_to, atomic_inject_mr_slice_lxor_to
    );
}

pub trait ConnectedAtomicWriteRemoteMemAddrSliceEp: ConnectedAtomicWriteEp {
    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<RT>
    ),   
        (
            buf,
            desc,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_min, atomic_max, atomic_sum,  atomic_prod, atomic_bor, atomic_band, atomic_bxor,, atomic_mr_slice_min, atomic_mr_slice_max, atomic_mr_slice_sum,  atomic_mr_slice_prod, atomic_mr_slice_bor, atomic_mr_slice_band, atomic_mr_slice_bxor
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<bool>
    ),   
        (
            buf,
            desc,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_lor, atomic_land, atomic_lxor,, atomic_mr_slice_lor, atomic_mr_slice_land, atomic_mr_slice_lxor
    );
    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<RT>,
        context: &mut Context
    ),   
        (
            buf,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_min_with_context, atomic_max_with_context, atomic_sum_with_context,  atomic_prod_with_context, atomic_bor_with_context, atomic_band_with_context, atomic_bxor_with_context,, atomic_mr_slice_min_with_context, atomic_mr_slice_max_with_context, atomic_mr_slice_sum_with_context,  atomic_mr_slice_prod_with_context, atomic_mr_slice_bor_with_context, atomic_mr_slice_band_with_context, atomic_mr_slice_bxor_with_context
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut Context
    ),   
        (
            buf,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_lor_with_context, atomic_land_with_context, atomic_lxor_with_context,, atomic_mr_slice_lor_with_context, atomic_mr_slice_land_with_context, atomic_mr_slice_lxor_with_context
    );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<RT>,
        context: &mut TriggeredContext
    ),   
        (
            buf,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_min_triggered, atomic_max_triggered, atomic_sum_triggered,  atomic_prod_triggered, atomic_bor_triggered, atomic_band_triggered, atomic_bxor_triggered,, atomic_mr_slice_min_triggered, atomic_mr_slice_max_triggered, atomic_mr_slice_sum_triggered,  atomic_mr_slice_prod_triggered, atomic_mr_slice_bor_triggered, atomic_mr_slice_band_triggered, atomic_mr_slice_bxor_triggered
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut TriggeredContext
    ),   
        (
            buf,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_lor_triggered, atomic_land_triggered, atomic_lxor_triggered,, atomic_mr_slice_lor_triggered, atomic_mr_slice_land_triggered, atomic_mr_slice_lxor_triggered
    );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<RT>
    ), 
        (
            ioc,
            desc,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomicv_min, atomicv_max, atomicv_sum,  atomicv_prod, atomicv_bor, atomicv_band, atomicv_bxor,, atomicv_mr_slice_min, atomicv_mr_slice_max, atomicv_mr_slice_sum,  atomicv_mr_slice_prod, atomicv_mr_slice_bor, atomicv_mr_slice_band, atomicv_mr_slice_bxor
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<bool>
    ), 
        (
            ioc,
            desc,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomicv_lor, atomicv_land, atomicv_lxor,, atomicv_mr_slice_lor, atomicv_mr_slice_land, atomicv_mr_slice_lxor
    );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<RT>,
        context: &mut Context
    ), 
        (
            ioc,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_min_with_context, atomicv_max_with_context, atomicv_sum_with_context,  atomicv_prod_with_context, atomicv_bor_with_context, atomicv_band_with_context, atomicv_bxor_with_context,, atomicv_mr_slice_min_with_context, atomicv_mr_slice_max_with_context, atomicv_mr_slice_sum_with_context,  atomicv_mr_slice_prod_with_context, atomicv_mr_slice_bor_with_context, atomicv_mr_slice_band_with_context, atomicv_mr_slice_bxor_with_context
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut Context
    ), 
        (
            ioc,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_lor_with_context, atomicv_land_with_context, atomicv_lxor_with_context,, atomicv_mr_slice_lor_with_context, atomicv_mr_slice_land_with_context, atomicv_mr_slice_lxor_with_context
    );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<RT>,
        context: &mut TriggeredContext
    ), 
        (
            ioc,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_min_triggered, atomicv_max_triggered, atomicv_sum_triggered,  atomicv_prod_triggered, atomicv_bor_triggered, atomicv_band_triggered, atomicv_bxor_triggered,, atomicv_mr_slice_min_triggered, atomicv_mr_slice_max_triggered, atomicv_mr_slice_sum_triggered,  atomicv_mr_slice_prod_triggered, atomicv_mr_slice_bor_triggered, atomicv_mr_slice_band_triggered, atomicv_mr_slice_bxor_triggered
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut TriggeredContext
    ), 
        (
            ioc,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_lor_triggered, atomicv_land_triggered, atomicv_lxor_triggered,, atomicv_mr_slice_lor_triggered, atomicv_mr_slice_land_triggered, atomicv_mr_slice_lxor_triggered
    );

    unsafe fn atomicmsg_slice<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        // assert!(msg.slice().mem_len() == std::mem::size_of_val(msg.buf()));
        self.atomicmsg(msg, options)
    }

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        dest_slice: &RemoteMemAddrSliceMut<RT>
    ), 
        (
            buf,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_inject_min, atomic_inject_max, atomic_inject_sum,  atomic_inject_prod, atomic_inject_bor, atomic_inject_band, atomic_inject_bxor,, atomic_inject_mr_slice_min, atomic_inject_mr_slice_max, atomic_inject_mr_slice_sum,  atomic_inject_mr_slice_prod, atomic_inject_mr_slice_bor, atomic_inject_mr_slice_band, atomic_inject_mr_slice_bxor
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        dest_slice: &RemoteMemAddrSliceMut<bool>
    ), 
        (
            buf,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_inject_lor, atomic_inject_land, atomic_inject_lxor,, atomic_inject_mr_slice_lor, atomic_inject_mr_slice_land, atomic_inject_mr_slice_lxor
    );
}

impl<EP: AtomicWriteEp> AtomicWriteRemoteMemAddrSliceEp for EP {}

impl<EP: AtomicWriteEpImpl + ConnlessEp> AtomicWriteEp for EP {
    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        atomic_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, None), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_min_to, atomic_max_to, atomic_sum_to, atomic_prod_to, atomic_bor_to, atomic_band_to, atomic_bxor_to
    );
    
    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
        atomic_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, None), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_lor_to, atomic_land_to, atomic_lxor_to
    );


    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
        atomic_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some(context.inner_mut())), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_min_to_with_context, atomic_max_to_with_context, atomic_sum_to_with_context, atomic_prod_to_with_context, atomic_bor_to_with_context, atomic_band_to_with_context, atomic_bxor_to_with_context
    );
    
    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
        atomic_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some(context.inner_mut())), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_lor_to_with_context, atomic_land_to_with_context, atomic_lxor_to_with_context
    );


    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
        atomic_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some(context.inner_mut())),  AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
        atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered, atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered
    );

    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
        atomic_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some(context.inner_mut())),  AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
        atomic_lor_to_triggered, atomic_land_to_triggered, atomic_lxor_to_triggered
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        atomicv_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, None), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
        atomicv_min_to, atomicv_max_to, atomicv_sum_to, atomicv_prod_to, atomicv_bor_to, atomicv_band_to, atomicv_bxor_to
    );

    gen_atomic_op_def!((), ( 
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
        atomicv_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, None), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
        atomicv_lor_to, atomicv_land_to, atomicv_lxor_to
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context
    ),
        atomicv_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, Some(ctx.inner_mut())), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
        atomicv_min_to_with_context, atomicv_max_to_with_context, atomicv_sum_to_with_context, atomicv_prod_to_with_context, atomicv_bor_to_with_context, atomicv_band_to_with_context, atomicv_bxor_to_with_context
    );

    gen_atomic_op_def!((), ( 
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context
    ),
        atomicv_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, Some(ctx.inner_mut())), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
        atomicv_lor_to_with_context, atomicv_land_to_with_context, atomicv_lxor_to_with_context
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut TriggeredContext
    ),
        atomicv_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, Some(ctx.inner_mut())), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
        atomicv_min_to_triggered, atomicv_max_to_triggered, atomicv_sum_to_triggered, atomicv_prod_to_triggered, atomicv_bor_to_triggered, atomicv_band_to_triggered, atomicv_bxor_to_triggered
    );

    gen_atomic_op_def!((), ( 
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut TriggeredContext
    ),
        atomicv_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, Some(ctx.inner_mut())), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
        atomicv_lor_to_triggered, atomicv_land_to_triggered, atomicv_lxor_to_triggered
    );

    #[inline]
    unsafe fn atomicmsg_to<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.atomicmsg_impl(Either::Left(msg), options)
    }

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        inject_atomic_impl(buf, Some(dest_addr), mem_addr, mapped_key), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
        atomic_inject_min_to, atomic_inject_max_to, atomic_inject_sum_to, atomic_inject_prod_to, atomic_inject_bor_to, atomic_inject_band_to, atomic_inject_bxor_to
    );

    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
        inject_atomic_impl(buf, Some(dest_addr), mem_addr, mapped_key), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
        atomic_inject_lor_to, atomic_inject_land_to, atomic_inject_lxor_to
    );
}

impl<EP: AtomicWriteEpImpl + ConnectedEp> ConnectedAtomicWriteEp for EP {
    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        atomic_impl(buf, desc, None, mem_addr, mapped_key, None), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_min, atomic_max, atomic_sum, atomic_prod, atomic_bor, atomic_band, atomic_bxor
    );
    
    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
        atomic_impl(buf, desc, None, mem_addr, mapped_key, None), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_lor, atomic_land, atomic_lxor
    );


    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
        atomic_impl(buf, desc, None, mem_addr, mapped_key, Some(context.inner_mut())), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_min_with_context, atomic_max_with_context, atomic_sum_with_context, atomic_prod_with_context, atomic_bor_with_context, atomic_band_with_context, atomic_bxor_with_context
    );
    
    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ),
        atomic_impl(buf, desc, None, mem_addr, mapped_key, Some(context.inner_mut())), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_lor_with_context, atomic_land_with_context, atomic_lxor_with_context
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
        atomic_impl(
            buf,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut())
        ),
        AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_min_triggered, atomic_max_triggered, atomic_sum_triggered, atomic_prod_triggered, atomic_bor_triggered, atomic_band_triggered, atomic_bxor_triggered
    );

    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ),
        atomic_impl(
            buf,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut())
        ),
        AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_lor_triggered, atomic_land_triggered, atomic_lxor_triggered
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ), 
        atomicv_impl(ioc, desc, None, mem_addr, mapped_key, None),
        AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomicv_min, atomicv_max, atomicv_sum, atomicv_prod, atomicv_bor, atomicv_band, atomicv_bxor
    );

    gen_atomic_op_def!((), ( 
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ), 
        atomicv_impl(ioc, desc, None, mem_addr, mapped_key, None),
        AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomicv_lor, atomicv_land, atomicv_lxor
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ), 
        atomicv_impl(ioc, desc, None, mem_addr, mapped_key, Some(context.inner_mut())),
        AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomicv_min_with_context, atomicv_max_with_context, atomicv_sum_with_context, atomicv_prod_with_context, atomicv_bor_with_context, atomicv_band_with_context, atomicv_bxor_with_context
    );

    gen_atomic_op_def!((), ( 
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ), 
        atomicv_impl(ioc, desc, None, mem_addr, mapped_key, Some(context.inner_mut())),
        AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomicv_lor_with_context, atomicv_land_with_context, atomicv_lxor_with_context
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ), 
        atomicv_impl(ioc, desc, None, mem_addr, mapped_key, Some(context.inner_mut())),
        AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomicv_min_triggered, atomicv_max_triggered, atomicv_sum_triggered, atomicv_prod_triggered, atomicv_bor_triggered, atomicv_band_triggered, atomicv_bxor_triggered
    );

    gen_atomic_op_def!((), ( 
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext
    ), 
        atomicv_impl(ioc, desc, None, mem_addr, mapped_key, Some(context.inner_mut())),
        AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomicv_lor_triggered, atomicv_land_triggered, atomicv_lxor_triggered
    );

    unsafe fn atomicmsg<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.atomicmsg_impl(Either::Right(msg), options)
    }

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ),
        inject_atomic_impl(buf, None, mem_addr, mapped_key),
        AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_inject_min, atomic_inject_max, atomic_inject_sum, atomic_inject_prod, atomic_inject_bor, atomic_inject_band, atomic_inject_bxor
    );

    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    ),
        inject_atomic_impl(buf, None, mem_addr, mapped_key),
        AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_inject_lor, atomic_inject_land, atomic_inject_lxor
    );
}

impl<EP: ConnectedAtomicWriteEp> ConnectedAtomicWriteRemoteMemAddrSliceEp for EP {}

// impl<E: AtomicCap+ WriteMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> EndpointBase<E> {
impl<EP: AtomicCap + WriteMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AtomicWriteEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + WriteMod, STATE: EpState, CQ: ?Sized + ReadCq> AtomicWriteEpImpl
    for TxContextImplBase<I, STATE, CQ>
{
}

impl<I: AtomicCap + WriteMod, STATE: EpState, CQ: ?Sized + ReadCq> AtomicWriteEpImpl
    for TxContextBase<I, STATE, CQ>
{
}

impl<E: AtomicWriteEpImpl> AtomicWriteEpImpl for EndpointBase<E, Connected> {}
impl<E: AtomicWriteEpImpl> AtomicWriteEpImpl for EndpointBase<E, Connectionless> {}

pub(crate) trait AtomicFetchEpImpl: AsTypedFid<EpRawFid> + AtomicValidEp {
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic_impl<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_fetch_atomic(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                res.as_mut_ptr().cast(),
                res_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_impl<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_fetch_atomicv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                ioc.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                ioc.len(),
                resultv.as_mut_ptr().cast(),
                res_desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                resultv.len(),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    fn fetch_atomicmsg_impl<T: AsFiType>(
        &self,
        msg: Either<&crate::msg::MsgFetchAtomic<T>, &crate::msg::MsgFetchAtomicConnected<T>>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_atomic_msg = match msg {
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_fetch_atomicmsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_atomic_msg,
                resultv.as_mut_ptr().cast(),
                res_desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                resultv.len(),
                options.as_raw(),
            )
        };
        check_error(err)
    }
}

pub trait AtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    unsafe fn fetch_atomicmsg_from<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<(), crate::error::Error>;
}
pub trait ConnectedAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    unsafe fn fetch_atomicmsg<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

pub trait AtomicFetchEpMrSlice: AtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_mr_slice_from<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        res_slice: &mut MemoryRegionSliceMut,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        let result_desc = res_slice.desc();

        self.fetch_atomic_from(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            res_slice.as_mut_slice(),
            Some(result_desc),
            dest_addr,
            mem_addr,
            mapped_key,
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_mr_slice_from_with_context<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        res_slice: &mut MemoryRegionSliceMut,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        let result_desc = res_slice.desc();

        self.fetch_atomic_from_with_context(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            res_slice.as_mut_slice(),
            Some(result_desc),
            dest_addr,
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_mr_slice_from_triggered<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        res_slice: &mut MemoryRegionSliceMut,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        let result_desc = res_slice.desc();

        self.fetch_atomic_from_triggered(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            res_slice.as_mut_slice(),
            Some(result_desc),
            dest_addr,
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }
}

impl<EP: AtomicFetchEp> AtomicFetchEpMrSlice for EP {}

pub trait ConnectedAtomicFetchEpMrSlice: ConnectedAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_mr_slice<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        res_slice: &mut MemoryRegionSliceMut,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        let result_desc = res_slice.desc();

        self.fetch_atomic(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            res_slice.as_mut_slice(),
            Some(result_desc),
            mem_addr,
            mapped_key,
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_mr_slice_with_context<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        res_slice: &mut MemoryRegionSliceMut,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        let result_desc = res_slice.desc();

        self.fetch_atomic_with_context(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            res_slice.as_mut_slice(),
            Some(result_desc),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_mr_slice_triggered<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        res_slice: &mut MemoryRegionSliceMut,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        let result_desc = res_slice.desc();

        self.fetch_atomic_triggered(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            res_slice.as_mut_slice(),
            Some(result_desc),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }
}

impl<EP: ConnectedAtomicFetchEp> ConnectedAtomicFetchEpMrSlice for EP {}

impl<EP: AtomicFetchEpImpl + ConnlessEp> AtomicFetchEp for EP {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomic_impl(
            buf,
            desc,
            res,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            None,
        )
    }
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomic_impl(
            buf,
            desc,
            res,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomic_impl(
            buf,
            desc,
            res,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_impl(
            ioc,
            desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            None,
        )
    }
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_impl(
            ioc,
            desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_impl(
            ioc,
            desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }
    unsafe fn fetch_atomicmsg_from<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicmsg_impl(Either::Left(msg), resultv, res_desc, options)
    }
}
impl<EP: AtomicFetchEpImpl + ConnectedEp> ConnectedAtomicFetchEp for EP {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomic_impl(
            buf, desc, res, res_desc, None, mem_addr, mapped_key, op, None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomic_impl(
            buf,
            desc,
            res,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomic_impl(
            buf,
            desc,
            res,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_impl(
            ioc, desc, resultv, res_desc, None, mem_addr, mapped_key, op, None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_impl(
            ioc,
            desc,
            resultv,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_impl(
            ioc,
            desc,
            resultv,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    unsafe fn fetch_atomicmsg<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicmsg_impl(Either::Right(msg), resultv, res_desc, options)
    }
}

pub trait AtomicFetchRemoteMemAddrSliceEp: AtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_slice_from<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        assert!(src_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(src_slice.mem_size() == std::mem::size_of_val(res));
        self.fetch_atomic_from(
            buf,
            desc,
            res,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            src_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_slice_from_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        assert!(src_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(src_slice.mem_size() == std::mem::size_of_val(res));
        self.fetch_atomic_from_with_context(
            buf,
            desc,
            res,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            src_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_slice_from_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        assert!(src_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(src_slice.mem_size() == std::mem::size_of_val(res));
        self.fetch_atomic_from_triggered(
            buf,
            desc,
            res,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            src_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_slice_from<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        // Optionally, you can check total length of ioc/resultv matches src_slice.mem_len()
        self.fetch_atomicv_from(
            ioc,
            desc,
            resultv,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            src_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_slice_from_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_from_with_context(
            ioc,
            desc,
            resultv,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            src_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_slice_from_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_from_triggered(
            ioc,
            desc,
            resultv,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            src_slice.key(),
            op,
            context,
        )
    }

    unsafe fn fetch_atomicmsg_slice_from<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<(), crate::error::Error> {
        // Optionally, you can check msg.slice().mem_len() == total resultv size
        self.fetch_atomicmsg_from(msg, resultv, res_desc, options)
    }
}

impl<EP: AtomicFetchEp> AtomicFetchRemoteMemAddrSliceEp for EP {}

pub trait ConnectedAtomicFetchRemoteMemAddrSliceEp: ConnectedAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_slice<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        assert!(src_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(src_slice.mem_size() == std::mem::size_of_val(res));
        self.fetch_atomic(
            buf,
            desc,
            res,
            res_desc,
            src_slice.mem_address(),
            src_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_slice_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        assert!(src_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(src_slice.mem_size() == std::mem::size_of_val(res));
        self.fetch_atomic_with_context(
            buf,
            desc,
            res,
            res_desc,
            src_slice.mem_address(),
            src_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_slice_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        assert!(src_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(src_slice.mem_size() == std::mem::size_of_val(res));
        self.fetch_atomic_triggered(
            buf,
            desc,
            res,
            res_desc,
            src_slice.mem_address(),
            src_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_slice<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        // Optionally, you can check total length of ioc/resultv matches src_slice.mem_len()
        self.fetch_atomicv(
            ioc,
            desc,
            resultv,
            res_desc,
            src_slice.mem_address(),
            src_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_slice_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_with_context(
            ioc,
            desc,
            resultv,
            res_desc,
            src_slice.mem_address(),
            src_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_slice_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_triggered(
            ioc,
            desc,
            resultv,
            res_desc,
            src_slice.mem_address(),
            src_slice.key(),
            op,
            context,
        )
    }

    unsafe fn fetch_atomicmsg_slice<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<(), crate::error::Error> {
        // Optionally, you can check msg.slice().mem_len() == total resultv size
        self.fetch_atomicmsg(msg, resultv, res_desc, options)
    }
}

impl<EP: ConnectedAtomicFetchEp> ConnectedAtomicFetchRemoteMemAddrSliceEp for EP {}

impl<EP: AtomicCap + ReadMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AtomicFetchEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + ReadMod, STATE: EpState, CQ: ?Sized + ReadCq> AtomicFetchEpImpl
    for TxContextImplBase<I, STATE, CQ>
{
}

impl<I: AtomicCap + ReadMod, STATE: EpState, CQ: ?Sized + ReadCq> AtomicFetchEpImpl
    for TxContextBase<I, STATE, CQ>
{
}

impl<E: AtomicFetchEpImpl> AtomicFetchEpImpl for EndpointBase<E, Connected> {}
impl<E: AtomicFetchEpImpl> AtomicFetchEpImpl for EndpointBase<E, Connectionless> {}

pub(crate) trait AtomicCASImpl: AsTypedFid<EpRawFid> + AtomicValidEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_impl<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_compare_atomic(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                compare.as_ptr().cast(),
                compare_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                result.as_mut_ptr().cast(),
                result_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_impl<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_compare_atomicv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                ioc.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                ioc.len(),
                comparetv.as_ptr().cast(),
                compare_desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                comparetv.len(),
                resultv.as_mut_ptr().cast(),
                res_desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                resultv.len(),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_impl<T: AsFiType>(
        &self,
        msg: Either<&crate::msg::MsgCompareAtomic<T>, &crate::msg::MsgCompareAtomicConnected<T>>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_atomic_msg = match msg {
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };

        let err: isize = unsafe {
            libfabric_sys::inlined_fi_compare_atomicmsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_atomic_msg,
                comparev.as_ptr().cast(),
                compare_desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                comparev.len(),
                resultv.as_mut_ptr().cast(),
                res_desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                resultv.len(),
                options.as_raw(),
            )
        };

        check_error(err)
    }
}

pub trait AtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_to<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

pub trait AtomicCASEpMrSlice: AtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_mr_slice_to<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        compare_slice: &MemoryRegionSlice,
        result_slice: &mut MemoryRegionSliceMut,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        let desc = result_slice.desc();
        
        self.compare_atomic_to(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            compare_slice.as_slice(),
            Some(compare_slice.desc()),
            result_slice.as_mut_slice(),
            Some(desc),
            dest_addr,
            mem_addr,
            mapped_key,
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_mr_slice_to_with_context<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        compare_slice: &MemoryRegionSlice,
        result_slice: &mut MemoryRegionSliceMut,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        let desc = result_slice.desc();
        self.compare_atomic_to_with_context(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            compare_slice.as_slice(),
            Some(compare_slice.desc()),
            result_slice.as_mut_slice(),
            Some(desc),
            dest_addr,
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_mr_slice_to_triggered<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        compare_slice: &MemoryRegionSlice,
        result_slice: &mut MemoryRegionSliceMut,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        let result_desc = result_slice.desc();

        self.compare_atomic_to_triggered(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            compare_slice.as_slice(),
            Some(compare_slice.desc()),
            result_slice.as_mut_slice(),
            Some(result_desc),
            dest_addr,
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }
}

impl<EP: AtomicCASEp> AtomicCASEpMrSlice for EP {}

pub trait ConnectedAtomicCASEpMrSlice: ConnectedAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_mr_slice<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        compare_slice: &MemoryRegionSlice,
        result_slice: &mut MemoryRegionSliceMut,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        let result_desc = result_slice.desc();

        self.compare_atomic(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            compare_slice.as_slice(),
            Some(compare_slice.desc()),
            result_slice.as_mut_slice(),
            Some(result_desc),
            mem_addr,
            mapped_key,
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_mr_slice_with_context<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        compare_slice: &MemoryRegionSlice,
        result_slice: &mut MemoryRegionSliceMut,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        let result_desc = result_slice.desc();

        self.compare_atomic_with_context(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            compare_slice.as_slice(),
            Some(compare_slice.desc()),
            result_slice.as_mut_slice(),
            Some(result_desc),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_mr_slice_triggered<T: AsFiType>(
        &self,
        mr_slice: &MemoryRegionSlice,
        compare_slice: &MemoryRegionSlice,
        result_slice: &mut MemoryRegionSliceMut,
        mem_addr: RemoteMemoryAddress<T>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        let result_desc = result_slice.desc();

        self.compare_atomic_triggered(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            compare_slice.as_slice(),
            Some(compare_slice.desc()),
            result_slice.as_mut_slice(),
            Some(result_desc),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }
}

impl<EP: ConnectedAtomicCASEp> ConnectedAtomicCASEpMrSlice for EP {}

pub trait ConnectedAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

impl<EP: AtomicCASImpl + ConnlessEp> AtomicCASEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomic_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            None,
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomic_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomic_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            None,
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_to<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicmsg_impl(
            Either::Left(msg),
            comparev,
            compare_desc,
            resultv,
            res_desc,
            options,
        )
    }
}

impl<EP: AtomicCASImpl + ConnectedEp> ConnectedAtomicCASEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomic_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            None,
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomic_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomic_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            None,
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_with_context<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_triggered<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicmsg_impl(
            Either::Right(msg),
            comparev,
            compare_desc,
            resultv,
            res_desc,
            options,
        )
    }
}

pub trait AtomicCASRemoteMemAddrSliceEp: AtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_slice_to<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        assert!(dest_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(compare));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(result));
        self.compare_atomic_to(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_slice_to_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        assert!(dest_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(compare));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(result));
        self.compare_atomic_to_with_context(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_slice_to_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        assert!(dest_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(compare));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(result));
        self.compare_atomic_to_triggered(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_slice_to<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        // Optionally, check lengths of ioc/comparetv/resultv match dest_slice.mem_len()
        self.compare_atomicv_to(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_slice_to_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_to_with_context(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_slice_to_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_to_triggered(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_slice_to<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        // Optionally, check msg.slice().mem_len() == total resultv size
        self.compare_atomicmsg_to(msg, comparev, compare_desc, resultv, res_desc, options)
    }
}

impl<EP: AtomicCASEp> AtomicCASRemoteMemAddrSliceEp for EP {}

pub trait ConnectedAtomicCASRemoteMemAddrSliceEp: ConnectedAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_slice<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        assert!(dest_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(compare));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(result));
        self.compare_atomic(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_slice_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        assert!(dest_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(compare));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(result));
        self.compare_atomic_with_context(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_slice_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        assert!(dest_slice.mem_size() == std::mem::size_of_val(buf));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(compare));
        assert!(dest_slice.mem_size() == std::mem::size_of_val(result));
        self.compare_atomic_triggered(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_slice<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error> {
        // Optionally, check lengths of ioc/comparetv/resultv match dest_slice.mem_len()
        self.compare_atomicv(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_slice_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_with_context(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_slice_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.compare_atomicv_triggered(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_slice<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        // Optionally, check msg.slice().mem_len() == total resultv size
        self.compare_atomicmsg(msg, comparev, compare_desc, resultv, res_desc, options)
    }
}

impl<EP: ConnectedAtomicCASEp> ConnectedAtomicCASRemoteMemAddrSliceEp for EP {}

impl<EP: AtomicCap + ReadMod + WriteMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AtomicCASImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + ReadMod + WriteMod, STATE: EpState, CQ: ?Sized + ReadCq> AtomicCASImpl
    for TxContextImplBase<I, STATE, CQ>
{
}

impl<I: AtomicCap + ReadMod + WriteMod, STATE: EpState, CQ: ?Sized + ReadCq> AtomicCASImpl
    for TxContextBase<I, STATE, CQ>
{
}

impl<E: AtomicCASImpl> AtomicCASImpl for EndpointBase<E, Connected> {}
impl<E: AtomicCASImpl> AtomicCASImpl for EndpointBase<E, Connectionless> {}

pub trait AtomicValidEp: AsTypedFid<EpRawFid> {
    unsafe fn atomicvalid<T: AsFiType>(
        &self,
        op: crate::enums::AtomicOp,
    ) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_atomicvalid(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                T::as_fi_datatype(),
                op.as_raw(),
                &mut count as *mut usize,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(count)
        }
    }

    unsafe fn fetch_atomicvalid<T: AsFiType>(
        &self,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_fetch_atomicvalid(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                T::as_fi_datatype(),
                op.as_raw(),
                &mut count as *mut usize,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(count)
        }
    }

    unsafe fn compare_atomicvalid<T: AsFiType>(
        &self,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_compare_atomicvalid(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                T::as_fi_datatype(),
                op.as_raw(),
                &mut count as *mut usize,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(count)
        }
    }
}

impl<E: AtomicValidEp> AtomicValidEp for EndpointBase<E, Connected> {}
impl<E: AtomicValidEp> AtomicValidEp for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AtomicValidEp
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap, STATE: EpState, CQ: ?Sized + ReadCq> AtomicValidEp
    for TxContextImplBase<I, STATE, CQ>
{
}

impl<I: AtomicCap, STATE: EpState, CQ: ?Sized + ReadCq> AtomicValidEp
    for TxContextBase<I, STATE, CQ>
{
}

impl<I: AtomicCap, STATE: EpState, CQ: ?Sized + ReadCq> AtomicValidEp
    for RxContextImplBase<I, STATE, CQ>
{
}

impl<I: AtomicCap, STATE: EpState, CQ: ?Sized + ReadCq> AtomicValidEp
    for RxContextBase<I, STATE, CQ>
{
}

pub struct AtomicAttr {
    pub(crate) c_attr: libfabric_sys::fi_atomic_attr,
}

impl AtomicAttr {
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_atomic_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_atomic_attr {
        &mut self.c_attr
    }
}
