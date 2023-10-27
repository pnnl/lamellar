
//================== Fabric (fi_fabric) ==================//

pub struct Fabric {
    pub(crate) c_fabric: *mut libfabric_sys::fid_fabric,
}


impl Fabric {
    pub fn new(mut attr: FabricAttr) -> Self {
        let mut c_fabric: *mut libfabric_sys::fid_fabric  = std::ptr::null_mut();
        let c_fabric_ptr: *mut *mut libfabric_sys::fid_fabric = &mut c_fabric;

        let err = unsafe {libfabric_sys::fi_fabric(attr.get_mut(), c_fabric_ptr, std::ptr::null_mut())};
        if err != 0 || c_fabric == std::ptr::null_mut() {
            panic!("fi_fabric failed {}", err);
        }

        Fabric { c_fabric }
    }

    pub fn domain(&self, info: &crate::InfoEntry) -> crate::domain::Domain {
        crate::domain::Domain::new(self, info)
    } 

    pub fn domain2(&self, info: &crate::InfoEntry, flags: u64) -> crate::domain::Domain {
        crate::domain::Domain::new2(self, info, flags)
    }

    pub fn passive_ep(&self, info: &crate::InfoEntry) -> crate::ep::PassiveEndPoint {
        crate::ep::PassiveEndPoint::new(self, info)
    }

    pub fn eq_open(&self, attr: crate::eq::EventQueueAttr) -> crate::eq::EventQueue {
        crate::eq::EventQueue::new(self, attr)
    }

    pub fn wait_open(&self, wait_attr: crate::sync::WaitAttr) -> crate::sync::Wait {
        crate::sync::Wait::new(self, wait_attr)
    }

    pub fn trywait(&self, fids: &Vec<&impl crate::FID>) {
        let mut raw_fids: Vec<*mut libfabric_sys::fid> = fids.iter().map(|x| x.fid()).collect();
        let err = unsafe { libfabric_sys::inlined_fi_trywait(self.c_fabric, raw_fids.as_mut_ptr(), raw_fids.len() as i32) } ;
        if err != 0 {
            panic!("fi_trywait failed {}", err);
        }
    }
}


impl crate::FID for Fabric {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_fabric).fid }
    }
}

//================== Fabric attribute ==================//

#[derive(Clone)]
pub struct FabricAttr {
    c_attr : libfabric_sys::fi_fabric_attr,
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

        Self { c_attr }
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_fabric_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_fabric_attr {
        &mut self.c_attr
    }    
}
