use std::{
    marker::PhantomData,
    os::fd::{AsFd, BorrowedFd},
};

use libfabric_sys::{
    fi_hmem_iface_FI_HMEM_CUDA, fi_hmem_iface_FI_HMEM_ZE, fi_wait_obj_FI_WAIT_FD,
    inlined_fi_control, FI_BACKLOG, FI_GETOPSFLAG,
};

use crate::{av::AVSyncMode, connless_ep::UninitConnectionlessEndpoint, eq::ConnReqEvent};
use crate::{
    av::{AddressVector, AddressVectorBase, AddressVectorImplBase, AddressVectorImplT},
    cntr::{Counter, ReadCntr},
    conn_ep::UninitUnconnectedEndpoint,
    cq::{CompletionQueue, ReadCq},
    domain::DomainImplT,
    enums::{EndpointType, HmemIface, HmemP2p, Protocol, TransferOptions},
    eq::{EventQueueBase, ReadEq},
    fabric::FabricImpl,
    fid::{
        AsRawFid, AsRawTypedFid, AsTypedFid, BorrowedTypedFid, EpRawFid, OwnedEpFid, OwnedPepFid,
        PepRawFid,
    },
    info::{InfoEntry, Version},
    trigger::TriggerXpu,
    utils::check_error,
    Context, MyOnceCell, MyRc, MyRefCell, SyncSend,
};

#[cfg(feature = "threading-completion")]
use crate::fid::EpCompletionOwnedTypedFid;

#[repr(C)]
#[derive(Clone)]
/// A unmapped network address.
///
/// This struct encapsulates a raw byte representation of a network address.
pub struct Address {
    pub(crate) address: Vec<u8>,
}

impl Address {
    /// Creates a new Address object from a raw pointer, usually received from the network.
    ///
    /// # Safety
    /// This function is unsafe since the contents of the pointer are not checked
    pub unsafe fn from_raw_parts(raw: *const u8, len: usize) -> Self {
        let mut address = vec![0u8; len];
        address.copy_from_slice(std::slice::from_raw_parts(raw, len));
        Self { address }
    }

    /// Creates a new Address object from a slice of bytes, usually received from the network.
    ///
    /// # Safety
    /// This function is unsafe since the contents of the slice are not checked
    pub unsafe fn from_bytes(raw: &[u8]) -> Self {
        Address {
            address: raw.to_vec(),
        }
    }

    /// Returns the raw byte representation of the address.
    ///
    /// This can be used for low-level operations or interfacing with C libraries.
    pub fn as_bytes(&self) -> &[u8] {
        &self.address
    }
}

pub(crate) enum EpCq<CQ: ?Sized> {
    Separate(MyRc<CQ>, MyRc<CQ>),
    Shared(MyRc<CQ>),
}

pub struct EndpointImplBase<T, EQ: ?Sized, CQ: ?Sized> {
    #[cfg(not(feature = "threading-completion"))]
    pub(crate) c_ep: OwnedEpFid,
    #[cfg(feature = "threading-completion")]
    pub(crate) c_ep: EpCompletionOwnedTypedFid<EpRawFid>,
    pub(crate) cq: MyOnceCell<EpCq<CQ>>,
    pub(crate) eq: MyOnceCell<MyRc<EQ>>,
    _bound_cntrs: MyRefCell<Vec<MyRc<dyn ReadCntr>>>,
    _bound_av: MyOnceCell<MyRc<dyn AddressVectorImplT>>,
    _domain_rc: MyRc<dyn DomainImplT>,
    phantom: PhantomData<fn() -> T>, // fn() -> T because we only need to track the Endpoint capabilities requested but avoid requiring caps to implement Sync+Send
    pub(crate) has_conn_req: bool,
}

// pub type Endpoint<T, STATE: EpState> =
//     EndpointBase<EndpointImplBase<T, dyn ReadEq, dyn ReadCq>, STATE>;

/// A trait representing the state of an endpoint.
pub trait EpState: SyncSend {}
pub struct Connected;
pub struct Unconnected;
pub struct PendingAccept;
pub struct UninitUnconnected;
pub struct Connectionless;
pub struct UninitConnectionless;

impl SyncSend for Connected {}
impl SyncSend for PendingAccept {}
impl SyncSend for Unconnected {}
impl SyncSend for UninitUnconnected {}
impl SyncSend for Connectionless {}
impl SyncSend for UninitConnectionless {}

impl EpState for Connected {}
impl EpState for PendingAccept {}
impl EpState for Unconnected {}
impl EpState for UninitUnconnected {}
impl EpState for Connectionless {}
impl EpState for UninitConnectionless {}

pub struct EndpointBase<EP, STATE: EpState> {
    pub(crate) inner: MyRc<EP>,
    pub(crate) phantom: PhantomData<STATE>,
}

// pub(crate) trait BaseEndpointImpl: AsRawTypedFid<Output = EpRawFid> {
//     fn bind<T: crate::Bind + AsRawFid>(&self, res: &T, flags: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.as_raw_typed_fid(), res.as_raw_fid(), flags) };
//
//         if err != 0 {
//             Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
//         }
//         else {
//             Ok(())
//         }
//     }
// }

/// A trait that provides common operations for all endpoint types.
pub trait BaseEndpoint<FID: AsRawFid>: AsTypedFid<FID> + SyncSend {
    /// Retrieves the local address of the endpoint.
    fn getname(&self) -> Result<Address, crate::error::Error> {
        let mut len = 0;
        let err: i32 = unsafe {
            libfabric_sys::inlined_fi_getname(
                self.as_typed_fid_mut().as_raw_fid(),
                std::ptr::null_mut(),
                &mut len,
            )
        };
        if -err as u32 == libfabric_sys::FI_ETOOSMALL {
            let mut address = vec![0; len];
            let err: i32 = unsafe {
                libfabric_sys::inlined_fi_getname(
                    self.as_typed_fid_mut().as_raw_fid(),
                    address.as_mut_ptr().cast(),
                    &mut len,
                )
            };
            if err < 0 {
                Err(crate::error::Error::from_err_code(
                    (-err).try_into().unwrap(),
                ))
            } else {
                Ok(Address { address })
            }
        } else {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        }
    }

