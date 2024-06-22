

use crate::{FI_ADDR_UNSPEC, ep::{ActiveEndpointImpl, Endpoint}, infocapsoptions::{SendMod, TagCap, RecvMod}, utils::check_error, cq::{AsyncCtx, AsyncTransferCq, SingleCompletionFormat}, mr::DataDescriptor};

impl<E: TagCap + RecvMod> Endpoint<E> {

    async fn trecv_async_impl<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddress>, tag: u64, ignore:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), raw_addr, tag, ignore, (&mut async_ctx as *mut AsyncCtx).cast()) };


        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").request();
            let cq = self.inner.rx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
            return AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

    pub async fn trecv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag: u64, ignore:u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.trecv_async_impl(buf, desc, Some(mapped_addr), tag, ignore, None).await
    }

    pub async fn trecv_connected_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.trecv_async_impl(buf, desc, None, tag, ignore, None).await
    }
    
    pub async fn trecv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag: u64, ignore:u64, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.trecv_async_impl(buf, desc, Some(mapped_addr), tag, ignore, Some((context as *mut T0).cast())).await
    }
    
    pub async fn trecv_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.trecv_async_impl(buf, desc, None, tag, ignore, Some((context as *mut T0).cast())).await
    }

	async fn trecvv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&crate::MappedAddress>, tag: u64, ignore:u64, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = src_mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(),raw_addr, tag, ignore, (&mut async_ctx as *mut AsyncCtx).cast()) };

        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").request();
            let cq = self.inner.rx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
            return AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }


	pub async fn trecvv_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &crate::MappedAddress, tag: u64, ignore:u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, Some(src_mapped_addr), tag, ignore, None).await
    }

	pub async fn trecvv_connected_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, None, tag, ignore, None).await
    }

	pub async fn trecvv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &crate::MappedAddress, tag: u64, ignore:u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> { //[TODO]
        self.trecvv_async_impl(iov, desc, Some(src_mapped_addr), tag, ignore, Some((context as *mut T0).cast())).await
    }

	pub async fn trecvv_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, None, tag, ignore, Some((context as *mut T0).cast())).await
    }

//     pub async fn trecvmsg_async(&self, msg: &crate::msg::MsgTagged, options: TaggedRecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
    
//         check_error(err)
//     }
}

impl<E: TagCap + SendMod> Endpoint<E> {

    async fn tsend_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddress>, tag:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), raw_addr, tag, (&mut async_ctx as *mut AsyncCtx).cast()) };

        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").request();
            let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
            return AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }


    pub async fn tsend_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag:u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsend_async_impl(buf, desc, Some(mapped_addr), tag, None).await
    }

    pub async fn tsend_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsend_async_impl(buf, desc, None, tag, None).await
    }

    pub async fn tsend_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag:u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsend_async_impl(buf, desc, Some(mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

    pub async fn tsend_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsend_async_impl(buf, desc, None, tag, Some((context as *mut T0).cast())).await
    }
    
	pub async fn tsendv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&crate::MappedAddress>, tag:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = dest_mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, tag, (&mut async_ctx as *mut AsyncCtx).cast()) };

        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").request();
            let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
            return AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }
    
	pub async fn tsendv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &crate::MappedAddress, tag:u64) -> Result<SingleCompletionFormat, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, Some(dest_mapped_addr), tag, None).await
    }
    
	pub async fn tsendv_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64) -> Result<SingleCompletionFormat, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, None, tag, None).await
    } 

    pub async fn tsendv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &crate::MappedAddress, tag:u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, Some(dest_mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

	pub async fn tsendv_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsendv_async_impl(iov, desc, None, tag, Some((context as *mut T0).cast())).await
    }

    //// [TODO]
    // pub async fn tsendmsg_async(&self, msg: &crate::msg::MsgTagged, options: TaggedSendMsgOptions) -> Result<(), crate::error::Error> {
    //     let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
    //     if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").request();
    //         let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
    //         AsyncTransferCq{cq}.await?;
    //     } 

    //     check_error(err)
    // }

    async fn tsenddata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&crate::MappedAddress>, tag: u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };
        
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, tag, (&mut async_ctx as *mut AsyncCtx).cast()) };


        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").request();
            let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
            return AsyncTransferCq{cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }


    pub async fn tsenddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, tag: u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, Some(mapped_addr), tag, None).await
    }

    pub async fn tsenddata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, None, tag, None).await
    }

    pub async fn tsenddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, tag: u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, Some(mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

    pub async fn tsenddata_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, None, tag, Some((context as *mut T0).cast())).await
    }
}


// impl TransmitContext {

//     pub async fn tsend_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag:u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub async fn tsend_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), tag, FI_ADDR_UNSPEC, std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub async fn tsend_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub async fn tsend_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };
    
//         check_error(err)
//     }
    
// 	pub async fn 'a, tsendv_async<T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &crate::MappedAddress, tag:u64) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };

//         check_error(err)
//     }
    
// 	pub async fn 'a, tsendv_connected_async<T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, std::ptr::null_mut()) };

//         check_error(err)
//     }

// 	pub async fn 'a, tsendv_with_context_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &crate::MappedAddress, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };

//         check_error(err)
//     }

// 	pub async fn 'a, tsendv_connected_with_context_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };

//         check_error(err)
//     }

//     pub async fn tsendmsg_async(&self, msg: &crate::msg::MsgTagged, options: TaggedSendMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
    
//         check_error(err)
//     }

//     pub async fn tsenddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub async fn tsenddata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, tag, std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub async fn tsenddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub async fn tsenddata_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub async fn tinject_async<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddress, tag:u64 ) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr(), tag) };
    
//         check_error(err)
//     }

//     pub async fn tinject_connected_async<T>(&self, buf: &[T], tag:u64 ) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC, tag) };
    
//         check_error(err)
//     }

//     pub async fn tinjectdata_async<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddress, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr(), tag) };
    
//         check_error(err)
//     }

//     pub async fn tinjectdata_connected_async<T>(&self, buf: &[T], data: u64, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, tag) };
    
//         check_error(err)
//     }
// }

// impl ReceiveContext {

//     pub async fn trecv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, ignore, std::ptr::null_mut()) };
        
//         check_error(err)
//     }

//     pub async fn trecv_connected_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, tag, ignore, std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async fn trecv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag: u64, ignore:u64, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, ignore, (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async fn trecv_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, tag, ignore, (context as *mut T0).cast()) };
        
//         check_error(err)
//     }

// 	pub async fn 'a, trecvv_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &crate::MappedAddress, tag: u64, ignore:u64) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), src_mapped_addr.raw_addr(), tag, ignore, std::ptr::null_mut()) };

//         check_error(err)   
//     }

// 	pub async fn 'a, trecvv_connected_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, ignore, std::ptr::null_mut()) };

//         check_error(err)   
//     }

// 	pub async fn 'a, trecvv_with_context_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &crate::MappedAddress, tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), src_mapped_addr.raw_addr(), tag, ignore, (context as *mut T0).cast()) };

//         check_error(err)   
//     }

// 	pub async fn 'a, trecvv_connected_with_context_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, ignore, (context as *mut T0).cast()) };

//         check_error(err)   
//     }

//     pub async fn trecvmsg_async(&self, msg: &crate::msg::MsgTagged, options: TaggedRecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
    
//         check_error(err)   
//     }
// }
