use std::ffi::{CStr, CString};

//================== Fabric (fi_fabric) ==================//
#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{
    fid::{
        AsRawFid, AsRawTypedFid, AsTypedFid, BorrowedFid, BorrowedTypedFid, FabricRawFid,
        OwnedFabricFid, RawFid,
    },
    info::{InfoEntry, Version},
    utils::check_error,
    Context, MyRc,
};

pub(crate) struct FabricImpl {
    pub(crate) c_fabric: OwnedFabricFid,
}

/// Owned wrapper around a libfabric `fid_fabric`.
///
/// This type wraps an instance of a `fid_fabric`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_fabric.3.html).
///
/// Note that other objects that rely on a `Fabric` (e.g., [`PassiveEndpoint`](crate::ep::PassiveEndpoint)) will extend its lifetime until they
/// are also dropped.
pub struct Fabric {
    pub(crate) inner: MyRc<FabricImpl>,
}

impl FabricImpl {
    pub(crate) fn new(
        attr: FabricAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_fabric: FabricRawFid = std::ptr::null_mut();
        let mut c_attr = unsafe { attr.get() };
        let err = unsafe { libfabric_sys::fi_fabric(&mut c_attr, &mut c_fabric, context) };

        if err != 0 || c_fabric.is_null() {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                c_fabric: OwnedFabricFid::from(c_fabric),
            })
        }
    }

    pub(crate) fn trywait_slice(&self, fids: &[&impl AsFid]) -> Result<(), crate::error::Error> {
        // [TODO] Move this into the WaitSet struct
        let mut raw_fids: Vec<*mut libfabric_sys::fid> =
            fids.iter().map(|x| x.as_fid().as_raw_fid()).collect();
        let err = unsafe {
            libfabric_sys::inlined_fi_trywait(
                self.as_raw_typed_fid(),
                raw_fids.as_mut_ptr(),
                raw_fids.len() as i32,
            )
        };

        check_error(err.try_into().unwrap())
    }

    pub(crate) fn trywait(&self, fid: &impl AsFid) -> Result<(), crate::error::Error> {
        // [TODO] Move this into the WaitSet struct
        let mut raw_fid = fid.as_fid().as_raw_fid();
        let err =
            unsafe { libfabric_sys::inlined_fi_trywait(self.as_raw_typed_fid(), &mut raw_fid, 1) };

        check_error(err.try_into().unwrap())
    }
}

impl Fabric {
    pub(crate) fn new(
        attr: FabricAttr,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(FabricImpl::new(attr, c_void)?),
        })
    }

    pub fn trywait_slice(&self, fids: &[&impl AsFid]) -> Result<(), crate::error::Error> {
        // [TODO] Move this into the WaitSet struct
        self.inner.trywait_slice(fids)
    }

    pub fn trywait(&self, fid: &impl AsFid) -> Result<(), crate::error::Error> {
        // [TODO] Move this into the WaitSet struct
        self.inner.trywait(fid)
    }
}

impl AsFid for FabricImpl {
    fn as_fid(&self) -> BorrowedFid<'_> {
        self.c_fabric.as_fid()
    }
}

impl AsRawFid for FabricImpl {
    fn as_raw_fid(&self) -> RawFid {
        self.c_fabric.as_raw_fid()
    }
}

impl AsTypedFid<FabricRawFid> for FabricImpl {
    fn as_typed_fid(&self) -> BorrowedTypedFid<FabricRawFid> {
        self.c_fabric.as_typed_fid()
    }
}

impl AsRawTypedFid for FabricImpl {
    type Output = FabricRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_fabric.as_raw_typed_fid()
    }
}

impl AsFid for Fabric {
    fn as_fid(&self) -> BorrowedFid {
        self.inner.as_fid()
    }
}

impl AsRawFid for Fabric {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}

impl AsTypedFid<FabricRawFid> for Fabric {
    fn as_typed_fid(&self) -> BorrowedTypedFid<FabricRawFid> {
        self.inner.as_typed_fid()
    }
}

