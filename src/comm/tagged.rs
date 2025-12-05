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
use crate::ep::EpState;
use crate::eq::ReadEq;
use crate::fid::AsRawTypedFid;
use crate::fid::AsTypedFid;
use crate::fid::EpRawFid;
use crate::infocapsoptions::RecvMod;
use crate::infocapsoptions::SendMod;
use crate::infocapsoptions::TagCap;
use crate::mr::MemoryRegionDesc;
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
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ignore: Option<u64>,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_trecv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_mut_ptr() as *mut std::ffi::c_void,
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                tag,
                ignore.unwrap_or(0),
                ctx,
            )
        };
        check_error(err)
    }

    fn trecvv_impl(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ignore: Option<u64>,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_trecvv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                iov.len(),
                raw_addr,
                tag,
                ignore.unwrap_or(0),
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
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_trecvmsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_tagged_msg,
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
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
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
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_any_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecv_from_any_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_any(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_any_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_from_any_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
}

pub trait ConnectedTagRecvEp {
    fn trecv<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error>;
    fn trecv_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecv_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvv(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn trecvv_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn trecvmsg(
        &self,
        msg: &crate::msg::MsgTaggedConnectedMut,
        options: TaggedRecvMsgOptions,
    ) -> Result<(), crate::error::Error>;
}


pub trait ConnectedTagRecvEpMrSlice: ConnectedTagRecvEp {
    fn trecv_mr_slice(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv(
            mr_slice.as_mut_slice(),
            Some(desc),
            tag,
            ignore
        )
    }

    fn trecv_mr_slice_with_context(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        context: &mut Context,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv_with_context(
            mr_slice.as_mut_slice(),
            Some(desc),
            tag,
            ignore,
            context,
        )
    }
    fn trecv_mr_slice_triggered(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        context: &mut TriggeredContext,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv_triggered(
            mr_slice.as_mut_slice(),
            Some(desc),
            tag,
            ignore,
            context,
        )
    }
}

pub trait TagRecvEpMrSlice: TagRecvEp {
    fn trecv_mr_slice_from(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv_from(
            mr_slice.as_mut_slice(),
            Some(desc),
            mapped_addr,
            tag,
            ignore,
        )
    }

    fn trecv_mr_slice_from_with_context(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv_from_with_context(
            mr_slice.as_mut_slice(),
            Some(desc),
            mapped_addr,
            tag,
            ignore,
            context,
        )
    }

    fn trecv_mr_slice_from_triggered(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv_from_triggered(
            mr_slice.as_mut_slice(),
            Some(desc),
            mapped_addr,
            tag,
            ignore,
            context,
        )
    }

    fn trecv_mr_slice_from_any(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv_from_any(
            mr_slice.as_mut_slice(),
            Some(desc),
            tag,
            ignore,
        )
    }

    fn trecv_mr_slice_from_any_with_context(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv_from_any_with_context(
            mr_slice.as_mut_slice(),
            Some(desc),
            tag,
            ignore,
            context,
        )
    }

    fn trecv_mr_slice_from_any_triggered(
        &self,
        mr_slice: &mut crate::mr::MemoryRegionSliceMut,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        let desc = mr_slice.desc();
        
        self.trecv_from_any_triggered(
            mr_slice.as_mut_slice(),
            Some(desc),
            tag,
            ignore,
            context,
        )
    }
}

impl<EP: ConnectedTagRecvEp> ConnectedTagRecvEpMrSlice for EP {}
impl<EP: TagRecvEp> TagRecvEpMrSlice for EP {}

impl<EP: TagRecvEpImpl + ConnlessEp> TagRecvEp for EP {
    #[inline]
    fn trecv_from<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, Some(mapped_addr), tag, ignore, None)
    }

    #[inline]
    fn trecv_from_any<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, None)
    }

    #[inline]
    fn trecv_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
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
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecv_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
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
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvv_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        self.trecvv_impl(iov, desc, Some(src_mapped_addr), tag, ignore, None)
    }

    #[inline]
    fn trecvv_from_any(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        self.trecvv_impl(iov, desc, None, tag, ignore, None)
    }

    #[inline]
    fn trecvv_from_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
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
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.trecvv_impl(iov, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvv_from_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
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
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
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
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, None)
    }

    #[inline]
    fn trecv_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecv_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvv(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
    ) -> Result<(), crate::error::Error> {
        self.trecvv_impl(iov, desc, None, tag, ignore, None)
    }

    #[inline]
    fn trecvv_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.trecvv_impl(iov, desc, None, tag, ignore, Some(context.inner_mut()))
    }

    #[inline]
    fn trecvv_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
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

impl<I: TagCap + RecvMod, STATE: EpState, CQ: ?Sized + ReadCq> TagRecvEpImpl
    for RxContextImplBase<I, STATE, CQ>
{
}

impl<I: TagCap + RecvMod, STATE: EpState, CQ: ?Sized + ReadCq> TagRecvEpImpl
    for RxContextBase<I, STATE, CQ>
{
}

