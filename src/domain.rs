use crate::fid::{AsTypedFid, BorrowedTypedFid};
#[allow(unused_imports)]
// use crate::fid::AsFid;
use crate::{
    enums::{
        AddressVectorType, AtomicOperation, DomainCaps, Mode, MrMode, Progress, ResourceMgmt,
        Threading, TrafficClass,
    },
    eq::{EventQueue, EventQueueBase, ReadEq},
    fabric::FabricImpl,
    fid::{self, AsRawFid, AsRawTypedFid, DomainRawFid, OwnedDomainFid},
    info::InfoEntry,
    utils::check_error,
    AsFiType, Context, MyOnceCell, MyRc, SyncSend,
};
use core::slice;
use std::ffi::CString;
pub struct NoEventQueue {}
impl SyncSend for NoEventQueue {}

pub(crate) struct DomainImplBase<EQ: ?Sized> {
    pub(crate) c_domain: OwnedDomainFid,
    pub(crate) mr_mode: MrMode,
    pub(crate) mr_key_size: usize,
    pub(crate) _eq_rc: MyOnceCell<(MyRc<EQ>, bool)>,
    _fabric_rc: MyRc<FabricImpl>,
}
impl<EQ: ?Sized + SyncSend> SyncSend for DomainImplBase<EQ> {}

pub(crate) trait DomainImplT: AsTypedFid<DomainRawFid> + SyncSend {
    fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error>;
    #[allow(unused)]
    fn fabric_impl(&self) -> MyRc<FabricImpl>;
    fn mr_mode(&self) -> MrMode;
    fn mr_key_size(&self) -> usize;
}

impl<EQ: ?Sized + SyncSend> DomainImplT for DomainImplBase<EQ> {
    fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        self.unmap_key(key)
    }

    fn fabric_impl(&self) -> MyRc<FabricImpl> {
        self._fabric_rc.clone()
    }

    fn mr_mode(&self) -> MrMode {
        self.mr_mode
    }

    fn mr_key_size(&self) -> usize {
        self.mr_key_size
    }
}

//================== Domain (fi_domain) ==================//

impl<EQ: ?Sized> DomainImplBase<EQ> {
    pub(crate) fn new<E>(
        fabric: &MyRc<crate::fabric::FabricImpl>,
        info: &InfoEntry<E>,
        flags: u64,
        domain_attr: DomainAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_domain: DomainRawFid = std::ptr::null_mut();
        let err = if flags == 0 {
            unsafe {
                libfabric_sys::inlined_fi_domain(
                    fabric.as_typed_fid_mut().as_raw_typed_fid(),
                    info.info.as_raw(),
                    &mut c_domain,
                    context,
                )
            }
        } else {
            unsafe {
                libfabric_sys::inlined_fi_domain2(
                    fabric.as_typed_fid_mut().as_raw_typed_fid(),
                    info.info.as_raw(),
                    &mut c_domain,
                    flags,
                    context,
                )
            }
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                #[cfg(feature = "threading-domain")]
                c_domain: OwnedDomainFid::from(
                    c_domain,
                    std::sync::Arc::new(parking_lot::Mutex::new(fid::TypedFid(c_domain))),
                ),
                #[cfg(not(feature = "threading-domain"))]
                c_domain: OwnedDomainFid::from(c_domain),
                mr_key_size: domain_attr.mr_key_size,
                mr_mode: domain_attr.mr_mode,
                _fabric_rc: fabric.clone(),
                _eq_rc: MyOnceCell::new(),
            })
        }
    }
}

impl DomainImplBase<dyn ReadEq> {
    pub(crate) fn bind(
        &self,
        eq: MyRc<dyn ReadEq>,
        async_mem_reg: bool,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_domain_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                eq.as_typed_fid().as_raw_fid(),
                if async_mem_reg {
                    libfabric_sys::FI_REG_MR
                } else {
                    0
                },
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self._eq_rc.set((eq, async_mem_reg)).is_err() {
                panic!("Domain is alread bound to an EventQueue");
            }
            Ok(())
        }
    }
}

