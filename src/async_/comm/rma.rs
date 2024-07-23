use crate::comm::rma::{ReadEpImpl, WriteEpImpl};
use crate::ep::EndpointImplBase;
use crate::infocapsoptions::RmaCap;
use crate::{infocapsoptions::{WriteMod, ReadMod}, mr::{DataDescriptor, MappedMemoryRegionKey}, cq::SingleCompletion, enums::{ReadMsgOptions, WriteMsgOptions}, async_::{AsyncCtx, cq::AsyncReadCq, eq::AsyncReadEq}, MappedAddress, ep::EndpointBase};

pub(crate) trait AsyncReadEpImpl: ReadEpImpl {
    async unsafe  fn read_async_impl<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_mapped_addr: Option<&MappedAddress>, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    async unsafe fn readv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    async unsafe  fn readmsg_async_impl(&self, msg: &mut crate::msg::MsgRma, options: ReadMsgOptions) -> Result<SingleCompletion, crate::error::Error> ;
}

pub trait AsyncReadEp {
    unsafe  fn read_from_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe  fn read_from_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe  fn read_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe  fn read_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe  fn readv_from_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ; //[TODO]
    unsafe  fn readv_from_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ; //[TODO]
    unsafe  fn readv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe  fn readv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ; //[TODO]
    unsafe  fn readmsg_async(&self, msg: &mut crate::msg::MsgRma, options: ReadMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}

// impl<E: ReadMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointBase<E, EQ, CQ> {
impl<EP: RmaCap + ReadMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointImplBase<EP, EQ, CQ> {

    #[inline]
    async unsafe  fn read_async_impl<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_mapped_addr: Option<&MappedAddress>, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.read_impl(buf, desc, src_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
    async unsafe fn readv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.readv_impl(iov, desc, src_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
    async unsafe  fn readmsg_async_impl(&self, msg: &mut crate::msg::MsgRma, options: ReadMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        let real_user_ctx = msg.c_msg_rma.context;
        let mut async_ctx = AsyncCtx{user_ctx: 
            if real_user_ctx.is_null() {
                None
            }
            else {
                Some(real_user_ctx)
            }
        };

        msg.c_msg_rma.context = (&mut async_ctx as *mut AsyncCtx).cast();
        
        if let Err(err) = self.readmsg_impl(msg, options) {
            msg.c_msg_rma.context = real_user_ctx;
            return Err(err);
        }

        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
        let res =  cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg_rma.context = real_user_ctx;
        res
    }
}

impl<E: RmaCap + ReadMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> AsyncReadEpImpl for EndpointBase<E, EQ, CQ> {

    #[inline]
    async unsafe  fn read_async_impl<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_mapped_addr: Option<&MappedAddress>, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.read_async_impl(buf, desc, src_mapped_addr, mem_addr, mapped_key, user_ctx).await
    }

    #[inline]
    async unsafe fn readv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        self.inner.readv_async_impl(iov, desc, src_mapped_addr, mem_addr, mapped_key, user_ctx).await
    }

    #[inline]
    async unsafe  fn readmsg_async_impl(&self, msg: &mut crate::msg::MsgRma, options: ReadMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.readmsg_async_impl(msg, options).await
    }
}

impl<EP: AsyncReadEpImpl> AsyncReadEp for EP {
    async unsafe  fn read_from_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
        self.read_async_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, None).await
    }
    
    async unsafe  fn read_from_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.read_async_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    
    async unsafe  fn read_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
        self.read_async_impl(buf, desc, None, mem_addr, mapped_key, None).await
    }
    
    async unsafe  fn read_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.read_async_impl(buf, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    async unsafe  fn readv_from_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
        self.readv_async_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, None).await
    }
    
    async unsafe   fn readv_from_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
        self.readv_async_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    async unsafe  fn readv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
        self.readv_async_impl(iov, desc, None, mem_addr, mapped_key, None).await
    }
    
    async unsafe   fn readv_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
        self.readv_async_impl(iov, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    
    async unsafe  fn readmsg_async(&self, msg: &mut crate::msg::MsgRma, options: ReadMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.readmsg_async_impl(msg, options).await
    }

}

pub(crate) trait AsyncWriteEpImpl: WriteEpImpl {
    async unsafe fn write_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error>  ;
    async unsafe fn writev_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    async unsafe  fn writemsg_async_impl(&self, msg: &mut crate::msg::MsgRma, options: WriteMsgOptions) -> Result<SingleCompletion, crate::error::Error> ;        
    async unsafe  fn writedata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ; 
}


