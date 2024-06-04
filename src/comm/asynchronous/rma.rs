use std::rc::Rc;

use async_io::Async;

use crate::FI_ADDR_UNSPEC;
use crate::cq::AsyncCompletionQueueImpl;
use crate::cq::AsyncTransferCq;
use crate::cq::CompletionQueueImpl;
use crate::enums::ReadMsgOptions;
use crate::enums::WriteMsgOptions;
use crate::ep::ActiveEndpointImpl;
use crate::ep::Endpoint;
use crate::infocapsoptions::ReadMod;
use crate::infocapsoptions::RmaCap;
use crate::infocapsoptions::WriteMod;
use crate::mr::DataDescriptor;
use crate::mr::MappedMemoryRegionKey;
use crate::utils::check_error;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;

// struct AsyncTransferCq{
//     req: usize,
//     // buf: &'a [T],
//     cq: Rc<AsyncCompletionQueueImpl>,
// }

// impl async_std::future::Future for AsyncTransferCq {
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


// impl<E: RmaCap + ReadMod> Endpoint<E> {

//     pub async unsafe  fn read_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn read_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn readv_asyn'a, c<T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe   fn readv_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };

//         check_error(err)
//     }

//     pub async unsafe  fn read_connected_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn read_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn readv_connected_asyn'a, c<T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe   fn readv_connected_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };

//         check_error(err)
//     }
    
    
//     pub async unsafe  fn readmsg_async(&self, msg: &crate::msg::MsgRma, options: ReadMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_readmsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        
//         check_error(err)
//     }
// }

impl<E: RmaCap + WriteMod> Endpoint<E> {

    pub async unsafe  fn write_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn write_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn writev_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }

    pub async unsafe  fn writev_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn writemsg_async(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn writedata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn writedata_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key(),  (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }

    pub async unsafe  fn inject_write_async<T>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }     

    pub async unsafe  fn inject_writedata_async<T>(&self, buf: &[T], data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }

    pub async unsafe  fn write_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn write_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn writev_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }

    pub async unsafe  fn writev_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn writedata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }
    
    pub async unsafe  fn writedata_connected_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(),  (context as *mut T0).cast()) };
        if err == 0 {
            let req = self.inner.tx_cq.borrow().as_ref().unwrap().request();
            let cq = self.inner.tx_cq.borrow().as_ref().unwrap().clone(); 
            AsyncTransferCq{req, cq}.await?;
        } 
        check_error(err)
    }

    pub async unsafe  fn inject_write_connected_async<T>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
    
        check_error(err)
    }     

    pub async unsafe  fn inject_writedata_connected_async<T>(&self, buf: &[T], data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
    
        check_error(err)
    }
}

// impl TransmitContext {

//     pub async unsafe  fn write_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
//         let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn write_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
//         let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn writev_asyn'a, c<T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };

//         check_error(err)
//     }

//     pub async unsafe  fn writev_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };

//         check_error(err)
//     }
    
//     pub async unsafe  fn writemsg_async(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn writedata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn writedata_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key(),  (context as *mut T0).cast()) };
        
//         check_error(err)
//     }

//     pub async unsafe  fn inject_write_async<T>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
    
//         check_error(err)
//     }     

//     pub async unsafe  fn inject_writedata_async<T>(&self, buf: &[T], data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
    
//         check_error(err)
//     }

//     pub async unsafe  fn write_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
//         let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn write_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
//         let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn writev_connected_asyn'a, c<T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };

//         check_error(err)
//     }

//     pub async unsafe  fn writev_connected_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };

//         check_error(err)
//     }
    
//     pub async unsafe  fn writedata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn writedata_connected_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(),  (context as *mut T0).cast()) };
        
//         check_error(err)
//     }

//     pub async unsafe  fn inject_write_connected_async<T>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
    
//         check_error(err)
//     }     

//     pub async unsafe  fn inject_writedata_connected_async<T>(&self, buf: &[T], data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
    
//         check_error(err)
//     }
// }

// impl ReceiveContext {

//     pub async unsafe  fn read_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn read_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn readv_asyn'a, c<T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe   fn readv_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };

//         check_error(err)
//     }

//     pub async unsafe  fn read_connected_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn read_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn readv_connected_asyn'a, c<T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe   fn readv_connected_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };

//         check_error(err)
//     }
    
    
//     pub async unsafe  fn readmsg_async(&self, msg: &crate::msg::MsgRma, options: ReadMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_readmsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        
//         check_error(err)
//     }
// }