impl<EQ: ?Sized> DomainImplBase<EQ> {
    // pub(crate) fn srx_context<T0>(&self, rx_attr: crate::RxAttr) -> Result<crate::ep::Endpoint, crate::error::Error> { //[TODO]
    //     crate::ep::Endpoint::from_attr(self, rx_attr)
    // }

    // pub(crate) fn srx_context_with_context<T0>(&self, rx_attr: crate::RxAttr, context: &mut T0) -> Result<crate::ep::Endpoint, crate::error::Error> { //[TODO]
    //     crate::ep::Endpoint::from_attr_with_context(self, rx_attr, context)
    // }

    pub(crate) fn query_atomic<T: AsFiType>(
        &self,
        op: impl AtomicOperation,
        mut attr: crate::comm::atomic::AtomicAttr,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_query_atomic(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                T::as_fi_datatype(),
                op.as_raw(),
                attr.get_mut(),
                flags,
            )
        };

        check_error(err.try_into().unwrap())
    }

    pub(crate) fn map_raw(
        &self,
        mr_key: &mut crate::mr::OwnedMemoryRegionKey,
        flags: u64,
    ) -> Result<u64, crate::error::Error> {
        let mut mapped_key = 0;
        let err = match mr_key {
            crate::mr::OwnedMemoryRegionKey::Key(simple_key) => {
                return Ok(*simple_key);
                // unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.handle(), base_addr, simple_key as *mut u64 as *mut u8, std::mem::size_of::<u64>(), &mut mapped_key, flags) }
            }
            crate::mr::OwnedMemoryRegionKey::RawKey(raw_key) => unsafe {
                libfabric_sys::inlined_fi_mr_map_raw(
                    self.as_typed_fid_mut().as_raw_typed_fid(),
                    raw_key.1,
                    raw_key.0.as_mut_ptr().cast(),
                    raw_key.0.len(),
                    &mut mapped_key,
                    flags,
                )
            },
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(mapped_key)
        }
    }

    // pub fn map_raw(&self, base_addr: u64, raw_key: &mut u8, key_size: usize, key: &mut u64, flags: u64) -> Result<(), crate::error::Error> {
    //     let err = unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.as_raw_typed_fid(), base_addr, raw_key, key_size, key, flags) };

    //     check_error(err.try_into().unwrap())
    // }

    pub(crate) fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_unmap_key(self.as_typed_fid_mut().as_raw_typed_fid(), key)
        };

        check_error(err.try_into().unwrap())
    }

    // pub fn stx_context<T0>(&self, attr: crate::TxAttr) -> Result<crate::Stx, crate::error::Error> { //[TODO]
    //     crate::Stx::new(self, attr, std::ptr::null_mut())
    // }

    // pub fn stx_context_with_context<T0>(&self, attr: crate::TxAttr , context: &mut T0) -> Result<crate::Stx, crate::error::Error> { //[TODO]
    //     crate::Stx::new(self, attr, context)
    // }

    pub(crate) fn query_collective<T: AsFiType>(
        &self,
        coll: crate::enums::CollectiveOp,
        attr: &mut crate::comm::collective::CollectiveAttr<T>,
    ) -> Result<bool, crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_query_collective(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                coll.as_raw(),
                attr.get_mut(),
                0,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(true)
        }
    }

    pub(crate) fn query_collective_scatter<T: AsFiType>(
        &self,
        coll: crate::enums::CollectiveOp,
        attr: &mut crate::comm::collective::CollectiveAttr<T>,
    ) -> Result<bool, crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_query_collective(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                coll.as_raw(),
                attr.get_mut(),
                libfabric_sys::fi_collective_op_FI_SCATTER.into(),
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(true)
        }
    }
}

/// Owned wrapper around a libfabric `fid_domain`.
///
/// This type wraps an instance of a `fid_domain`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.22.0/man/fi_domain.3.html).
///
/// Note that other objects that rely on a Domain (e.g., [`Endpoint`](crate::ep::Endpoint)) will extend its lifetime until they
/// are also dropped.
// pub type Domain = DomainBase<dyn EventQueueImplT>;

pub struct DomainBase<EQ: ?Sized> {
    pub(crate) inner: MyRc<DomainImplBase<EQ>>,
}

impl<EQ: ?Sized + SyncSend> SyncSend for DomainBase<EQ> {}

