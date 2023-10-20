
pub struct Fabric {
    pub(crate) c_fabric: *mut libfabric_sys::fid_fabric,
}


impl Fabric {
    pub fn new(mut attr: crate::FabricAttr) -> Self {
        let mut c_fabric: *mut libfabric_sys::fid_fabric  = std::ptr::null_mut();
        let c_fabric_ptr: *mut *mut libfabric_sys::fid_fabric = &mut c_fabric;

        let err = unsafe {libfabric_sys::fi_fabric(attr.get_mut(), c_fabric_ptr, std::ptr::null_mut())};
        if err != 0 {
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

    pub fn eq_open(&self, attr: crate::eq::EqAttr) -> crate::eq::Eq {
        crate::eq::Eq::new(self, attr)
    }

    pub fn wait_open(&self, wait_attr: crate::eq::WaitAttr) -> crate::eq::Wait {
        crate::eq::Wait::new(self, wait_attr)
    }

    pub fn trywait(&self, fids: &Vec<&impl crate::FID>) {
        let mut raw_fids: Vec<*mut libfabric_sys::fid> = fids.iter().map(|x| x.fid()).collect();
        let err = unsafe { libfabric_sys::inlined_fi_trywait(self.c_fabric, raw_fids.as_mut_ptr(), raw_fids.len() as i32) } ;
        if err != 0 {
            panic!("fi_trywait failed {}", err);
        }
    }
}