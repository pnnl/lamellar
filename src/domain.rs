use core::panic;

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

    pub fn query_atomic(&self, datatype: crate::DataType, op: crate::Op, mut attr: crate::AtomicAttr, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_query_atomic(self.c_domain, datatype, op, attr.get_mut(), flags )};

        if err != 0 {
            panic!("fi_query_atomic failed {}", err);
        }
    }
    pub fn cq_open(&self, attr: crate::eq::CqAttr) -> crate::eq::Cq {
        crate::eq::Cq::new(self, attr)
    }

    pub fn cntr_open(&self, attr: crate::CounterAttr) -> crate::Counter {
        crate::Counter::new(self, attr)
    }


    pub fn poll_open(&self, attr: crate::PollAttr) -> crate::Poll {
        crate::Poll::new(self, attr)
    }


    pub fn mr_reg<T0>(&self, buf: &[T0], acs: u64, offset: u64, requested_key: u64, flags: u64) -> crate::MemoryRegion {
        crate::MemoryRegion::from_buffer(self, buf, acs, offset, requested_key, flags)
    }

    pub fn mr_regv<T0>(&self,  iov : &crate::IoVec, count: usize, acs: u64, offset: u64, requested_key: u64, flags: u64) -> crate::MemoryRegion {
        crate::MemoryRegion::from_iovec(self, iov, count, acs, offset, requested_key, flags)
    }

    pub fn mr_regattr<T0>(&self, attr: crate::MemoryRegionAttr ,  flags: u64) -> crate::MemoryRegion {
        crate::MemoryRegion::from_attr(self, attr,  flags)
    }

    pub fn map_raw(&self, base_addr: u64, raw_key: &mut u8, key_size: usize, key: &mut u64, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.c_domain, base_addr, raw_key as *mut u8, key_size, key as *mut u64, flags) };
        
        if err != 0 {
            panic!("fi_mr_map_raw failed {}", err);
        }
    }

    pub fn unmap_key(&self, key: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_unmap_key(self.c_domain, key) };

        if err != 0 {
            panic!("fi_mr_unmap_key {}", err);
        }
    }

    pub fn stx_context<T0>(&self, attr:crate::TxAttr , context: &mut T0) -> crate::Stx {
        crate::Stx::new(&self, attr, context)
    }

}