impl<EQ: ?Sized + SyncSend> DomainImplT for DomainBase<EQ> {
    fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        self.inner.unmap_key(key)
    }

    fn fabric_impl(&self) -> MyRc<FabricImpl> {
        self.inner.fabric_impl()
    }

    fn mr_mode(&self) -> MrMode {
        self.inner.mr_mode()
    }

    fn mr_key_size(&self) -> usize {
        self.inner.mr_key_size()
    }
}

impl<EQ: ?Sized> DomainBase<EQ> {
    pub(crate) fn new<E>(
        fabric: &crate::fabric::Fabric,
        info: &InfoEntry<E>,
        flags: u64,
        domain_attr: DomainAttr,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(DomainImplBase::new(
                &fabric.inner,
                info,
                flags,
                domain_attr,
                c_void,
            )?),
        })
    }
}

impl DomainBase<dyn ReadEq> {
    /// Associates an [crate::eq::EventQueue] with the domain.
    ///
    /// If `async_mem_reg` is true, the provider should perform all memory registration operations asynchronously, with the completion reported through the event queue
    ///
    /// Corresponds to `fi_domain_bind`, with flag `FI_REG_MR` if `async_mem_reg` is true.
    pub(crate) fn bind_eq<EQ: ReadEq + 'static>(
        &self,
        eq: &EventQueueBase<EQ>,
        async_mem_reg: bool,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind(eq.inner.clone(), async_mem_reg)
    }
}

impl<EQ: ?Sized> DomainBase<EQ> {
    /// Indicates if a provider supports a specific atomic operation
    ///
    /// Returns true if the provider supports operations `op` for datatype `T` and atomic ops as reflected in `flags`.
    ///
    /// Corresponds to `fi_query_atomic` with `datatype` automatically inferred from `T`.
    pub fn query_atomic<T: AsFiType>(
        &self,
        op: impl AtomicOperation,
        attr: crate::comm::atomic::AtomicAttr,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        //[TODO] Flags

        self.inner.query_atomic::<T>(op, attr, flags)
    }

    pub(crate) fn map_raw(
        &self,
        mr_key: &mut crate::mr::OwnedMemoryRegionKey,
        flags: u64,
    ) -> Result<u64, crate::error::Error> {
        self.inner.map_raw(mr_key, flags)
    }

    #[allow(dead_code)]
    pub(crate) fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        self.inner.unmap_key(key)
    }

    /// Returns information about which collective operations are supported by a provider, and limitations on the collective.
    ///
    /// Direclty corresponds to `fi_query_collective` with `flags` = 0
    pub fn query_collective<T: AsFiType>(
        &self,
        coll: crate::enums::CollectiveOp,
        attr: &mut crate::comm::collective::CollectiveAttr<T>,
    ) -> Result<bool, crate::error::Error> {
        self.inner.query_collective::<T>(coll, attr)
    }

    /// Requests attribute information on the reduce-scatter collective operation.
    ///
    /// Direclty corresponds to `fi_query_collective` with `flags` = `FI_SCATTER`
    pub fn query_collective_scatter<T: AsFiType>(
        &self,
        coll: crate::enums::CollectiveOp,
        attr: &mut crate::comm::collective::CollectiveAttr<T>,
    ) -> Result<bool, crate::error::Error> {
        self.inner.query_collective_scatter::<T>(coll, attr)
    }
}

// impl<EQ: ?Sized> AsFid for DomainImplBase<EQ> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.c_domain.as_fid()
//     }
// }

impl<EQ: ?Sized> AsTypedFid<DomainRawFid> for DomainImplBase<EQ> {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<'_, DomainRawFid> {
        self.c_domain.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> fid::MutBorrowedTypedFid<'_, DomainRawFid> {
        self.c_domain.as_typed_fid_mut()
    }
}

// impl<EQ: ?Sized> AsRawTypedFid for DomainImplBase<EQ> {
//     type Output = DomainRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.c_domain.as_raw_typed_fid()
//     }
// }

// impl<EQ> AsFid for DomainBase<EQ> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.inner.as_fid()
//     }
// }

// impl<EQ: ?Sized> AsRawFid for DomainImplBase<EQ> {
//     fn as_raw_fid(&self) -> fid::RawFid {
//         self.c_domain.as_raw_fid()
//     }
// }

