#![warn(missing_docs)]
#![warn(unreachable_pub)]

//! PMI crate
//!
//! Provides a thin abstraction over multiple PMI backends used by examples in
//! this workspace. Features enable specific backends:
//!
//! - `with-pmi1`: enable PMI1 backend
//! - `with-pmi2`: enable PMI2 backend
//! - `with-pmix`: enable PMIx backend
//!
//! Behavior notes:
//! - Node detection: when backends don't provide stable node indices, ranks
//!   exchange hostnames and the crate derives contiguous node ids by sorting
//!   and deduplicating hostnames.
//! - Job id: numeric job id strings are preferred and returned directly; if a
//!   job id is non-numeric the crate computes a deterministic `usize` via
//!   hashing.

#[cfg(not(any(feature = "with-pmi1", feature = "with-pmi2", feature = "with-pmix")))]
compile_error!(
    "At least one of the features 'with-pmi1', 'with-pmi2' or 'with-pmix' must be enabled"
);
/// Core PMI trait, types and helpers.
pub mod pmi;
#[cfg(feature = "with-pmi1")]
/// PMI1 backend implementation.
pub mod pmi1;
#[cfg(feature = "with-pmi2")]
/// PMI2 backend implementation.
pub mod pmi2;
#[cfg(feature = "with-pmix")]
/// PMIx backend implementation.
pub mod pmix;

/// Re-exported for crate users: core PMI trait and helpers.
#[doc(no_inline)]
pub use crate::pmi::*;
