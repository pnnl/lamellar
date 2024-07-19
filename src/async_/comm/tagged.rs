

use crate::{FI_ADDR_UNSPEC, infocapsoptions::{SendMod, TagCap, RecvMod}, cq::SingleCompletion, mr::DataDescriptor, async_::{AsyncCtx, cq::AsyncReadCq, eq::AsyncReadEq}, fid::AsRawTypedFid, enums::{TaggedSendMsgOptions, TaggedRecvMsgOptions}, MappedAddress, ep::EndpointBase};

impl<E: TagCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> EndpointBase<E, EQ, CQ> {
    
    #[inline]
    async fn trecv_async_impl<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, tag: u64, ignore:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.trecv_impl(buf, desc, mapped_addr, tag, ignore, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.inner.rx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    pub async fn trecv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag: u64, ignore:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, Some(mapped_addr), tag, ignore, None).await
    }

    pub async fn trecv_connected_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, None, tag, ignore, None).await
    }
    
    pub async fn trecv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag: u64, ignore:u64, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, Some(mapped_addr), tag, ignore, Some((context as *mut T0).cast())).await
    }
    
    pub async fn trecv_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, None, tag, ignore, Some((context as *mut T0).cast())).await
    }

    #[inline]
	async fn trecvv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddress>, tag: u64, ignore:u64, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.trecvv_impl(iov, desc, src_mapped_addr, tag, ignore, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.inner.rx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }


	pub async fn trecvv_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddress, tag: u64, ignore:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, Some(src_mapped_addr), tag, ignore, None).await
    }

	pub async fn trecvv_connected_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, None, tag, ignore, None).await
    }

	pub async fn trecvv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddress, tag: u64, ignore:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
        self.trecvv_async_impl(iov, desc, Some(src_mapped_addr), tag, ignore, Some((context as *mut T0).cast())).await
    }

	pub async fn trecvv_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, None, tag, ignore, Some((context as *mut T0).cast())).await
    }

    pub async fn trecvmsg_async(&self, msg: &mut crate::msg::MsgTagged, options: TaggedRecvMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
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

        if let Err(err) = self.trecvmsg(msg, options) {
            msg.c_msg_tagged.context = real_user_ctx;
            return Err(err);
        }        

        let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        let res =  cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg_tagged.context = real_user_ctx;
        res
    }
}

impl<E: TagCap + SendMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> EndpointBase<E, EQ, CQ> {

    #[inline]
    async fn tsend_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddress>, tag:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.tsend_impl(buf, desc, mapped_addr, tag, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }


    pub async fn tsend_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, Some(mapped_addr), tag, None).await
    }

    pub async fn tsend_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, None, tag, None).await
    }

    pub async fn tsend_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, Some(mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

    pub async fn tsend_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, None, tag, Some((context as *mut T0).cast())).await
    }
    
	pub async fn tsendv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, tag:u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.tsendv_impl(iov, desc, dest_mapped_addr, tag, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
    
	pub async fn tsendv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddress, tag:u64) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, Some(dest_mapped_addr), tag, None).await
    }
    
	pub async fn tsendv_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, None, tag, None).await
    } 

    pub async fn tsendv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddress, tag:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> { // [TODO]
        self.tsendv_async_impl(iov, desc, Some(dest_mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

	pub async fn tsendv_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsendv_async_impl(iov, desc, None, tag, Some((context as *mut T0).cast())).await
    }

    pub async fn tsendmsg_async(&self, msg: &mut crate::msg::MsgTagged, options: TaggedSendMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
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
        if let Err(err) = self.tsendmsg(msg, options) {
            msg.c_msg_tagged.context = real_user_ctx;
            return Err(err);
        }
        let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        let res =  cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg_tagged.context = real_user_ctx;
        res
    }

    #[inline]
    async fn tsenddata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&MappedAddress>, tag: u64, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.tsenddata_impl(buf, desc, data, mapped_addr, tag, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }


    pub async fn tsenddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, tag: u64) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, Some(mapped_addr), tag, None).await
    }

    pub async fn tsenddata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, None, tag, None).await
    }

    pub async fn tsenddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, tag: u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, Some(mapped_addr), tag, Some((context as *mut T0).cast())).await
    }

    pub async fn tsenddata_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, None, tag, Some((context as *mut T0).cast())).await
    }
}


// impl TransmitContext {

//     pub async fn tsend_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag:u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub async fn tsend_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), tag, FI_ADDR_UNSPEC, std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub async fn tsend_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub async fn tsend_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };
    
//         check_error(err)
//     }
    
// 	pub async fn 'a, tsendv_async<T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddress, tag:u64) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };

//         check_error(err)
//     }
    
// 	pub async fn 'a, tsendv_connected_async<T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, std::ptr::null_mut()) };

//         check_error(err)
//     }

// 	pub async fn 'a, tsendv_with_context_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddress, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };

//         check_error(err)
//     }

// 	pub async fn 'a, tsendv_connected_with_context_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };

//         check_error(err)
//     }

//     pub async fn tsendmsg_async(&self, msg: &crate::msg::MsgTagged, options: TaggedSendMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.as_raw_typed_fid(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
    
//         check_error(err)
//     }

//     pub async fn tsenddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub async fn tsenddata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, tag, std::ptr::null_mut()) };
    
//         check_error(err)
//     }

//     pub async fn tsenddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddress, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub async fn tsenddata_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };
    
//         check_error(err)
//     }

//     pub async fn tinject_async<T>(&self, buf: &[T], mapped_addr: &MappedAddress, tag:u64 ) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr(), tag) };
    
//         check_error(err)
//     }

//     pub async fn tinject_connected_async<T>(&self, buf: &[T], tag:u64 ) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC, tag) };
    
//         check_error(err)
//     }

//     pub async fn tinjectdata_async<T>(&self, buf: &[T], data: u64, mapped_addr: &MappedAddress, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr(), tag) };
    
//         check_error(err)
//     }

//     pub async fn tinjectdata_connected_async<T>(&self, buf: &[T], data: u64, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, tag) };
    
//         check_error(err)
//     }
// }

// impl ReceiveContext {

//     pub async fn trecv_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.as_raw_typed_fid(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, ignore, std::ptr::null_mut()) };
        
//         check_error(err)
//     }

//     pub async fn trecv_connected_async<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.as_raw_typed_fid(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, tag, ignore, std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async fn trecv_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddress, tag: u64, ignore:u64, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.as_raw_typed_fid(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, ignore, (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async fn trecv_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.as_raw_typed_fid(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, tag, ignore, (context as *mut T0).cast()) };
        
//         check_error(err)
//     }

// 	pub async fn 'a, trecvv_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddress, tag: u64, ignore:u64) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.as_raw_typed_fid(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), src_mapped_addr.raw_addr(), tag, ignore, std::ptr::null_mut()) };

//         check_error(err)   
//     }

// 	pub async fn 'a, trecvv_connected_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.as_raw_typed_fid(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, ignore, std::ptr::null_mut()) };

//         check_error(err)   
//     }

// 	pub async fn 'a, trecvv_with_context_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddress, tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.as_raw_typed_fid(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), src_mapped_addr.raw_addr(), tag, ignore, (context as *mut T0).cast()) };

//         check_error(err)   
//     }

// 	pub async fn 'a, trecvv_connected_with_context_async<T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.as_raw_typed_fid(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, ignore, (context as *mut T0).cast()) };

//         check_error(err)   
//     }

//     pub async fn trecvmsg_async(&self, msg: &crate::msg::MsgTagged, options: TaggedRecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.as_raw_typed_fid(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
    
//         check_error(err)   
//     }
// }
