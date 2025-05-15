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
pub mod xcontext;

pub trait AsyncFid {
    fn trywait(&self) -> Result<(), Error>;
}
