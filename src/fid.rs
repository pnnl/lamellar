use std::marker::PhantomData;
#[cfg(feature = "threading-completion")]
use std::sync::OnceLock;
use crate::{error, MyRc};
pub(crate) type RawFid = *mut libfabric_sys::fid;

#[derive(Hash, Clone, Copy)]
pub struct Fid(pub(crate) usize);

pub(crate) struct TypedFid<FID: AsRawFid>(pub(crate) FID);
unsafe impl<FID: AsRawFid> Send for TypedFid<FID> {}
#[cfg(feature = "thread-safe")]
unsafe impl<FID: AsRawFid> Sync for TypedFid<FID> {}
impl PartialEq for Fid {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Fid {}

// impl AsRawFid for Fid {
//     fn as_raw_fid(&self) -> RawFid {
//         self.0
//     }
// }

#[cfg(all(feature = "thread-safe", not(feature = "threading-thread-safe")))]
pub struct OwnedTypedFid<FID: AsRawFid> {
    pub(crate) typed_fid: std::sync::Arc<parking_lot::RwLock<TypedFid<FID>>>,
}

#[cfg(not(all(feature = "thread-safe", not(feature = "threading-thread-safe"))))]
pub struct OwnedTypedFid<FID: AsRawFid> {
    pub(crate) typed_fid: TypedFid<FID>,
}

#[cfg(feature = "threading-endpoint")]
pub struct XContextOwnedTypedFid<FID: AsRawFid> {
    pub(crate) typed_fid: OwnedTypedFid<FID>,
    pub(crate) parent_ep: std::sync::Arc<parking_lot::RwLock<TypedFid<EpRawFid>>>,
}

#[cfg(feature = "threading-completion")]
pub struct EpCompletionOwnedTypedFid<FID: AsRawFid> {
    pub(crate) typed_fid: OwnedTypedFid<FID>,
    pub(crate) bound_cq0:
        std::sync::OnceLock<std::sync::Arc<parking_lot::RwLock<TypedFid<CqRawFid>>>>,
    pub(crate) bound_cq1:
        std::sync::OnceLock<std::sync::Arc<parking_lot::RwLock<TypedFid<CqRawFid>>>>,
    pub(crate) bound_cntr:
        std::sync::OnceLock<std::sync::Arc<parking_lot::RwLock<TypedFid<CntrRawFid>>>>,
}

#[cfg(feature = "threading-domain")]
pub struct DomainOwnedTypedFid<FID: AsRawFid> {
    pub(crate) typed_fid: OwnedTypedFid<FID>,
    pub(crate) domain: std::sync::Arc<parking_lot::Mutex<TypedFid<DomainRawFid>>>,
}

#[cfg(feature = "thread-safe")]
mod thread_safe {
    unsafe impl<FID: super::AsRawFid> Send for super::OwnedTypedFid<FID> {}
    unsafe impl Send for super::Fid {}
}

#[cfg(feature = "threading-thread-safe")]
mod threading_thread_safe {
    unsafe impl<FID: super::AsRawFid> Sync for super::OwnedTypedFid<FID> {}
    unsafe impl Sync for super::Fid {}
}

// #[cfg(feature="threading-fid")]
impl<FID: AsRawFid + AsRawTypedFid> OwnedTypedFid<FID> {
    pub(crate) fn from(typed_fid: FID) -> Self {
        #[cfg(any(
            feature = "threading-fid",
            feature = "threading-domain",
            feature = "threading-completion"
        ))]
        return Self {
            typed_fid: std::sync::Arc::new(parking_lot::RwLock::new(TypedFid(typed_fid))),
        };
        #[cfg(not(any(
            feature = "threading-fid",
            feature = "threading-domain",
            feature = "threading-completion"
        )))]
        return Self {
            typed_fid: TypedFid(typed_fid),
        };
    }
}

#[cfg(feature = "threading-domain")]
impl<FID: AsRawFid + AsRawTypedFid> DomainOwnedTypedFid<FID> {
    pub(crate) fn from(
        typed_fid: FID,
        domain: std::sync::Arc<parking_lot::Mutex<TypedFid<DomainRawFid>>>,
    ) -> Self {
        return Self {
            typed_fid: OwnedTypedFid::from(typed_fid),
            domain,
        };
    }
}

