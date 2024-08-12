pub(crate) fn check_error(err: isize) -> Result<(), crate::error::Error> {
    if err != 0 {
        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
    }
    else {
        Ok(())
    }
}

pub trait AsFiType {
    fn as_fi_datatype() -> libfabric_sys::fi_datatype;
}

macro_rules! impl_as_fi_type {
    ($(($rtype: ty, $fitype: path)),*) => {
        $(impl AsFiType for $rtype {
            fn as_fi_datatype() -> libfabric_sys::fi_datatype {
                $fitype
            }
        })*
    };
}

impl_as_fi_type!(
    ((), libfabric_sys::fi_datatype_FI_VOID),
    (i8, libfabric_sys::fi_datatype_FI_INT8),
    (i16, libfabric_sys::fi_datatype_FI_INT16),
    (i32, libfabric_sys::fi_datatype_FI_INT32),
    (i64, libfabric_sys::fi_datatype_FI_INT64),
    (i128, libfabric_sys::fi_datatype_FI_INT128),
    (u8, libfabric_sys::fi_datatype_FI_UINT8),
    (u16, libfabric_sys::fi_datatype_FI_UINT16),
    (u32, libfabric_sys::fi_datatype_FI_UINT32),
    (u64, libfabric_sys::fi_datatype_FI_UINT64),
    (u128, libfabric_sys::fi_datatype_FI_UINT128),
    (f32, libfabric_sys::fi_datatype_FI_FLOAT),
    (f64, libfabric_sys::fi_datatype_FI_DOUBLE)
);

impl AsFiType for usize {
    fn as_fi_datatype() -> libfabric_sys::fi_datatype {
        if std::mem::size_of::<usize>() == 8 {libfabric_sys::fi_datatype_FI_UINT64}
        else if std::mem::size_of::<usize>() == 4 {libfabric_sys::fi_datatype_FI_UINT32}
        else if std::mem::size_of::<usize>() == 2 {libfabric_sys::fi_datatype_FI_UINT16}
        else if std::mem::size_of::<usize>() == 1 {libfabric_sys::fi_datatype_FI_UINT8}
        else {panic!("Unhandled usize datatype size")}
    }
}

impl AsFiType for isize {
    fn as_fi_datatype() -> libfabric_sys::fi_datatype {
        if std::mem::size_of::<isize>() == 8 {libfabric_sys::fi_datatype_FI_INT64}
        else if std::mem::size_of::<isize>() == 4 {libfabric_sys::fi_datatype_FI_INT32}
        else if std::mem::size_of::<isize>() == 2 {libfabric_sys::fi_datatype_FI_INT16}
        else if std::mem::size_of::<isize>() == 1 {libfabric_sys::fi_datatype_FI_INT8}
        else {panic!("Unhandled isize datatype size")}
    }
}

pub(crate) fn error_to_string(errnum: i64) -> String {
    let ret = unsafe { libfabric_sys::fi_strerror(errnum as i32) };
    let str = unsafe { std::ffi::CStr::from_ptr(ret) };
    str.to_str().unwrap().to_string()
}

pub enum Either<L, R> {
    Left(L),
    Right(R)
}