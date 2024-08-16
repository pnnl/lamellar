use crate::async_::ep::{AsyncRxEp, AsyncTxEp};
use crate::async_::xcontext::{TransmitContext, ReceiveContext, TransmitContextImpl, ReceiveContextImpl};
use crate::comm::message::{RecvEpImpl, SendEpImpl};
use crate::ep::EndpointImplBase;
use crate::infocapsoptions::{MsgCap, RecvMod, SendMod};
use crate::utils::Either;
use crate::{cq::SingleCompletion, mr::DataDescriptor, async_::{AsyncCtx, cq::AsyncReadCq, eq::AsyncReadEq}, enums::{SendMsgOptions, RecvMsgOptions}, MappedAddress, ep::EndpointBase};

pub(crate) trait AsyncRecvEpImpl: AsyncRxEp + RecvEpImpl {
    async fn recv_async_imp<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.recv_impl(buf, desc, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void))?;
        let cq = self.retrieve_rx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
	async fn recvv_async_impl<'a>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.recvv_impl(iov, desc, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void))?;
        let cq = self.retrieve_rx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async fn recvmsg_async_impl<'a>(&self, mut msg: Either<&mut crate::msg::MsgMut<'a>, &mut crate::msg::MsgConnectedMut<'a>>, options: RecvMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx;
        let real_user_ctx  = {
            let c_msg = match &mut msg {
                Either::Left(msg) => &mut msg.c_msg,
                Either::Right(msg) => &mut msg.c_msg,
            };

            let real_user_ctx = c_msg.context;
            async_ctx = 
                AsyncCtx {
                    user_ctx: if real_user_ctx.is_null() {
                        None
                    }
                    else {
                        Some(real_user_ctx)
                    }
                };
            
            c_msg.context = (&mut async_ctx as *mut AsyncCtx).cast();
            real_user_ctx
        };

        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };
        
        let err  =  self.recvmsg_impl(imm_msg, options);
        let c_msg = match &mut msg {
            Either::Left(msg) => &mut msg.c_msg,
            Either::Right(msg) => &mut msg.c_msg,
        };

        if err.is_err() {
            c_msg.context = real_user_ctx;
            err?
        }

        let cq = self.retrieve_rx_cq();
        let res = cq.wait_for_ctx_async(&mut async_ctx).await;
        c_msg.context = real_user_ctx;
        res
    }
}


pub trait AsyncRecvEp {
    fn recv_from_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recv_from_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recvv_from_async<'a>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn recvv_async<'a>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor]) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn recvv_from_with_context_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress,  context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn recvv_with_contex_asynct<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recvmsg_async(&self, msg: &mut crate::msg::MsgConnectedMut, options: RecvMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recvmsg_from_async(&self, msg: &mut crate::msg::MsgMut, options: RecvMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}

impl<E: AsyncRecvEpImpl> AsyncRecvEpImpl for  EndpointBase<E> {}

impl<EP: MsgCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> AsyncRecvEpImpl for  EndpointImplBase<EP, EQ, CQ> {}

impl<EP: AsyncRecvEpImpl> AsyncRecvEp for EP {
    fn recv_from_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>  {
        self.recv_async_imp(buf, desc, Some(mapped_addr), None)
    }

    fn recv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>  {
        self.recv_async_imp(buf, desc, None, None)
    }

    fn recv_from_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recv_async_imp(buf, desc, Some(mapped_addr), Some((context as *mut T0).cast()))
    }
         
    fn recv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recv_async_imp(buf, desc, None, Some((context as *mut T0).cast()))
    }
        
    fn recvv_from_async<'a>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, Some(mapped_addr), None)
    }

    fn recvv_async<'a>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor]) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, None, None)
    }
    
    fn recvv_from_with_context_async<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress,  context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, Some(mapped_addr), Some((context as *mut T0).cast()))
    }
    
    fn recvv_with_contex_asynct<'a, T0>(&self, iov: &[crate::iovec::IoVecMut<'a>], desc: &mut [impl DataDescriptor], context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, None, Some((context as *mut T0).cast()))
    }

    fn recvmsg_from_async(&self, msg: &mut crate::msg::MsgMut, options: RecvMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvmsg_async_impl(Either::Left(msg), options)
    }

    fn recvmsg_async(&self, msg: &mut crate::msg::MsgConnectedMut, options: RecvMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvmsg_async_impl(Either::Right(msg), options)
    }
}