impl AsRawTypedFid for Fabric {
    type Output = FabricRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

//================== Fabric attribute ==================//

#[derive(Clone, Debug)]
pub struct FabricAttr {
    fabric_id: usize,
    name: String,
    prov_name: String,
    _c_name: CString,
    _c_prov_name: CString,
    prov_version: Version,
    api_version: Version,
}

impl FabricAttr {
    pub(crate) fn from_raw_ptr(c_fab_att: *const libfabric_sys::fi_fabric_attr) -> Self {
        assert!(!c_fab_att.is_null());
        let c_name = unsafe {
            if !(*c_fab_att).name.is_null() {
                CStr::from_ptr((*c_fab_att).name).into()
            } else {
                CString::new("").unwrap()
            }
        };
        let c_prov_name = unsafe {
            if !(*c_fab_att).prov_name.is_null() {
                CStr::from_ptr((*c_fab_att).prov_name).into()
            } else {
                CString::new("").unwrap()
            }
        };
        Self {
            fabric_id: unsafe { *c_fab_att }.fabric as usize,
            name: c_name.to_str().unwrap().to_string(),
            prov_name: c_prov_name.to_str().unwrap().to_string(),
            _c_name: c_name,
            _c_prov_name: c_prov_name,
            prov_version: Version::from_raw(unsafe { *c_fab_att }.prov_version),
            api_version: Version::from_raw(unsafe { *c_fab_att }.api_version),
        }
    }

    pub(crate) unsafe fn get(&self) -> libfabric_sys::fi_fabric_attr {
        libfabric_sys::fi_fabric_attr {
            fabric: self.fabric_id as *mut libfabric_sys::fid_fabric,
            prov_name: unsafe {
                std::mem::transmute::<*const i8, *mut i8>(self._c_prov_name.as_ptr())
            },
            name: unsafe { std::mem::transmute::<*const i8, *mut i8>(self._c_name.as_ptr()) },
            prov_version: self.prov_version.as_raw(),
            api_version: self.api_version.as_raw(),
        }
    }

    pub fn fabric_id(&self) -> usize {
        self.fabric_id
    }

    pub fn prov_name(&self) -> &str {
        &self.prov_name
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn prov_version(&self) -> Version {
        self.prov_version
    }

    pub fn api_version(&self) -> Version {
        self.api_version
    }
}

// impl Drop for FabricAttr {
//     fn drop(&mut self) {
//         if ! self.c_attr.name.is_null() {
//             let _ = unsafe{CString::from_raw(self.c_attr.name)}; // Reclaim ptr from C backend and free
//         }
//         if ! self.c_attr.prov_name.is_null() {
//             let _ = unsafe{CString::from_raw(self.c_attr.prov_name)}; // Reclaim ptr from C backend and free
//         }
//     }
// }

/// Builder for the [`Fabric`] type.
///
/// `FabricBuilder` is used to configure and build a new `Fabric`.
/// It encapsulates an incremental configuration of the address vector, as provided by a `fi_fabric_attr`,
/// followed by a call to `fi_fabric`  
pub struct FabricBuilder<'a> {
    ctx: Option<&'a mut Context>,
}

impl<'a> FabricBuilder<'a> {
    /// Initiates the creation of a new [Fabric] based on the respective field of the `info` entry.
    ///
    /// The initial configuration is what is set in the `fi_info::fabric_attr` field and no `context` is provided.
    pub fn new() -> FabricBuilder<'a> {
        FabricBuilder { ctx: None }
    }
}

impl<'a> FabricBuilder<'a> {
    /// Sets the context to be passed to the `Fabric`.
    ///
    /// Corresponds to passing a non-NULL `context` value to `fi_fabric`.
    pub fn context(self, ctx: &'a mut Context) -> FabricBuilder<'a> {
        FabricBuilder { ctx: Some(ctx) }
    }

    /// Constructs a new [Fabric] with the configurations requested so far.
    ///
    /// Corresponds to retrieving the `fabric_attr` field of the provided `fi_info` entry (from [`new`](Self::new))
    /// and passing it along with an optional `context` to `fi_fabric`
    pub fn build<E>(self, info: &InfoEntry<E>) -> Result<Fabric, crate::error::Error> {
        Fabric::new(info.fabric_attr().clone(), self.ctx)
    }
}
