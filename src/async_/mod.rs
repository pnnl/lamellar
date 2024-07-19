use crate::error::Error;

pub mod eq;
pub mod domain;
pub mod av;
pub mod ep;
pub mod mr;
pub mod comm;
pub mod cq;

pub struct AsyncCtx {
    pub(crate) user_ctx: Option<*mut std::ffi::c_void>,
}

pub trait AsyncFid {
    fn trywait(&self) -> Result<(), Error>;
}