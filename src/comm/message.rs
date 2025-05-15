use crate::{
    conn_ep::ConnectedEp,
    connless_ep::ConnlessEp,
    cq::ReadCq,
    enums::{RecvMsgOptions, SendMsgOptions},
    ep::{ActiveEndpoint, Connected, Connectionless, EndpointBase, EndpointImplBase, EpState},
    eq::ReadEq,
    fid::{AsRawTypedFid, AsTypedFid, EpRawFid},
    infocapsoptions::{MsgCap, RecvMod, SendMod},
    mr::MemoryRegionDesc,
    trigger::TriggeredContext,
    utils::{check_error, Either},
    xcontext::{RxContextBase, RxContextImplBase, TxContextBase, TxContextImplBase},
    Context, MappedAddress, FI_ADDR_UNSPEC,
};

pub(crate) fn extract_raw_ctx(context: Option<*mut std::ffi::c_void>) -> *mut std::ffi::c_void {
    if let Some(ctx) = context {
        ctx
    } else {
        std::ptr::null_mut()
    }
}

pub(crate) fn extract_raw_addr_and_ctx(
    mapped_addr: Option<&MappedAddress>,
    context: Option<*mut std::ffi::c_void>,
) -> (u64, *mut std::ffi::c_void) {
    let ctx = if let Some(ctx) = context {
        ctx
    } else {
        std::ptr::null_mut()
    };

    let raw_addr = if let Some(addr) = mapped_addr {
        addr.raw_addr()
    } else {
        FI_ADDR_UNSPEC
    };

    (raw_addr, ctx)
}

pub(crate) trait RecvEpImpl: AsTypedFid<EpRawFid> {
    fn recv_impl<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: Option<&crate::MappedAddress>,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_recv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                ctx,
            )
        };
        check_error(err)
    }

    fn recvv_impl(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: Option<&crate::MappedAddress>,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_recvv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                iov.len(),
                raw_addr,
                ctx,
            )
        };
        check_error(err)
    }

    fn recvmsg_impl(
        &self,
        msg: Either<&crate::msg::MsgMut, &crate::msg::MsgConnectedMut>,
        options: RecvMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_msg = match msg {
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_recvmsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_msg,
                options.as_raw(),
            )
        };
        check_error(err)
    }
}

pub trait ConnectedRecvEp {
    fn recv<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
    ) -> Result<(), crate::error::Error>;
    fn recv_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn recv_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn recvv(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
    ) -> Result<(), crate::error::Error>;
    fn recvv_with_context<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn recvv_triggered<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn recvmsg(
        &self,
        msg: &crate::msg::MsgConnectedMut,
        options: RecvMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

impl<EP: RecvEpImpl + ConnectedEp> ConnectedRecvEp for EP {
    #[inline]
    fn recv<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl::<T>(buf, desc, None, None)
    }
    #[inline]
    fn recv_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, None, Some(context.inner_mut()))
    }
    #[inline]
    fn recv_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, None, Some(context.inner_mut()))
    }
    #[inline]
    fn recvv(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, None, None)
    }
    #[inline]
    fn recvv_with_context<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, None, Some(context.inner_mut()))
    }
    #[inline]
    fn recvv_triggered<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, None, Some(context.inner_mut()))
    }
    #[inline]
    fn recvmsg(
        &self,
        msg: &crate::msg::MsgConnectedMut,
        options: RecvMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.recvmsg_impl(Either::Right(msg), options)
    }
}

