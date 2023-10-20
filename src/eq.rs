use core::panic;

pub struct Eq {
    c_eq: *mut libfabric_sys::fid_eq,
}

impl Eq {
    pub fn new(fabric: &crate::fabric::Fabric, mut attr: EqAttr) -> Self {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, std::ptr::null_mut())};
        if err != 0 {
            panic!("fi_eq_open failed {}", err);
        }

        Self { c_eq }
    }

    pub fn read<T0>(&self, event: &mut u32, buf: &mut [T0], flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_eq_read(self.c_eq, event as *mut u32, buf.as_mut_ptr() as *mut std::ffi::c_void, buf.len(), flags) };

        if err != 0 {
            panic!("fi_eq_read failed {}", err);
        }
    }

    pub fn write<T0>(&self, event: u32, buf: & [T0], flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_eq_write(self.c_eq, event, buf.as_ptr() as *const std::ffi::c_void, buf.len(), flags) };

        if err != 0 {
            panic!("fi_eq_write failed {}", err);
        }
    }


    pub fn sread<T0>(&self, event: &mut u32, buf: &mut [T0], timeout: i32, flags: u64) -> isize { // [TODO] Check return
        unsafe { libfabric_sys::inlined_fi_eq_sread(self.c_eq, event as *mut u32, buf.as_mut_ptr() as *mut std::ffi::c_void, buf.len(), timeout, flags) }

        // if err != 0 {
        //     panic!("fi_eq_sread failed {}", err);
        // }
    }
}

impl crate::FID for Eq {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_eq).fid }
    }
}

pub struct Cq {
    pub(crate) c_cq: *mut libfabric_sys::fid_cq,
}

pub struct CqAttr {
    pub(crate) c_attr: libfabric_sys::fi_cq_attr,
}

impl CqAttr {
    
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_cq_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_cq_attr {
        &mut self.c_attr
    }
}

impl Cq {
    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: CqAttr) -> Self {
        let mut c_cq: *mut libfabric_sys::fid_cq  = std::ptr::null_mut();
        let c_cq_ptr: *mut *mut libfabric_sys::fid_cq = &mut c_cq;

        let err = unsafe {libfabric_sys::inlined_fi_cq_open(domain.c_domain, attr.get_mut(), c_cq_ptr, std::ptr::null_mut())};
        if err != 0 {
            panic!("fi_cq_open failed {}", err);
        }

        Self { c_cq } 
    }

    pub fn read<T0>(&self, buf: &mut [T0], count: usize) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_read(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count) }
    }

    pub fn readfrom<T0>(&self, buf: &mut [T0], count: usize, address: &mut crate::Address) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_readfrom(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, address as *mut crate::Address) }
    }

    pub fn sread<T0, T1>(&self, buf: &mut [T0], count: usize, cond: &T1, timeout: i32) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_sread(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, cond as *const T1 as *const std::ffi::c_void, timeout) }
    }

    pub fn sreadfrom<T0, T1>(&self, buf: &mut [T0], count: usize, address: &mut crate::Address, cond: &T1, timeout: i32) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_sreadfrom(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, address as *mut crate::Address, cond as *const T1 as *const std::ffi::c_void, timeout) }
    }

    pub fn signal(&self) {
        let err = unsafe { libfabric_sys::inlined_fi_cq_signal(self.c_cq) };

        if err != 0 {
            panic!("fi_cq_signal failed {}", err);
        }
    }
}


pub struct EqAttr {
    c_attr: libfabric_sys::fi_eq_attr,
}

impl EqAttr {

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_eq_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_eq_attr {
        &mut self.c_attr
    }    
}

pub struct Wait {
    pub(crate) c_wait: *mut libfabric_sys::fid_wait,
}

impl Wait {
    pub(crate) fn new(fabric: &crate::fabric::Fabric, mut attr: WaitAttr) -> Self {
        let mut c_wait: *mut libfabric_sys::fid_wait  = std::ptr::null_mut();
        let c_wait_ptr: *mut *mut libfabric_sys::fid_wait = &mut c_wait;

        let err = unsafe {libfabric_sys::inlined_fi_wait_open(fabric.c_fabric, attr.get_mut(), c_wait_ptr)};
        if err != 0 {
            panic!("fi_eq_open failed {}", err);
        }

        Self { c_wait }        
    }

    pub fn wait(&self, timeout: i32) -> i32 { // [TODO] Probably returns error when timeout occurs, 0 if done, or error
        unsafe { libfabric_sys::inlined_fi_wait(self.c_wait, timeout) } 
    }
}

pub struct WaitAttr {
    pub(crate) c_attr: libfabric_sys::fi_wait_attr,
}


impl WaitAttr {
    
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_wait_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_wait_attr {
        &mut self.c_attr
    }   
}