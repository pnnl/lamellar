use super::message::extract_raw_addr_and_ctx;
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::cq::ReadCq;
use crate::enums::AtomicFetchMsgOptions;
use crate::enums::AtomicMsgOptions;
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
use crate::trigger::TriggeredContext;
use crate::utils::check_error;
use crate::utils::Either;
use crate::xcontext::RxContextBase;
use crate::xcontext::RxContextImplBase;
use crate::xcontext::TxContextBase;
use crate::xcontext::TxContextImplBase;
use crate::AsFiType;
use crate::Context;
use crate::FI_ADDR_UNSPEC;

pub(crate) trait AtomicWriteEpImpl: AsTypedFid<EpRawFid> + AtomicValidEp {
    #[allow(clippy::too_many_arguments)]
    fn atomic_impl<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_atomic(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                mem_addr,
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn atomicv_impl<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_atomicv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                ioc.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                ioc.len(),
                raw_addr,
                mem_addr,
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
    fn inject_atomic_impl<T: AsFiType>(
        &self,
        buf: &[T],
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
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
                mem_addr,
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
            )
        };
        check_error(err)
    }
}

pub trait AtomicWriteEp {
    unsafe fn atomic_to<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_to_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_to_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_to<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_to_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_to_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    unsafe fn atomicmsg_to<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error>;
    unsafe fn inject_atomic_to<T: AsFiType>(
        &self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error>;
}

pub trait ConnectedAtomicWriteEp {
    unsafe fn atomic<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error>;
    unsafe fn atomic_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    unsafe fn atomic_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    unsafe fn atomicv<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error>;
    unsafe fn atomicv_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    unsafe fn atomicv_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    unsafe fn atomicmsg<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error>;
    unsafe fn inject_atomic<T: AsFiType>(
        &self,
        buf: &[T],
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error>;
}

impl<EP: AtomicWriteEpImpl + ConnlessEp> AtomicWriteEp for EP {
    #[inline]
    unsafe fn atomic_to<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.atomic_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, op, None)
    }

    #[inline]
    unsafe fn atomic_to_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.atomic_impl(
            buf,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn atomic_to_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.atomic_impl(
            buf,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn atomicv_to<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.atomicv_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, op, None)
    }

    #[inline]
    unsafe fn atomicv_to_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.atomicv_impl(
            ioc,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn atomicv_to_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.atomicv_impl(
            ioc,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn atomicmsg_to<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.atomicmsg_impl(Either::Left(msg), options)
    }

    #[inline]
    unsafe fn inject_atomic_to<T: AsFiType>(
        &self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.inject_atomic_impl(buf, Some(dest_addr), mem_addr, mapped_key, op)
    }
}
impl<EP: AtomicWriteEpImpl + ConnectedEp> ConnectedAtomicWriteEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.atomic_impl(buf, desc, None, mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.atomic_impl(
            buf,
            desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.atomic_impl(
            buf,
            desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.atomicv_impl(ioc, desc, None, mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.atomicv_impl(
            ioc,
            desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.atomicv_impl(
            ioc,
            desc,
            None,
            mem_addr,
            mapped_key,
            op,
            Some(context.inner_mut()),
        )
    }

    unsafe fn atomicmsg<T: AsFiType>(
        &self,
        msg: &crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.atomicmsg_impl(Either::Right(msg), options)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn inject_atomic<T: AsFiType>(
        &self,
        buf: &[T],
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.inject_atomic_impl(buf, None, mem_addr, mapped_key, op)
    }
}

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
    fn fetch_atomic_impl<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
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
                mem_addr,
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_impl<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
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
                mem_addr,
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
    unsafe fn fetch_atomic_from<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn fetch_atomic<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
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

impl<EP: AtomicFetchEpImpl + ConnlessEp> AtomicFetchEp for EP {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn fetch_atomic_from_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn fetch_atomic_from_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn fetch_atomicv_from<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn fetch_atomicv_from_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn fetch_atomicv_from_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn fetch_atomic<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomic_impl(
            buf, desc, res, res_desc, None, mem_addr, mapped_key, op, None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
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
    unsafe fn fetch_atomic_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
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
    unsafe fn fetch_atomicv<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<(), crate::error::Error> {
        self.fetch_atomicv_impl(
            ioc, desc, resultv, res_desc, None, mem_addr, mapped_key, op, None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
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
    unsafe fn fetch_atomicv_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
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
    unsafe fn compare_atomic_impl<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
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
                mem_addr,
                mapped_key.key(),
                T::as_fi_datatype(),
                op.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_impl<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
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
                mem_addr,
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
    unsafe fn compare_atomic_to<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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

pub trait ConnectedAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
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
    unsafe fn compare_atomic_to<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn compare_atomic_to_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn compare_atomic_to_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn compare_atomicv_to<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn compare_atomicv_to_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn compare_atomicv_to_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn compare_atomic<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
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
    unsafe fn compare_atomic_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
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
    unsafe fn compare_atomic_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
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
    unsafe fn compare_atomicv<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
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
    unsafe fn compare_atomicv_with_context<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
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
    unsafe fn compare_atomicv_triggered<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
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