pub trait RecvEp {
    fn recv_from<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error>;
    fn recv_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn recv_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn recvv_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error>;
    fn recvv_from_with_context<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn recvv_from_triggered<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn recv_from_any<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
    ) -> Result<(), crate::error::Error>;
    fn recv_from_any_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn recvv_from_any(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
    ) -> Result<(), crate::error::Error>;
    fn recvv_from_any_with_context<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn recv_from_any_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn recvv_from_any_triggered<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn recvmsg_from(
        &self,
        msg: &crate::msg::MsgMut,
        options: RecvMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

impl<EP: RecvEpImpl + ConnlessEp> RecvEp for EP {
    #[inline]
    fn recv_from_any<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl::<T>(buf, desc, None, None)
    }

    #[inline]
    fn recv_from_any_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, None, Some(context.inner_mut()))
    }

    #[inline]
    fn recv_from<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl::<T>(buf, desc, Some(mapped_addr), None)
    }

    #[inline]
    fn recv_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, Some(mapped_addr), Some(context.inner_mut()))
    }

    #[inline]
    fn recv_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, Some(mapped_addr), Some(context.inner_mut()))
    }

    #[inline]
    fn recvv_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, Some(mapped_addr), None)
    }

    #[inline]
    fn recvv_from_with_context<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, Some(mapped_addr), Some(context.inner_mut()))
    }

    #[inline]
    fn recvv_from_triggered<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, Some(mapped_addr), Some(context.inner_mut()))
    }

    #[inline]
    fn recv_from_any_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, None, Some(context.inner_mut()))
    }

    #[inline]
    fn recvv_from_any(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, None, None)
    }

    #[inline]
    fn recvv_from_any_with_context<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, None, Some(context.inner_mut()))
    }

    #[inline]
    fn recvv_from_any_triggered<T0>(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, None, Some(context.inner_mut()))
    }

    #[inline]
    fn recvmsg_from(
        &self,
        msg: &crate::msg::MsgMut,
        options: RecvMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.recvmsg_impl(Either::Left(msg), options)
    }
}

impl<EP: MsgCap + RecvMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> RecvEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

// impl<E: MsgCap + RecvMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> EndpointBase<E> {
impl<E: RecvEpImpl> RecvEpImpl for EndpointBase<E, Connected> {}
impl<E: RecvEpImpl> RecvEpImpl for EndpointBase<E, Connectionless> {}


impl<EP: MsgCap + RecvMod, STATE: EpState, CQ: ?Sized + ReadCq> RecvEpImpl for RxContextBase<EP, STATE, CQ> {}
impl<EP: MsgCap + RecvMod, STATE: EpState, CQ: ?Sized + ReadCq> RecvEpImpl for RxContextImplBase<EP, STATE, CQ> {}

pub(crate) trait SendEpImpl: AsTypedFid<EpRawFid> {
    fn sendv_impl(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: Option<&crate::MappedAddress>,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_sendv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                iov.len(),
                raw_addr,
                ctx,
            )
        };
        check_error(err)
    }

    fn send_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: Option<&crate::MappedAddress>,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_send(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                ctx,
            )
        };
        check_error(err)
    }

    fn sendmsg_impl(
        &self,
        msg: Either<&crate::msg::Msg, &crate::msg::MsgConnected>,
        options: SendMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_msg = match msg {
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_sendmsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_msg,
                options.as_raw(),
            )
        };
        check_error(err)
    }

    fn senddata_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: Option<&crate::MappedAddress>,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_senddata(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                data,
                raw_addr,
                ctx,
            )
        };
        check_error(err)
    }

    fn inject_impl<T>(
        &self,
        buf: &[T],
        mapped_addr: Option<&crate::MappedAddress>,
    ) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        } else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_inject(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                raw_addr,
            )
        };
        check_error(err)
    }

    fn injectdata_impl<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: Option<&crate::MappedAddress>,
    ) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        } else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_injectdata(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                data,
                raw_addr,
            )
        };
        check_error(err)
    }
}

pub trait SendEp {
    fn sendv_to(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error>;
    fn sendv_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn sendv_to_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn send_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error>;
    fn send_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn send_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn sendmsg_to(
        &self,
        msg: &crate::msg::Msg,
        options: SendMsgOptions,
    ) -> Result<(), crate::error::Error>;
    fn senddata_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error>;
    fn senddata_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn senddata_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn inject_to<T>(
        &self,
        buf: &[T],
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error>;
    fn injectdata_to<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error>;
}

pub trait ConnectedSendEp {
    fn sendv(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
    ) -> Result<(), crate::error::Error>;
    fn sendv_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn sendv_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn send<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
    ) -> Result<(), crate::error::Error>;
    fn send_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn send_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn sendmsg(
        &self,
        msg: &crate::msg::MsgConnected,
        options: SendMsgOptions,
    ) -> Result<(), crate::error::Error>;
    fn senddata<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
    ) -> Result<(), crate::error::Error>;
    fn senddata_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn senddata_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn inject<T>(&self, buf: &[T]) -> Result<(), crate::error::Error>;
    fn injectdata<T>(&self, buf: &[T], data: u64) -> Result<(), crate::error::Error>;
}

