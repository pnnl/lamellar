

use std::{ffi::CString, rc::Rc};

//================== Fabric (fi_fabric) ==================//
#[allow(unused_imports)]
use crate::AsFid;
use crate::{OwnedFid, check_error};

// impl Drop for FabricImpl {
//     fn drop(&mut self) {
//        println!("Dropping FabricImpl\n");
//     }
// }

pub(crate) struct FabricImpl {
    pub(crate) c_fabric: *mut libfabric_sys::fid_fabric,
    fid: OwnedFid,
}

pub struct Fabric {
    pub(crate) inner: Rc<FabricImpl>,
}


impl Fabric {
    pub(crate) fn new(mut attr: FabricAttr) -> Result<Fabric, crate::error::Error> {
        let mut c_fabric: *mut libfabric_sys::fid_fabric  = std::ptr::null_mut();
        let c_fabric_ptr: *mut *mut libfabric_sys::fid_fabric = &mut c_fabric;

        let err = unsafe {libfabric_sys::fi_fabric(attr.get_mut(), c_fabric_ptr, std::ptr::null_mut())};
        
        if err != 0 || c_fabric.is_null() {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { 
                    inner: Rc::new( FabricImpl {
                        c_fabric, 
                        fid: crate::OwnedFid { fid: unsafe{ &mut (*c_fabric).fid } } 
                    })
                })
        }
    }

    pub(crate) fn new_with_context<T0>(mut attr: FabricAttr, ctx: &mut T0) -> Result<Fabric, crate::error::Error> {
        let mut c_fabric: *mut libfabric_sys::fid_fabric  = std::ptr::null_mut();
        let c_fabric_ptr: *mut *mut libfabric_sys::fid_fabric = &mut c_fabric;

        let err = unsafe {libfabric_sys::fi_fabric(attr.get_mut(), c_fabric_ptr, ctx as *mut T0 as *mut std::ffi::c_void)};
        
        if err != 0 || c_fabric.is_null() {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { 
                    inner: Rc::new( FabricImpl {
                        c_fabric, 
                        fid: crate::OwnedFid { fid: unsafe{ &mut (*c_fabric).fid } } 
                    })
                })
        }
    }

    // #[allow(dead_code)]
    // pub(crate) fn from(c_fabric: *mut libfabric_sys::fid_fabric) -> Self {
        
    //     Self { 
    //         inner: Rc::new (FabricImpl {
    //             c_fabric,
    //             fid: crate::OwnedFid { fid: unsafe{&mut (*c_fabric).fid} },
    //         })
    //     }
    // }

    // pub fn domain(&self, info: &crate::InfoEntry) -> Result<crate::domain::Domain, crate::error::Error> {
    //     crate::domain::Domain::new(self, info)
    // } 

    // pub fn domain2(&self, info: &crate::InfoEntry, flags: u64) -> Result<crate::domain::Domain, crate::error::Error> {
    //     crate::domain::Domain::new2(self, info, flags)
    // }

    pub fn trywait(&self, fids: &[&impl crate::AsFid]) -> Result<(), crate::error::Error> { // [TODO] Move this into the WaitSet struct
        let mut raw_fids: Vec<*mut libfabric_sys::fid> = fids.iter().map(|x| x.as_fid()).collect();
        let err = unsafe { libfabric_sys::inlined_fi_trywait(self.inner.c_fabric, raw_fids.as_mut_ptr(), raw_fids.len() as i32) } ;
        
        check_error(err.try_into().unwrap())
    }
}


impl crate::AsFid for Fabric {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.inner.fid.as_fid()
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

pub struct FabricBuilder<'a, T> {
    fab_attr: FabricAttr,
    ctx: Option<&'a mut T>,
}

impl<'a> FabricBuilder<'a, ()> {
    pub fn new<E>(info: &crate::InfoEntry<E>) -> FabricBuilder<()> {
        FabricBuilder::<()> {
            fab_attr: info.get_fabric_attr().clone(),
            ctx: None,
        }
    }

    pub fn from_attr(fab_attr: FabricAttr) -> FabricBuilder<'a, ()> {
        FabricBuilder::<()> {
            fab_attr,
            ctx: None,
        }
    }
}

impl<'a, T> FabricBuilder<'a, T> {
    // pub fn fabric(mut self, fabric: &Fabric) -> Self {
    //     self.fab_attr.fabric(fabric);
    //     self
    // }

    pub fn name(mut self, name: String) -> Self {
        self.fab_attr.name(name);
        self
    }

    pub fn prov_name(mut self, name: String) -> Self {
        self.fab_attr.prov_name(name);
        self
    }

    pub fn prov_version(mut self, version: u32) -> Self {
        self.fab_attr.prov_version(version);
        self
    }

    pub fn api_version(mut self, version: u32) -> Self {
        self.fab_attr.api_version(version);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> FabricBuilder<'a, T> {
        FabricBuilder {
            fab_attr: self.fab_attr,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<Fabric, crate::error::Error> {
        if let Some(ctx) = self.ctx {
            Fabric::new_with_context(self.fab_attr, ctx)
        }
        else {
            Fabric::new(self.fab_attr)
        }
    }    
}