#[cfg(feature = "threading-endpoint")]
impl<FID: AsRawFid + AsRawTypedFid> XContextOwnedTypedFid<FID> {
    pub(crate) fn from(
        typed_fid: FID,
        parent_ep: std::sync::Arc<parking_lot::RwLock<TypedFid<EpRawFid>>>,
    ) -> Self {
        return Self {
            typed_fid: OwnedTypedFid::from(typed_fid),
            parent_ep,
        };
    }
}

#[cfg(feature = "threading-completion")]
impl<FID: AsRawFid + AsRawTypedFid> EpCompletionOwnedTypedFid<FID> {
    pub(crate) fn from(typed_fid: FID) -> Self {
        return Self {
            typed_fid: OwnedTypedFid::from(typed_fid),
            bound_cq0: OnceLock::new(),
            bound_cq1: OnceLock::new(),
            bound_cntr: OnceLock::new(),
        };
    }
}

#[cfg(any(
    feature = "threading-fid",
    feature = "threading-domain",
    feature = "threading-completion"
))]
pub struct ProtectedBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: parking_lot::RwLockReadGuard<'a, TypedFid<FID>>,
    phantom: PhantomData<&'a OwnedTypedFid<FID>>,
}

#[cfg(any(
    feature = "threading-fid",
    feature = "threading-domain",
    feature = "threading-completion"
))]
pub struct MutProtectedBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: parking_lot::RwLockWriteGuard<'a, TypedFid<FID>>,
    phantom: PhantomData<&'a OwnedTypedFid<FID>>,
}

pub enum BorrowedTypedFid<'a, FID: AsRawFid> {
    #[cfg(any(not(feature = "thread-safe"), feature = "threading-thread-safe"))]
    Unprotected(UnprotectedBorrowedTypedFid<'a, FID>),
    #[cfg(any(
        feature = "threading-fid",
        feature = "threading-domain",
        feature = "threading-completion"
    ))]
    Protected(ProtectedBorrowedTypedFid<'a, FID>),
    #[cfg(feature = "threading-endpoint")]
    XContext(XContextBorrowedTypedFid<'a, FID>),
    #[cfg(feature = "threading-domain")]
    Domain(DomainBorrowedTypedFid<'a, FID>),
    #[cfg(feature = "threading-completion")]
    Completion(EpCompletionBorrowedTypedFid<'a, FID>),
}

pub enum MutBorrowedTypedFid<'a, FID: AsRawFid> {
    #[cfg(any(not(feature = "thread-safe"), feature = "threading-thread-safe"))]
    Unprotected(UnprotectedBorrowedTypedFid<'a, FID>),
    #[cfg(any(
        feature = "threading-fid",
        feature = "threading-domain",
        feature = "threading-completion"
    ))]
    Protected(MutProtectedBorrowedTypedFid<'a, FID>),
    #[cfg(feature = "threading-endpoint")]
    XContext(MutXContextBorrowedTypedFid<'a, FID>),
    #[cfg(feature = "threading-domain")]
    Domain(MutDomainBorrowedTypedFid<'a, FID>),
    #[cfg(feature = "threading-completion")]
    Completion(MutEpCompletionBorrowedTypedFid<'a, FID>),
}

#[cfg(feature = "threading-endpoint")]
pub struct XContextBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: parking_lot::RwLockReadGuard<'a, TypedFid<FID>>,
    // _parent_ep: parking_lot::MutexGuard<'a, TypedFid<EpRawFid>>,
    phantom: PhantomData<&'a OwnedTypedFid<EpRawFid>>,
}

#[cfg(feature = "threading-endpoint")]
pub struct MutXContextBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: parking_lot::RwLockWriteGuard<'a, TypedFid<FID>>,
    _parent_ep: parking_lot::RwLockWriteGuard<'a, TypedFid<EpRawFid>>,
    phantom: PhantomData<&'a OwnedTypedFid<EpRawFid>>,
}

#[cfg(feature = "threading-domain")]
pub struct DomainBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: parking_lot::RwLockReadGuard<'a, TypedFid<FID>>,
    // _domain_fid: parking_lot::MutexGuard<'a, TypedFid<DomainRawFid>>,
    phantom: PhantomData<&'a OwnedTypedFid<FID>>,
}

#[cfg(feature = "threading-completion")]
pub struct EpCompletionBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: parking_lot::RwLockReadGuard<'a, TypedFid<FID>>,
    // _domain_fid: parking_lot::MutexGuard<'a, TypedFid<DomainRawFid>>,
    phantom: PhantomData<&'a OwnedTypedFid<FID>>,
}

