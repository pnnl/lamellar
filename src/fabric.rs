

use std::{ffi::CString, rc::Rc};

//================== Fabric (fi_fabric) ==================//
#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{utils::check_error, info::InfoEntry, fid::{OwnedFid, self, AsRawFid}};

// impl Drop for FabricImpl {
//     fn drop(&mut self) {
//        println!("Dropping FabricImpl\n");
//     }
// }

pub(crate) struct FabricImpl {
    pub(crate) c_fabric: *mut libfabric_sys::fid_fabric,
    fid: OwnedFid,
}

/// Owned wrapper around a libfabric `fid_fabric`.
/// 
/// This type wraps an instance of a `fid_fabric`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_fabric.3.html).
/// 
/// Note that other objects that rely on a `Fabric` (e.g., [`PassiveEndpoint`](crate::ep::PassiveEndpoint)) will extend its lifetime until they
/// are also dropped.
pub struct Fabric {
    pub(crate) inner: Rc<FabricImpl>,
}

impl FabricImpl {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_fabric {
        self.c_fabric
    }

    pub(crate) fn new<T0>(mut attr: FabricAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_fabric: *mut libfabric_sys::fid_fabric  = std::ptr::null_mut();
        let c_fabric_ptr: *mut *mut libfabric_sys::fid_fabric = &mut c_fabric;

        let err = 
            if let Some(ctx) = context {
                unsafe {libfabric_sys::fi_fabric(attr.get_mut(), c_fabric_ptr, (ctx as *mut T0).cast())}
            }
            else {
                unsafe {libfabric_sys::fi_fabric(attr.get_mut(), c_fabric_ptr, std::ptr::null_mut())}
            };
        
        if err != 0 || c_fabric.is_null() {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { 
                        c_fabric, 
                        fid: OwnedFid::from(unsafe{ &mut (*c_fabric).fid }), 
                })
        }
    }

    pub(crate) fn trywait(&self, fids: &[&impl AsFid]) -> Result<(), crate::error::Error> { // [TODO] Move this into the WaitSet struct
        let mut raw_fids: Vec<*mut libfabric_sys::fid> = fids.iter().map(|x| x.as_fid().as_raw_fid()).collect();
        let err = unsafe { libfabric_sys::inlined_fi_trywait(self.c_fabric, raw_fids.as_mut_ptr(), raw_fids.len() as i32) } ;
        
        check_error(err.try_into().unwrap())
    }
}

impl Fabric {
    
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_fabric {
        self.inner.handle()
    }

    pub(crate) fn new<T0>(attr: FabricAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(
            Self { inner: 
                Rc::new(FabricImpl::new(attr, context)?) 
            }
        )
    }
    
    pub fn trywait(&self, fids: &[&impl AsFid]) -> Result<(), crate::error::Error> { // [TODO] Move this into the WaitSet struct
        self.inner.trywait(fids)
    }
}

impl AsFid for FabricImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.fid.as_fid()
    }
}

impl AsFid for Fabric{
    fn as_fid(&self) -> fid::BorrowedFid {
        self.inner.as_fid()
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
        self.c_attr.name = unsafe{std::mem::transmute(self.f_name.as_ptr())};
        self
    }

    pub fn prov_name(&mut self, name: String) -> &mut Self { //[TODO] Possible memory leak
        let name = CString::new(name).unwrap();
        self.prov_name = name;
        self.c_attr.prov_name = unsafe{std::mem::transmute(self.prov_name.as_ptr())};
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
    fab_attr: FabricAttr,
    ctx: Option<&'a mut T>,
}

impl<'a> FabricBuilder<'a, ()> {
    
    /// Initiates the creation of a new [Fabric] based on the respective field of the `info` entry.
    /// 
    /// The initial configuration is what is set in the `fi_info::fabric_attr` field and no `context` is provided.
    pub fn new<E>(info: &InfoEntry<E>) -> FabricBuilder<()> {
        FabricBuilder::<()> {
            fab_attr: info.get_fabric_attr().clone(),
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
            fab_attr: self.fab_attr,
            ctx: Some(ctx),
        }
    }

    /// Constructs a new [Fabric] with the configurations requested so far.
    /// 
    /// Corresponds to retrieving the `fabric_attr` field of the provided `fi_info` entry (from [`new`](Self::new))
    /// and passing it along with an optional `context` to `fi_fabric`
    pub fn build(self) -> Result<Fabric, crate::error::Error> {
        Fabric::new(self.fab_attr, self.ctx)
    }    
}