// impl<EQ: ?Sized> AsRawFid for DomainBase<EQ> {
//     fn as_raw_fid(&self) -> fid::RawFid {
//         self.inner.as_raw_fid()
//     }
// }

impl<EQ: ?Sized> AsTypedFid<DomainRawFid> for DomainBase<EQ> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<'_, DomainRawFid> {
        self.inner.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<'_, DomainRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

// impl<EQ: ?Sized> AsRawTypedFid for DomainBase<EQ> {
//     type Output = DomainRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.inner.as_raw_typed_fid()
//     }
// }

//================== Domain attribute ==================//
/// Represents a read-only version of the a `fi_info`'s `fi_domain_attr` field returned by
/// a call to `fi_getinfo()`  
#[derive(Clone, Debug)]
pub struct DomainAttr {
    _c_name: CString,
    domain_id: usize,
    name: String,
    threading: crate::enums::Threading,
    control_progress: crate::enums::Progress,
    data_progress: crate::enums::Progress,
    resource_mgmt: crate::enums::ResourceMgmt,
    av_type: crate::enums::AddressVectorType,
    mr_mode: crate::enums::MrMode,
    mr_key_size: usize,
    cq_data_size: usize,
    cq_cnt: usize,
    ep_cnt: usize,
    tx_ctx_cnt: usize,
    rx_ctx_cnt: usize,
    max_ep_tx_ctx: usize,
    max_ep_rx_ctx: usize,
    max_ep_stx_ctx: usize,
    max_ep_srx_ctx: usize,
    cntr_cnt: usize,
    mr_iov_limit: usize,
    caps: crate::enums::DomainCaps,
    mode: crate::enums::Mode,
    auth_key: Option<Vec<u8>>,
    max_err_data: usize,
    mr_cnt: usize,
    traffic_class: crate::enums::TrafficClass,
    max_ep_auth_key: usize,
}

