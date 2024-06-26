use crate::FI_ADDR_UNSPEC;
use crate::enums::ReadMsgOptions;
use crate::enums::WriteMsgOptions;
use crate::ep::ActiveEndpointImpl;
use crate::ep::Endpoint;
use crate::ep::EndpointBase;
use crate::eq::EventQueueImplT;
use crate::infocapsoptions::ReadMod;
use crate::infocapsoptions::RmaCap;
use crate::infocapsoptions::WriteMod;
use crate::mr::DataDescriptor;
use crate::mr::MappedMemoryRegionKey;
use crate::utils::check_error;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;

use super::message::extract_raw_addr_and_ctx;


impl<E: RmaCap + ReadMod, EQ: EventQueueImplT, CQ> EndpointBase<E, EQ, CQ> {

    unsafe fn read_impl<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: Option<&crate::MappedAddress>, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }

    pub unsafe fn read<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.read_impl::<T,()>(buf, desc, Some(src_addr), mem_addr, mapped_key, None)
    }
    
    pub unsafe fn read_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.read_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn read_connected<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.read_impl::<T,()>(buf, desc, None, mem_addr, mapped_key, None)
    }
    
    pub unsafe fn read_connected_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.read_impl(buf, desc, None, mem_addr, mapped_key, Some(context))
    }
    
    unsafe fn readv_impl<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }

    pub unsafe fn readv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl::<T,()>(iov, desc, Some(src_addr), mem_addr, mapped_key, None)
    }
    
    pub unsafe  fn readv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn readv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl::<T,()>(iov, desc, None, mem_addr, mapped_key, None)
    }
    
    pub unsafe  fn readv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl(iov, desc, None, mem_addr, mapped_key, Some(context))
    }
    
    
    pub unsafe fn readmsg(&self, msg: &crate::msg::MsgRma, options: ReadMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_readmsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        
        check_error(err)
    }
}

impl<E: RmaCap + WriteMod, EQ: EventQueueImplT, CQ> EndpointBase<E, EQ, CQ> {

    unsafe fn write_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error>  {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }

    pub unsafe fn write<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        self.write_impl::<T,()>(buf, desc, Some(dest_addr), mem_addr, mapped_key, None)
    }
    
    pub unsafe fn write_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
        self.write_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn write_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        self.write_impl::<T,()>(buf, desc, None, mem_addr, mapped_key, None)
    }
    
    pub unsafe fn write_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
        self.write_impl(buf, desc, None, mem_addr, mapped_key, Some(context))
    }

    unsafe fn writev_impl<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> { 
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }

    pub unsafe fn writev<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl::<T,()>(iov, desc, Some(dest_addr), mem_addr, mapped_key, None)
    }

    pub unsafe fn writev_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }
    
    pub unsafe fn writev_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl::<T,()>(iov, desc, None, mem_addr, mapped_key, None)
    }

    pub unsafe fn writev_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl(iov, desc, None, mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn writemsg(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        check_error(err)
    }
    
    unsafe fn writedata_impl<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, mem_addr, mapped_key.get_key(),  ctx) };
        check_error(err)
    }

    pub unsafe fn writedata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.writedata_impl::<T,()>(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, None)
    }
    
    pub unsafe fn writedata_with_context<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.writedata_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }
        
    pub unsafe fn writedata_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.writedata_impl::<T,()>(buf, desc, data, None, mem_addr, mapped_key, None)
    }
    
    pub unsafe fn writedata_connected_with_context<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.writedata_impl(buf, desc, data, None, mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn inject_write<T>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
        check_error(err)
    }     

    pub unsafe fn inject_writedata<T>(&self, buf: &[T], data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
        check_error(err)
    }

    pub unsafe fn inject_write_connected<T>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
        check_error(err)
    }     

    pub unsafe fn inject_writedata_connected<T>(&self, buf: &[T], data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
        check_error(err)
    }
}

impl TransmitContext {

    unsafe fn write_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error>  {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }

    pub unsafe fn write<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        self.write_impl::<T,()>(buf, desc, Some(dest_addr), mem_addr, mapped_key, None)
    }
    
    pub unsafe fn write_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
        self.write_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn write_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        self.write_impl::<T,()>(buf, desc, None, mem_addr, mapped_key, None)
    }
    
    pub unsafe fn write_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
        self.write_impl(buf, desc, None, mem_addr, mapped_key, Some(context))
    }

    unsafe fn writev_impl<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> { 
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_writev(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }

    pub unsafe fn writev<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl::<T,()>(iov, desc, Some(dest_addr), mem_addr, mapped_key, None)
    }

    pub unsafe fn writev_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }
    
    pub unsafe fn writev_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl::<T,()>(iov, desc, None, mem_addr, mapped_key, None)
    }

    pub unsafe fn writev_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl(iov, desc, None, mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn writemsg(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        check_error(err)
    }
    
    unsafe fn writedata_impl<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, mem_addr, mapped_key.get_key(),  ctx) };
        check_error(err)
    }

    pub unsafe fn writedata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.writedata_impl::<T,()>(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, None)
    }
    
    pub unsafe fn writedata_with_context<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.writedata_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }
        
    pub unsafe fn writedata_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.writedata_impl::<T,()>(buf, desc, data, None, mem_addr, mapped_key, None)
    }
    
    pub unsafe fn writedata_connected_with_context<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.writedata_impl(buf, desc, data, None, mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn inject_write<T>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
        check_error(err)
    }     

    pub unsafe fn inject_writedata<T>(&self, buf: &[T], data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, dest_addr.raw_addr(), mem_addr, mapped_key.get_key()) };
        check_error(err)
    }

    pub unsafe fn inject_write_connected<T>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
        check_error(err)
    }     

    pub unsafe fn inject_writedata_connected<T>(&self, buf: &[T], data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.handle(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key()) };
        check_error(err)
    }
}

impl ReceiveContext {

    unsafe fn read_impl<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: Option<&crate::MappedAddress>, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_read(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }

    pub unsafe fn read<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.read_impl::<T,()>(buf, desc, Some(src_addr), mem_addr, mapped_key, None)
    }
    
    pub unsafe fn read_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.read_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn read_connected<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.read_impl::<T,()>(buf, desc, None, mem_addr, mapped_key, None)
    }
    
    pub unsafe fn read_connected_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.read_impl(buf, desc, None, mem_addr, mapped_key, Some(context))
    }
    
    unsafe fn readv_impl<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_readv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }

    pub unsafe fn readv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl::<T,()>(iov, desc, Some(src_addr), mem_addr, mapped_key, None)
    }
    
    pub unsafe  fn readv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, Some(context))
    }

    pub unsafe fn readv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl::<T,()>(iov, desc, None, mem_addr, mapped_key, None)
    }
    
    pub unsafe  fn readv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl(iov, desc, None, mem_addr, mapped_key, Some(context))
    }
    
    
    pub unsafe fn readmsg(&self, msg: &crate::msg::MsgRma, options: ReadMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_readmsg(self.handle(), &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, options.get_value()) };
        
        check_error(err)
    }
}


