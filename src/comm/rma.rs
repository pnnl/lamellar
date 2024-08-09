use crate::cq::ReadCq;
use crate::enums::ReadMsgOptions;
use crate::enums::WriteMsgOptions;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::eq::ReadEq;
use crate::fid::AsRawTypedFid;
use crate::fid::EpRawFid;
use crate::infocapsoptions::ReadMod;
use crate::infocapsoptions::RmaCap;
use crate::infocapsoptions::WriteMod;
use crate::mr::DataDescriptor;
use crate::mr::MappedMemoryRegionKey;
use crate::utils::check_error;
use crate::xcontext::RxContextBase;
use crate::xcontext::RxContextImplBase;
use crate::xcontext::TxContextBase;
use crate::xcontext::TxContextImplBase;
use crate::FI_ADDR_UNSPEC;

use super::message::extract_raw_addr_and_ctx;


pub(crate) trait ReadEpImpl: ReadEp + AsRawTypedFid<Output = EpRawFid>{
    unsafe fn read_impl<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: Option<&crate::MappedAddress>, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_read(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }
    
    unsafe fn readv_impl<T0>(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], src_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_readv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }
    
    unsafe fn readmsg_impl(&self, msg: &crate::msg::MsgRmaMut, options: ReadMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_readmsg(self.as_raw_typed_fid(), msg.get() as *const libfabric_sys::fi_msg_rma, options.as_raw()) };
        check_error(err)
    }
}





pub trait ReadEp {
    /// Read data from a remote memory region into local buffer `buf`
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` without a provided context
    unsafe fn read_from<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
    
    /// Similar to [ReadEp::read_from] but with a context argument provided
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read`
    unsafe fn read_from_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
    
    /// Similar to [ReadEp::read_from] but without specifying a src network address
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` with no context and `src_addr` = `FI_ADDR_UNSPEC`
    unsafe fn read<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
    
    /// Similar to [ReadEp::read] but providing a context
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` with `src_addr` = `FI_ADDR_UNSPEC`
    unsafe fn read_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
    
