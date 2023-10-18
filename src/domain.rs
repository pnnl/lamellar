
pub struct Domain {
    pub(crate) c_domain: *mut libfabric_sys::fid_domain,
}

impl Domain {
    // pub fn inlined_fi_domain(
    //     fabric: *mut fid_fabric,
    //     info: *mut fi_info,
    //     domain: *mut *mut fid_domain,
    //     context: *mut ::std::os::raw::c_void,
    // ) -> ::std::os::raw::c_int;
    pub fn new(fabric: &crate::fabric::Fabric, info: &crate::InfoEntry) -> Self {
        let mut c_domain: *mut libfabric_sys::fid_domain = std::ptr::null_mut();
        let c_domain_ptr: *mut *mut libfabric_sys::fid_domain = &mut c_domain;
        let err = unsafe { libfabric_sys::inlined_fi_domain(fabric.c_fabric, info.c_info, c_domain_ptr, std::ptr::null_mut()) };

        if err != 0 {
            panic!("fi_domain failed {}", err);
        }

        Self { c_domain }
    }

    pub fn new2(fabric: &crate::fabric::Fabric, info: &crate::InfoEntry, flags: u64) -> Self {
        let mut c_domain: *mut libfabric_sys::fid_domain = std::ptr::null_mut();
        let c_domain_ptr: *mut *mut libfabric_sys::fid_domain = &mut c_domain;
        let err = unsafe { libfabric_sys::inlined_fi_domain2(fabric.c_fabric, info.c_info, c_domain_ptr, flags, std::ptr::null_mut()) };

        if err != 0 {
            panic!("fi_domain failed {}", err);
        }

        Self { c_domain }
    }
    // pub fn inlined_fi_domain_bind(
    //     domain: *mut fid_domain,
    //     fid: *mut fid,
    //     flags: u64,
    // ) -> ::std::os::raw::c_int;
    pub fn bind(self, fid: &impl crate::FID, flags: u64)  /*[TODO] Change to Result*/ {
        let err = unsafe{ libfabric_sys::inlined_fi_domain_bind(self.c_domain, fid.fid(), flags)} ;

        if err != 0 {
            panic!("fi_domain_bind failed {}", err);
        }
    } 

        // static inline int
    // fi_stx_context(struct fid_domain *domain, struct fi_tx_attr *attr,
    //            struct fid_stx **stx, void *context) [TODO]
    // pub fn stx_context(&self, idx: i32, stxattr: &crate::StxAttr) -> Self {
    //     let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
    //     let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

    //     let err = unsafe {libfabric_sys::inlined_fi_stx_context(self.c_ep, idx, stxattr.c_stx_attr, c_ep_ptr, std::ptr::null_mut())};
        
    //     if err != 0 {
    //         panic!("fi_tx_context_failed {}", err);
    //     }

    //     Self { c_ep }
    // }

    //     static inline int
    // fi_srx_context(struct fid_domain *domain, struct fi_rx_attr *attr,
    // 	       struct fid_ep **rx_ep, void *context)  [TODO]
    // pub fn srx_context(&self, idx: i32, srxattr: &crate::SrxAttr) -> Self {
    //     let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
    //     let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

    //     let err = unsafe {libfabric_sys::inlined_fi_srx_context(self.c_ep, idx, srxattr.c_srx_attr, c_ep_ptr, std::ptr::null_mut())};
        
    //     if err != 0 {
    //         panic!("fi_tx_context_failed {}", err);
    //     }

    //     Self { c_ep }
    // }
}