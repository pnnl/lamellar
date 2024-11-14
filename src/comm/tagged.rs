use super::message::extract_raw_addr_and_ctx;
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::cq::ReadCq;
use crate::enums::TaggedRecvMsgOptions;
use crate::enums::TaggedSendMsgOptions;
use crate::ep::Connected;
use crate::ep::Connectionless;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::eq::ReadEq;
use crate::fid::AsRawTypedFid;
use crate::fid::AsTypedFid;
use crate::fid::EpRawFid;
use crate::infocapsoptions::RecvMod;
use crate::infocapsoptions::SendMod;
use crate::infocapsoptions::TagCap;
use crate::mr::DataDescriptor;
use crate::trigger::TriggeredContext;
use crate::utils::check_error;
use crate::utils::Either;
use crate::xcontext::RxContextBase;
use crate::xcontext::RxContextImplBase;
use crate::xcontext::TxContextBase;
use crate::xcontext::TxContextImplBase;
use crate::Context;
use crate::MappedAddress;
use crate::FI_ADDR_UNSPEC;

pub(crate) trait TagRecvEpImpl: AsTypedFid<EpRawFid> {
    fn trecv_impl<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ignore: u64,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_trecv(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr() as *mut std::ffi::c_void,
                std::mem::size_of_val(buf),
                desc.get_desc(),
                raw_addr,
                tag,
                ignore,
                ctx,
            )
        };
        check_error(err)
    }

    fn trecvv_impl(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ignore: u64,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_trecvv(
                self.as_typed_fid().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                desc.as_mut_ptr().cast(),
                iov.len(),
                raw_addr,
                tag,
                ignore,
                ctx,
            )
        };
        check_error(err)
    }

    fn trecvmsg_impl(
        &self,
        msg: Either<&crate::msg::MsgTaggedMut, &crate::msg::MsgTaggedConnectedMut>,
        options: TaggedRecvMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_tagged_msg = match msg {
            Either::Left(msg) => msg.c_msg_tagged,
            Either::Right(msg) => msg.c_msg_tagged,
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_trecvmsg(
                self.as_typed_fid().as_raw_typed_fid(),
                &c_tagged_msg,
                options.as_raw(),
            )
        };
        check_error(err)
    }
}

pub trait TagRecvEp {
    fn trecv_from<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvmsg_from(
        &self,
        msg: &crate::msg::MsgTaggedMut,
        options: TaggedRecvMsgOptions,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_any<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_any_with_context<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_any_triggered<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_any(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_any_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_any_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
}

pub trait ConnectedTagRecvEp {
    fn trecv<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error>;
    fn trecv_with_context<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecv_triggered<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvv(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvmsg(
        &self,
        msg: &crate::msg::MsgTaggedConnectedMut,
        options: TaggedRecvMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

impl<EP: TagRecvEpImpl + ConnlessEp> TagRecvEp for EP {
    #[inline]
    fn trecv_from<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, Some(mapped_addr), tag, ignore, None)
    }

    #[inline]
    fn trecv_from_any<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, None)
    }

    #[inline]
    fn trecv_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(
            buf,
            desc,
            Some(mapped_addr),
            tag,
            ignore,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn trecv_from_any_with_context<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecv_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(
            buf,
            desc,
            Some(mapped_addr),
            tag,
            ignore,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn trecv_from_any_triggered<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvv_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error> {
        self.trecvv_impl(iov, desc, Some(src_mapped_addr), tag, ignore, None)
    }

    #[inline]
    fn trecvv_from_any(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error> {
        self.trecvv_impl(iov, desc, None, tag, ignore, None)
    }

    #[inline]
    fn trecvv_from_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        //[TODO]
        self.trecvv_impl(
            iov,
            desc,
            Some(src_mapped_addr),
            tag,
            ignore,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn trecvv_from_any_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        //[TODO]
        self.trecvv_impl(iov, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvv_from_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        //[TODO]
        self.trecvv_impl(
            iov,
            desc,
            Some(src_mapped_addr),
            tag,
            ignore,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn trecvv_from_any_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        //[TODO]
        self.trecvv_impl(iov, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvmsg_from(
        &self,
        msg: &crate::msg::MsgTaggedMut,
        options: TaggedRecvMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.trecvmsg_impl(Either::Left(msg), options)
    }
}

impl<EP: TagRecvEpImpl + ConnectedEp> ConnectedTagRecvEp for EP {
    #[inline]
    fn trecv<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, None)
    }

    #[inline]
    fn trecv_with_context<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecv_triggered<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvv(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
    ) -> Result<(), crate::error::Error> {
        //[TODO]
        self.trecvv_impl(iov, desc, None, tag, ignore, None)
    }

    #[inline]
    fn trecvv_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        //[TODO]
        self.trecvv_impl(iov, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvv_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        //[TODO]
        self.trecvv_impl(iov, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvmsg(
        &self,
        msg: &crate::msg::MsgTaggedConnectedMut,
        options: TaggedRecvMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.trecvmsg_impl(Either::Right(msg), options)
    }
}

// impl<E: TagCap + RecvMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> EndpointBase<E> {
impl<EP: TagCap + RecvMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> TagRecvEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<E: TagRecvEpImpl> TagRecvEpImpl for EndpointBase<E, Connected> {}
impl<E: TagRecvEpImpl> TagRecvEpImpl for EndpointBase<E, Connectionless> {}

pub(crate) trait TagSendEpImpl: AsTypedFid<EpRawFid> {
    fn tsend_impl<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_tsend(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                desc.get_desc(),
                raw_addr,
                tag,
                ctx,
            )
        };
        check_error(err)
    }

    fn tsendv_impl(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        dest_mapped_addr: Option<&MappedAddress>,
        tag: u64,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_tsendv(
                self.as_typed_fid().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                desc.as_mut_ptr().cast(),
                iov.len(),
                raw_addr,
                tag,
                ctx,
            )
        };
        check_error(err)
    }

    fn tsendmsg_impl(
        &self,
        msg: Either<&crate::msg::MsgTagged, &crate::msg::MsgTaggedConnected>,
        options: TaggedSendMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_tagged_msg = match msg {
            Either::Left(msg) => msg.c_msg_tagged,
            Either::Right(msg) => msg.c_msg_tagged,
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_tsendmsg(
                self.as_typed_fid().as_raw_typed_fid(),
                &c_tagged_msg as *const libfabric_sys::fi_msg_tagged,
                options.as_raw(),
            )
        };
        check_error(err)
    }

    fn tsenddata_impl<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_tsenddata(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                desc.get_desc(),
                data,
                raw_addr,
                tag,
                ctx,
            )
        };
        check_error(err)
    }

    fn tinject_impl<T>(
        &self,
        buf: &[T],
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        } else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_tinject(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                raw_addr,
                tag,
            )
        };
        check_error(err)
    }

    fn tinjectdata_impl<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        } else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_tinjectdata(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                data,
                raw_addr,
                tag,
            )
        };
        check_error(err)
    }
}

