use std::rc::Rc;

use async_io::Async;

use crate::FI_ADDR_UNSPEC;
use crate::cq::AsyncCompletionQueueImpl;
use crate::cq::AsyncTransferCq;
use crate::cq::CompletionQueueImpl;
use crate::enums::TaggedRecvMsgOptions;
use crate::enums::TaggedSendMsgOptions;
use crate::ep::ActiveEndpointImpl;
use crate::ep::Endpoint;
use crate::infocapsoptions::RecvMod;
use crate::infocapsoptions::SendMod;
use crate::infocapsoptions::TagCap;
use crate::mr::DataDescriptor;
use crate::utils::check_error;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;

// struct LibfabricAsyncTransferCq{
//     req: usize,
//     // buf: &'a [T],
//     cq: Rc<AsyncCompletionQueueImpl>,
// }

// impl async_std::future::Future for LibfabricAsyncTransferCq {
//     type Output=();

//     fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        
//         if  self.cq.completions() >= self.req {
//             std::task::Poll::Ready(())
//         }
//         else {
//             match self.cq.read(1) {
//                 Ok(_) => {
                    
//                     if  self.cq.completions() >= self.req {
//                         std::task::Poll::Ready(())
//                     }
//                     else {
//                         cx.waker().wake_by_ref(); 
//                         std::task::Poll::Pending
//                     }
//                 }
//                 Err(ref err) => {
//                     if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
//                         panic!("Could not read cq");
//                     }
//                     else {
//                         cx.waker().wake_by_ref(); 
//                         std::task::Poll::Pending
//                     }
//                 }
                
//             }
//         }
//     }
// }


// impl<E: TagCap + RecvMod> Endpoint<E> {

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

impl<E: TagCap + SendMod> Endpoint<E> {

    pub async fn tsend_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tsend_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), tag, FI_ADDR_UNSPEC, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tsend_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };
        if err == 0 {
            println!("tsend_with_context_async ");
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tsend_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }
    
	pub async fn tsendv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &crate::MappedAddress, tag:u64) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }
    
	pub async fn tsendv_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

	pub async fn tsendv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &crate::MappedAddress, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

	pub async fn tsendv_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tsendmsg_async(&self, msg: &crate::msg::MsgTagged, options: TaggedSendMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tsenddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), tag, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tsenddata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, tag, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tsenddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), tag, (context as *mut T0).cast()) };
        println!("tsenddata_with_context_async");
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tsenddata_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, tag, (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 

        check_error(err)
    }

    pub async fn tinject_async<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddress, tag:u64 ) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr(), tag) };
    
        check_error(err)
    }

    pub async fn tinject_connected_async<T>(&self, buf: &[T], tag:u64 ) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC, tag) };
    
        check_error(err)
    }

    pub async fn tinjectdata_async<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddress, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr(), tag) };
    
        check_error(err)
    }

    pub async fn tinjectdata_connected_async<T>(&self, buf: &[T], data: u64, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, tag) };
    
        check_error(err)
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