    /// Similar to [ReadEp::read_from] with a list of buffers `iov` instead of a single buffer to store the data ranges read
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with no context`
    unsafe fn readv_from(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
            
    /// Similar to [ReadEp::readv_from] but providing a context
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv`
    unsafe fn readv_from_with_context<T0>(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
                
    /// Similar to [ReadEp::readv_from] but without specifying a network address 
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with no context and `src_addr` set to `FI_ADDR_UNSPEC`
    unsafe fn readv(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
                    
    /// Similar to [ReadEp::readv] but with a provided context 
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with `src_addr` set to `FI_ADDR_UNSPEC`
    unsafe fn readv_with_context<T0>(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
    
    /// Read from remote node with the specifications provided by the `msg` argument 
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readmsg`
    unsafe fn readmsg(&self, msg: &crate::msg::MsgRmaMut, options: ReadMsgOptions) -> Result<(), crate::error::Error> ;
}

impl<EP: ReadEpImpl> ReadEp for EP {
    unsafe fn read_from<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.read_impl::<T,()>(buf, desc, Some(src_addr), mem_addr, mapped_key, None)
    }
    
    unsafe fn read_from_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, src_addr: &crate::MappedAddress, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.read_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, Some(context))
    }

    unsafe fn read<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.read_impl::<T,()>(buf, desc, None, mem_addr, mapped_key, None)
    }
    
    unsafe fn read_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mem_addr: u64,  mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.read_impl(buf, desc, None, mem_addr, mapped_key, Some(context))
    }

    unsafe fn readv_from(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl::<()>(iov, desc, Some(src_addr), mem_addr, mapped_key, None)
    }
        
    unsafe  fn readv_from_with_context<T0>(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], src_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, Some(context))
    }

    unsafe fn readv(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl::<()>(iov, desc, None, mem_addr, mapped_key, None)
    }
    
    unsafe  fn readv_with_context<T0>(&self, iov: &[crate::iovec::IoVecMut], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.readv_impl(iov, desc, None, mem_addr, mapped_key, Some(context))
    }
    
    unsafe fn readmsg(&self, msg: &crate::msg::MsgRmaMut, options: ReadMsgOptions) -> Result<(), crate::error::Error> {
        self.readmsg_impl(msg, options)
    }
}




impl<EP: RmaCap + ReadMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> ReadEpImpl for EndpointImplBase<EP, EQ, CQ> { }
impl<E: ReadEpImpl> ReadEpImpl for EndpointBase<E> {}


pub(crate) trait WriteEpImpl: WriteEp + AsRawTypedFid<Output = EpRawFid>{
    unsafe fn write_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error>  {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }
    
    unsafe fn inject_write_impl<T>(&self, buf: &[T], dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        let raw_addr = if let Some(addr) = dest_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), raw_addr, mem_addr, mapped_key.get_key()) };
        check_error(err)
    }
    
    unsafe fn writev_impl<T0>(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> { 
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_writev(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, mem_addr, mapped_key.get_key(), ctx) };
        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    unsafe fn writedata_impl<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, mem_addr, mapped_key.get_key(),  ctx) };
        check_error(err)
    }

    unsafe fn inject_writedata_impl<T>(&self, buf: &[T], data: u64, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        let raw_addr = if let Some(addr) = dest_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), data, raw_addr, mem_addr, mapped_key.get_key()) };
        check_error(err)
    }

    unsafe fn writemsg_impl(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.as_raw_typed_fid(), msg.get() as *const libfabric_sys::fi_msg_rma, options.as_raw()) };
        check_error(err)
    }
}

pub trait WriteEp {
    /// Write data to a remote memory region from local buffer `buf` to remote address mem_addr
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` without a provided context
    unsafe fn write_to<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
    
    /// Similar to [WriteEp::write_to] but with a provided context
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write`.
    unsafe fn write_to_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
        
    /// Similar to [WriteEp::write_to] but without specifying a destination network address (e.g., for connected endpoints)
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` with dest_addr = FI_ADDR_UNSPEC and no context.
    unsafe fn write<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
            
    /// Similar to [WriteEp::write] but with a provided context
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn write_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
    
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject`.
    unsafe fn inject_write_to<T>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
    
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn inject_write<T>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
    
        
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject_writedata`.
    unsafe fn inject_writedata_to<T>(&self, buf: &[T], data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
        
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject_writedata` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn inject_writedata<T>(&self, buf: &[T], data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
    
    /// Similar to [WriteEp::write_to] but with a list of buffers `iov` instead of a single buffer to transfer
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` without a provided context
    unsafe fn writev_to(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
        
    /// Similar to [WriteEp::writev_to] but with a provided context
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev`.
    unsafe fn writev_to_with_context<T0>(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
            
    /// Similar to [WriteEp::writev_to] but without specifying a network address
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` with `dest_addr` = `FI_ADDR_UNSPEC` and no contex.
    unsafe fn writev(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;
                
    /// Similar to [WriteEp::writev] but with a provided context
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writev_with_context<T0>(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
    
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` without providing a context.
    unsafe fn writedata_to<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;    
            
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` .
    #[allow(clippy::too_many_arguments)]
    unsafe fn writedata_to_with_context<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
    
            
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` with no context and `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writedata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> ;    
    
                
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writedata_with_context<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> ;
    
    /// Transfer data base on the specifications provided by the `msg`
    /// 
    /// The operation is only expected to have completed when a respective Completion has been generated
    /// 
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writemsg`
    unsafe fn writemsg(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> ;
}

impl<EP: WriteEpImpl> WriteEp for EP {
    #[inline]
    unsafe fn write_to<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        self.write_impl::<T,()>(buf, desc, Some(dest_addr), mem_addr, mapped_key, None)
    }
    
    #[inline]
    unsafe fn write_to_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
        self.write_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }

    #[inline]
    unsafe fn write<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        self.write_impl::<T,()>(buf, desc, None, mem_addr, mapped_key, None)
    }
    
    #[inline]
    unsafe fn write_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error>  {
        self.write_impl(buf, desc, None, mem_addr, mapped_key, Some(context))
    }

    #[inline]
    unsafe fn inject_write_to<T>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.inject_write_impl(buf, Some(dest_addr), mem_addr, mapped_key)
    } 
    #[inline]
    unsafe fn inject_write<T>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.inject_write_impl(buf, None, mem_addr, mapped_key)
    }   

    #[inline]
    unsafe fn writev_to(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl::<()>(iov, desc, Some(dest_addr), mem_addr, mapped_key, None)
    }

    #[inline]
    unsafe fn writev_to_with_context<T0>(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }
    
    #[inline]
    unsafe fn writev(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl::<()>(iov, desc, None, mem_addr, mapped_key, None)
    }

    #[inline]
    unsafe fn writev_with_context<T0>(&self, iov: &[crate::iovec::IoVec], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.writev_impl(iov, desc, None, mem_addr, mapped_key, Some(context))
    }

    #[inline]
    unsafe fn writedata_to<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.writedata_impl::<T,()>(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, None)
    }
    
    #[inline]
    unsafe fn writedata_to_with_context<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.writedata_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, Some(context))
    }
        
    #[inline]
    unsafe fn writedata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error> {
        self.writedata_impl::<T,()>(buf, desc, data, None, mem_addr, mapped_key, None)
    }
    
    #[inline]
    unsafe fn writedata_with_context<T,T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, context: &mut T0) -> Result<(), crate::error::Error> {
        self.writedata_impl(buf, desc, data, None, mem_addr, mapped_key, Some(context))
    }

    #[inline]
    unsafe fn inject_writedata_to<T>(&self, buf: &[T], data: u64, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        self.inject_writedata_impl(buf, data, Some(dest_addr), mem_addr, mapped_key)
    }
    
    #[inline]
    unsafe fn inject_writedata<T>(&self, buf: &[T], data: u64, mem_addr: u64, mapped_key: &MappedMemoryRegionKey) -> Result<(), crate::error::Error>  {
        self.inject_writedata_impl(buf, data, None, mem_addr, mapped_key)
    }

    #[inline]
    unsafe fn writemsg(&self, msg: &crate::msg::MsgRma, options: WriteMsgOptions) -> Result<(), crate::error::Error> {
        self.writemsg_impl(msg, options)
    }
}

impl<EP: RmaCap + WriteMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> WriteEpImpl for EndpointImplBase<EP, EQ, CQ> {}
impl<E: WriteEpImpl> WriteEpImpl for EndpointBase<E> {}

impl<CQ: ?Sized + ReadCq> WriteEpImpl  for TxContextBase<CQ> {}
impl<CQ: ?Sized + ReadCq> WriteEpImpl  for TxContextImplBase<CQ> {}
impl<CQ: ?Sized + ReadCq> ReadEpImpl for RxContextBase<CQ> {}
impl<CQ: ?Sized + ReadCq> ReadEpImpl for RxContextImplBase<CQ> {}

pub trait ReadWriteEp: ReadEp + WriteEp {}
impl<EP: ReadEp + WriteEp> ReadWriteEp for EP {} 