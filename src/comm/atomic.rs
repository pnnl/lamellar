use std::any::TypeId;
use crate::DataType;
use crate::check_error;
use crate::ep::Endpoint;
use crate::ep::ActiveEndpoint;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;

fn to_fi_datatype<T: 'static>() -> DataType {
    let isize_t: TypeId = TypeId::of::<isize>();
    let usize_t: TypeId = TypeId::of::<usize>();
    let i8_t: TypeId = TypeId::of::<i8>();
    let i16_t: TypeId = TypeId::of::<i16>();
    let i32_t: TypeId = TypeId::of::<i32>();
    let i64_t: TypeId = TypeId::of::<i64>();
    let i128_t: TypeId = TypeId::of::<i128>();
    let u8_t: TypeId = TypeId::of::<u8>();
    let u16_t: TypeId = TypeId::of::<u16>();
    let u32_t: TypeId = TypeId::of::<u32>();
    let u64_t: TypeId = TypeId::of::<u64>();
    let u128_t: TypeId = TypeId::of::<u128>();
    let f32_t: TypeId = TypeId::of::<f32>();
    let f64_t: TypeId = TypeId::of::<f64>();

    if TypeId::of::<T>()  == isize_t{
        if std::mem::size_of::<isize>() == 8 {libfabric_sys::fi_datatype_FI_INT64}
        else if std::mem::size_of::<isize>() == 4 {libfabric_sys::fi_datatype_FI_INT32}
        else if std::mem::size_of::<isize>() == 2 {libfabric_sys::fi_datatype_FI_INT16}
        else if std::mem::size_of::<isize>() == 1 {libfabric_sys::fi_datatype_FI_INT8}
        else {panic!("Unhandled isize datatype size")}
    }
    else if TypeId::of::<T>() == usize_t {
        if std::mem::size_of::<usize>() == 8 {libfabric_sys::fi_datatype_FI_UINT64}
        else if std::mem::size_of::<usize>() == 4 {libfabric_sys::fi_datatype_FI_UINT32}
        else if std::mem::size_of::<usize>() == 2 {libfabric_sys::fi_datatype_FI_UINT16}
        else if std::mem::size_of::<usize>() == 1 {libfabric_sys::fi_datatype_FI_UINT8}
        else {panic!("Unhandled usize datatype size")}
    }
    else if TypeId::of::<T>() == i8_t {libfabric_sys::fi_datatype_FI_INT8}
    else if TypeId::of::<T>() == i16_t {libfabric_sys::fi_datatype_FI_INT16}
    else if TypeId::of::<T>() == i32_t {libfabric_sys::fi_datatype_FI_INT32}
    else if TypeId::of::<T>() == i64_t {libfabric_sys::fi_datatype_FI_INT64}
    else if TypeId::of::<T>() == i128_t {libfabric_sys::fi_datatype_FI_INT128}
    else if TypeId::of::<T>() == u8_t {libfabric_sys::fi_datatype_FI_UINT8}
    else if TypeId::of::<T>() == u16_t {libfabric_sys::fi_datatype_FI_UINT16}
    else if TypeId::of::<T>() == u32_t {libfabric_sys::fi_datatype_FI_UINT32}
    else if TypeId::of::<T>() == u64_t {libfabric_sys::fi_datatype_FI_UINT64}
    else if TypeId::of::<T>() == u128_t {libfabric_sys::fi_datatype_FI_UINT128}
    else if TypeId::of::<T>() == f32_t {libfabric_sys::fi_datatype_FI_FLOAT}
    else if TypeId::of::<T>() == f64_t {libfabric_sys::fi_datatype_FI_DOUBLE}
    else {panic!("Type not supported")}
}

impl Endpoint {

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