    /// Retrieves the endpoint type.
    fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        let mut res = 0_usize;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe {
            libfabric_sys::inlined_fi_getopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_BUFFERED_LIMIT as i32,
                (&mut res as *mut usize).cast(),
                &mut len,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(res)
        }
    }

    /// Retrieves the minimum buffered size for the endpoint.
    fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        let mut res = 0_usize;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe {
            libfabric_sys::inlined_fi_getopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_BUFFERED_MIN as i32,
                (&mut res as *mut usize).cast(),
                &mut len,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(res)
        }
    }

    /// Retrieves the size of connection management data.
    fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        let mut res = 0_usize;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe {
            libfabric_sys::inlined_fi_getopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_CM_DATA_SIZE as i32,
                (&mut res as *mut usize).cast(),
                &mut len,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(res)
        }
    }

    /// Retrieves the minimum multi-receive size for the endpoint.
    fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        let mut res = 0_usize;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe {
            libfabric_sys::inlined_fi_getopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_MIN_MULTI_RECV as i32,
                (&mut res as *mut usize).cast(),
                &mut len,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(res)
        }
    }

    /// Retrieves the peer-to-peer memory access capabilities of the endpoint.
    fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        let mut res = 0_u32;
        let mut len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_getopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_FI_HMEM_P2P as i32,
                (&mut res as *mut u32).cast(),
                &mut len,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(HmemP2p::from_raw(res))
        }
    }

    /// Checks if CUDA API is permitted on the endpoint.
    fn cuda_api_permitted(&self) -> Result<bool, crate::error::Error> {
        let mut permitted = 0_u32;
        let mut len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_getopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_CUDA_API_PERMITTED as i32,
                (&mut permitted as *mut u32).cast(),
                &mut len,
            )
        };

        check_error(err.try_into().unwrap())?;
        Ok(permitted == 1)
    }

    /// Retrieves the trigger settings for a specific heterogeneous memory interface.
    fn xpu_trigger(&self, iface: &HmemIface) -> Result<TriggerXpu, crate::error::Error> {
        let (dev_type, device) = match iface {
            HmemIface::Cuda(dev_id) => (
                fi_hmem_iface_FI_HMEM_CUDA,
                libfabric_sys::fi_trigger_xpu__bindgen_ty_1 { cuda: *dev_id },
            ),
            HmemIface::Ze(drv_id, dev_id) => (
                fi_hmem_iface_FI_HMEM_ZE,
                libfabric_sys::fi_trigger_xpu__bindgen_ty_1 {
                    ze: unsafe { libfabric_sys::inlined_fi_hmem_ze_device(*drv_id, *dev_id) },
                },
            ),
            _ => panic!("Device type not supported"),
        };
        let mut res = libfabric_sys::fi_trigger_xpu {
            count: 0,
            iface: dev_type,
            device,
            var: std::ptr::null_mut(),
        };
        let mut len = std::mem::size_of::<libfabric_sys::fi_trigger_xpu>();

        let err = unsafe {
            libfabric_sys::inlined_fi_getopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_XPU_TRIGGER as i32,
                (&mut res as *mut libfabric_sys::fi_trigger_xpu).cast(),
                &mut len,
            )
        };
        let var = libfabric_sys::fi_trigger_var {
            datatype: 0,
            count: 0,
            addr: std::ptr::null_mut(),
            value: libfabric_sys::fi_trigger_var__bindgen_ty_1 { val64: 0 },
        };
        let mut trigger_vars = vec![var; res.count as usize];
        res.var = trigger_vars.as_mut_ptr();

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            let err = unsafe {
                libfabric_sys::inlined_fi_getopt(
                    self.as_typed_fid_mut().as_raw_fid(),
                    libfabric_sys::FI_OPT_ENDPOINT as i32,
                    libfabric_sys::FI_OPT_XPU_TRIGGER as i32,
                    (&mut res as *mut libfabric_sys::fi_trigger_xpu).cast(),
                    &mut len,
                )
            };
            if err != 0 {
                Err(crate::error::Error::from_err_code(
                    (-err).try_into().unwrap(),
                ))
            } else {
                let vars: Vec<libfabric_sys::fi_trigger_var> = (0..res.count)
                    .map(|i| unsafe { *res.var.add(i as usize) })
                    .collect();
                Ok(TriggerXpu::new(*iface, vars))
            }
        }
    }

    /// Retrieves a file descriptor that can be used to wait for events on the endpoint.
    fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        let mut fd = 0;

        let err = unsafe {
            libfabric_sys::inlined_fi_control(
                self.as_typed_fid_mut().as_raw_fid(),
                fi_wait_obj_FI_WAIT_FD as i32,
                (&mut fd as *mut i32).cast(),
            )
        };
        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(unsafe { BorrowedFd::borrow_raw(fd) })
        }
    }
}

impl<T, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> BaseEndpoint<EpRawFid>
    for EndpointImplBase<T, EQ, CQ>
{
}
// impl<T, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> SyncSend for EndpointImplBase<T, EQ, CQ> {}
impl<T: BaseEndpoint<EpRawFid>, STATE: EpState> BaseEndpoint<EpRawFid> for EndpointBase<T, STATE> {}
impl<T: BaseEndpoint<EpRawFid>, STATE: EpState> SyncSend for EndpointBase<T, STATE> {}

impl<T, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> ActiveEndpoint for EndpointImplBase<T, EQ, CQ> {
    fn fid(&self) -> &OwnedEpFid {
        #[cfg(feature = "threading-completion")]
        return &self.c_ep.typed_fid;
        #[cfg(not(feature = "threading-completion"))]
        return &self.c_ep;
    }
}
impl<T, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> SyncSend for EndpointImplBase<T, EQ, CQ> {}

impl<T: BaseEndpoint<EpRawFid> + ActiveEndpoint, STATE: EpState> ActiveEndpoint
    for EndpointBase<T, STATE>
{
    fn fid(&self) -> &OwnedEpFid {
        self.inner.fid()
    }
}
// impl<T: ActiveEndpoint, STATE: EpState> SyncSend for EndpointBase<T, STATE> {}

impl<E: AsFd, STATE: EpState> AsFd for EndpointBase<E, STATE> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

impl<T, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AsFd for EndpointImplBase<T, EQ, CQ> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.wait_fd().unwrap()
    }
}

//================== Scalable Endpoint (fi_scalable_ep) ==================//
pub(crate) struct ScalableEndpointImpl {
    #[cfg(not(feature = "threading-completion"))]
    pub(crate) c_sep: OwnedEpFid,
    #[cfg(feature = "threading-completion")]
    pub(crate) c_sep: EpCompletionOwnedTypedFid<EpRawFid>,
    _domain_rc: MyRc<dyn DomainImplT>,
}

/// A scalable endpoint that can manage multiple connections.
///
/// This endpoint type is suitable for applications that require high scalability and flexibility in managing connections.
/// Corresponds to `fi_scalable_ep` in libfabric.
pub struct ScalableEndpoint<E> {
    inner: MyRc<ScalableEndpointImpl>,
    phantom: PhantomData<fn() -> E>,
}

impl ScalableEndpoint<()> {
    pub fn new<E, EQ: ?Sized + 'static + SyncSend>(
        domain: &crate::domain::DomainBase<EQ>,
        info: &InfoEntry<E>,
        context: Option<&mut Context>,
    ) -> Result<ScalableEndpoint<E>, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(ScalableEndpoint::<E> {
            inner: MyRc::new(ScalableEndpointImpl::new(&domain.inner, info, c_void)?),
            phantom: PhantomData,
        })
    }
}

impl SyncSend for ScalableEndpointImpl {}

impl ScalableEndpointImpl {
    pub fn new<E, EQ: ?Sized + 'static + SyncSend>(
        domain: &MyRc<crate::domain::DomainImplBase<EQ>>,
        info: &InfoEntry<E>,
        context: *mut std::ffi::c_void,
    ) -> Result<ScalableEndpointImpl, crate::error::Error> {
        let mut c_sep: EpRawFid = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_scalable_ep(
                domain.as_typed_fid_mut().as_raw_typed_fid(),
                info.info.as_raw(),
                &mut c_sep,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(ScalableEndpointImpl {
                #[cfg(not(any(feature = "threading-domain", feature = "threading-completion")))]
                c_sep: OwnedEpFid::from(c_sep),
                #[cfg(feature = "threading-domain")]
                c_sep: OwnedEpFid::from(c_sep, domain.c_domain.domain.clone()),
                #[cfg(feature = "threading-completion")]
                c_sep: EpCompletionOwnedTypedFid::from(c_sep),

                // #[cfg(not(feature="threading-domain"))]
                // c_sep: OwnedEpFid::from(c_sep),
                // #[cfg(feature="threading-domain")]
                // c_sep: OwnedEpFid::from(c_sep, domain.c_domain.domain.clone()),
                _domain_rc: domain.clone(),
            })
        }
    }
    fn bind<T: crate::fid::AsTypedFid<impl AsRawFid> + ?Sized>(
        &self,
        res: &MyRc<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_scalable_ep_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                res.as_typed_fid().as_raw_fid(),
                flags,
            )
        };

        check_error(err.try_into().unwrap())
    }

    // pub(crate) fn bind_av(&self, av: &MyRc<AddressVectorImpl>) -> Result<(), crate::error::Error> {

    //     self.bind(&av, 0)
    // }

    // pub(crate) fn alias(&self, flags: u64) -> Result<ScalableEndpointImpl, crate::error::Error> {
    //     let mut c_sep: EpRawFid = std::ptr::null_mut();
    //     let c_sep_ptr: *mut EpRawFid = &mut c_sep;
    //     let err = unsafe {
    //         libfabric_sys::inlined_fi_ep_alias(self.as_typed_fid().as_raw_typed_fid(), c_sep_ptr, flags)
    //     };

    //     if err != 0 {
    //         Err(crate::error::Error::from_err_code(
    //             (-err).try_into().unwrap(),
    //         ))
    //     } else {
    //         Ok(ScalableEndpointImpl {
    //             c_sep: OwnedEpFid::from(c_sep),
    //             _domain_rc: self._domain_rc.clone(),
    //         })
    //     }
    // }
}

