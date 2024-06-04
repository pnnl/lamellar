use std::rc::Rc;

use async_io::Async;
use futures_io::AsyncRead;

use crate::FI_ADDR_UNSPEC;
use crate::cq::AsyncCompletionQueueImpl;
use crate::cq::AsyncCtx;
use crate::cq::CompletionQueueImpl;
use crate::enums::RecvMsgOptions;
use crate::enums::SendMsgOptions;
use crate::ep::ActiveEndpointImpl;
use crate::ep::Endpoint;
use crate::error::Error;
use crate::infocapsoptions::MsgCap;
use crate::infocapsoptions::RecvMod;
use crate::infocapsoptions::SendMod;
use crate::mr::DataDescriptor;
use crate::utils::check_error;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;





// impl<E: MsgCap + RecvMod> Endpoint<E> {

//     pub fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }

//         check_error(err)
//     }

//     pub fn recv_connected<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, std::ptr::null_mut()) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }
//         check_error(err)
//     }
    
//     pub fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast() ) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }

//         check_error(err)
//     }
    
//     pub fn recv_connected_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, (context as *mut T0).cast() ) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }
        
//         check_error(err)
//     }
    
//     pub async fn recv_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, (context as *mut T0).cast() ) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//             let cq = self.inner.rx_cq.borrow().as_ref().unwrap().clone(); 
//             LibfabricAsyncTransferCq{req, buf, cq}.await;
//         }
        
//         check_error(err)
//     }

// 	pub fn recvv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }
//         check_error(err)
//     }

// 	pub fn recvv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, std::ptr::null_mut()) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }
//         check_error(err)
//     }
    
// 	pub fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress,  context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }
//         check_error(err)
//     }
    
// 	pub fn recvv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, (context as *mut T0).cast()) };
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }
//         check_error(err)
//     }

//     pub fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        
//         if err == 0 {
//             let req = self.inner.rx_cq.borrow().as_ref().unwrap().request();
//         }
//         check_error(err)
//     }
// }

impl<E: MsgCap + SendMod> Endpoint<E> {

	pub async fn sendv_async<'a, T>(&self, iov: & [crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> { 
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }

	pub async fn sendv_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> { 
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
	pub async fn sendv_with_context_async<'a, T,T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
	pub async fn sendv_connected_with_context_async<'a, T,T0>(&self, iov: &[crate::iovec::IoVec<'a, T>], desc: &mut [impl DataDescriptor], context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }

    pub async fn send_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        }

        check_error(err)
    }

    pub async fn send_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        }
    
        check_error(err)
    }

    pub async fn send_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx: Some((context as *mut T0).cast())};
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
            // crate::cq::AsyncTransferCq{req, cq, ctx: &mut async_ctx as *mut AsyncCtx as usize}.await?;
        }
    
        check_error(err)
    }

    pub async fn send_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        }
    
        check_error(err)
    }

    pub async fn sendmsg_async(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        }

        check_error(err)
    }


    pub async fn senddata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        }

        check_error(err)
    }

    pub async fn senddata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        }

        check_error(err)
    }

    pub async fn senddata_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        }

        check_error(err)
    }

    pub async fn senddata_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            crate::cq::AsyncTransferCq{req, cq}.await?;
        }

        check_error(err)
    }

    pub async fn inject_connected_async<T>(&self, buf: &[T]) -> Result<(), crate::error::Error> { // Inject does not generate completions
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC) };
        // let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();

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