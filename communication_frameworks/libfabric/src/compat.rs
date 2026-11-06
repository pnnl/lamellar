//! Items that exist in some libfabric releases but not others (see lamellar-libfabric-sys's build script, which decides
//! the `ofi_const_*` cfgs from the generated bindings). When a constant is missing the flag is 0, so setting it is a
//! no-op and testing it is always false.

#[cfg(ofi_const_fi_reg_mr)]
pub(crate) const FI_REG_MR: u64 = libfabric_sys::FI_REG_MR;
#[cfg(not(ofi_const_fi_reg_mr))]
pub(crate) const FI_REG_MR: u64 = 0;

#[cfg(ofi_const_fi_local_mr)]
pub(crate) const FI_LOCAL_MR: u64 = libfabric_sys::FI_LOCAL_MR;
#[cfg(not(ofi_const_fi_local_mr))]
pub(crate) const FI_LOCAL_MR: u64 = 0;

#[cfg(ofi_const_fi_buffered_recv)]
pub(crate) const FI_BUFFERED_RECV: u64 = libfabric_sys::FI_BUFFERED_RECV;
#[cfg(not(ofi_const_fi_buffered_recv))]
pub(crate) const FI_BUFFERED_RECV: u64 = 0;

#[cfg(ofi_const_fi_order_strict)]
pub(crate) const FI_ORDER_STRICT: u64 = libfabric_sys::FI_ORDER_STRICT as u64;
#[cfg(not(ofi_const_fi_order_strict))]
pub(crate) const FI_ORDER_STRICT: u64 = 0;

#[cfg(ofi_const_fi_order_data)]
pub(crate) const FI_ORDER_DATA: u64 = libfabric_sys::FI_ORDER_DATA as u64;
#[cfg(not(ofi_const_fi_order_data))]
pub(crate) const FI_ORDER_DATA: u64 = 0;

#[cfg(ofi_const_fi_mr_unspec)]
pub(crate) const FI_MR_UNSPEC: u32 = libfabric_sys::fi_mr_mode_FI_MR_UNSPEC;
#[cfg(not(ofi_const_fi_mr_unspec))]
pub(crate) const FI_MR_UNSPEC: u32 = 0;

#[cfg(ofi_const_fi_mr_basic)]
pub(crate) const FI_MR_BASIC: u32 = libfabric_sys::fi_mr_mode_FI_MR_BASIC;
#[cfg(not(ofi_const_fi_mr_basic))]
pub(crate) const FI_MR_BASIC: u32 = 0;

#[cfg(ofi_const_fi_mr_scalable)]
pub(crate) const FI_MR_SCALABLE: u32 = libfabric_sys::fi_mr_mode_FI_MR_SCALABLE;
#[cfg(not(ofi_const_fi_mr_scalable))]
pub(crate) const FI_MR_SCALABLE: u32 = 0;

// 1.x: FI_INT128 == FI_DATATYPE_LAST (a compatibility alias); releases that dropped the alias use FI_INT128 itself.
#[cfg(ofi_const_fi_datatype_last)]
pub(crate) const FI_DATATYPE_LAST: libfabric_sys::fi_datatype = libfabric_sys::fi_datatype_FI_DATATYPE_LAST;
#[cfg(not(ofi_const_fi_datatype_last))]
pub(crate) const FI_DATATYPE_LAST: libfabric_sys::fi_datatype = libfabric_sys::fi_datatype_FI_INT128;