impl<E> ScalableEndpoint<E> {
    pub fn bind_av(&self, av: &AddressVector) -> Result<(), crate::error::Error> {
        self.inner.bind(&av.inner, 0)
    }

    // pub fn alias(&self, flags: u64) -> Result<ScalableEndpoint<E>, crate::error::Error> {
    //     Ok(Self {
    //         inner: MyRc::new(self.inner.alias(flags)?),
    //         phantom: PhantomData,
    //     })
    // }
}

// impl AsFid for ScalableEndpointImpl {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.c_sep.as_fid()
//     }
// }
// impl<E> AsFid for ScalableEndpoint<E> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.inner.as_fid()
//     }
// }

// impl AsRawFid for ScalableEndpointImpl {
//     fn as_raw_fid(&self) -> RawFid {
//         self.c_sep.as_raw_fid()
//     }
// }
// impl<E> AsRawFid for ScalableEndpoint<E> {
//     fn as_raw_fid(&self) -> RawFid {
//         self.inner.as_raw_fid()
//     }
// }

impl AsTypedFid<EpRawFid> for ScalableEndpointImpl {
    fn as_typed_fid(&self) -> BorrowedTypedFid<EpRawFid> {
        self.c_sep.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<EpRawFid> {
        self.c_sep.as_typed_fid_mut()
    }
}

impl<E> AsTypedFid<EpRawFid> for ScalableEndpoint<E> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<EpRawFid> {
        self.inner.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<EpRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

// impl AsRawTypedFid for ScalableEndpointImpl {
//     type Output = EpRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.c_sep.as_raw_typed_fid()
//     }
// }

// impl<E> AsRawTypedFid for ScalableEndpoint<E> {
//     type Output = EpRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.inner.as_raw_typed_fid()
//     }
// }

impl<E> BaseEndpoint<EpRawFid> for ScalableEndpoint<E> {}
impl<E> SyncSend for ScalableEndpoint<E> {}

impl BaseEndpoint<EpRawFid> for ScalableEndpointImpl {}

impl ActiveEndpoint for ScalableEndpointImpl {
    fn fid(&self) -> &OwnedEpFid {
        #[cfg(feature = "threading-completion")]
        return &self.c_sep.typed_fid;
        #[cfg(not(feature = "threading-completion"))]
        return &self.c_sep;
    }
}

impl<E: ActiveEndpoint> ActiveEndpoint for ScalableEndpoint<E> {
    fn fid(&self) -> &OwnedEpFid {
        self.inner.fid()
    }
}

impl<E> AsFd for ScalableEndpoint<E> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.wait_fd().unwrap()
    }
}

//================== Passive Endpoint (fi_passive_ep) ==================//

pub(crate) struct PassiveEndpointImplBase<E, EQ: ?Sized> {
    pub(crate) c_pep: OwnedPepFid,
    pub(crate) eq: MyOnceCell<MyRc<EQ>>,
    phantom: PhantomData<fn() -> E>,
    _fabric_rc: MyRc<FabricImpl>,
}

pub type PassiveEndpoint<E> = PassiveEndpointBase<E, dyn ReadEq>;

/// A passive endpoint that listens for incoming connection requests.
///
/// Corresponds to `fi_passive_ep` in libfabric.
pub struct PassiveEndpointBase<E, EQ: ?Sized> {
    pub(crate) inner: MyRc<PassiveEndpointImplBase<E, EQ>>,
}

impl<EQ: ?Sized> PassiveEndpointBase<(), EQ> {
    pub fn new<E>(
        fabric: &crate::fabric::Fabric,
        info: &InfoEntry<E>,
        context: Option<&mut Context>,
    ) -> Result<PassiveEndpointBase<E, EQ>, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(PassiveEndpointBase::<E, EQ> {
            inner: MyRc::new(PassiveEndpointImplBase::new(&fabric.inner, info, c_void)?),
        })
    }
}

impl<EQ: ?Sized> PassiveEndpointImplBase<(), EQ> {
    pub fn new<E>(
        fabric: &MyRc<crate::fabric::FabricImpl>,
        info: &InfoEntry<E>,
        context: *mut std::ffi::c_void,
    ) -> Result<PassiveEndpointImplBase<E, EQ>, crate::error::Error> {
        let mut c_pep: PepRawFid = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_passive_ep(
                fabric.as_typed_fid_mut().as_raw_typed_fid(),
                info.info.as_raw(),
                &mut c_pep,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(PassiveEndpointImplBase::<E, EQ> {
                c_pep: OwnedPepFid::from(c_pep),
                eq: MyOnceCell::new(),
                _fabric_rc: fabric.clone(),
                phantom: PhantomData,
            })
        }
    }
}

impl<E> PassiveEndpointImplBase<E, dyn ReadEq> {
    pub(crate) fn bind<T: ReadEq + 'static>(
        &self,
        res: &MyRc<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_pep_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                res.as_typed_fid().as_raw_fid(),
                flags,
            )
        };
        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self.eq.set(res.clone()).is_err() {
                panic!("Could not set oncecell")
            }
            Ok(())
        }
    }
}

impl<E> PassiveEndpointBase<E, dyn ReadEq> {
    /// Binds a resource to the passive endpoint.
    pub fn bind<T: ReadEq + 'static>(
        &self,
        res: &EventQueueBase<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind(&res.inner, flags)
    }
}

impl<E, EQ: ?Sized + ReadEq> PassiveEndpointImplBase<E, EQ> {
    pub fn listen(&self) -> Result<(), crate::error::Error> {
        let err =
            unsafe { libfabric_sys::inlined_fi_listen(self.as_typed_fid_mut().as_raw_typed_fid()) };

        check_error(err.try_into().unwrap())
    }

    pub fn reject<T0>(
        &self,
        fid: &impl AsRawFid,
        params: Option<&[T0]>,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_reject(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                fid.as_raw_fid(),
                params.map_or_else(std::ptr::null, |v| v.as_ptr().cast()),
                params.map_or(0, |v| v.len()),
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets the backlog size for incoming connection requests.
    ///
    /// Corresponds to `FI_BACKLOG` control operation in libfabric.
    pub fn set_backlog_size(&self, size: i32) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_control(
                self.as_typed_fid_mut().as_raw_fid(),
                FI_BACKLOG as i32,
                (&mut size.clone() as *mut i32).cast(),
            )
        };
        check_error(err.try_into().unwrap())
    }
}

impl<E, EQ: ?Sized + ReadEq> PassiveEndpointBase<E, EQ> {
    /// Starts listening for incoming connection requests.
    ///
    /// Corresponds to `fi_listen` in libfabric.
    pub fn listen(&self) -> Result<(), crate::error::Error> {
        self.inner.listen()
    }

    /// Rejects an incoming connection request.
    ///
    /// Corresponds to `fi_reject` in libfabric.
    pub fn reject(&self, event: ConnReqEvent) -> Result<(), crate::error::Error> {
        self.inner.reject::<()>(&event.fid(), None)
    }

    /// Rejects an incoming connection request and sends back the provided params.
    ///
    /// Corresponds to `fi_reject` in libfabric.
    pub fn reject_with_params<T0>(
        &self,
        event: ConnReqEvent,
        params: &[T0],
    ) -> Result<(), crate::error::Error> {
        self.inner.reject(&event.fid(), Some(params))
    }

    /// Sets the backlog size for incoming connection requests.
    ///
    /// Corresponds to `FI_BACKLOG` control operation in libfabric.
    pub fn set_backlog_size(&self, size: i32) -> Result<(), crate::error::Error> {
        self.inner.set_backlog_size(size)
    }
}

impl<E, EQ: ?Sized + SyncSend> SyncSend for PassiveEndpointBase<E, EQ> {}
impl<E, EQ: ?Sized + SyncSend + ReadEq> BaseEndpoint<PepRawFid> for PassiveEndpointBase<E, EQ> {}