pub(crate) trait AsyncSendEpImpl: AsyncTxEp + SendEpImpl {
	async fn sendv_async_impl<'a>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.sendv_impl(iov, desc, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async fn send_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.send_impl(buf, desc, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async fn sendmsg_async_impl<'a>(&self, mut msg: Either<&mut crate::msg::Msg<'a>, &mut crate::msg::MsgConnected<'a>>, options: SendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx;
        let real_user_ctx  = {
            let c_msg = match &mut msg {
                Either::Left(msg) => &mut msg.c_msg,
                Either::Right(msg) => &mut msg.c_msg,
            };

            let real_user_ctx = c_msg.context;

            async_ctx = 
            AsyncCtx {
                user_ctx: if real_user_ctx.is_null() {
                    None
                }
                else {
                    Some(real_user_ctx)
                }
            };
            c_msg.context = (&mut async_ctx as *mut AsyncCtx).cast();
            real_user_ctx
        };

        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let err  = self.sendmsg_impl(imm_msg, options);
        let c_msg = match &mut msg {
            Either::Left(msg) => &mut msg.c_msg,
            Either::Right(msg) => &mut msg.c_msg,
        };


        if err.is_err() {
            c_msg.context = real_user_ctx;   
            err? 
        }
        
        let cq = self.retrieve_tx_cq();
        let res = cq.wait_for_ctx_async(&mut async_ctx).await;
        c_msg.context = real_user_ctx;
        res
    }

    async fn senddata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.senddata_impl(buf, desc, data, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
}

pub trait AsyncSendEp {
    fn sendv_to_async<'a>(&self, iov: & [crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn sendv_async<'a>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor]) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn sendv_to_with_context_async<'a,T0>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn sendv_with_context_async<'a,T0>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn send_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn send_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn send_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn send_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn sendmsg_async(&self, msg: &mut crate::msg::MsgConnected, options: SendMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn sendmsg_to_async(&self, msg: &mut crate::msg::Msg, options: SendMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn senddata_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn senddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn senddata_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn senddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}

// impl<E, EQ: ?Sized +  AsyncReadEq,  CQ: AsyncReadCq + ? Sized> EndpointBase<E> {

impl<EP: AsyncSendEpImpl> AsyncSendEp for EP {
    async fn sendv_to_async<'a>(&self, iov: & [crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> Result<SingleCompletion, crate::error::Error> { 
	    self.sendv_async_impl(iov, desc, Some(mapped_addr), None).await 
    }

	async fn sendv_async<'a>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor]) -> Result<SingleCompletion, crate::error::Error> { 
	    self.sendv_async_impl(iov, desc, None, None).await 
    }
    
	async fn sendv_to_with_context_async<'a,T0>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
	    self.sendv_async_impl(iov, desc, Some(mapped_addr), Some((context as *mut T0).cast())).await 
    }
    
	async fn sendv_with_context_async<'a,T0>(&self, iov: &[crate::iovec::IoVec<'a>], desc: &mut [impl DataDescriptor], context : &mut T0) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
	    self.sendv_async_impl(iov, desc, None, Some((context as *mut T0).cast())).await 
    }

    async fn send_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress) -> Result<SingleCompletion, crate::error::Error> {
        self.send_async_impl(buf, desc, Some(mapped_addr), None).await
    }

    async fn send_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> Result<SingleCompletion, crate::error::Error> {
        self.send_async_impl(buf, desc, None, None).await
    }

    async fn send_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.send_async_impl(buf, desc, Some(mapped_addr), Some((context as *mut T0).cast())).await
    }

    async fn send_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.send_async_impl(buf, desc, None, Some((context as *mut T0).cast())).await
    }

    async fn sendmsg_async<'a>(&self, msg: &mut crate::msg::MsgConnected<'a>, options: SendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.sendmsg_async_impl(Either::Right(msg), options).await
    }

    async fn sendmsg_to_async<'a>(&self, msg: &mut crate::msg::Msg<'a>, options: SendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.sendmsg_async_impl(Either::Left(msg), options).await
    }

    async fn senddata_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, Some(mapped_addr), None).await
    }

    async fn senddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, None, None).await
    }

    async fn senddata_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, Some(mapped_addr), Some((context as *mut T0).cast())).await
    }

    async fn senddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, None, Some((context as *mut T0).cast())).await
    }
}

impl<EP: MsgCap + SendMod, EQ: ?Sized +  AsyncReadEq,  CQ: AsyncReadCq + ? Sized> AsyncSendEpImpl for EndpointImplBase<EP, EQ, CQ> {}
impl<E: AsyncSendEpImpl> AsyncSendEpImpl for EndpointBase<E> {}
impl AsyncSendEpImpl for TransmitContext {}
impl AsyncSendEpImpl for TransmitContextImpl {}
impl AsyncRecvEpImpl for ReceiveContext {}
impl AsyncRecvEpImpl for ReceiveContextImpl {}