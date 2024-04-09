use crate::check_error;
use crate::ep::Endpoint;
use crate::ep::ActiveEndpoint;
use crate::infocapsoptions::AtomicCap;
use crate::infocapsoptions::ReadMod;
use crate::infocapsoptions::WriteMod;
use crate::to_fi_datatype;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;


impl<E: AtomicCap+ WriteMod> Endpoint<E> {

    #[allow(clippy::too_many_arguments)]
    pub fn atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    pub fn atomicmsg(&self, msg: &MsgAtomic, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.handle(), msg.c_msg_atomic, flags) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic<T: 'static>(&self, buf: &[T], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }
}

impl<E: AtomicCap+ ReadMod + WriteMod> Endpoint<E> {


    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, res: &mut [T], res_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, res: &mut [T], res_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, resultv: &mut crate::Ioc<T>,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, resultv.get_mut(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    pub fn fetch_atomicmsg<T: 'static>(&self, msg: &MsgAtomic,  resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.handle(), msg.c_msg_atomic, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, flags) };
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, compare: &mut [T], compare_desc: &mut impl crate::DataDescriptor, 
            result: &mut [T], result_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.get_desc().cast(), result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut [impl crate::DataDescriptor], compare: &mut [T], compare_desc: &mut [impl crate::DataDescriptor], 
            result: &mut [T], result_desc: &mut [impl crate::DataDescriptor], dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.as_mut_ptr()  as *mut std::ffi::c_void, result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.as_mut_ptr()  as *mut std::ffi::c_void, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut crate::Ioc<T>,  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut crate::Ioc<T>,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.get_mut(), compare_desc.as_mut_ptr().cast(), compare_count, resultv.get_mut(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut crate::Ioc<T>,  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut crate::Ioc<T>,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.get_mut(), compare_desc.as_mut_ptr().cast(), compare_count, resultv.get_mut(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicmsg<T: 'static>(&self, msg: &MsgAtomic, comparev: &[crate::Ioc<T>], compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, resultv: &mut crate::Ioc<T>,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.handle(), msg.c_msg_atomic, comparev.as_ptr().cast(), compare_desc.as_mut_ptr().cast(), compare_count, resultv.get_mut(), res_desc.as_mut_ptr().cast(), res_count, flags) };

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
    pub fn atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    pub fn atomicmsg(&self, msg: &MsgAtomic, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.handle(), msg.c_msg_atomic, flags) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic<T: 'static>(&self, buf: &[T], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, res: &mut [T], res_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, res: &mut [T], res_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, resultv: &mut crate::Ioc<T>,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, resultv.get_mut(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, resultv: &mut crate::Ioc<T>,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, resultv.get_mut(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    pub fn fetch_atomicmsg<T: 'static>(&self, msg: &MsgAtomic,  resultv: &mut crate::Ioc<T>,  res_desc: &mut impl crate::DataDescriptor, res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.handle(), msg.c_msg_atomic, resultv.get_mut(), res_desc.get_desc().cast(), res_count, flags) };
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, compare: &mut [T], compare_desc: &mut impl crate::DataDescriptor, 
            result: &mut [T], result_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.get_desc().cast(), result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, compare: &mut [T], compare_desc: &mut impl crate::DataDescriptor, 
            result: &mut [T], result_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.get_desc().cast(), result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut [crate::Ioc<T>],  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), compare_count, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut [crate::Ioc<T>],  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), compare_count, resultv.as_mut_ptr().cast() , res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicmsg<T: 'static>(&self, msg: &MsgAtomic, comparev: &[crate::Ioc<T>], compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.handle(), msg.c_msg_atomic, comparev.as_ptr().cast() , compare_desc.as_mut_ptr().cast(), compare_count, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, flags) };

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
    pub fn atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    pub fn atomicmsg(&self, msg: &MsgAtomic, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.handle(), msg.c_msg_atomic, flags) };
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic<T: 'static>(&self, buf: &[T], count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, res: &mut [T], res_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, res: &mut [T], res_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), count, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        check_error(err)
    }

    pub fn fetch_atomicmsg<T: 'static>(&self, msg: &MsgAtomic,  resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.handle(), msg.c_msg_atomic, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, flags) };
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, compare: &mut [T], compare_desc: &mut impl crate::DataDescriptor, 
            result: &mut [T], result_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.get_desc().cast(), result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_with_context<T: 'static>(&self, buf: &[T], count : usize, desc: &mut impl crate::DataDescriptor, compare: &mut [T], compare_desc: &mut impl crate::DataDescriptor, 
            result: &mut [T], result_desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc().cast(), compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.get_desc().cast(), result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.get_desc().cast(), dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut [crate::Ioc<T>],  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), compare_count, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), std::ptr::null_mut())};
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_with_context<T: 'static>(&self, iov: &[crate::Ioc<T>], desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut [crate::Ioc<T>],  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), compare_count, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, dest_addr, addr, key, to_fi_datatype::<T>(), op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicmsg<T: 'static>(&self, msg: &MsgAtomic, comparev: &[crate::Ioc<T>], compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, resultv: &mut [crate::Ioc<T>],  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.handle(), msg.c_msg_atomic, comparev.as_ptr().cast() , compare_desc.as_mut_ptr().cast(), compare_count, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), res_count, flags) };

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

pub struct MsgAtomic {
    c_msg_atomic: *mut libfabric_sys::fi_msg_atomic,
}