impl<E, EQ: ?Sized + ReadEq> SyncSend for PassiveEndpointImplBase<E, EQ> {}
impl<E, EQ: ?Sized + ReadEq> BaseEndpoint<PepRawFid> for PassiveEndpointImplBase<E, EQ> {}

// impl<E, EQ: ?Sized + ReadEq> AsFid for PassiveEndpointImplBase<E, EQ> {
//     fn as_fid(&self) -> fid::BorrowedFid {
//         self.c_pep.as_fid()
//     }
// }

// impl<E, EQ: ?Sized + ReadEq> AsFid for PassiveEndpointBase<E, EQ> {
//     fn as_fid(&self) -> fid::BorrowedFid {
//         self.inner.as_fid()
//     }
// }

// impl<E, EQ: ?Sized> AsRawFid for PassiveEndpointImplBase<E, EQ> {
//     fn as_raw_fid(&self) -> RawFid {
//         self.c_pep.as_raw_fid()
//     }
// }

// impl<E, EQ: ?Sized> AsRawFid for PassiveEndpointBase<E, EQ> {
//     fn as_raw_fid(&self) -> RawFid {
//         self.inner.as_raw_fid()
//     }
// }

impl<E, EQ: ?Sized + ReadEq> AsTypedFid<PepRawFid> for PassiveEndpointImplBase<E, EQ> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<PepRawFid> {
        self.c_pep.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<PepRawFid> {
        self.c_pep.as_typed_fid_mut()
    }
}

impl<E, EQ: ?Sized + ReadEq> AsTypedFid<PepRawFid> for PassiveEndpointBase<E, EQ> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<PepRawFid> {
        self.inner.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<PepRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

// impl<E, EQ: ?Sized + ReadEq> AsRawTypedFid for PassiveEndpointImplBase<E, EQ> {
//     type Output = PepRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.c_pep.as_raw_typed_fid()
//     }
// }

// impl<E, EQ: ?Sized + ReadEq> AsRawTypedFid for PassiveEndpointBase<E, EQ> {
//     type Output = PepRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.inner.as_raw_typed_fid()
//     }
// }

impl<E, EQ: ?Sized + ReadEq> AsFd for PassiveEndpointImplBase<E, EQ> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.wait_fd().unwrap()
    }
}

impl<E> AsFd for PassiveEndpoint<E> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.wait_fd().unwrap()
    }
}

//================== Endpoint (fi_endpoint) ==================//

// pub struct IncompleteBindCq<'a, EP> {
//     pub(crate) ep: &'a EP,
//     pub(crate) flags: u64,
// }

// impl<'a, EP> IncompleteBindCq<'a, EP> {
//     pub fn recv(&mut self, selective: bool) -> &mut Self {
//         if selective {
//             self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_RECV  as u64 ;

//             self
//         }
//         else {
//             self.flags |= libfabric_sys::FI_RECV as u64;

//             self
//         }
//     }

//     pub fn transmit(&mut self, selective: bool) -> &mut Self {
//         if selective {
//             self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_TRANSMIT as u64;

//             self
//         }
//         else {
//             self.flags |= libfabric_sys::FI_TRANSMIT as u64;

//             self
//         }
//     }
// }

// impl<'a, EP> IncompleteBindCq<'a, EndpointImplBase<EP, dyn ReadEq, dyn ReadCq>> {

//     pub fn cq<T: AsRawFid+ ReadCq + 'static>(&mut self, cq: &crate::cq::CompletionQueueBase<T>) -> Result<(), crate::error::Error> {
//         self.ep.bind_cq_(&cq.inner, self.flags)
//     }
// }

pub struct IncompleteBindCntr<'a, EP, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> {
    pub(crate) ep: &'a EndpointImplBase<EP, EQ, CQ>,
    pub(crate) flags: u64,
}

impl<EP, EQ: ?Sized + ReadEq + 'static, CQ: ?Sized + ReadCq>
    IncompleteBindCntr<'_, EP, EQ, CQ>
{
    pub fn read(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_READ as u64;

        self
    }

    pub fn recv(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_RECV as u64;

        self
    }

    pub fn remote_read(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_REMOTE_READ as u64;

        self
    }

    pub fn remote_write(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_REMOTE_WRITE as u64;

        self
    }

    pub fn send(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_SEND as u64;

        self
    }

    pub fn write(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_WRITE as u64;

        self
    }

    pub fn cntr(
        &mut self,
        cntr: &Counter<impl ReadCntr + 'static>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cntr_(&cntr.inner, self.flags)
    }
}

impl<T, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> EndpointImplBase<T, EQ, CQ> {
    pub(crate) fn new<E, DEQ: ?Sized + 'static + SyncSend>(
        domain: &MyRc<crate::domain::DomainImplBase<DEQ>>,
        info: &InfoEntry<E>,
        flags: u64,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_ep: EpRawFid = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_endpoint2(
                domain.as_typed_fid_mut().as_raw_typed_fid(),
                info.info.as_raw(),
                &mut c_ep,
                flags,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                #[cfg(not(any(feature = "threading-domain", feature = "threading-completion")))]
                c_ep: OwnedEpFid::from(c_ep),
                #[cfg(feature = "threading-domain")]
                c_ep: OwnedEpFid::from(c_ep, domain.c_domain.domain.clone()),
                #[cfg(feature = "threading-completion")]
                c_ep: EpCompletionOwnedTypedFid::from(c_ep),
                _bound_av: MyOnceCell::new(),
                _bound_cntrs: MyRefCell::new(Vec::new()),
                cq: MyOnceCell::new(),
                eq: MyOnceCell::new(),
                _domain_rc: domain.clone(),
                phantom: PhantomData,
                has_conn_req: !unsafe { *info.info.0 }.handle.is_null(),
            })
        }
    }
}

impl EndpointBase<EndpointImplBase<(), dyn ReadEq, dyn ReadCq>, UninitConnectionless> {
    #[allow(clippy::type_complexity)]
    pub fn new<E, DEQ: ?Sized + 'static + SyncSend>(
        domain: &crate::domain::DomainBase<DEQ>,
        info: &InfoEntry<E>,
        flags: u64,
        context: Option<&mut Context>,
    ) -> Result<
        EndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>, UninitConnectionless>,
        crate::error::Error,
    > {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(
            EndpointBase::<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>, UninitConnectionless> {
                inner: MyRc::new(EndpointImplBase::new(&domain.inner, info, flags, c_void)?),
                phantom: PhantomData,
            },
        )
    }
}

impl EndpointBase<EndpointImplBase<(), dyn ReadEq, dyn ReadCq>, UninitUnconnected> {
    #[allow(clippy::type_complexity)]
    pub fn new<E, DEQ: ?Sized + 'static + SyncSend>(
        domain: &crate::domain::DomainBase<DEQ>,
        info: &InfoEntry<E>,
        flags: u64,
        context: Option<&mut Context>,
    ) -> Result<
        EndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>, UninitUnconnected>,
        crate::error::Error,
    > {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(
            EndpointBase::<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>, UninitUnconnected> {
                inner: MyRc::new(EndpointImplBase::new(&domain.inner, info, flags, c_void)?),
                phantom: PhantomData,
            },
        )
    }
}

impl<EP, EQ: ?Sized + ReadEq + 'static, CQ: ?Sized + ReadCq> EndpointImplBase<EP, EQ, CQ> {
    pub(crate) fn bind_av_<AVEQ: ?Sized + ReadEq + 'static>(
        &self,
        res: &MyRc<AddressVectorImplBase<AVEQ>>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                res.as_typed_fid().as_raw_fid(),
                flags,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self._bound_av.set(res.clone()).is_err() {
                panic!("Endpoint already bound to an AddressVector");
            }
            Ok(())
        }
    }

    pub(crate) fn bind_cntr_(
        &self,
        res: &MyRc<impl ReadCntr + 'static>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                res.as_typed_fid().as_raw_fid(),
                flags,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            #[cfg(not(feature = "thread-safe"))]
            self._bound_cntrs.borrow_mut().push(res.clone());
            #[cfg(feature = "thread-safe")]
            self._bound_cntrs.write().push(res.clone());
            #[cfg(feature = "threading-completion")]
            let _ = self.c_ep.bound_cntr.set(res.fid().typed_fid.clone());
            Ok(())
        }
    }
}

