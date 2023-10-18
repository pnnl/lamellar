pub struct Eq {
    c_eq: *mut libfabric_sys::fid_eq,
}

impl Eq {
    pub fn new(fabric: &crate::fabric::Fabric, attr: &EqAttr) -> Self {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let mut c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.c_eq_attr, c_eq_ptr, std::ptr::null_mut())};
        if err != 0 {
            panic!("fi_eq_open failed {}", err);
        }

        Self { c_eq }
    }
}

impl crate::FID for Eq {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_eq).fid }
    }
}

impl Eq {

}

pub struct EqAttr {
    c_eq_attr: *mut libfabric_sys::fi_eq_attr,
}

pub struct Wait {
    c_wait: *mut libfabric_sys::fid_wait,
}

impl Wait {
    pub(crate) fn new(fabric: &crate::fabric::Fabric, attr: &WaitAttr) -> Self {
        let mut c_wait: *mut libfabric_sys::fid_wait  = std::ptr::null_mut();
        let mut c_wait_ptr: *mut *mut libfabric_sys::fid_wait = &mut c_wait;

        let err = unsafe {libfabric_sys::inlined_fi_wait_open(fabric.c_fabric, attr.c_wait_attr, c_wait_ptr)};
        if err != 0 {
            panic!("fi_eq_open failed {}", err);
        }

        Self { c_wait }        
    }
}

pub struct WaitAttr {
    c_wait_attr: *mut libfabric_sys::fi_wait_attr,
}