use crate::error::Error;

pub mod eq;
pub mod domain;
pub mod av;
pub mod ep;
pub mod mr;
pub mod comm;
pub mod cq;

pub(crate) struct AsyncCtx {
    pub(crate) user_ctx: Option<*mut std::ffi::c_void>,
}

pub(crate) trait AsyncFid {
    fn trywait(&self) -> Result<(), Error>;
}