impl<EP, EQ: ?Sized + ReadEq + 'static> EndpointImplBase<EP, EQ, dyn ReadCq> {
    pub(crate) fn bind_shared_cq<T: ReadCq + 'static>(
        &self,
        cq: &MyRc<T>,
        selective: bool,
    ) -> Result<(), crate::error::Error> {
        let mut flags = libfabric_sys::FI_TRANSMIT as u64 | libfabric_sys::FI_RECV as u64;
        if selective {
            flags |= libfabric_sys::FI_SELECTIVE_COMPLETION;
        }

        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                cq.as_typed_fid().as_raw_fid(),
                flags,
            )
        };

        check_error(err as isize)?;
        if self.cq.set(EpCq::Shared(cq.clone())).is_err() {
            panic!("Endpoint already bound with another shared Completion Queueu");
        }

        #[cfg(feature = "threading-completion")]
        let _ = self.c_ep.bound_cq0.set(cq.fid().typed_fid.clone());

        Ok(())
    }

    pub(crate) fn bind_separate_cqs<T: ReadCq + 'static>(
        &self,
        tx_cq: &MyRc<T>,
        tx_selective: bool,
        rx_cq: &MyRc<T>,
        rx_selective: bool,
    ) -> Result<(), crate::error::Error> {
        let mut tx_flags = libfabric_sys::FI_TRANSMIT as u64;
        if tx_selective {
            tx_flags |= libfabric_sys::FI_SELECTIVE_COMPLETION;
        }

        let mut rx_flags = libfabric_sys::FI_RECV as u64;
        if rx_selective {
            rx_flags |= libfabric_sys::FI_SELECTIVE_COMPLETION;
        }

        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                tx_cq.as_typed_fid().as_raw_fid(),
                tx_flags,
            )
        };
        check_error(err as isize)?;

        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                rx_cq.as_typed_fid().as_raw_fid(),
                rx_flags,
            )
        };
        check_error(err as isize)?;

        if self
            .cq
            .set(EpCq::Separate(tx_cq.clone(), rx_cq.clone()))
            .is_err()
        {
            panic!("Endpoint already bound with other  Completion Queueus");
        }
        #[cfg(feature = "threading-completion")]
        let _ = self.c_ep.bound_cq0.set(tx_cq.fid().typed_fid.clone());
        #[cfg(feature = "threading-completion")]
        let _ = self.c_ep.bound_cq1.set(rx_cq.fid().typed_fid.clone());

        Ok(())
    }
}

impl<EP, CQ: ?Sized + ReadCq> EndpointImplBase<EP, dyn ReadEq, CQ> {
    pub(crate) fn bind_eq<T: ReadEq + 'static>(
        &self,
        eq: &MyRc<T>,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                eq.as_typed_fid().as_raw_fid(),
                0,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self.eq.set(eq.clone()).is_err() {
                panic!("Endpoint is already bound to another EventQueue"); // Should never reach this since inlined_fi_ep_bind will throw an error ealier
                                                                           // but keep it here to satisfy the compiler.
            }
            Ok(())
        }
    }
}

impl<EP, EQ: ?Sized + 'static + ReadEq, CQ: ?Sized + ReadCq> EndpointImplBase<EP, EQ, CQ> {
    pub(crate) fn bind_cntr(&self) -> IncompleteBindCntr<EP, EQ, CQ> {
        IncompleteBindCntr { ep: self, flags: 0 }
    }

    pub(crate) fn bind_av<Mode: AVSyncMode, AVEQ: ?Sized + ReadEq + 'static>(
        &self,
        av: &AddressVectorBase<Mode, AVEQ>,
    ) -> Result<(), crate::error::Error> {
        self.bind_av_(&av.inner, 0)
    }

    // #[allow(dead_code)]
    // pub(crate) fn alias(&self, flags: u64) -> Result<Self, crate::error::Error> {
    //     let mut c_ep: EpRawFid = std::ptr::null_mut();
    //     let err = unsafe {
    //         libfabric_sys::inlined_fi_ep_alias(self.as_typed_fid().as_raw_typed_fid(), &mut c_ep, flags)
    //     };

    //     if err != 0 {
    //         Err(crate::error::Error::from_err_code(
    //             (-err).try_into().unwrap(),
    //         ))
    //     } else {
    //         Ok(Self {
    //             #[cfg(not(feature="threading-domain"))]
    //             c_ep: OwnedEpFid::from(c_ep),
    //             #[cfg(feature="threading-domain")]
    //             c_ep: OwnedEpFid::from(c_ep,self._domain_rc.c_domain.typed_fid.typed_fid.clone()),
    //             _bound_av: MyOnceCell::new(),
    //             _bound_cntrs: MyRefCell::new(Vec::new()),
    //             cq: MyOnceCell::new(),
    //             eq: MyOnceCell::new(),
    //             _domain_rc: self._domain_rc.clone(),
    //             phantom: PhantomData,
    //         })
    //     }
    // }
}

impl<EP> EndpointBase<EndpointImplBase<EP, dyn ReadEq, dyn ReadCq>, UninitConnectionless> {
    /// Binds a completion queue to the endpoint for both transmit and receive operations.
    ///
    /// If `selective` is true, selective completion is enabled.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_shared_cq<T: ReadCq + 'static>(
        &self,
        cq: &CompletionQueue<T>,
        selective: bool,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind_shared_cq(&cq.inner, selective)
    }

    /// Binds separate completion queues to the endpoint for transmit and receive operations.
    ///
    /// `tx_selective` enables selective completion for the transmit queue if true.
    /// `rx_selective` enables selective completion for the receive queue if true.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_separate_cqs<T: ReadCq + 'static>(
        &self,
        tx_cq: &CompletionQueue<T>,
        tx_selective: bool,
        rx_cq: &CompletionQueue<T>,
        rx_selective: bool,
    ) -> Result<(), crate::error::Error> {
        self.inner
            .bind_separate_cqs(&tx_cq.inner, tx_selective, &rx_cq.inner, rx_selective)
    }

    /// Binds an event queue to the endpoint.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_eq<T: ReadEq + 'static>(
        &self,
        eq: &EventQueueBase<T>,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind_eq(&eq.inner)
    }

    /// Binds a counter to the endpoint for specified operations.
    ///
    /// Use the returned `IncompleteBindCntr` to specify which operations to bind the counter to.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_cntr(&self) -> IncompleteBindCntr<EP, dyn ReadEq, dyn ReadCq> {
        self.inner.bind_cntr()
    }

    /// Binds an address vector to the endpoint.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_av<Mode: AVSyncMode, EQ: ?Sized + ReadEq + 'static>(
        &self,
        av: &AddressVectorBase<Mode, EQ>,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind_av(av)
    }

    // pub fn alias(&self, flags: u64) -> Result<Self, crate::error::Error> {
    //     Ok(
    //         Self {
    //             inner: MyRc::new (self.inner.alias(flags)?),
    //         }
    //     )
    // }
}

