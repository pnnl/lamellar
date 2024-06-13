use crate::{FI_ADDR_UNSPEC, cq::{AsyncCtx, SingleCompletionFormat}, ep::{ActiveEndpointImpl, Endpoint}, infocapsoptions::{MsgCap, SendMod, RecvMod}, mr::DataDescriptor, utils::check_error};

impl<E: MsgCap + RecvMod> Endpoint<E> {

    async fn recv_async_imp<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            return crate::cq::AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        }
        
        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

    pub async fn recv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.recv_async_imp(buf, desc, Some(mapped_addr), None).await
    }

    pub async fn recv_connected_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.recv_async_imp(buf, desc, None, None).await
    }
    
    pub async fn recv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.recv_async_imp(buf, desc, Some(mapped_addr), Some((context as *mut T0).cast())).await
    }
 
    pub async fn recv_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.recv_async_imp(buf, desc, None, Some((context as *mut T0).cast())).await
    }

	async fn recvv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&crate::MappedAddress>, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            return crate::cq::AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        }
        
        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

	pub async fn recvv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.recvv_async_impl(iov, desc, Some(mapped_addr), None).await
    }

	pub async fn recvv_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor]) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.recvv_async_impl(iov, desc, None, None).await
    }
    
	pub async fn recvv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress,  context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.recvv_async_impl(iov, desc, Some(mapped_addr), Some((context as *mut T0).cast())).await
    }
    
	pub async fn recvv_connected_with_contex_asynct<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.recvv_async_impl(iov, desc, None, Some((context as *mut T0).cast())).await
    }

//     pub fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }
//         check_error(err)
//     }
}

impl<E: MsgCap + SendMod> Endpoint<E> {

	async fn sendv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&crate::MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), raw_addr, (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            return crate::cq::AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        }
        
        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

	pub async fn sendv_async<'a, T>(&self, iov: & [crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<SingleCompletionFormat, crate::error::Error> { 
	    self.sendv_async_impl(iov, desc, Some(mapped_addr), None).await 
    }

	pub async fn sendv_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor]) -> Result<SingleCompletionFormat, crate::error::Error> { 
	    self.sendv_async_impl(iov, desc, None, None).await 
    }
    
	pub async fn sendv_with_context_async<'a, T,T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> { // [TODO]
	    self.sendv_async_impl(iov, desc, Some(mapped_addr), Some((context as *mut T0).cast())).await 
    }
    
	pub async fn sendv_connected_with_context_async<'a, T,T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> { // [TODO]
	    self.sendv_async_impl(iov, desc, None, Some((context as *mut T0).cast())).await 
    }

    async fn send_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), raw_addr, (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            return crate::cq::AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        }
        
        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

    pub async fn send_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.send_async_impl(buf, desc, Some(mapped_addr), None).await
    }

    pub async fn send_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.send_async_impl(buf, desc, None, None).await
    }

    pub async fn send_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.send_async_impl(buf, desc, Some(mapped_addr), Some((context as *mut T0).cast())).await
    }

    pub async fn send_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.send_async_impl(buf, desc, None, Some((context as *mut T0).cast())).await
    }

    //// [TODO]
    // pub async fn sendmsg_async(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
    //     let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
    //     if err == 0 {
    //         let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
    //         let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
    //         crate::cq::AsyncTransferCq{cq}.await?;
    //     }

    //     check_error(err)
    // }

    async fn senddata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&crate::MappedAddress>, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            return crate::cq::AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        }
        
        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }


    pub async fn senddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, Some(mapped_addr), None).await
    }

    pub async fn senddata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, None, None).await
    }

    pub async fn senddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, Some(mapped_addr), Some((context as *mut T0).cast())).await
    }

    pub async fn senddata_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, None, Some((context as *mut T0).cast())).await
    }

    pub async fn inject_connected_async<T>(&self, buf: &[T]) -> Result<(), crate::error::Error> { // Inject does not generate completions
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC) };

        check_error(err)
    }

    pub async fn inject_async<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> { // Inject does not generate completions
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr()) };
        // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();

        check_error(err)
    }

    pub async fn injectdata_async<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> { // Inject does not generate completions
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr()) };
        // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();

        check_error(err)
    }

    pub async fn injectdata_connected_async<T>(&self, buf: &[T], data: u64) -> Result<(), crate::error::Error> { // Inject does not generate completions
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC) };
        // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();

        check_error(err)
    }
}

// impl TransmitContext {
    
// 	pub fn sendv<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

//         check_error(err)
//     }
    
// 	pub fn sendv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };

//         check_error(err)
//     }

//     pub fn send<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub fn send_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub fn sendmsg(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
    
//         check_error(err)
//     }


//     pub fn senddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub fn senddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub fn inject<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr()) };
    
//         check_error(err)
//     }

//     pub fn injectdata<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr()) };
    
//         check_error(err)
//     }
// }

// impl ReceiveContext {

//     pub fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

//         check_error(err)
//     }

//     pub fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast() ) };
    
//         check_error(err)
//     }
    
// 	pub fn recvv<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

//         check_error(err)
//     }
    
// 	pub fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };

//         check_error(err)
//     }
    
//     pub fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        
//         check_error(err)
//     }
// }