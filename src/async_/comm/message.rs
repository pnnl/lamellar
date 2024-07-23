use crate::comm::message::{RecvEpImpl, SendEpImpl};
use crate::ep::EndpointImplBase;
use crate::infocapsoptions::{MsgCap, RecvMod, SendMod};
use crate::{cq::SingleCompletion, mr::DataDescriptor, async_::{AsyncCtx, cq::AsyncReadCq, eq::AsyncReadEq}, enums::{SendMsgOptions, RecvMsgOptions}, MappedAddress, ep::EndpointBase};

pub(crate) trait AsyncRecvEpImpl: RecvEpImpl {
    async fn recv_async_imp<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
	async fn recvv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    async fn recvmsg_async_impl(&self, msg: &mut crate::msg::Msg, options: RecvMsgOptions) -> Result<SingleCompletion, crate::error::Error> ;
}


pub trait AsyncRecvEp {
    fn recv_from_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recv_from_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recvv_from_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn recvv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor]) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn recvv_from_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress,  context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn recvv_with_contex_asynct<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn recvmsg_async(&self, msg: &mut crate::msg::Msg, options: RecvMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}

impl<E: MsgCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> AsyncRecvEpImpl for  EndpointBase<E, EQ, CQ> {
    async fn recv_async_imp<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.recv_async_imp(buf, desc, mapped_addr, user_ctx).await
    }

    #[inline]
	async fn recvv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.recvv_async_impl(iov, desc, mapped_addr, user_ctx).await
    }

    async fn recvmsg_async_impl(&self, msg: &mut crate::msg::Msg, options: RecvMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.recvmsg_async_impl(msg, options).await
    }
}

impl<EP: MsgCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> AsyncRecvEpImpl for  EndpointImplBase<EP, EQ, CQ> {
    async fn recv_async_imp<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.recv_impl(buf, desc, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.rx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
	async fn recvv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.recvv_impl(iov, desc, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.rx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async fn recvmsg_async_impl(&self, msg: &mut crate::msg::Msg, options: RecvMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        let real_user_ctx = msg.c_msg.context;
        let mut async_ctx = AsyncCtx{user_ctx: 
            if real_user_ctx.is_null() {
                None
            }
            else {
                Some(real_user_ctx)
            }
        };
        msg.c_msg.context = (&mut async_ctx as *mut AsyncCtx).cast();
        
        if let Err(err) = self.recvmsg_impl(msg, options) {
            msg.c_msg.context = real_user_ctx;
            return Err(err);
        }
        let cq = self.rx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        let res = cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg.context = real_user_ctx;
        res
    }
}

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
        
    fn recvv_from_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, Some(mapped_addr), None)
    }

    fn recvv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor]) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, None, None)
    }
    
    fn recvv_from_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress,  context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, Some(mapped_addr), Some((context as *mut T0).cast()))
    }
    
    fn recvv_with_contex_asynct<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, None, Some((context as *mut T0).cast()))
    }

    fn recvmsg_async(&self, msg: &mut crate::msg::Msg, options: RecvMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvmsg_async_impl(msg, options)
    }
}

pub(crate) trait AsyncSendEpImpl: SendEpImpl {
	async fn sendv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    async fn send_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    async fn sendmsg_async_impl(&self, msg: &mut crate::msg::Msg, options: SendMsgOptions) -> Result<SingleCompletion, crate::error::Error> ;
    async fn senddata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
}

pub trait AsyncSendEp {
    fn sendv_to_async<'a, T>(&self, iov: & [crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn sendv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor]) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn sendv_to_with_context_async<'a, T,T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
	fn sendv_with_context_async<'a, T,T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn send_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn send_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn send_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn send_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn sendmsg_async(&self, msg: &mut crate::msg::Msg, options: SendMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn senddata_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn senddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn senddata_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn senddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}

// impl<E, EQ: ?Sized +  AsyncReadEq,  CQ: AsyncReadCq + ? Sized> EndpointBase<E, EQ, CQ> {
impl<EP: MsgCap + SendMod, EQ: ?Sized +  AsyncReadEq,  CQ: AsyncReadCq + ? Sized> AsyncSendEpImpl for EndpointImplBase<EP, EQ, CQ> {

