use crate::ep::Endpoint;
use crate::ep::BaseEndpoint;

impl Endpoint {

    #[allow(clippy::too_many_arguments)]
    pub fn atomic<T0,T1>(&self, buf: &[T0], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomic_with_context<T0,T1>(&self, buf: &[T0], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn atomicv_with_context<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.handle(), iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn atomicmsg(&self, msg: &MsgAtomic, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.handle(), msg.c_msg_atomic, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_atomic<T0,T1>(&self, buf: &[T0], count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, dest_addr, addr, key, datatype, op.get_value())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic<T0,T1>(&self, buf: &[T0], count : usize, desc: &mut [T1], res: &mut [T0], res_desc: &mut [T1], dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.as_mut_ptr() as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomic_with_context<T0,T1>(&self, buf: &[T0], count : usize, desc: &mut [T1], res: &mut [T0], res_desc: &mut [T1], dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.as_mut_ptr() as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [T1], count : usize, resultv: &mut crate::Ioc,  res_desc: &mut [T1], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fetch_atomicv_with_context<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [T1], count : usize, resultv: &mut crate::Ioc,  res_desc: &mut [T1], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.handle(), iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn fetch_atomicmsg<T0>(&self, msg: &MsgAtomic,  resultv: &mut crate::Ioc,  res_desc: &mut [T0], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.handle(), msg.c_msg_atomic, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, flags) };
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic<T0, T1>(&self, buf: &[T0], count : usize, desc: &mut [T1], compare: &mut [T0], compare_desc: &mut [T1], 
            result: &mut [T0], result_desc: &mut [T1], dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.as_mut_ptr()  as *mut std::ffi::c_void, result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.as_mut_ptr()  as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomic_with_context<T0, T1>(&self, buf: &[T0], count : usize, desc: &mut [T1], compare: &mut [T0], compare_desc: &mut [T1], 
            result: &mut [T0], result_desc: &mut [T1], dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.handle(), buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.as_mut_ptr()  as *mut std::ffi::c_void, result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.as_mut_ptr()  as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv(&self, iov: &crate::Ioc, desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut crate::Ioc,  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut crate::Ioc,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.get_mut(), compare_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, compare_count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicv_with_context(&self, iov: &crate::Ioc, desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut crate::Ioc,  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut crate::Ioc,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.handle(), iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.get_mut(), compare_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, compare_count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_atomicmsg<T0>(&self, msg: &MsgAtomic, comparev: &crate::Ioc, compare_desc: &mut [T0], compare_count : usize, resultv: &mut crate::Ioc,  res_desc: &mut [T0], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.handle(), msg.c_msg_atomic, comparev.get(), compare_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, compare_count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn atomicvalid(&self, datatype: crate::DataType, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_atomicvalid(self.handle(), datatype, op.get_value(), &mut count as *mut usize)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn fetch_atomicvalid(&self, datatype: crate::DataType, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_fetch_atomicvalid(self.handle(), datatype, op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn compare_atomicvalid(&self, datatype: crate::DataType, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_compare_atomicvalid(self.handle(), datatype, op.get_value(), &mut count as *mut usize)};

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