#[cfg(feature = "threading-completion")]
pub struct MutEpCompletionBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: parking_lot::RwLockWriteGuard<'a, TypedFid<FID>>,
    bound_cq0: Option<parking_lot::RwLockWriteGuard<'a, TypedFid<CqRawFid>>>,
    bound_cq1: Option<parking_lot::RwLockWriteGuard<'a, TypedFid<CqRawFid>>>,
    bound_cntr: Option<parking_lot::RwLockWriteGuard<'a, TypedFid<CntrRawFid>>>,
    phantom: PhantomData<&'a OwnedTypedFid<FID>>,
}

#[cfg(feature = "threading-domain")]
pub struct MutDomainBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: parking_lot::RwLockWriteGuard<'a, TypedFid<FID>>,
    _domain_fid: parking_lot::MutexGuard<'a, TypedFid<DomainRawFid>>,
    phantom: PhantomData<&'a OwnedTypedFid<FID>>,
}

#[cfg(not(any(feature = "threading-fid", feature = "threading-domain")))]
pub struct UnprotectedBorrowedTypedFid<'a, FID: AsRawFid> {
    typed_fid: &'a TypedFid<FID>,
    phantom: PhantomData<&'a OwnedTypedFid<FID>>,
}

// impl BorrowedFid<'_> {
//     #[inline]
//     pub const unsafe fn borrow_raw(fid: RawFid) -> Self {
//         Self {
//             fid,
//             phantom: PhantomData,
//         }
//     }
// }

// pub struct BorrowedTypedFid<'a, FID: AsRawFid > {
//     typed_fid: FID,
//     phantom: PhantomData<&'a OwnedTypedFid<FID>>,
// }

// impl<FID: AsRawFid > BorrowedTypedFid<'_, FID> {
//     #[inline]
//     pub const unsafe fn borrow_raw(typed_fid: FID) -> Self {
//         Self {
//             typed_fid,
//             phantom: PhantomData,
//         }
//     }
// }

impl<'a, FID: AsRawFid + AsRawTypedFid<Output = FID>> AsRawTypedFid for BorrowedTypedFid<'a, FID> {
    type Output = FID;
    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        match self {
            #[cfg(any(
                feature = "threading-fid",
                feature = "threading-domain",
                feature = "threading-completion"
            ))]
            Self::Protected(this) => this.typed_fid.0.as_raw_typed_fid(),
            #[cfg(any(not(feature = "thread-safe"), feature = "threading-thread-safe"))]
            Self::Unprotected(this) => this.typed_fid.0.as_raw_typed_fid(),
            #[cfg(feature = "threading-endpoint")]
            Self::XContext(this) => this.typed_fid.0.as_raw_typed_fid(),
            #[cfg(feature = "threading-domain")]
            Self::Domain(this) => this.typed_fid.0.as_raw_typed_fid(),
            #[cfg(feature = "threading-completion")]
            Self::Completion(this) => this.typed_fid.0.as_raw_typed_fid(),
        }
    }
}

impl<'a, FID: AsRawFid + AsRawTypedFid<Output = FID>> AsRawTypedFid
    for MutBorrowedTypedFid<'a, FID>
{
    type Output = FID;
    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        match self {
            #[cfg(any(
                feature = "threading-fid",
                feature = "threading-domain",
                feature = "threading-completion"
            ))]
            Self::Protected(this) => this.typed_fid.0.as_raw_typed_fid(),
            #[cfg(any(not(feature = "thread-safe"), feature = "threading-thread-safe"))]
            Self::Unprotected(this) => this.typed_fid.0.as_raw_typed_fid(),
            #[cfg(feature = "threading-endpoint")]
            Self::XContext(this) => this.typed_fid.0.as_raw_typed_fid(),
            #[cfg(feature = "threading-domain")]
            Self::Domain(this) => this.typed_fid.0.as_raw_typed_fid(),
            #[cfg(feature = "threading-completion")]
            Self::Completion(this) => this.typed_fid.0.as_raw_typed_fid(),
        }
    }
}

