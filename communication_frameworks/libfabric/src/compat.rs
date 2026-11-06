//! Items that exist in libfabric 1.x headers but were removed in 2.x. Under `cfg(ofi2)` the flag constants are 0, so
//! setting them is a no-op and testing them is always false.

#[cfg(not(ofi2))]
mod v {
    pub(crate) const FI_REG_MR: u64 = libfabric_sys::FI_REG_MR;
    pub(crate) const FI_LOCAL_MR: u64 = libfabric_sys::FI_LOCAL_MR;
    pub(crate) const FI_BUFFERED_RECV: u64 = libfabric_sys::FI_BUFFERED_RECV;
    pub(crate) const FI_ORDER_STRICT: u64 = libfabric_sys::FI_ORDER_STRICT as u64;
    pub(crate) const FI_ORDER_DATA: u64 = libfabric_sys::FI_ORDER_DATA as u64;
    pub(crate) const FI_MR_UNSPEC: u32 = libfabric_sys::fi_mr_mode_FI_MR_UNSPEC;
    pub(crate) const FI_MR_BASIC: u32 = libfabric_sys::fi_mr_mode_FI_MR_BASIC;
    pub(crate) const FI_MR_SCALABLE: u32 = libfabric_sys::fi_mr_mode_FI_MR_SCALABLE;
    pub(crate) const FI_DATATYPE_LAST: libfabric_sys::fi_datatype = libfabric_sys::fi_datatype_FI_DATATYPE_LAST;
}

#[cfg(ofi2)]
mod v {
    pub(crate) const FI_REG_MR: u64 = 0;
    pub(crate) const FI_LOCAL_MR: u64 = 0;
    pub(crate) const FI_BUFFERED_RECV: u64 = 0;
    pub(crate) const FI_ORDER_STRICT: u64 = 0;
    pub(crate) const FI_ORDER_DATA: u64 = 0;
    pub(crate) const FI_MR_UNSPEC: u32 = 0;
    pub(crate) const FI_MR_BASIC: u32 = 0;
    pub(crate) const FI_MR_SCALABLE: u32 = 0;
    // 1.x: FI_INT128 == FI_DATATYPE_LAST (a compatibility alias); 2.x dropped the alias.
    pub(crate) const FI_DATATYPE_LAST: libfabric_sys::fi_datatype = libfabric_sys::fi_datatype_FI_INT128;
}

pub(crate) use v::*;
