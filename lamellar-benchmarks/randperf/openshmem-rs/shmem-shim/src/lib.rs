#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clippy::too_many_arguments,
    clippy::useless_transmute,
    rustdoc::broken_intra_doc_links,
    missing_docs,
    improper_ctypes
)]
// the shmem_* symbols come from this crate; the extern block below only declares them
#[cfg(any(feature = "lamellar-openshmem", feature = "lamellar-openshmem-ucx", feature = "lamellar-openshmem-libfabric"))]
extern crate shmem as _;   // the lamellar-openshmem crate's library is named `shmem`

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe extern "C" {
    static shmemc_sync_all_psync: *const std::ffi::c_long;
}
