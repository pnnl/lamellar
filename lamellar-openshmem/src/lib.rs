//! OpenSHMEM 1.4 C API on top of Lamellar.
//!
//! Phase A: everything goes through Lamellar's public API. RMA to/from
//! arbitrary user pointers is staged through a registered scratch area carved
//! out of the symmetric heap (see `heap.rs`); raw zero-copy ops arrive with the
//! runtime `ffi` module (Phase B).

mod amo;
mod coll;
mod heap;
mod rma;
mod state;
mod sync;

pub use amo::*;
pub use coll::*;
pub use heap::*;
pub use rma::*;
pub use state::*;
pub use sync::*;
