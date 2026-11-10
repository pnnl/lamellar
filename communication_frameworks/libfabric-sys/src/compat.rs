//! Helpers for the places where the libfabric C API differs between releases and the bindgen output therefore does too.
//! The `ofi_*` cfgs are decided by this crate's build script from what was actually generated (see `emit_api_cfgs`).
//! Code that fills these structs in directly (rather than through the `lamellar-libfabric` wrapper) can use these
//! instead of naming version-specific fields and constants.

use crate::{fi_domain_attr, fi_progress};

/// Sets `data_progress` in a `fi_domain_attr`: a plain field in libfabric 1.x, inside an anonymous union in 2.x.
///
/// # Safety
/// `attr` must point to a valid, writable `fi_domain_attr`.
#[cfg(ofi_domain_data_progress_flat)]
pub unsafe fn set_domain_data_progress(attr: *mut fi_domain_attr, progress: fi_progress) {
    (*attr).data_progress = progress;
}

/// Sets `data_progress` in a `fi_domain_attr`: a plain field in libfabric 1.x, inside an anonymous union in 2.x.
///
/// # Safety
/// `attr` must point to a valid, writable `fi_domain_attr`.
#[cfg(not(ofi_domain_data_progress_flat))]
pub unsafe fn set_domain_data_progress(attr: *mut fi_domain_attr, progress: fi_progress) {
    (*attr).__bindgen_anon_1.data_progress = progress;
}

/// `FI_MR_BASIC`, the legacy memory-registration mode. libfabric 2.x removed it, so there it is 0 (no bits set).
#[cfg(ofi_const_fi_mr_basic)]
pub const FI_MR_BASIC: u32 = crate::fi_mr_mode_FI_MR_BASIC;
/// `FI_MR_BASIC`, the legacy memory-registration mode. libfabric 2.x removed it, so there it is 0 (no bits set).
#[cfg(not(ofi_const_fi_mr_basic))]
pub const FI_MR_BASIC: u32 = 0;
