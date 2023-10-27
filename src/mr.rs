
//================== Memory Region (fi_mr) ==================//
pub struct MemoryRegion {
    pub(crate) c_mr: *mut libfabric_sys::fid_mr,
}

impl MemoryRegion {
    pub(crate) fn from_buffer<T0>(domain: &crate::domain::Domain, buf: &[T0], acs: u64, offset: u64, requested_key: u64, flags: u64) -> Self {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_reg(domain.c_domain, buf.as_ptr() as *const std::ffi::c_void, buf.len(), acs, offset, requested_key, flags, c_mr_ptr, std::ptr::null_mut()) };
    
        if err != 0 {
            panic!("fi_mr_reg failed {}", err);
        }
    
        Self { c_mr }        
    }

    pub(crate) fn from_attr(domain: &crate::domain::Domain, attr: MemoryRegionAttr, flags: u64) -> Self {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_regattr(domain.c_domain, attr.get(), flags, c_mr_ptr) };
    
        if err != 0 {
            panic!("fi_mr_regattr failed {}", err);
        }
    
        Self { c_mr }           
    }
    
    pub(crate) fn from_iovec(domain: &crate::domain::Domain,  iov : &crate::IoVec, count: usize, acs: u64, offset: u64, requested_key: u64, flags: u64) -> Self {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_regv(domain.c_domain, iov.get(), count, acs, offset, requested_key, flags, c_mr_ptr, std::ptr::null_mut()) };
    
        if err != 0 {
            panic!("fi_mr_regv failed {}", err);
        }
    
        Self { c_mr }    
    }


    pub fn desc<T0>(&mut self) -> &mut T0 {
        let ret: *mut T0 = (unsafe { libfabric_sys::inlined_fi_mr_desc(self.c_mr) }) as *mut T0;
        unsafe { &mut *ret }
    }


    pub fn key(&mut self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_mr_key(self.c_mr) }
    }

    pub fn bind(&self, fid: &impl crate::FID, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_bind(self.c_mr, fid.fid(), flags) } ;
        
        if err != 0 {
            panic!("fi_mr_bind failed {}", err);
        }
    }

    pub fn refresh(&self, iov: &crate::IoVec, count: usize, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_refresh(self.c_mr, iov.get(), count, flags) };

        if err != 0 {
            panic!("fi_mr_refresh failed {}", err);
        }
    }

    pub fn enable(&self) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_enable(self.c_mr) };

        if err != 0 {
            panic!("fi_mr_enable failed {}", err);
        }
    }

    pub fn raw_attr(&self, base_addr: &mut u64, raw_key: &mut u8, key_size: &mut usize, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_raw_attr(self.c_mr, base_addr as *mut u64, raw_key as *mut u8, key_size as *mut usize, flags) };

        if err != 0 {
            panic!("fi_mr_raw_attr failed {}", err);
        }        
    }
}

//================== Memory Region attribute ==================//

pub struct MemoryRegionAttr {
    pub(crate) c_attr: libfabric_sys::fi_mr_attr,
}

impl MemoryRegionAttr {
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_mr_attr {
        &self.c_attr
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_mr_attr {
        &mut self.c_attr
    }
}