impl<EP: SendEpImpl + ConnlessEp> SendEp for EP {
    #[inline]
    fn sendv_to(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.sendv_impl(iov, desc, Some(mapped_addr), None)
    }

    #[inline]
    fn sendv_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.sendv_impl(iov, desc, Some(mapped_addr), Some(context.inner_mut()))
    }

    #[inline]
    fn sendv_to_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.sendv_impl(iov, desc, Some(mapped_addr), Some(context.inner_mut()))
    }

    #[inline]
    fn send_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.send_impl::<T>(buf, desc, Some(mapped_addr), None)
    }

    #[inline]
    fn send_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.send_impl(buf, desc, Some(mapped_addr), Some(context.inner_mut()))
    }

    #[inline]
    fn send_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.send_impl(buf, desc, Some(mapped_addr), Some(context.inner_mut()))
    }

    #[inline]
    fn sendmsg_to(
        &self,
        msg: &crate::msg::Msg,
        options: SendMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.sendmsg_impl(Either::Left(msg), options)
    }

    #[inline]
    fn senddata_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.senddata_impl::<T>(buf, desc, data, Some(mapped_addr), None)
    }

    #[inline]
    fn senddata_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &crate::MappedAddress,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.senddata_impl(
            buf,
            desc,
            data,
            Some(mapped_addr),
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn senddata_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &crate::MappedAddress,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.senddata_impl(
            buf,
            desc,
            data,
            Some(mapped_addr),
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn inject_to<T>(
        &self,
        buf: &[T],
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.inject_impl(buf, Some(mapped_addr))
    }

    #[inline]
    fn injectdata_to<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.injectdata_impl(buf, data, Some(mapped_addr))
    }
}

impl<EP: SendEpImpl + ConnectedEp> ConnectedSendEp for EP {
    #[inline]
    fn sendv(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
    ) -> Result<(), crate::error::Error> {
        self.sendv_impl(iov, desc, None, None)
    }

    #[inline]
    fn sendv_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.sendv_impl(iov, desc, None, Some(context.inner_mut()))
    }

    #[inline]
    fn sendv_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.sendv_impl(iov, desc, None, Some(context.inner_mut()))
    }

    #[inline]
    fn send<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
    ) -> Result<(), crate::error::Error> {
        self.send_impl::<T>(buf, desc, None, None)
    }

    #[inline]
    fn send_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.send_impl(buf, desc, None, Some(context.inner_mut()))
    }

    #[inline]
    fn send_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.send_impl(buf, desc, None, Some(context.inner_mut()))
    }

    #[inline]
    fn sendmsg(
        &self,
        msg: &crate::msg::MsgConnected,
        options: SendMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.sendmsg_impl(Either::Right(msg), options)
    }

    #[inline]
    fn senddata<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
    ) -> Result<(), crate::error::Error> {
        self.senddata_impl::<T>(buf, desc, data, None, None)
    }

    #[inline]
    fn senddata_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.senddata_impl(buf, desc, data, None, Some(context.inner_mut()))
    }

    #[inline]
    fn senddata_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.senddata_impl(buf, desc, data, None, Some(context.inner_mut()))
    }

    #[inline]
    fn inject<T>(&self, buf: &[T]) -> Result<(), crate::error::Error> {
        self.inject_impl(buf, None)
    }

    #[inline]
    fn injectdata<T>(&self, buf: &[T], data: u64) -> Result<(), crate::error::Error> {
        self.injectdata_impl(buf, data, None)
    }
}

// impl<E: MsgCap + SendMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> EndpointBase<E> {
impl<EP: MsgCap + SendMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> SendEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: MsgCap + SendMod, STATE: EpState, CQ: ?Sized + ReadCq> SendEpImpl
    for TxContextImplBase<I, STATE, CQ>
{
}

impl<I: MsgCap + SendMod, STATE: EpState, CQ: ?Sized + ReadCq> SendEpImpl
    for TxContextBase<I, STATE, CQ>
{
}

impl<E: SendEpImpl> SendEpImpl for EndpointBase<E, Connected> {}
impl<E: SendEpImpl> SendEpImpl for EndpointBase<E, Connectionless> {}