pub trait TagSendEp {
    fn tsend_to<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsend_to_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsend_to_triggered<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_to(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        dest_mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_to_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tsendmsg_to(
        &self,
        msg: &crate::msg::MsgTagged,
        options: TaggedSendMsgOptions,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_to<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_to_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_to_triggered<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tinject_to<T>(
        &self,
        buf: &[T],
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tinjectdata_to<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
}

pub trait ConnectedTagSendEp {
    fn tsend<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsend_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsend_triggered<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tsendv(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tsendmsg(
        &self,
        msg: &crate::msg::MsgTaggedConnected,
        options: TaggedSendMsgOptions,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_triggered<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tinject<T>(&self, buf: &[T], tag: u64) -> Result<(), crate::error::Error>;
    fn tinjectdata<T>(&self, buf: &[T], data: u64, tag: u64) -> Result<(), crate::error::Error>;
}

impl<EP: TagSendEpImpl + ConnlessEp> TagSendEp for EP {
    #[inline]
    fn tsend_to<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, Some(mapped_addr), tag, None)
    }

    #[inline]
    fn tsend_to_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, Some(mapped_addr), tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsend_to_triggered<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, Some(mapped_addr), tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsendv_to(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        dest_mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        // [TODO]
        self.tsendv_impl(iov, desc, Some(dest_mapped_addr), tag, None)
    }

    #[inline]
    fn tsendv_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        // [TODO]
        self.tsendv_impl(
            iov,
            desc,
            Some(dest_mapped_addr),
            tag,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn tsendv_to_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        // [TODO]
        self.tsendv_impl(
            iov,
            desc,
            Some(dest_mapped_addr),
            tag,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn tsenddata_to<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(buf, desc, data, Some(mapped_addr), tag, None)
    }

    #[inline]
    fn tsenddata_to_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(
            buf,
            desc,
            data,
            Some(mapped_addr),
            tag,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn tsenddata_to_triggered<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(
            buf,
            desc,
            data,
            Some(mapped_addr),
            tag,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    fn tinject_to<T>(
        &self,
        buf: &[T],
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinject_impl(buf, Some(mapped_addr), tag)
    }

    #[inline]
    fn tinjectdata_to<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinjectdata_impl(buf, data, Some(mapped_addr), tag)
    }

    #[inline]
    fn tsendmsg_to(
        &self,
        msg: &crate::msg::MsgTagged,
        options: TaggedSendMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.tsendmsg_impl(Either::Left(msg), options)
    }
}

impl<EP: TagSendEpImpl + ConnectedEp> ConnectedTagSendEp for EP {
    #[inline]
    fn tsend<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, None, tag, None)
    }

    #[inline]
    fn tsend_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsend_triggered<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsendv(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        // [TODO]
        self.tsendv_impl(iov, desc, None, tag, None)
    }

    #[inline]
    fn tsendv_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        // [TODO]
        self.tsendv_impl(iov, desc, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsendv_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: &mut [impl DataDescriptor],
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        // [TODO]
        self.tsendv_impl(iov, desc, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsenddata<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(buf, desc, data, None, tag, None)
    }

    #[inline]
    fn tsenddata_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(buf, desc, data, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsenddata_triggered<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(buf, desc, data, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tinject<T>(&self, buf: &[T], tag: u64) -> Result<(), crate::error::Error> {
        self.tinject_impl(buf, None, tag)
    }

    #[inline]
    fn tinjectdata<T>(&self, buf: &[T], data: u64, tag: u64) -> Result<(), crate::error::Error> {
        self.tinjectdata_impl(buf, data, None, tag)
    }

    #[inline]
    fn tsendmsg(
        &self,
        msg: &crate::msg::MsgTaggedConnected,
        options: TaggedSendMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.tsendmsg_impl(Either::Right(msg), options)
    }
}

impl<EP: TagCap + SendMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> TagSendEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}
impl<E: TagSendEpImpl> TagSendEpImpl for EndpointBase<E, Connected> {}
impl<E: TagSendEpImpl> TagSendEpImpl for EndpointBase<E, Connectionless> {}

impl<CQ: ?Sized + ReadCq> TagSendEpImpl for TxContextBase<CQ> {}
impl<CQ: ?Sized + ReadCq> TagSendEpImpl for TxContextImplBase<CQ> {}
impl<CQ: ?Sized + ReadCq> TagRecvEpImpl for RxContextBase<CQ> {}
impl<CQ: ?Sized + ReadCq> TagRecvEpImpl for RxContextImplBase<CQ> {}
