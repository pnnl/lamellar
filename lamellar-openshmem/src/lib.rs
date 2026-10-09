//! OpenSHMEM 1.4 C API on top of Lamellar.
//!
//! Everything goes through Lamellar's public memory-region API: the symmetric heap is a `SharedMemoryRegion`, the
//! program's `.data`/`.bss` is registered with `try_register_external_shared_mem_region`, and RMA to/from arbitrary
//! user pointers uses the region's pointer put/get (see `sym.rs`).

mod amo;
mod coll;
mod heap;
mod rma;
mod state;
mod sym;
mod sync;

pub use amo::*;
pub use coll::*;
pub use heap::*;
pub use rma::*;
pub use state::*;
pub use sync::*;
