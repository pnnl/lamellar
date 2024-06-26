use crate::{FI_ADDR_UNSPEC, infocapsoptions::{RmaCap, WriteMod, ReadMod}, mr::DataDescriptor, utils::check_error, cq::SingleCompletionFormat, enums::ReadMsgOptions, async_::{ep::Endpoint, AsyncCtx, cq::AsyncTransferCq, mr::{MappedAddress, MappedMemoryRegionKey}}, ep::ActiveEndpointImpl};

impl<E: RmaCap + ReadMod> Endpoint<E> {

    async unsafe  fn read_async_impl<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_mapped_addr: Option<&MappedAddress>, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = src_mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), (&mut async_ctx as *mut AsyncCtx).cast()) };

        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
            let cq = self.inner.rx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
            return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

    pub async unsafe  fn read_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.read_async_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, None).await
    }
    
    pub async unsafe  fn read_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.read_async_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    
    pub async unsafe  fn read_connected_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.read_async_impl(buf, desc, None, mem_addr, mapped_key, None).await
    }
    
    pub async unsafe  fn read_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.read_async_impl(buf, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    async unsafe   fn readv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = src_mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), (&mut async_ctx as *mut AsyncCtx).cast()) };

        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
            let cq = self.inner.rx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
            return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

    pub async unsafe  fn readv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> { //[TODO]
        self.readv_async_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, None).await
    }
    
    pub async unsafe   fn readv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> { //[TODO]
        self.readv_async_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    
    pub async unsafe  fn readv_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.readv_async_impl(iov, desc, None, mem_addr, mapped_key, None).await

    }
    
    pub async unsafe   fn readv_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> { //[TODO]
        self.readv_async_impl(iov, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    
    
    pub async unsafe  fn readmsg_async(&self, msg: &crate::msg::MsgRma, options: ReadMsgOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        let user_ctx = if !msg.c_msg_rma.context.is_null() {
            Some(msg.c_msg_rma.context.cast())
        }
        else {
            None
        };
        let mut async_ctx = AsyncCtx{user_ctx};
        
        let err = unsafe{ libfabric_sys::inlined_fi_readmsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
            let cq = self.inner.rx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
            return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }
}

impl<E: RmaCap + WriteMod> Endpoint<E> {

    async unsafe fn write_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error>  {
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = dest_mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
            let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
            return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

    pub async unsafe  fn write_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error>  {
        self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, None).await
    }
    
    pub async unsafe  fn write_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error>  {
        self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    
    pub async unsafe  fn write_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error>  {
        self.write_async_impl(buf, desc, None, mem_addr, mapped_key, None).await
    }


        
    pub async unsafe  fn write_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error>  {
        self.write_async_impl(buf, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    

    async unsafe fn writev_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = dest_mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
            let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
            return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }


    pub async unsafe  fn writev_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, None).await
    }

    pub async unsafe  fn writev_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    
    pub async unsafe  fn writev_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> { //[TODO]
        self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, None).await
    }

    pub async unsafe  fn writev_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> { //[TODO]
        self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    // pub async unsafe  fn writemsg_async(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
    //     let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
    //     if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
    //         let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
    //         AsyncTransferCq{cq}.await?;
    //     } 
    //     check_error(err)
    // }
    
    async unsafe  fn writedata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        let raw_addr = if let Some(addr) = dest_mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, mem_addr, mapped_key.get_key(),  (&mut async_ctx as *mut AsyncCtx).cast()) };
        if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
            let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
            return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
        } 

        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
    }

    pub async unsafe  fn writedata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, None).await
    }

    pub async unsafe  fn writedata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, None).await
    }
    
    pub async unsafe  fn writedata_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    pub async unsafe  fn writedata_connected_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
}

// impl TransmitContext {
//     async unsafe fn write_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error>  {
//         let mut async_ctx = AsyncCtx{user_ctx};
//         let raw_addr = if let Some(addr) = dest_mapped_addr {
//             addr.raw_addr()
//         }
//         else {
//             FI_ADDR_UNSPEC
//         };

//         let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), (&mut async_ctx as *mut AsyncCtx).cast()) };
//         if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
//             let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
//             return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
//         } 

//         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
//     }

//     pub async unsafe  fn write_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error>  {
//         self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, None).await
//     }
    
//     pub async unsafe  fn write_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error>  {
//         self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }
    
//     pub async unsafe  fn write_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error>  {
//         self.write_async_impl(buf, desc, None, mem_addr, mapped_key, None).await
//     }


        
//     pub async unsafe  fn write_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error>  {
//         self.write_async_impl(buf, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }
    

//     async unsafe fn writev_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> { 
//         let mut async_ctx = AsyncCtx{user_ctx};
//         let raw_addr = if let Some(addr) = dest_mapped_addr {
//             addr.raw_addr()
//         }
//         else {
//             FI_ADDR_UNSPEC
//         };

//         let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), (&mut async_ctx as *mut AsyncCtx).cast()) };
//         if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
//             let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
//             return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
//         } 

//         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
//     }


//     pub async unsafe  fn writev_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
//         self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, None).await
//     }

//     pub async unsafe  fn writev_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
//         self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }
    
//     pub async unsafe  fn writev_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> { //[TODO]
//         self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, None).await
//     }

//     pub async unsafe  fn writev_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> { //[TODO]
//         self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }

//     /// [TODO]
//     // pub async unsafe  fn writemsg_async(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
//     //     let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
//     //     if err == 0 {
    //         let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
//     //         let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
//     //         AsyncTransferCq{cq}.await?;
//     //     } 
//     //     check_error(err)
//     // }
    
//     async unsafe  fn writedata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> { 
//         let mut async_ctx = AsyncCtx{user_ctx};
//         let raw_addr = if let Some(addr) = dest_mapped_addr {
//             addr.raw_addr()
//         }
//         else {
//             FI_ADDR_UNSPEC
//         };

//         let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, mem_addr, mapped_key.get_key(),  (&mut async_ctx as *mut AsyncCtx).cast()) };
//         if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
//             let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
//             return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
//         } 

//         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
//     }

//     pub async unsafe  fn writedata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
//         self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, None).await
//     }

//     pub async unsafe  fn writedata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletionFormat, crate::error::Error> {
//         self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, None).await
//     }
    
//     pub async unsafe  fn writedata_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
//         self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }

//     pub async unsafe  fn writedata_connected_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
//         self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }

//     pub async unsafe  fn inject_write_async<T>(&self, buf: &[T], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {  // Inject does not generate completions
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
//         check_error(err)
//     }     

//     pub async unsafe  fn inject_writedata_async<T>(&self, buf: &[T], data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { // Inject does not generate completions
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
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

//     pub async unsafe  fn read_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn read_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn readv_asyn'a, c<T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe   fn readv_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
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