pub trait AsyncWriteEp {
    unsafe fn write_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>  ;
    unsafe fn write_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn write_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn write_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writev_to_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writev_to_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writev_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writev_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writemsg_async(&self, msg: &mut crate::msg::MsgRma, options: WriteMsgOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writedata_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writedata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writedata_to_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    unsafe fn writedata_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}


// impl<E: WriteMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointBase<E, EQ, CQ> {
impl<EP: RmaCap + WriteMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> AsyncWriteEpImpl for EndpointImplBase<EP, EQ, CQ> {

    async unsafe fn write_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error>  {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.write_impl(buf, desc, dest_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async unsafe fn writev_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.writev_impl(iov, desc, dest_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async unsafe  fn writedata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.writedata_impl(buf, desc, data, dest_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async unsafe  fn writemsg_async_impl(&self, msg: &mut crate::msg::MsgRma, options: WriteMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        let real_user_ctx = msg.c_msg_rma.context;
        let mut async_ctx = AsyncCtx{user_ctx: 
            if real_user_ctx.is_null() {
                None
            }
            else {
                Some(real_user_ctx)
            }
        };
        msg.c_msg_rma.context = (&mut async_ctx as *mut AsyncCtx).cast();
        if let Err(err) = self.writemsg_impl(msg, options) {
            msg.c_msg_rma.context = real_user_ctx;
            return Err(err);
        }
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
        let res =  cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg_rma.context = real_user_ctx;
        res
    }
}


impl<E: RmaCap + WriteMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> AsyncWriteEpImpl for EndpointBase<E, EQ, CQ> {

    async unsafe fn write_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error>  {
        self.inner.write_async_impl(buf, desc, dest_mapped_addr, mem_addr, mapped_key, user_ctx).await
    }

    async unsafe fn writev_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        self.inner.writev_async_impl(iov, desc, dest_mapped_addr, mem_addr, mapped_key, user_ctx).await
    }

    async unsafe  fn writedata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        self.inner.writedata_async_impl(buf, desc, data, dest_mapped_addr, mem_addr, mapped_key, user_ctx).await
    }

    async unsafe  fn writemsg_async_impl(&self, msg: &mut crate::msg::MsgRma, options: WriteMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.writemsg_async_impl(msg, options).await
    }
}

impl<EP: AsyncWriteEpImpl> AsyncWriteEp for EP {

    #[inline]
    async unsafe  fn write_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error>  {
        self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, None).await
    }
    
    #[inline]
    async unsafe  fn write_to_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error>  {
        self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    
    #[inline]
    async unsafe  fn write_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error>  {
        self.write_async_impl(buf, desc, None, mem_addr, mapped_key, None).await
    }
    
    #[inline]
    async unsafe  fn write_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error>  {
        self.write_async_impl(buf, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    #[inline]
    async unsafe  fn writev_to_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
        self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, None).await
    }

    #[inline]
    async unsafe  fn writev_to_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
    
    #[inline]
    async unsafe  fn writev_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
        self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, None).await
    }

    #[inline]
    async unsafe  fn writev_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
        self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    #[inline]
    async unsafe  fn writemsg_async(&self, msg: &mut crate::msg::MsgRma, options: WriteMsgOptions) -> Result<SingleCompletion, crate::error::Error> {
        self.writemsg_async_impl(msg, options).await
    }

    #[inline]
    async unsafe  fn writedata_to_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, None).await
    }

    #[inline]
    async unsafe  fn writedata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, None).await
    }
    
    #[inline]
    async unsafe  fn writedata_to_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }

    #[inline]
    async unsafe  fn writedata_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
    }
}

pub trait AsyncReadWriteEp: AsyncReadEp + AsyncWriteEp {}
impl<EP: AsyncReadEp + AsyncWriteEp> AsyncReadWriteEp for EP {}

// impl TransmitContext {
//     async unsafe fn write_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error>  {
//         let mut async_ctx = AsyncCtx{user_ctx};
//         let raw_addr = if let Some(addr) = dest_mapped_addr {
//             addr.raw_addr()
//         }
//         else {
//             FI_ADDR_UNSPEC
//         };