impl<'a, FID: AsRawFid> AsRawFid for BorrowedTypedFid<'a, FID> {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        match self {
            #[cfg(any(
                feature = "threading-fid",
                feature = "threading-domain",
                feature = "threading-completion"
            ))]
            Self::Protected(this) => this.typed_fid.0.as_raw_fid(),
            #[cfg(any(not(feature = "thread-safe"), feature = "threading-thread-safe"))]
            Self::Unprotected(this) => this.typed_fid.0.as_raw_fid(),
            #[cfg(feature = "threading-endpoint")]
            Self::XContext(this) => this.typed_fid.0.as_raw_fid(),
            #[cfg(feature = "threading-domain")]
            Self::Domain(this) => this.typed_fid.0.as_raw_fid(),
            #[cfg(feature = "threading-completion")]
            Self::Completion(this) => this.typed_fid.0.as_raw_fid(),
        }
    }
}

impl<'a, FID: AsRawFid> AsRawFid for MutBorrowedTypedFid<'a, FID> {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        match self {
            #[cfg(any(
                feature = "threading-fid",
                feature = "threading-domain",
                feature = "threading-completion"
            ))]
            Self::Protected(this) => this.typed_fid.0.as_raw_fid(),
            #[cfg(any(not(feature = "thread-safe"), feature = "threading-thread-safe"))]
            Self::Unprotected(this) => this.typed_fid.0.as_raw_fid(),
            #[cfg(feature = "threading-endpoint")]
            Self::XContext(this) => this.typed_fid.0.as_raw_fid(),
            #[cfg(feature = "threading-domain")]
            Self::Domain(this) => this.typed_fid.0.as_raw_fid(),
            #[cfg(feature = "threading-completion")]
            Self::Completion(this) => this.typed_fid.0.as_raw_fid(),
        }
    }
}

impl<FID: AsRawFid> Drop for OwnedTypedFid<FID> {
    #[inline]
    fn drop(&mut self) {
        let err = unsafe { libfabric_sys::inlined_fi_close(self.as_typed_fid().as_raw_fid()) };
        if err != 0 {
            panic!(
                "{}",
                error::Error::from_err_code((-err).try_into().unwrap())
            );
        }
    }
}

impl<T: AsRawFid> AsRawFid for MyRc<T> {
    fn as_raw_fid(&self) -> RawFid {
        (**self).as_raw_fid()
    }
}

// impl<FID: AsRawFid > AsRawTypedFid for OwnedTypedFid<FID> {
//     type Output = FID;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.typed_fid
//     }
// }

// impl<FID: AsRawFid> AsRawFid for OwnedTypedFid<FID> {
//     fn as_raw_fid(&self) -> RawFid {
//         self.typed_fid.as_raw_fid()
//     }
// }

// impl<FID: AsRawFid> AsFid for OwnedTypedFid<FID> {
//     fn as_fid(&self) -> BorrowedFid<'_> {
//         unsafe { BorrowedFid::borrow_raw(self.as_raw_fid()) }
//     }
// }

impl<FID: AsRawFid> AsTypedFid<FID> for OwnedTypedFid<FID> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<'_, FID> {
        #[cfg(any(
            feature = "threading-fid",
            feature = "threading-domain",
            feature = "threading-completion"
        ))]
        return BorrowedTypedFid::Protected(ProtectedBorrowedTypedFid {
            typed_fid: self.typed_fid.read(),
            phantom: PhantomData,
        });
        #[cfg(any(not(feature = "thread-safe"), feature = "threading-thread-safe"))]
        return BorrowedTypedFid::Unprotected(UnprotectedBorrowedTypedFid {
            typed_fid: &self.typed_fid,
            phantom: PhantomData,
        });
        // #[cfg(feature="threading-domain")]
        // return BorrowedTypedFid::Unprotected(UnprotectedBorrowedTypedFid {
        //     typed_fid: &self.typed_fid,
        //     phantom: PhantomData,
        // });
    }

    fn as_typed_fid_mut(&self) -> MutBorrowedTypedFid<'_, FID> {
        #[cfg(any(
            feature = "threading-fid",
            feature = "threading-domain",
            feature = "threading-completion"
        ))]
        return MutBorrowedTypedFid::Protected(MutProtectedBorrowedTypedFid {
            typed_fid: self.typed_fid.write(),
            phantom: PhantomData,
        });
        #[cfg(any(not(feature = "thread-safe"), feature = "threading-thread-safe"))]
        return MutBorrowedTypedFid::Unprotected(UnprotectedBorrowedTypedFid {
            typed_fid: &self.typed_fid,
            phantom: PhantomData,
        });
        // #[cfg(feature="threading-domain")]
        // return BorrowedTypedFid::Unprotected(UnprotectedBorrowedTypedFid {
        //     typed_fid: &self.typed_fid,
        //     phantom: PhantomData,
        // });
    }
}

