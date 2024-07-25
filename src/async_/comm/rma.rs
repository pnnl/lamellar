use crate::async_::ep::AsyncTxEp;
use crate::async_::xcontext::{TransmitContext, TransmitContextImpl, ReceiveContext, ReceiveContextImpl};
use crate::comm::rma::{ReadEpImpl, WriteEpImpl};
use crate::ep::EndpointImplBase;
use crate::infocapsoptions::RmaCap;
use crate::{infocapsoptions::{WriteMod, ReadMod}, mr::{DataDescriptor, MappedMemoryRegionKey}, cq::SingleCompletion, enums::{ReadMsgOptions, WriteMsgOptions}, async_::{AsyncCtx, cq::AsyncReadCq, eq::AsyncReadEq}, MappedAddress, ep::EndpointBase};

pub(crate) trait AsyncReadEpImpl: AsyncTxEp + ReadEpImpl {
    async unsafe  fn read_async_impl<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_mapped_addr: Option<&MappedAddress>, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.read_impl(buf, desc, src_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async unsafe fn readv_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.readv_impl(iov, desc, src_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

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

        let cq = self.retrieve_tx_cq();
        let res =  cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg_rma.context = real_user_ctx;
        res
    }
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

// impl<E: ReadMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointBase<E> {
impl<EP: RmaCap + ReadMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointImplBase<EP, EQ, CQ> {}

impl<E: AsyncReadEpImpl> AsyncReadEpImpl for EndpointBase<E> {}

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

pub(crate) trait AsyncWriteEpImpl: AsyncTxEp + WriteEpImpl {
    async unsafe fn write_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error>  {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.write_impl(buf, desc, dest_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async unsafe fn writev_async_impl<'a, T>(&self, iov: &[crate::iovec::IoVec<'a,T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.writev_impl(iov, desc, dest_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async unsafe  fn writedata_async_impl<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_mapped_addr: Option<&MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.writedata_impl(buf, desc, data, dest_mapped_addr, mem_addr, mapped_key, Some(&mut async_ctx as *mut AsyncCtx))?;
        let cq = self.retrieve_tx_cq();
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
        let cq = self.retrieve_tx_cq();
        let res =  cq.wait_for_ctx_async(&mut async_ctx).await;
        msg.c_msg_rma.context = real_user_ctx;
        res
    }
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


// impl<E: WriteMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointBase<E> {
impl<EP: RmaCap + WriteMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> AsyncWriteEpImpl for EndpointImplBase<EP, EQ, CQ> {}


impl<E: AsyncWriteEpImpl> AsyncWriteEpImpl for EndpointBase<E> {

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

// impl AsyncWriteEpImpl for TransmitContext {}
// impl AsyncWriteEpImpl for TransmitContextImpl {}
// impl AsyncReadEpImpl for ReceiveContext {}
// impl AsyncReadEpImpl for ReceiveContextImpl {}