impl DomainAttr {
    pub(crate) fn from_raw_ptr(value: *const libfabric_sys::fi_domain_attr) -> Self {
        assert!(!value.is_null());
        let c_name = if !unsafe { *value }.name.is_null() {
            unsafe { std::ffi::CStr::from_ptr((*value).name) }.into()
        } else {
            CString::new("").unwrap()
        };
        Self {
            domain_id: unsafe { *value }.domain as usize,
            name: c_name.to_str().unwrap().to_string(),
            _c_name: c_name,
            threading: crate::enums::Threading::from_raw(unsafe { *value }.threading),
            control_progress: crate::enums::Progress::from_raw(unsafe { *value }.control_progress),
            data_progress: crate::enums::Progress::from_raw(unsafe { *value }.data_progress),
            resource_mgmt: crate::enums::ResourceMgmt::from_raw(unsafe { *value }.resource_mgmt),
            av_type: crate::enums::AddressVectorType::from_raw(unsafe { *value }.av_type),
            mr_mode: crate::enums::MrMode::from_raw(unsafe { *value }.mr_mode as u32),
            mr_key_size: unsafe { *value }.mr_key_size,
            cq_data_size: unsafe { *value }.cq_data_size,
            cq_cnt: unsafe { *value }.cq_cnt,
            ep_cnt: unsafe { *value }.ep_cnt,
            tx_ctx_cnt: unsafe { *value }.tx_ctx_cnt,
            rx_ctx_cnt: unsafe { *value }.rx_ctx_cnt,
            max_ep_tx_ctx: unsafe { *value }.max_ep_tx_ctx,
            max_ep_rx_ctx: unsafe { *value }.max_ep_rx_ctx,
            max_ep_stx_ctx: unsafe { *value }.max_ep_stx_ctx,
            max_ep_srx_ctx: unsafe { *value }.max_ep_srx_ctx,
            cntr_cnt: unsafe { *value }.cntr_cnt,
            mr_iov_limit: unsafe { *value }.mr_iov_limit,
            caps: crate::enums::DomainCaps::from_raw(unsafe { *value }.caps),
            mode: crate::enums::Mode::from_raw(unsafe { *value }.mode),
            auth_key: {
                if !unsafe { *value }.auth_key.is_null() {
                    Some(
                        unsafe { slice::from_raw_parts((*value).auth_key, (*value).auth_key_size) }
                            .to_vec(),
                    )
                } else {
                    None
                }
            },
            max_err_data: unsafe { *value }.max_err_data,
            mr_cnt: unsafe { *value }.mr_cnt,
            traffic_class: crate::enums::TrafficClass::from_raw(unsafe { *value }.tclass),
            max_ep_auth_key: unsafe { *value }.max_ep_auth_key,
        }
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn get(&self) -> libfabric_sys::fi_domain_attr {
        libfabric_sys::fi_domain_attr {
            domain: self.domain_id as *mut libfabric_sys::fid_domain,
            name: std::mem::transmute::<*const i8, *mut i8>(self._c_name.as_ptr()),
            threading: self.threading.as_raw(),
            control_progress: self.control_progress.as_raw(),
            data_progress: self.data_progress.as_raw(),
            resource_mgmt: self.resource_mgmt.as_raw(),
            mr_key_size: self.mr_key_size,
            cq_data_size: self.cq_data_size,
            cq_cnt: self.cq_cnt,
            ep_cnt: self.ep_cnt,
            tx_ctx_cnt: self.tx_ctx_cnt,
            rx_ctx_cnt: self.rx_ctx_cnt,
            max_ep_tx_ctx: self.max_ep_tx_ctx,
            max_ep_rx_ctx: self.max_ep_rx_ctx,
            max_ep_stx_ctx: self.max_ep_stx_ctx,
            max_ep_srx_ctx: self.max_ep_srx_ctx,
            cntr_cnt: self.cntr_cnt,
            mr_iov_limit: self.mr_iov_limit,
            max_err_data: self.max_err_data,
            mr_cnt: self.mr_cnt,
            av_type: self.av_type.as_raw(),
            mr_mode: self.mr_mode.as_raw() as i32,
            caps: self.caps.as_raw(),
            mode: self.mode.as_raw(),
            tclass: self.traffic_class.as_raw(),
            max_ep_auth_key: self.max_ep_auth_key,
            auth_key: if let Some(auth_key) = &self.auth_key {
                std::mem::transmute::<*const u8, *mut u8>(auth_key.as_ptr())
            } else {
                std::ptr::null_mut()
            },
            auth_key_size: if let Some(auth_key) = &self.auth_key {
                auth_key.len()
            } else {
                0
            },
        }
    }

    /// Returns the domain ID.
    ///
    /// Corresponds to accessing the `fi_domain_attr::domain` field.
    pub fn domain_id(&self) -> usize {
        self.domain_id
    }

    /// Returns the name of the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::name` field.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the threading model used by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::threading` field.
    pub fn threading(&self) -> &Threading {
        &self.threading
    }

    /// Returns the progress model used for control operations in the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::control_progress` field.
    pub fn control_progress(&self) -> &Progress {
        &self.control_progress
    }

    /// Returns the progress model used for data operations in the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::data_progress` field.
    pub fn data_progress(&self) -> &Progress {
        &self.data_progress
    }

    /// Returns the resource management model used by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::resource_mgmt` field.
    pub fn resource_mgmt(&self) -> &ResourceMgmt {
        &self.resource_mgmt
    }

    /// Returns the address vector type used by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::av_type` field.
    pub fn av_type(&self) -> &AddressVectorType {
        &self.av_type
    }

    /// Returns the memory registration mode used by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::mr_mode` field.
    pub fn mr_mode(&self) -> &MrMode {
        &self.mr_mode
    }

    /// Returns the size of the memory registration key used by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::mr_key_size` field.
    pub fn mr_key_size(&self) -> usize {
        self.mr_key_size
    }

    /// Returns the size of the completion queue data used by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::cq_data_size` field.
    pub fn cq_data_size(&self) -> usize {
        self.cq_data_size
    }

    /// Returns the number of completion queues supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::cq_cnt` field.
    pub fn cq_cnt(&self) -> usize {
        self.cq_cnt
    }

    ///  Returns the number of endpoints supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::ep_cnt` field.
    pub fn ep_cnt(&self) -> usize {
        self.ep_cnt
    }

    /// Returns the number of transmit contexts supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::tx_ctx_cnt` field.
    pub fn tx_ctx_cnt(&self) -> usize {
        self.tx_ctx_cnt
    }

    /// Returns the number of receive contexts supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::rx_ctx_cnt` field.
    pub fn rx_ctx_cnt(&self) -> usize {
        self.rx_ctx_cnt
    }

    /// Returns the maximum number of transmit contexts per endpoint supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::max_ep_tx_ctx` field.
    pub fn max_ep_tx_ctx(&self) -> usize {
        self.max_ep_tx_ctx
    }

    /// Returns the maximum number of receive contexts per endpoint supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::max_ep_rx_ctx` field.
    pub fn max_ep_rx_ctx(&self) -> usize {
        self.max_ep_rx_ctx
    }

    /// Returns the maximum number of shared transmit contexts per endpoint supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::max_ep_stx_ctx` field
    pub fn max_ep_stx_ctx(&self) -> usize {
        self.max_ep_stx_ctx
    }

    /// Returns the maximum number of shared receive contexts per endpoint supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::max_ep_srx_ctx` field
    pub fn max_ep_srx_ctx(&self) -> usize {
        self.max_ep_srx_ctx
    }

    /// Returns the number of counters supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::cntr_cnt` field
    pub fn cntr_cnt(&self) -> usize {
        self.cntr_cnt
    }

    /// Returns the maximum number of memory regions that can be registered.
    ///
    /// Corresponds to accessing the `fi_domain_attr::mr_iov_limit` field
    pub fn mr_iov_limit(&self) -> usize {
        self.mr_iov_limit
    }

    /// Returns the maximum size of error data that can be reported.
    ///
    /// Corresponds to accessing the `fi_domain_attr::max_err_data` field
    pub fn max_err_data(&self) -> usize {
        self.max_err_data
    }

    pub fn mr_cnt(&self) -> usize {
        self.mr_cnt
    }

    /// Returns the capabilities supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::caps` field
    pub fn caps(&self) -> &DomainCaps {
        &self.caps
    }

    /// Returns the operational modes supported by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::mode` field
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Returns the authentication key used by the domain, if any.
    ///
    /// Corresponds to accessing the `fi_domain_attr::auth_key` field
    pub fn auth_key(&self) -> &Option<Vec<u8>> {
        &self.auth_key
    }

    /// Return the traffica class used by the domain.
    ///
    /// Corresponds to accessing the `fi_domain_attr::tclass` field
    pub fn traffic_class(&self) -> &TrafficClass {
        &self.traffic_class
    }
}

/// Builder for the [Domain] type.
///
/// `DomainBuilder` is used to configure and build a new [Domain].
/// It encapsulates an incremental configuration of the address vector set, as provided by a `fi_domain_attr`,
/// followed by a call to `fi_domain_open`  
pub struct DomainBuilder<'a, E> {
    pub(crate) fabric: &'a crate::fabric::Fabric,
    pub(crate) info: &'a InfoEntry<E>,
    pub(crate) ctx: Option<&'a mut Context>,
    pub(crate) peer_ctx: Option<&'a mut PeerDomainCtx>,
    pub(crate) flags: u64,
}

