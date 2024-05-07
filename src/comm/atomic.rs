use crate::FI_ADDR_UNSPEC;
use crate::enums::AtomicFetchMsgOptions;
use crate::enums::AtomicMsgOptions;
use crate::ep::ActiveEndpointImpl;
use crate::ep::Endpoint;
use crate::infocapsoptions::AtomicCap;
use crate::infocapsoptions::ReadMod;
use crate::infocapsoptions::WriteMod;
use crate::mr::DataDescriptor;
use crate::mr::MappedMemoryRegionKey;
use crate::utils::check_error;
use crate::utils::to_fi_datatype;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;


impl<E: AtomicCap+ WriteMod> Endpoint<E> {

    #[allow(clippy::too_many_arguments)]
    pub fn atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_with_context<T: 'static, T0>(&self, buf: &[T],  desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    pub fn atomicmsg(&self, msg: &crate::msg::MsgAtomic, options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.handle(), msg.c_msg_atomic, options.get_value()) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic<T: 'static>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic_connected<T: 'static>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }
}

impl<E: AtomicCap+ ReadMod + WriteMod> Endpoint<E> {


    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    pub fn fetch_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic,  resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicFetchMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.handle(), msg.c_msg_atomic, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), options.get_value()) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_connected_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_connected_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic, comparev: &[crate::iovec::Ioc<T>], compare_desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.handle(), msg.c_msg_atomic, comparev.as_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparev.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), options.get_value()) };

        check_error(err)
    }

    pub fn atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn fetch_atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_fetch_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn compare_atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_compare_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }
}

impl TransmitContext {

    #[allow(clippy::too_many_arguments)]
    pub fn atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_with_context<T: 'static, T0>(&self, buf: &[T],  desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    pub fn atomicmsg(&self, msg: &crate::msg::MsgAtomic, options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.handle(), msg.c_msg_atomic, options.get_value()) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic<T: 'static>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic_connected<T: 'static>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    pub fn fetch_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic,  resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicFetchMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.handle(), msg.c_msg_atomic, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), options.get_value()) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_connected_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_connected_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic, comparev: &[crate::iovec::Ioc<T>], compare_desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.handle(), msg.c_msg_atomic, comparev.as_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparev.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), options.get_value()) };

        check_error(err)
    }

    pub fn atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn fetch_atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_fetch_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn compare_atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_compare_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }
}

impl ReceiveContext {

    #[allow(clippy::too_many_arguments)]
    pub fn atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_with_context<T: 'static, T0>(&self, buf: &[T],  desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    pub fn atomicmsg(&self, msg: &crate::msg::MsgAtomic, options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.handle(), msg.c_msg_atomic, options.get_value()) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic<T: 'static>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic_connected<T: 'static>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    pub fn fetch_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic,  resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicFetchMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.handle(), msg.c_msg_atomic, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), options.get_value()) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), dest_addr.raw_addr(), mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_connected_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_connected<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_connected_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_connected<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_connected_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), FI_ADDR_UNSPEC, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), (context as *mut T0).cast())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic, comparev: &[crate::iovec::Ioc<T>], compare_desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.handle(), msg.c_msg_atomic, comparev.as_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparev.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), options.get_value()) };

        check_error(err)
    }

    pub fn atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn fetch_atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_fetch_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn compare_atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_compare_atomicvalid(self.handle(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }
}

pub struct AtomicAttr {
    pub(crate) c_attr : libfabric_sys::fi_atomic_attr,
}

impl AtomicAttr {
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_atomic_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_atomic_attr {
        &mut self.c_attr
    }
}