impl<E: TagRecvEpImpl> TagRecvEpImpl for EndpointBase<E, Connected> {}
impl<E: TagRecvEpImpl> TagRecvEpImpl for EndpointBase<E, Connectionless> {}

pub(crate) trait TagSendEpImpl: AsTypedFid<EpRawFid> {
    fn tsend_impl<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_tsend(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: Option<&MappedAddress>,
        tag: u64,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_tsendv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
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
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_tsendmsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_tagged_msg,
                options.as_raw(),
            )
        };
        check_error(err)
    }

    fn tsenddata_impl<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_tsenddata(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr() as *const std::ffi::c_void,
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
                self.as_typed_fid_mut().as_raw_typed_fid(),
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
                self.as_typed_fid_mut().as_raw_typed_fid(),
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
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsend_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsend_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_to(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_to_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
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
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
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
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsend_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsend_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tsendv(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsendv_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
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
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        tag: u64,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn tsenddata_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn tinject<T>(&self, buf: &[T], tag: u64) -> Result<(), crate::error::Error>;
    fn tinjectdata<T>(&self, buf: &[T], data: u64, tag: u64) -> Result<(), crate::error::Error>;
}

pub trait ConnectedTagSendEpMrSlice: ConnectedTagSendEp {
    fn tsend_mr_slice(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsend(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            tag,
        )
    }

    fn tsend_mr_slice_with_context(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsend_with_context(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            tag,
            context,
        )
    }

    fn tsend_mr_slice_triggered(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsend_triggered(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            tag,
            context,
        )
    }
    fn tinject_mr_slice(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinject(mr_slice.as_slice(), tag)
    }

    fn tinjectdata_mr_slice(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        data: u64,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinjectdata(mr_slice.as_slice(), data, tag)
    }

    fn tsenddata_mr_slice(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        data: u64,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            data,
            tag,
        )
    }

    fn tsenddata_mr_slice_with_context(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        data: u64,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_with_context(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            data,
            tag,
            context,
        )
    }

    fn tsenddata_mr_slice_triggered(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        data: u64,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_triggered(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            data,
            tag,
            context,
        )
    }
}

pub trait TagSendEpMrSlice: TagSendEp {
    fn tsend_mr_slice_to(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsend_to(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            mapped_addr,
            tag,
        )
    }

    fn tsend_mr_slice_to_with_context(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsend_to_with_context(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            mapped_addr,
            tag,
            context,
        )
    }

    fn tsend_mr_slice_to_triggered(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsend_to_triggered(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            mapped_addr,
            tag,
            context,
        )
    }

    fn tsenddata_mr_slice_to(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_to(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            data,
            mapped_addr,
            tag,
        )
    }

    fn tsenddata_mr_slice_to_with_context(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_to_with_context(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            data,
            mapped_addr,
            tag,
            context,
        )
    }

    fn tsenddata_mr_slice_to_triggered(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_to_triggered(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            data,
            mapped_addr,
            tag,
            context,
        )
    }

    fn tinject_mr_slice_to(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinject_to(mr_slice.as_slice(), mapped_addr, tag)
    }

    fn tinjectdata_mr_slice_to(
        &self,
        mr_slice: &crate::mr::MemoryRegionSlice<'_>,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinjectdata_to(mr_slice.as_slice(), data, mapped_addr, tag)
    }
}

impl<EP: ConnectedTagSendEp> ConnectedTagSendEpMrSlice for EP {}
impl<EP: TagSendEp> TagSendEpMrSlice for EP {}

impl<EP: TagSendEpImpl + ConnlessEp> TagSendEp for EP {
    #[inline]
    fn tsend_to<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, Some(mapped_addr), tag, None)
    }

    #[inline]
    fn tsend_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
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
        desc: Option<MemoryRegionDesc<'_>>,
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
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsendv_impl(iov, desc, Some(dest_mapped_addr), tag, None)
    }

    #[inline]
    fn tsendv_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
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
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
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
        desc: Option<MemoryRegionDesc<'_>>,
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
        desc: Option<MemoryRegionDesc<'_>>,
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
        desc: Option<MemoryRegionDesc<'_>>,
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
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, None, tag, None)
    }

    #[inline]
    fn tsend_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsend_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsendv(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsendv_impl(iov, desc, None, tag, None)
    }

    #[inline]
    fn tsendv_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.tsendv_impl(iov, desc, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsendv_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.tsendv_impl(iov, desc, None, tag, Some(context.inner_mut()))
    }

    #[inline]
    fn tsenddata<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(buf, desc, data, None, tag, None)
    }

    #[inline]
    fn tsenddata_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
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
        desc: Option<MemoryRegionDesc<'_>>,
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

impl<I: TagCap + SendMod, STATE: EpState, CQ: ?Sized + ReadCq> TagSendEpImpl
    for TxContextImplBase<I, STATE, CQ>
{
}

impl<I: TagCap + SendMod, STATE: EpState, CQ: ?Sized + ReadCq> TagSendEpImpl
    for TxContextBase<I, STATE, CQ>
{
}

impl<E: TagSendEpImpl> TagSendEpImpl for EndpointBase<E, Connected> {}
impl<E: TagSendEpImpl> TagSendEpImpl for EndpointBase<E, Connectionless> {}