impl<'a> DomainBuilder<'a, ()> {
    /// Initiates the creation of new [Domain] on `fabric`, using the configuration found in `info`.
    ///
    /// The initial configuration is what would be set if no `fi_domain_attr` or `context` was provided to
    /// the `fi_domain` call.
    pub fn new<E>(
        fabric: &'a crate::fabric::Fabric,
        info: &'a InfoEntry<E>,
    ) -> DomainBuilder<'a, E> {
        DomainBuilder::<E> {
            fabric,
            info,
            flags: 0,
            ctx: None,
            peer_ctx: None,
        }
    }

    /// Initiates the creation of new [Domain] on `fabir`, using the configuration found in `info`.
    ///
    /// The initial configuration is what would be set if no `fi_domain_attr` was provided to
    /// the `fi_domain2` call and `context` was set to a `fi_peer_context`.
    pub fn new_with_peer<E>(
        fabric: &'a crate::fabric::Fabric,
        info: &'a InfoEntry<E>,
        peer_ctx: &'a mut PeerDomainCtx,
    ) -> DomainBuilder<'a, E> {
        DomainBuilder::<E> {
            fabric,
            info,
            flags: libfabric_sys::FI_PEER,
            ctx: None,
            peer_ctx: Some(peer_ctx),
        }
    }
}

