use crate::error::Error;

pub mod av;
pub mod comm;
pub mod conn_ep;
pub mod connless_ep;
pub mod cq;
pub mod domain;
pub mod ep;
pub mod eq;
pub mod mr;
mod xcontext;

// [TODO] Remove user ctx and replace with a proper Context
pub struct AsyncCtx {
    pub(crate) user_ctx: Option<*mut std::ffi::c_void>,
}

pub trait AsyncFid {
    fn trywait(&self) -> Result<(), Error>;
}