#[cfg(feature = "threading-endpoint")]
impl<FID: AsRawFid> AsTypedFid<FID> for XContextOwnedTypedFid<FID> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<'_, FID> {
        BorrowedTypedFid::XContext(XContextBorrowedTypedFid {
            // _parent_ep: self.parent_ep.lock(),
            typed_fid: self.typed_fid.typed_fid.read(),
            phantom: PhantomData,
        })
    }

    fn as_typed_fid_mut(&self) -> MutBorrowedTypedFid<'_, FID> {
        MutBorrowedTypedFid::XContext(MutXContextBorrowedTypedFid {
            _parent_ep: self.parent_ep.write(),
            typed_fid: self.typed_fid.typed_fid.write(),
            phantom: PhantomData,
        })
    }
}

#[cfg(feature = "threading-domain")]
impl<FID: AsRawFid> AsTypedFid<FID> for DomainOwnedTypedFid<FID> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<'_, FID> {
        BorrowedTypedFid::Domain(DomainBorrowedTypedFid {
            // _domain_fid: self.domain.lock(),
            typed_fid: self.typed_fid.typed_fid.read(),
            phantom: PhantomData,
        })
    }

    fn as_typed_fid_mut(&self) -> MutBorrowedTypedFid<'_, FID> {
        MutBorrowedTypedFid::Domain(MutDomainBorrowedTypedFid {
            _domain_fid: self.domain.lock(),
            typed_fid: self.typed_fid.typed_fid.write(),
            phantom: PhantomData,
        })
    }
}

#[cfg(feature = "threading-completion")]
impl<FID: AsRawFid> AsTypedFid<FID> for EpCompletionOwnedTypedFid<FID> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<'_, FID> {
        BorrowedTypedFid::Completion(EpCompletionBorrowedTypedFid {
            // _domain_fid: self.domain.lock(),
            typed_fid: self.typed_fid.typed_fid.read(),
            phantom: PhantomData,
        })
    }

    fn as_typed_fid_mut(&self) -> MutBorrowedTypedFid<'_, FID> {
        MutBorrowedTypedFid::Completion(MutEpCompletionBorrowedTypedFid {
            bound_cq0: self.bound_cq0.get().map(|v| v.write()),
            bound_cq1: self.bound_cq1.get().map(|v| v.write()),
            bound_cntr: self.bound_cntr.get().map(|v| v.write()),
            typed_fid: self.typed_fid.typed_fid.write(),
            phantom: PhantomData,
        })
    }
}

// pub trait AsFid {
//     fn as_fid(&self) -> BorrowedTypedFid<>;
// }
// pub trait AsTypedFid {
//     fn as_fid(&self) -> BorrowedTypedFid<>;
// }

pub trait AsRawFid {
    fn as_raw_fid(&self) -> RawFid;
}

pub trait AsTypedFid<FID: AsRawFid> {
    fn as_typed_fid(&self) -> BorrowedTypedFid<FID>;
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<FID>;
}

pub trait AsRawTypedFid {
    type Output: AsRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output;
}

impl AsRawFid for RawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        *self
    }
}

// impl<'a> AsRawFid for BorrowedFid<'a> {
//     #[inline]
//     fn as_raw_fid(&self) -> RawFid {
//         self.fid
//     }
// }

pub(crate) type DomainRawFid = *mut libfabric_sys::fid_domain;

#[cfg(not(feature = "threading-domain"))]
pub(crate) type OwnedDomainFid = OwnedTypedFid<DomainRawFid>;
#[cfg(feature = "threading-domain")]
pub(crate) type OwnedDomainFid = DomainOwnedTypedFid<DomainRawFid>;

