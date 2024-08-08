use crate::async_::ep::{AsyncRxEp, AsyncTxEp};
use crate::async_::xcontext::{TransmitContext, TransmitContextImpl, ReceiveContext, ReceiveContextImpl};
use crate::comm::tagged::{TagRecvEpImpl, TagSendEp, TagSendEpImpl};
use crate::ep::EndpointImplBase;
use crate::infocapsoptions::{RecvMod, TagCap, SendMod};
use crate::{cq::SingleCompletion, mr::DataDescriptor, async_::{AsyncCtx, cq::AsyncReadCq, eq::AsyncReadEq}, enums::{TaggedSendMsgOptions, TaggedRecvMsgOptions}, MappedAddress, ep::EndpointBase};

pub(crate) trait AsyncTagRecvEpImpl: AsyncRxEp + TagRecvEpImpl {
    async fn trecv_async_impl<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, tag: u64, ignore:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.trecv_impl(buf, desc, mapped_addr, tag, ignore, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_rx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

	async fn trecvv_async_impl<'a>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddress>, tag: u64, ignore:u64, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.trecvv_impl(iov, desc, src_mapped_addr, tag, ignore, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_rx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async fn trecvmsg_async_impl<'a>(&self, msg: &mut crate::msg::MsgTaggedMut<'a>, options: TaggedRecvMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        let real_user_ctx = msg.c_msg_tagged.context;
        let mut async_ctx = AsyncCtx{user_ctx: 
            if real_user_ctx.is_null() {
                None
            }
            else {
                Some(real_user_ctx)
            }
        };

        msg.c_msg_tagged.context = (&mut async_ctx as *mut AsyncCtx).cast();

        if let Err(err) = self.trecvmsg_impl(msg, options) {
            msg.c_msg_tagged.context = real_user_ctx;
            return Err(err);
        }        

        let cq = self.retrieve_rx_cq();
        let res =  cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg_tagged.context = real_user_ctx;
        res
    }
}

pub trait AsyncTagRecvEp {
    fn trecv_from_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag: u64, ignore:u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn trecv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn trecv_from_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag: u64, ignore:u64, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ; 
    fn trecv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn trecvv_from_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddress, tag: u64, ignore:u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn trecvv_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn trecvv_from_with_context_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddress, tag: u64, ignore:u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn trecvv_with_context_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn trecvmsg_async<'a>(&self, msg: &mut crate::msg::MsgTaggedMut<'a>, options: TaggedRecvMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}

// impl<E: TagCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> EndpointBase<E> {
// impl<E:, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> EndpointBase<E> {
impl<EP: TagCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> AsyncTagRecvEpImpl for EndpointImplBase<EP, EQ, CQ> {}

impl<E: AsyncTagRecvEpImpl> AsyncTagRecvEpImpl for EndpointBase<E> {}

impl<EP: AsyncTagRecvEpImpl> AsyncTagRecvEp for EP {
    #[inline]
    async fn trecv_from_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag: u64, ignore:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, Some(mapped_addr), tag, ignore, None).await
    }

    #[inline]
    async fn trecv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, None, tag, ignore, None).await
    }
    
    #[inline]
    async fn trecv_from_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag: u64, ignore:u64, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, Some(mapped_addr), tag, ignore, Some((context as *mut T0).cast())).await
    }
    
    #[inline]
    async fn trecv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, None, tag, ignore, Some((context as *mut T0).cast())).await
    }

	#[inline]
    async fn trecvv_from_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddress, tag: u64, ignore:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, Some(src_mapped_addr), tag, ignore, None).await
    }

	#[inline]
    async fn trecvv_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, None, tag, ignore, None).await
    }

	#[inline]
    async fn trecvv_from_with_context_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddress, tag: u64, ignore:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
        self.trecvv_async_impl(iov, desc, Some(src_mapped_addr), tag, ignore, Some((context as *mut T0).cast())).await
    }

	#[inline]
    async fn trecvv_with_context_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, None, tag, ignore, Some((context as *mut T0).cast())).await
    }

    #[inline]
    async fn trecvmsg_async<'a>(&self, msg: &mut crate::msg::MsgTaggedMut<'a>, options: TaggedRecvMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvmsg_async_impl(msg, options).await
    }
}

