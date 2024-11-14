use crate::{error::Error, Context, RawContext};

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


pub trait AsyncFid {
    fn trywait(&self) -> Result<(), Error>;
}
