

use std::ffi::CString;

//================== Fabric (fi_fabric) ==================//
#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{fid::{AsRawFid, AsRawTypedFid, AsTypedFid, BorrowedFid, BorrowedTypedFid, FabricRawFid, OwnedFabricFid, RawFid}, info::InfoEntry, utils::check_error, MyRc};

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

    pub(crate) fn new<T0>(mut attr: FabricAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_fabric: FabricRawFid  = std::ptr::null_mut();

        let err = 
            if let Some(ctx) = context {
                unsafe {libfabric_sys::fi_fabric(attr.get_mut(), &mut c_fabric, (ctx as *mut T0).cast())}
            }
            else {
                unsafe {libfabric_sys::fi_fabric(attr.get_mut(), &mut c_fabric, std::ptr::null_mut())}
            };
        
        if err != 0 || c_fabric.is_null() {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { 
                    c_fabric: OwnedFabricFid::from(c_fabric), 
                })
        }
    }

    pub(crate) fn trywait_slice(&self, fids: &[&impl AsFid]) -> Result<(), crate::error::Error> { // [TODO] Move this into the WaitSet struct
        let mut raw_fids: Vec<*mut libfabric_sys::fid> = fids.iter().map(|x| x.as_fid().as_raw_fid()).collect();
        let err = unsafe { libfabric_sys::inlined_fi_trywait(self.as_raw_typed_fid(), raw_fids.as_mut_ptr(), raw_fids.len() as i32) } ;
        
        check_error(err.try_into().unwrap())
    }

    pub(crate) fn trywait(&self, fid: &impl AsFid) -> Result<(), crate::error::Error> { // [TODO] Move this into the WaitSet struct
        let mut raw_fid = fid.as_fid().as_raw_fid();
        let err = unsafe { libfabric_sys::inlined_fi_trywait(self.as_raw_typed_fid(), &mut raw_fid, 1) } ;
        
        check_error(err.try_into().unwrap())
    }
}

impl Fabric {
    
    pub(crate) fn new<T0>(attr: FabricAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(
            Self { inner: 
                MyRc::new(FabricImpl::new(attr, context)?)
            }
        )
    }
    
    pub fn trywait_slice(&self, fids: &[&impl AsFid]) -> Result<(), crate::error::Error> { // [TODO] Move this into the WaitSet struct
        self.inner.trywait_slice(fids)
    }
    
    pub fn trywait(&self, fid: &impl AsFid) -> Result<(), crate::error::Error> { // [TODO] Move this into the WaitSet struct
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

impl AsFid for Fabric{
    fn as_fid(&self) -> BorrowedFid {
        self.inner.as_fid()
    }
}

impl AsRawFid for Fabric{
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
    c_attr : libfabric_sys::fi_fabric_attr,
    f_name : CString,
    prov_name : CString,
}

impl FabricAttr {

    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_fabric_attr {
            fabric: std::ptr::null_mut(),
            name: std::ptr::null_mut(),
            prov_name: std::ptr::null_mut(),
            prov_version: 0,
            api_version: 0,
        };

        Self { c_attr, f_name: CString::new("").unwrap(), prov_name: CString::new("").unwrap() }
    }

    // pub fn fabric(&mut self, fabric: &Fabric) -> &mut Self {
    //     self.c_attr.fabric = fabric.c_fabric;
    //     self
    // }

    pub fn name(&mut self, name: String) -> &mut Self { //[TODO] Possible memory leak
        let name = CString::new(name).unwrap();
        self.f_name = name;
        self.c_attr.name = unsafe{std::mem::transmute::<*const i8, *mut i8>(self.f_name.as_ptr())};
        self
    }

    pub fn prov_name(&mut self, name: String) -> &mut Self { //[TODO] Possible memory leak
        let name = CString::new(name).unwrap();
        self.prov_name = name;
        self.c_attr.prov_name = unsafe{std::mem::transmute::<*const i8, *mut i8>(self.prov_name.as_ptr())};
        self
    }

    pub fn prov_version(&mut self, version: u32) -> &mut Self {
        self.c_attr.prov_version = version;
        self
    }

    pub fn api_version(&mut self, version: u32) -> &mut Self {
        self.c_attr.api_version = version;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_fabric_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_fabric_attr {
        &mut self.c_attr
    }

    pub fn get_prov_name(&self) -> String {
        unsafe{ std::ffi::CStr::from_ptr(self.c_attr.prov_name).to_str().unwrap().to_string() }
    }    

    pub fn get_name(&self) -> String {
        unsafe{ std::ffi::CStr::from_ptr(self.c_attr.name).to_str().unwrap().to_string() }
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

impl Default for FabricAttr {
    fn default() -> Self {
        Self::new()
    }
}


/// Builder for the [`Fabric`] type.
/// 
/// `FabricBuilder` is used to configure and build a new `Fabric`.
/// It encapsulates an incremental configuration of the address vector, as provided by a `fi_fabric_attr`,
/// followed by a call to `fi_fabric`  
pub struct FabricBuilder<'a, T> {
    ctx: Option<&'a mut T>,
}

impl<'a> FabricBuilder<'a, ()> {
    
    /// Initiates the creation of a new [Fabric] based on the respective field of the `info` entry.
    /// 
    /// The initial configuration is what is set in the `fi_info::fabric_attr` field and no `context` is provided.
    pub fn new() -> FabricBuilder<'a, ()> {
        FabricBuilder::<()> {
            ctx: None,
        }
    }
}

impl<'a, T> FabricBuilder<'a, T> {

    /// Sets the context to be passed to the `Fabric`.
    /// 
    /// Corresponds to passing a non-NULL `context` value to `fi_fabric`.
    pub fn context(self, ctx: &'a mut T) -> FabricBuilder<'a, T> {
        FabricBuilder {
            ctx: Some(ctx),
        }
    }

    /// Constructs a new [Fabric] with the configurations requested so far.
    /// 
    /// Corresponds to retrieving the `fabric_attr` field of the provided `fi_info` entry (from [`new`](Self::new))
    /// and passing it along with an optional `context` to `fi_fabric`
    pub fn build<E>(self, info: &InfoEntry<E>) -> Result<Fabric, crate::error::Error> {
        Fabric::new(info.get_fabric_attr().clone(), self.ctx)
    }    
}