impl AsRawTypedFid for DomainRawFid {
    type Output = DomainRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for DomainRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

pub(crate) type AvRawFid = *mut libfabric_sys::fid_av;

impl AsRawTypedFid for AvRawFid {
    type Output = AvRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for AvRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

#[cfg(not(feature = "threading-domain"))]
pub(crate) type OwnedAVFid = OwnedTypedFid<AvRawFid>;
#[cfg(feature = "threading-domain")]
pub(crate) type OwnedAVFid = DomainOwnedTypedFid<AvRawFid>;

pub(crate) type AVSetRawFid = *mut libfabric_sys::fid_av_set;

impl AsRawTypedFid for AVSetRawFid {
    type Output = AVSetRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for AVSetRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

pub(crate) type OwnedAVSetFid = OwnedTypedFid<AVSetRawFid>;
// #[cfg(feature="threading-domain")]
// pub(crate) type OwnedAVSetFid = DomainOwnedTypedFid<AVSetRawFid>;

pub(crate) type CntrRawFid = *mut libfabric_sys::fid_cntr;

impl AsRawTypedFid for CntrRawFid {
    type Output = CntrRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for CntrRawFid {
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

#[cfg(not(feature = "threading-domain"))]
pub(crate) type OwnedCntrFid = OwnedTypedFid<CntrRawFid>;
#[cfg(feature = "threading-domain")]
pub(crate) type OwnedCntrFid = DomainOwnedTypedFid<CntrRawFid>;

pub(crate) type CqRawFid = *mut libfabric_sys::fid_cq;

impl AsRawTypedFid for CqRawFid {
    type Output = CqRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for CqRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

#[cfg(not(feature = "threading-domain"))]
pub(crate) type OwnedCqFid = OwnedTypedFid<CqRawFid>;
#[cfg(feature = "threading-domain")]
pub(crate) type OwnedCqFid = DomainOwnedTypedFid<CqRawFid>;

pub(crate) type FabricRawFid = *mut libfabric_sys::fid_fabric;

impl AsRawTypedFid for FabricRawFid {
    type Output = FabricRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for FabricRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

pub(crate) type OwnedFabricFid = OwnedTypedFid<FabricRawFid>;

pub(crate) type MrRawFid = *mut libfabric_sys::fid_mr;

impl AsRawTypedFid for MrRawFid {
    type Output = MrRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for MrRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

#[cfg(not(feature = "threading-domain"))]
pub(crate) type OwnedMrFid = OwnedTypedFid<MrRawFid>;
#[cfg(feature = "threading-domain")]
pub(crate) type OwnedMrFid = DomainOwnedTypedFid<MrRawFid>;

pub(crate) type EqRawFid = *mut libfabric_sys::fid_eq;

impl AsRawTypedFid for EqRawFid {
    type Output = EqRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for EqRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

pub(crate) type OwnedEqFid = OwnedTypedFid<EqRawFid>;

pub(crate) type WaitRawFid = *mut libfabric_sys::fid_wait;

impl AsRawTypedFid for WaitRawFid {
    type Output = WaitRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for WaitRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

pub(crate) type OwnedWaitFid = OwnedTypedFid<WaitRawFid>;

pub(crate) type EpRawFid = *mut libfabric_sys::fid_ep;

impl AsRawTypedFid for EpRawFid {
    type Output = EpRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for EpRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

#[cfg(not(feature = "threading-domain"))]
pub type OwnedEpFid = OwnedTypedFid<EpRawFid>;
#[cfg(feature = "threading-domain")]
pub type OwnedEpFid = DomainOwnedTypedFid<EpRawFid>;

pub(crate) type PepRawFid = *mut libfabric_sys::fid_pep;

impl AsRawTypedFid for PepRawFid {
    type Output = PepRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for PepRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

pub(crate) type OwnedPepFid = OwnedTypedFid<PepRawFid>;

pub(crate) type McRawFid = *mut libfabric_sys::fid_mc;

impl AsRawTypedFid for McRawFid {
    type Output = McRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for McRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

pub(crate) type OwnedMcFid = OwnedTypedFid<McRawFid>;

pub(crate) type PollRawFid = *mut libfabric_sys::fid_poll;

impl AsRawTypedFid for PollRawFid {
    type Output = PollRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}

impl AsRawFid for PollRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

#[cfg(not(feature = "threading-domain"))]
pub(crate) type OwnedPollFid = OwnedTypedFid<PollRawFid>;
#[cfg(feature = "threading-domain")]
pub(crate) type OwnedPollFid = DomainOwnedTypedFid<PollRawFid>;

pub(crate) type ProfileRawFid = *mut libfabric_sys::fid_profile;
impl AsRawTypedFid for ProfileRawFid {
    type Output = ProfileRawFid;

    #[inline]
    fn as_raw_typed_fid(&self) -> Self::Output {
        *self
    }
}
impl AsRawFid for ProfileRawFid {
    #[inline]
    fn as_raw_fid(&self) -> RawFid {
        unsafe { &mut (**self).fid }
    }
}

pub(crate) type OwnedProfileFid = OwnedTypedFid<ProfileRawFid>;