impl<'a, E> DomainBuilder<'a, E> {
    /// Sets the context to be passed to the domain.
    ///
    /// Corresponds to passing a non-NULL, non-`fi_peer_context` `context` value to `fi_domain`.
    pub fn context(self, ctx: &'a mut Context) -> DomainBuilder<'a, E> {
        DomainBuilder {
            fabric: self.fabric,
            info: self.info,
            flags: self.flags,
            ctx: Some(ctx),
            peer_ctx: self.peer_ctx,
        }
    }
}

impl<'a, E> DomainBuilder<'a, E> {
    /// Constructs a new [Domain] with the configurations requested so far.
    ///
    /// Corresponds to creating a `fi_domain_attr`, setting its fields to the requested ones,
    /// and passing it to a `fi_domain` call with an optional `context` (set by [Self::context]).
    /// Or a call to `fi_domain2` with `context` of type `fi_peer_context` and `flags` equal to `FI_PEER`
    pub fn build_and_bind<EQ: ReadEq + 'static>(
        self,
        eq: &EventQueue<EQ>,
        async_mem_reg: bool,
    ) -> Result<DomainBase<dyn ReadEq>, crate::error::Error> {
        let domain = DomainBase::<dyn ReadEq>::new(
            self.fabric,
            self.info,
            self.flags,
            self.info.domain_attr().clone(),
            self.ctx,
        )?;
        domain.bind_eq(eq, async_mem_reg)?;
        Ok(domain)
    }
}

impl<'a, E> DomainBuilder<'a, E> {
    /// Constructs a new [Domain] with the configurations requested so far.
    ///
    /// Corresponds to creating a `fi_domain_attr`, setting its fields to the requested ones,
    /// and passing it to a `fi_domain` call with an optional `context` (set by [Self::context]).
    /// Or a call to `fi_domain2` with `context` of type `fi_peer_context` and `flags` equal to `FI_PEER`
    pub fn build(self) -> Result<Domain, crate::error::Error> {
        let domain = DomainBase::new(
            self.fabric,
            self.info,
            self.flags,
            self.info.domain_attr().clone(),
            self.ctx,
        )?;
        Ok(domain)
    }
}

/// A Domain without an associated EventQueue
pub type Domain = DomainBase<NoEventQueue>;

/// A Domain with an associated EventQueue
pub type BoundDomain = DomainBase<dyn ReadEq>;

#[repr(C)]
/// Corresponds to `fi_peer_domain_context` in libfabric
/// Used to pass context information to `fi_domain2` when creating a peer domain
pub struct PeerDomainCtx {
    c_ctx: libfabric_sys::fi_peer_domain_context,
}

impl PeerDomainCtx {
    pub fn new<EQ>(size: usize, domain: &DomainBase<EQ>) -> Self {
        Self {
            c_ctx: {
                libfabric_sys::fi_peer_domain_context {
                    domain: domain.as_typed_fid_mut().as_raw_typed_fid(),
                    size,
                }
            },
        }
    }
}

//================== Domain tests ==================//

#[cfg(test)]
mod tests {
    use crate::info::Info;

    #[test]
    fn domain_test() {
        let info = Info::new(&crate::info::libfabric_version()).get().unwrap();
        let entry = info.into_iter().next().unwrap();
        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let count = 10;
        let mut doms = Vec::new();
        for _ in 0..count {
            let domain = crate::domain::DomainBuilder::new(&fab, &entry)
                .build()
                .unwrap();
            doms.push(domain);
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::info::Info;

    #[test]

    fn domain_drops_before_fabric() {
        let info = Info::new(&crate::info::libfabric_version()).get().unwrap();
        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let count = 10;
        let mut doms = Vec::new();
        for _ in 0..count {
            let domain = crate::domain::DomainBuilder::new(&fab, &entry)
                .build()
                .unwrap();
            doms.push(domain);
        }
        drop(fab);
    }
}