//         let err = unsafe{ libfabric_sys::inlined_fi_write(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), (&mut async_ctx as *mut AsyncCtx).cast()) };
//         if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
//             let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
//             return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
//         } 

//         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
//     }

//     pub async unsafe  fn write_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error>  {
//         self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, None).await
//     }
    
//     pub async unsafe  fn write_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error>  {
//         self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }
    
//     pub async unsafe  fn write_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error>  {
//         self.write_async_impl(buf, desc, None, mem_addr, mapped_key, None).await
//     }


        
//     pub async unsafe  fn write_connected_with_context_async<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error>  {
//         self.write_async_impl(buf, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }
    

//     async unsafe fn writev_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
//         let mut async_ctx = AsyncCtx{user_ctx};
//         let raw_addr = if let Some(addr) = dest_mapped_addr {
//             addr.raw_addr()
//         }
//         else {
//             FI_ADDR_UNSPEC
//         };

//         let err = unsafe{ libfabric_sys::inlined_fi_writev(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), (&mut async_ctx as *mut AsyncCtx).cast()) };
//         if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
//             let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
//             return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
//         } 

//         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
//     }


//     pub async unsafe  fn writev_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
//         self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, None).await
//     }

//     pub async unsafe  fn writev_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
//         self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }
    
//     pub async unsafe  fn writev_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
//         self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, None).await
//     }

//     pub async unsafe  fn writev_connected_with_context_async<'a, T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> { //[TODO]
//         self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }

//     /// [TODO]
//     // pub async unsafe  fn writemsg_async(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
//     //     let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.as_raw_typed_fid(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
//     //     if err == 0 {
    //         let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
//     //         let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
//     //         AsyncTransferCq{cq}.await?;
//     //     } 
//     //     check_error(err)
//     // }
    
//     async unsafe  fn writedata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
//         let mut async_ctx = AsyncCtx{user_ctx};
//         let raw_addr = if let Some(addr) = dest_mapped_addr {
//             addr.raw_addr()
//         }
//         else {
//             FI_ADDR_UNSPEC
//         };

//         let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, mem_addr, mapped_key.get_key(),  (&mut async_ctx as *mut AsyncCtx).cast()) };
//         if err == 0 {
            // let req = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").request();
//             let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion").clone(); 
//             return crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await;
//         } 

//         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
//     }

//     pub async unsafe  fn writedata_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
//         self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, None).await
//     }

//     pub async unsafe  fn writedata_connected_async<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<SingleCompletion, crate::error::Error> {
//         self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, None).await
//     }
    
//     pub async unsafe  fn writedata_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
//         self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }

//     pub async unsafe  fn writedata_connected_with_context_async<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<SingleCompletion, crate::error::Error> {
//         self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, Some((context as *mut T0).cast())).await
//     }

//     pub async unsafe  fn inject_write_async<T>(&self, buf: &[T], dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {  // Inject does not generate completions
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
//         check_error(err)
//     }     

//     pub async unsafe  fn inject_writedata_async<T>(&self, buf: &[T], data: u64, dest_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { // Inject does not generate completions
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
//         check_error(err)
//     }

//     pub async unsafe  fn inject_write_connected_async<T>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
//         check_error(err)
//     }     

//     pub async unsafe  fn inject_writedata_connected_async<T>(&self, buf: &[T], data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
//         check_error(err)
//     }
// }

// impl ReceiveContext {

//     pub async unsafe  fn read_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.as_raw_typed_fid(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn read_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.as_raw_typed_fid(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn readv_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe   fn readv_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_addr: &MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), src_addr.raw_addr(), mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };

//         check_error(err)
//     }

//     pub async unsafe  fn read_connected_async<T0>(&self, buf: &mut [T0], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.as_raw_typed_fid(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn read_connected_with_context_async<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_read(self.as_raw_typed_fid(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe  fn readv_connected_async<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), std::ptr::null_mut()) };
        
//         check_error(err)
//     }
    
//     pub async unsafe   fn readv_connected_with_context_asyn'a, c<T, T0>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         let err = unsafe{ libfabric_sys::inlined_fi_readv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), (context as *mut T0).cast()) };

//         check_error(err)
//     }
    
    
//     pub async unsafe  fn readmsg_async(&self, msg: &crate::msg::MsgRma, options: ReadMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_readmsg(self.as_raw_typed_fid(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        
//         check_error(err)
//     }
// }