impl<EP> EndpointBase<EndpointImplBase<EP, dyn ReadEq, dyn ReadCq>, UninitUnconnected> {
    /// Binds a completion queue to the endpoint for both transmit and receive operations.
    ///
    /// If `selective` is true, selective completion is enabled.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_shared_cq<T: ReadCq + 'static>(
        &self,
        cq: &CompletionQueue<T>,
        selective: bool,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind_shared_cq(&cq.inner, selective)
    }

    /// Binds separate completion queues to the endpoint for transmit and receive operations.
    ///
    /// `tx_selective` enables selective completion for the transmit queue if true.
    /// `rx_selective` enables selective completion for the receive queue if true.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_separate_cqs<T: ReadCq + 'static>(
        &self,
        tx_cq: &CompletionQueue<T>,
        tx_selective: bool,
        rx_cq: &CompletionQueue<T>,
        rx_selective: bool,
    ) -> Result<(), crate::error::Error> {
        self.inner
            .bind_separate_cqs(&tx_cq.inner, tx_selective, &rx_cq.inner, rx_selective)
    }

    /// Binds an event queue to the endpoint.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub(crate) fn bind_eq<T: ReadEq + 'static>(
        &self,
        eq: &EventQueueBase<T>,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind_eq(&eq.inner)
    }

    /// Binds a counter to the endpoint for specified operations.
    ///
    /// Use the returned `IncompleteBindCntr` to specify which operations to bind the counter
    /// to.
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_cntr(&self) -> IncompleteBindCntr<EP, dyn ReadEq, dyn ReadCq> {
        self.inner.bind_cntr()
    }

    /// Binds an address vector to the endpoint.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.   
    pub fn bind_av<Mode: AVSyncMode, EQ: ?Sized + ReadEq + 'static>(
        &self,
        av: &AddressVectorBase<Mode, EQ>,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind_av(av)
    }

    // pub fn alias(&self, flags: u64) -> Result<Self, crate::error::Error> {
    //     Ok(
    //         Self {
    //             inner: MyRc::new (self.inner.alias(flags)?),
    //         }
    //     )
    // }
}

// impl<E: AsFid, STATE: EpState> AsFid for EndpointBase<E, STATE> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.inner.as_fid()
//     }
// }

// impl<E: AsRawFid, STATE: EpState> AsRawFid for EndpointBase<E, STATE> {
//     fn as_raw_fid(&self) -> RawFid {
//         self.inner.as_raw_fid()
//     }
// }

impl<E: AsTypedFid<EpRawFid>, STATE: EpState> AsTypedFid<EpRawFid> for EndpointBase<E, STATE> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<EpRawFid> {
        self.inner.as_typed_fid()
    }

    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<EpRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

// impl<E: AsRawTypedFid<Output = *mut libfabric_sys::fid_ep>, STATE: EpState> AsRawTypedFid
//     for EndpointBase<E, STATE>
// {
//     type Output = EpRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.inner.as_raw_typed_fid()
//     }
// }

// impl<EP, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AsFid for EndpointImplBase<EP, EQ, CQ> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.c_ep.as_fid()
//     }
// }

// impl<EP, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AsRawFid for EndpointImplBase<EP, EQ, CQ> {
//     fn as_raw_fid(&self) -> RawFid {
//         self.c_ep.as_raw_fid()
//     }
// }

