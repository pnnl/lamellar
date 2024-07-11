use std::any::TypeId;
use crate::DataType;

pub(crate) fn check_error(err: isize) -> Result<(), crate::error::Error> {
    if err != 0 {
        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
    }
    else {
        Ok(())
    }
}


pub(crate) fn to_fi_datatype<T: 'static>() -> DataType {
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
    let void_t: TypeId = TypeId::of::<()>();

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
    else if TypeId::of::<T>() == void_t {libfabric_sys::fi_datatype_FI_VOID}
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


pub(crate) fn error_to_string(errnum: i64) -> String {
    let ret = unsafe { libfabric_sys::fi_strerror(errnum as i32) };
    let str = unsafe { std::ffi::CStr::from_ptr(ret) };
    str.to_str().unwrap().to_string()
}