pub(crate) trait AsyncTagSendEpImpl: AsyncTxEp + TagSendEpImpl {
    async fn tsend_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, tag:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.tsend_impl(buf, desc, mapped_addr, tag, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
    
	async fn tsendv_async_impl<'a>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, tag:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.tsendv_impl(iov, desc, dest_mapped_addr, tag, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async fn tsendmsg_async_impl<'a>(&self, msg: &mut crate::msg::MsgTagged<'a>, options: TaggedSendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        let real_user_ctx = msg.c_msg_tagged.context;
        let mut async_ctx = AsyncCtx{user_ctx: 
            if real_user_ctx.is_null() {
                None
            }
            else {
                Some(real_user_ctx)
            }
        };
        msg.c_msg_tagged.context = (&mut async_ctx as *mut AsyncCtx).cast();
        if let Err(err) = self.tsendmsg_impl(msg, options) {
            msg.c_msg_tagged.context = real_user_ctx;
            return Err(err);
        }
        let cq = self.retrieve_tx_cq();
        let res =  cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg_tagged.context = real_user_ctx;
        res
    }

    async fn tsenddata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&MappedAddress>, tag: u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.tsenddata_impl(buf, desc, data, mapped_addr, tag, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
}

pub trait AsyncTagSendEp: TagSendEp {
    fn tsend_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag:u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsend_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsend_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag:u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsend_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsendv_to_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddress, tag:u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;    
    fn tsendv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], tag:u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn tsendv_to_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddress, tag:u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsendv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsendmsg_async<'a>(&self, msg: &mut crate::msg::MsgTagged<'a>, options: TaggedSendMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsenddata_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, tag: u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsenddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsenddata_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, tag: u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsenddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

// impl<E: TagCap + SendMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> EndpointBase<E> {
impl<EP: TagCap + SendMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> AsyncTagSendEpImpl for EndpointImplBase<EP, EQ, CQ> {}

impl<E: AsyncTagSendEpImpl> AsyncTagSendEpImpl for EndpointBase<E> {}

impl<EP: AsyncTagSendEpImpl + TagSendEpImpl> AsyncTagSendEp for EP {
    #[inline]
    async fn tsend_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, Some(mapped_addr), tag, None).await
    }

    #[inline]
    async fn tsend_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, None, tag, None).await
    }

    #[inline]
    async fn tsend_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, Some(mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

    #[inline]
    async fn tsend_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, None, tag, Some((context as *mut T0).cast())).await
    }

	#[inline]
    async fn tsendv_to_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddress, tag:u64) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, Some(dest_mapped_addr), tag, None).await
    }
    
	#[inline]
    async fn tsendv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], tag:u64) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, None, tag, None).await
    } 

    #[inline]
    async fn tsendv_to_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddress, tag:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, Some(dest_mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

	#[inline]
    async fn tsendv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsendv_async_impl(iov, desc, None, tag, Some((context as *mut T0).cast())).await
    }

    #[inline]
    async fn tsendmsg_async<'a>(&self, msg: &mut crate::msg::MsgTagged<'a>, options: TaggedSendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.tsendmsg_async_impl(msg, options).await
    }

    #[inline]
    async fn tsenddata_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, tag: u64) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, Some(mapped_addr), tag, None).await
    }

    #[inline]
    async fn tsenddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, None, tag, None).await
    }

    #[inline]
    async fn tsenddata_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, tag: u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, Some(mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

    #[inline]
    async fn tsenddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, None, tag, Some((context as *mut T0).cast())).await
    }
}


impl AsyncTagSendEpImpl for TransmitContext {}
impl AsyncTagSendEpImpl for TransmitContextImpl {}
impl AsyncTagRecvEpImpl for ReceiveContext {}
impl AsyncTagRecvEpImpl for ReceiveContextImpl {}