impl<EP, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AsTypedFid<EpRawFid>
    for EndpointImplBase<EP, EQ, CQ>
{
    fn as_typed_fid(&self) -> BorrowedTypedFid<EpRawFid> {
        self.c_ep.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<EpRawFid> {
        self.c_ep.as_typed_fid_mut()
    }
}

// impl<EP, EQ: ?Sized, CQ: ?Sized + ReadCq> AsRawTypedFid for EndpointImplBase<EP, EQ, CQ> {
//     type Output = EpRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.c_ep.as_raw_typed_fid()
//     }
// }

/// Trait for active endpoints (i.e. not passive), providing common operations.
pub trait ActiveEndpoint: AsTypedFid<EpRawFid> + SyncSend {
    /// Returns a reference to the underlying `OwnedEpFid`.
    fn fid(&self) -> &OwnedEpFid;

    /// Cancels outstanding operations associated with the given context.
    ///
    /// Corresponds to `fi_cancel` in libfabric.
    fn cancel(&self, context: &mut Context) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_cancel(
                self.as_typed_fid_mut().as_raw_typed_fid().as_raw_fid(),
                context.inner_mut(),
            )
        };
        check_error(err)
    }

    #[deprecated]
    fn rx_size_left(&self) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_rx_size_left(self.as_typed_fid_mut().as_raw_typed_fid())
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            Ok(ret as usize)
        }
    }

    #[deprecated]
    fn tx_size_left(&self) -> Result<usize, crate::error::Error> {
        let ret = unsafe {
            libfabric_sys::inlined_fi_tx_size_left(self.as_typed_fid_mut().as_raw_typed_fid())
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code(
                (-ret).try_into().unwrap(),
            ))
        } else {
            Ok(ret as usize)
        }
    }

    /// Retrieves the current transmit options for the endpoint.
    ///
    /// Corresponds to `fi_control` with `FI_GETOPSFLAG` in libfabric.
    fn transmit_options(&self) -> Result<TransferOptions, crate::error::Error> {
        let mut ops = libfabric_sys::FI_TRANSMIT;
        let err = unsafe {
            inlined_fi_control(
                self.as_typed_fid_mut().as_raw_fid(),
                FI_GETOPSFLAG as i32,
                (&mut ops as *mut u32).cast(),
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(TransferOptions::from_raw(ops))
        }
    }

    /// Retrieves the current receive options for the endpoint.
    ///
    /// Corresponds to `fi_control` with `FI_GETOPSFLAG` in libfabric.
    fn receive_options(&self) -> Result<TransferOptions, crate::error::Error> {
        let mut ops = libfabric_sys::FI_RECV;
        let err = unsafe {
            inlined_fi_control(
                self.as_typed_fid_mut().as_raw_typed_fid().as_raw_fid(),
                FI_GETOPSFLAG as i32,
                (&mut ops as *mut u32).cast(),
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(TransferOptions::from_raw(ops))
        }
    }
}

/// Trait for uninitialized endpoints, providing configuration options before activation.
pub trait UninitEndpoint: AsTypedFid<EpRawFid> {
    /// Sets the transmit options for the endpoint.
    ///
    /// Corresponds to `fi_control` with `FI_SETOPSFLAG` in libfabric.
    fn set_transmit_options(&self, ops: TransferOptions) -> Result<(), crate::error::Error> {
        ops.transmit();
        let err = unsafe {
            inlined_fi_control(
                self.as_typed_fid_mut().as_raw_typed_fid().as_raw_fid(),
                libfabric_sys::FI_SETOPSFLAG as i32,
                (&mut ops.as_raw() as *mut u32).cast(),
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets the receive options for the endpoint.
    ///
    /// Corresponds to `fi_control` with `FI_SETOPSFLAG` in libfabric
    fn set_receive_options(&self, ops: TransferOptions) -> Result<(), crate::error::Error> {
        ops.recv();
        let err = unsafe {
            inlined_fi_control(
                self.as_typed_fid_mut().as_raw_typed_fid().as_raw_fid(),
                libfabric_sys::FI_SETOPSFLAG as i32,
                (&mut ops.as_raw() as *mut u32).cast(),
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets the buffered limit for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_BUFFERED_LIMIT` in lib
    fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        let mut res = size;
        let len = std::mem::size_of::<usize>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_BUFFERED_LIMIT as i32,
                (&mut res as *mut usize).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets the minimum buffered size for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_BUFFERED_MIN` in lib
    fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        let mut res = size;
        let len = std::mem::size_of::<usize>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_BUFFERED_MIN as i32,
                (&mut res as *mut usize).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets the connection manager data size for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_CM_DATA_SIZE` in lib
    fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        let mut res = size;
        let len = std::mem::size_of::<usize>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_CM_DATA_SIZE as i32,
                (&mut res as *mut usize).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets the minimum multi-receive size for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_MIN_MULTI_RECV` in
    fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        let mut res = size;
        let len = std::mem::size_of::<usize>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_MIN_MULTI_RECV as i32,
                (&mut res as *mut usize).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets the memory peer-to-peer capabilities for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_FI_HMEM_P2
    fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_FI_HMEM_P2P as i32,
                (&mut hmem.as_raw() as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets whether CUDA API usage is permitted for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_CUDA_API_PERMITTED`
    fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_CUDA_API_PERMITTED as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets whether shared memory usage is permitted for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_SHARED_MEMORY_PERMITTED`
    fn set_shared_memory_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_SHARED_MEMORY_PERMITTED as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets whether maximum message size operations are permitted for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_MAX_MSG_SIZE`
    fn set_max_msg_size(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_MAX_MSG_SIZE as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets whether maximum tagged message size operations are permitted for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_MAX_TAGGED_SIZE`
    fn set_max_tagged_size(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_MAX_TAGGED_SIZE as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    /// Sets whether maximum RMA size operations are permitted for the endpoint.
    ///
    /// Corresponds to `fi_setopt` with `FI_OPT_MAX_RMA_SIZE`
    fn set_max_rma_size(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_MAX_RMA_SIZE as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    fn set_max_atomic_size(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_MAX_ATOMIC_SIZE as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    fn set_inject_tagged_size(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_INJECT_TAGGED_SIZE as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    fn set_inject_rma_size(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_INJECT_RMA_SIZE as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }

    fn set_inject_atomic_size(&self, permitted: bool) -> Result<(), crate::error::Error> {
        let mut val = if permitted { 1_u32 } else { 0_u32 };
        let len = std::mem::size_of::<u32>();

        let err = unsafe {
            libfabric_sys::inlined_fi_setopt(
                self.as_typed_fid_mut().as_raw_fid(),
                libfabric_sys::FI_OPT_ENDPOINT as i32,
                libfabric_sys::FI_OPT_INJECT_ATOMIC_SIZE as i32,
                (&mut val as *mut u32).cast(),
                len,
            )
        };

        check_error(err.try_into().unwrap())
    }
}

//================== Endpoint attribute ==================//
#[derive(Clone)]
pub struct EndpointAttr {
    type_: EndpointType,
    protocol: Protocol,
    protocol_version: Version,
    max_msg_size: usize,
    msg_prefix_size: usize,
    max_order_raw_size: usize,
    max_order_war_size: usize,
    max_order_waw_size: usize,
    mem_tag_format: u64,
    tx_ctx_cnt: usize,
    rx_ctx_cnt: usize,
    auth_key: Option<Vec<u8>>,
}

impl EndpointAttr {
    pub(crate) fn new() -> Self {
        Self {
            type_: EndpointType::Unspec,
            protocol: Protocol::Unspec,
            protocol_version: Version { major: 0, minor: 0 },
            max_msg_size: 0,
            msg_prefix_size: 0,
            max_order_raw_size: 0,
            max_order_war_size: 0,
            max_order_waw_size: 0,
            mem_tag_format: 0,
            tx_ctx_cnt: 0,
            rx_ctx_cnt: 0,
            auth_key: None,
        }
    }

    pub(crate) fn from_raw_ptr(c_ep_attr: *const libfabric_sys::fi_ep_attr) -> Self {
        Self {
            type_: EndpointType::from_raw(unsafe { *c_ep_attr }.type_),
            protocol: Protocol::from_raw(unsafe { *c_ep_attr }.protocol),
            protocol_version: Version::from_raw(unsafe { *c_ep_attr }.protocol_version),
            max_msg_size: unsafe { *c_ep_attr }.max_msg_size,
            msg_prefix_size: unsafe { *c_ep_attr }.msg_prefix_size,
            max_order_raw_size: unsafe { *c_ep_attr }.max_order_raw_size,
            max_order_war_size: unsafe { *c_ep_attr }.max_order_war_size,
            max_order_waw_size: unsafe { *c_ep_attr }.max_order_waw_size,
            mem_tag_format: unsafe { *c_ep_attr }.mem_tag_format,
            tx_ctx_cnt: unsafe { *c_ep_attr }.tx_ctx_cnt,
            rx_ctx_cnt: unsafe { *c_ep_attr }.rx_ctx_cnt,
            auth_key: if !unsafe { *c_ep_attr }.auth_key.is_null() {
                Some(unsafe {
                    std::slice::from_raw_parts((*c_ep_attr).auth_key, (*c_ep_attr).auth_key_size)
                        .to_vec()
                })
            } else {
                None
            },
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> libfabric_sys::fi_ep_attr {
        let (auth_key, auth_key_size) = if let Some(auth_key) = &self.auth_key {
            (auth_key.as_ptr(), auth_key.len())
        } else {
            (std::ptr::null(), 0)
        };
        libfabric_sys::fi_ep_attr {
            type_: self.type_.as_raw(),
            protocol: self.protocol.as_raw(),
            protocol_version: self.protocol_version.as_raw(),
            max_msg_size: self.max_msg_size,
            msg_prefix_size: self.msg_prefix_size,
            max_order_raw_size: self.max_order_raw_size,
            max_order_war_size: self.max_order_war_size,
            max_order_waw_size: self.max_order_waw_size,
            mem_tag_format: self.mem_tag_format,
            tx_ctx_cnt: self.tx_ctx_cnt,
            rx_ctx_cnt: self.rx_ctx_cnt,
            auth_key: unsafe { std::mem::transmute::<*const u8, *mut u8>(auth_key) },
            auth_key_size,
        }
    }

    pub fn set_type(&mut self, type_: crate::enums::EndpointType) -> &mut Self {
        self.type_ = type_;
        self
    }

    pub fn set_protocol(&mut self, proto: crate::enums::Protocol) -> &mut Self {
        self.protocol = proto;
        self
    }

    pub fn set_protocol_version(&mut self, protocol_version: &Version) -> &mut Self {
        self.protocol_version = *protocol_version;
        self
    }

    pub fn set_max_msg_size(&mut self, size: usize) -> &mut Self {
        self.max_msg_size = size;
        self
    }

    pub fn set_msg_prefix_size(&mut self, size: usize) -> &mut Self {
        self.msg_prefix_size = size;
        self
    }

    pub fn set_max_order_raw_size(&mut self, size: usize) -> &mut Self {
        self.max_order_raw_size = size;
        self
    }

    pub fn set_max_order_war_size(&mut self, size: usize) -> &mut Self {
        self.max_order_war_size = size;
        self
    }

    pub fn set_max_order_waw_size(&mut self, size: usize) -> &mut Self {
        self.max_order_waw_size = size;
        self
    }

    pub fn set_mem_tag_format(&mut self, tag: u64) -> &mut Self {
        self.mem_tag_format = tag;
        self
    }

    pub fn set_tx_ctx_cnt(&mut self, size: usize) -> &mut Self {
        self.tx_ctx_cnt = size;
        self
    }

    pub fn set_rx_ctx_cnt(&mut self, size: usize) -> &mut Self {
        self.rx_ctx_cnt = size;
        self
    }

    pub fn set_auth_key(&mut self, key: &mut [u8]) -> &mut Self {
        self.auth_key = Some(key.to_vec());
        self
    }

    pub fn type_(&self) -> &crate::enums::EndpointType {
        &self.type_
    }

    pub fn protocol(&self) -> &crate::enums::Protocol {
        &self.protocol
    }

    pub fn protocol_version(&self) -> &Version {
        &self.protocol_version
    }

    pub fn max_msg_size(&self) -> usize {
        self.max_msg_size
    }

    pub fn msg_prefix_size(&self) -> usize {
        self.msg_prefix_size
    }

    pub fn max_order_raw_size(&self) -> usize {
        self.max_order_raw_size
    }

    pub fn max_order_war_size(&self) -> usize {
        self.max_order_war_size
    }

    pub fn max_order_waw_size(&self) -> usize {
        self.max_order_waw_size
    }

    pub fn tx_ctx_cnt(&self) -> usize {
        self.tx_ctx_cnt
    }

    pub fn rx_ctx_cnt(&self) -> usize {
        self.rx_ctx_cnt
    }

    pub fn mem_tag_format(&self) -> u64 {
        self.mem_tag_format
    }

    pub fn auth_key(&self) -> &Option<Vec<u8>> {
        &self.auth_key
    }
}

impl Default for EndpointAttr {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating endpoints with specific attributes and configurations.
pub struct EndpointBuilder<'a, E> {
    ep_attr: EndpointAttr,
    flags: u64,
    info: &'a InfoEntry<E>,
    ctx: Option<&'a mut Context>,
}

impl<'a> EndpointBuilder<'a, ()> {
    pub fn new<E>(info: &'a InfoEntry<E>) -> EndpointBuilder<'a, E> {
        EndpointBuilder::<E> {
            ep_attr: EndpointAttr::new(),
            flags: 0,
            info,
            ctx: None,
        }
    }
}

pub enum Endpoint<EP> {
    Connectionless(UninitConnectionlessEndpoint<EP>),
    ConnectionOriented(UninitUnconnectedEndpoint<EP>),
}

impl<EP> Endpoint<EP> {
    /// Binds a completion queue to the endpoint for both transmit and receive operations.
    /// If `selective` is true, selective completion is enabled.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_shared_cq<T: ReadCq + 'static>(
        &self,
        cq: &CompletionQueue<T>,
        selective: bool,
    ) -> Result<(), crate::error::Error> {
        match self {
            Endpoint::Connectionless(endpoint_base) => endpoint_base.bind_shared_cq(cq, selective),
            Endpoint::ConnectionOriented(endpoint_base) => {
                endpoint_base.bind_shared_cq(cq, selective)
            }
        }
    }

    /// Binds separate completion queues to the endpoint for transmit and receive operations.
    /// `tx_selective` enables selective completion for the transmit queue if true.
    /// `rx_selective` enables selective completion for the receive queue if true.
    ///
    /// Corresponds to `fi_ep_bind` in libfabric.
    pub fn bind_separate_cqs<T: ReadCq + 'static>(
        &self,
        tx_cq: &CompletionQueue<T>,
        tx_selective: bool,
        rx_cq: &CompletionQueue<T>,
        rx_selective: bool,
    ) -> Result<(), crate::error::Error> {
        match self {
            Endpoint::Connectionless(endpoint_base) => {
                endpoint_base.bind_separate_cqs(tx_cq, tx_selective, rx_cq, rx_selective)
            }
            Endpoint::ConnectionOriented(endpoint_base) => {
                endpoint_base.bind_separate_cqs(tx_cq, tx_selective, rx_cq, rx_selective)
            }
        }
    }
}

impl<E> EndpointBuilder<'_, E> {
    /// Builds an endpoint with separate completion queues for transmit and receive operations.
    /// `tx_selective_completion` enables selective completion for the transmit queue if true.
    /// `rx_selective_completion` enables selective completion for the receive queue if true.
    ///
    /// Corresponds to `fi_endpoint` and `fi_ep_bind` in libfabric.
    pub fn build_with_separate_cqs<EQ: ?Sized + 'static + SyncSend, CQ: ReadCq + 'static>(
        self,
        domain: &crate::domain::DomainBase<EQ>,
        tx_cq: &CompletionQueue<CQ>,
        tx_selective_completion: bool,
        rx_cq: &CompletionQueue<CQ>,
        rx_selective_completion: bool,
    ) -> Result<Endpoint<E>, crate::error::Error> {
        match self.info.ep_attr().type_() {
            EndpointType::Unspec => panic!("Should not be reachable."),
            EndpointType::Msg => {
                let conn_ep =
                    UninitUnconnectedEndpoint::new(domain, self.info, self.flags, self.ctx)?;
                conn_ep.bind_separate_cqs(
                    tx_cq,
                    tx_selective_completion,
                    rx_cq,
                    rx_selective_completion,
                )?;
                Ok(Endpoint::ConnectionOriented(conn_ep))
            }
            EndpointType::Dgram | EndpointType::Rdm => {
                let connless_ep =
                    UninitConnectionlessEndpoint::new(domain, self.info, self.flags, self.ctx)?;
                connless_ep.bind_separate_cqs(
                    tx_cq,
                    tx_selective_completion,
                    rx_cq,
                    rx_selective_completion,
                )?;
                Ok(Endpoint::Connectionless(connless_ep))
            }
        }
    }

    /// Builds an endpoint with a shared completion queue for both transmit and receive operations.
    /// If `selective_completion` is true, selective completion is enabled.
    ///
    /// Corresponds to `fi_endpoint` and `fi_ep_bind` in libfabric.
    pub fn build_with_shared_cq<EQ: ?Sized + 'static + SyncSend, CQ: ReadCq + 'static>(
        self,
        domain: &crate::domain::DomainBase<EQ>,
        cq: &CompletionQueue<CQ>,
        selective_completion: bool,
    ) -> Result<Endpoint<E>, crate::error::Error> {
        match self.info.ep_attr().type_() {
            EndpointType::Unspec => panic!("Should not be reachable."),
            EndpointType::Msg => {
                let conn_ep =
                    UninitUnconnectedEndpoint::new(domain, self.info, self.flags, self.ctx)?;
                conn_ep.bind_shared_cq(cq, selective_completion)?;
                Ok(Endpoint::ConnectionOriented(conn_ep))
            }
            EndpointType::Dgram | EndpointType::Rdm => {
                let connless_ep =
                    UninitConnectionlessEndpoint::new(domain, self.info, self.flags, self.ctx)?;
                connless_ep.bind_shared_cq(cq, selective_completion)?;
                Ok(Endpoint::Connectionless(connless_ep))
            }
        }
    }

    /// Builds a scalable endpoint associated with the given domain.
    ///
    /// Corresponds to `fi_scalable_ep` in libfabric.
    pub fn build_scalable<EQ: ?Sized + 'static + SyncSend>(
        self,
        domain: &crate::domain::DomainBase<EQ>,
    ) -> Result<ScalableEndpoint<E>, crate::error::Error> {
        ScalableEndpoint::new(domain, self.info, self.ctx)
    }

    /// Builds a passive endpoint associated with the given fabric.
    ///
    /// Corresponds to `fi_passive_ep` in libfabric.
    pub fn build_passive(
        self,
        fabric: &crate::fabric::Fabric,
    ) -> Result<PassiveEndpoint<E>, crate::error::Error> {
        PassiveEndpoint::new(fabric, self.info, self.ctx)
    }

    // pub(crate) fn from(c_ep_attr: *mut libfabric_sys::fi_ep_attr) -> Self {
    //     let c_attr = unsafe { *c_ep_attr };

    //     Self { c_attr }
    // }

    /// Sets flags for the endpoint.
    pub fn flags(mut self, flags: u64) -> Self {
        self.flags = flags;
        self
    }

    /// Sets the type of the endpoint.
    pub fn ep_type(mut self, type_: crate::enums::EndpointType) -> Self {
        self.ep_attr.set_type(type_);
        self
    }

    /// Sets the protocol for the endpoint.
    pub fn protocol(mut self, proto: crate::enums::Protocol) -> Self {
        self.ep_attr.set_protocol(proto);
        self
    }

    /// Sets maximum msgage size for the endpoint.
    pub fn max_msg_size(mut self, size: usize) -> Self {
        self.ep_attr.set_max_msg_size(size);
        self
    }

    /// Sets the message prefix size for the endpoint.
    pub fn msg_prefix_size(mut self, size: usize) -> Self {
        self.ep_attr.set_msg_prefix_size(size);
        self
    }

    /// Sets the maximum order RAW size for the endpoint.
    pub fn max_order_raw_size(mut self, size: usize) -> Self {
        self.ep_attr.set_max_order_raw_size(size);
        self
    }

    /// Sets the maximum order WAR size for the endpoint.
    pub fn max_order_war_size(mut self, size: usize) -> Self {
        self.ep_attr.set_max_order_war_size(size);
        self
    }

    /// Sets the maximum order WAW size for the endpoint.
    pub fn max_order_waw_size(mut self, size: usize) -> Self {
        self.ep_attr.set_max_order_waw_size(size);
        self
    }

    /// Sets the memory tag format for the endpoint.
    pub fn mem_tag_format(mut self, tag: u64) -> Self {
        self.ep_attr.set_mem_tag_format(tag);
        self
    }

    /// Sets the transmit context count for the endpoint.
    pub fn tx_ctx_cnt(mut self, size: usize) -> Self {
        self.ep_attr.set_tx_ctx_cnt(size);
        self
    }

    /// Sets the receive context count for the endpoint.
    pub fn rx_ctx_cnt(mut self, size: usize) -> Self {
        self.ep_attr.set_rx_ctx_cnt(size);
        self
    }

    /// Sets the authentication key for the endpoint.
    pub fn auth_key(mut self, key: &mut [u8]) -> Self {
        self.ep_attr.set_auth_key(key);
        self
    }
}
