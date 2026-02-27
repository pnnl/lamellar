pub(crate) fn check_error(err: isize) -> Result<(), crate::error::Error> {
    if err != 0 {
        Err(crate::error::Error::from_err_code(
            (-err).try_into().unwrap(),
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn error_to_string(errnum: i64) -> String {
    let ret = unsafe { libfabric_sys::fi_strerror(errnum as i32) };
    let str = unsafe { std::ffi::CStr::from_ptr(ret) };
    str.to_str().unwrap().to_string()
}

#[derive(Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