    #[inline]
	async fn sendv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.sendv_impl(iov, desc, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
    async fn send_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.send_impl(buf, desc, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
    async fn sendmsg_async_impl(&self, msg: &mut crate::msg::Msg, options: SendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        let real_user_ctx = msg.c_msg.context;
        let mut async_ctx = 
            AsyncCtx {
                user_ctx: if real_user_ctx.is_null() {
                    None
                }
                else {
                    Some(real_user_ctx)
                }
            };
        
        msg.c_msg.context = (&mut async_ctx as *mut AsyncCtx).cast();
        if let Err(err) = self.sendmsg_impl(msg, options) {
            msg.c_msg.context = real_user_ctx;
            return Err(err);
        }
        
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        let res = cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg.context = real_user_ctx;
        res
    }

    #[inline]
    async fn senddata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.senddata_impl(buf, desc, data, mapped_addr, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
}

impl<E: MsgCap + SendMod, EQ: ?Sized +  AsyncReadEq,  CQ: AsyncReadCq + ? Sized> AsyncSendEpImpl for EndpointBase<E, EQ, CQ> {
    #[inline]
	async fn sendv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        self.inner.sendv_async_impl(iov, desc, mapped_addr, user_ctx).await
    }

    #[inline]
    async fn send_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.send_async_impl(buf, desc, mapped_addr, user_ctx).await
    }

    #[inline]
    async fn sendmsg_async_impl(&self, msg: &mut crate::msg::Msg, options: SendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.sendmsg_async_impl(msg, options).await
    }

    #[inline]
    async fn senddata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.senddata_async_impl(buf, desc, data, mapped_addr, user_ctx).await
    }
}

impl<EP: AsyncSendEpImpl> AsyncSendEp for EP {
    async fn sendv_to_async<'a, T>(&self, iov: & [crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> Result<SingleCompletion, crate::error::Error> { 
	    self.sendv_async_impl(iov, desc, Some(mapped_addr), None).await 
    }

	async fn sendv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor]) -> Result<SingleCompletion, crate::error::Error> { 
	    self.sendv_async_impl(iov, desc, None, None).await 
    }
    
	async fn sendv_to_with_context_async<'a, T,T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
	    self.sendv_async_impl(iov, desc, Some(mapped_addr), Some((context as *mut T0).cast())).await 
    }
    
	async fn sendv_with_context_async<'a, T,T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], context : &mut T0) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
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

    async fn sendmsg_async(&self, msg: &mut crate::msg::Msg, options: SendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.sendmsg_async_impl(msg, options).await
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

// impl TransmitContext {
    
// 	pub fn sendv<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

//         check_error(err)
//     }
    
// 	pub fn sendv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };

//         check_error(err)
//     }

//     pub fn send<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_send(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub fn send_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_send(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub fn sendmsg(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.as_raw_typed_fid(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
    
//         check_error(err)
//     }


//     pub fn senddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub fn senddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub fn inject<T>(&self, buf: &[T], mapped_addr: &MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr()) };
    
//         check_error(err)
//     }

//     pub fn injectdata<T>(&self, buf: &[T], data: u64, mapped_addr: &MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr()) };
    
//         check_error(err)
//     }
// }

// impl ReceiveContext {

//     pub fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

//         check_error(err)
//     }

//     pub fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast() ) };
    
//         check_error(err)
//     }
    
// 	pub fn recvv<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

//         check_error(err)
//     }
    
// 	pub fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };

//         check_error(err)
//     }
    
//     pub fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.as_raw_typed_fid(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        
//         check_error(err)
//     }
// }