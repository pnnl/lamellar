use core::panic;
#[allow(unused_imports)]
use crate::FID;

//================== Domain (fi_domain) ==================//

pub struct Domain {
    pub(crate) c_domain: *mut libfabric_sys::fid_domain,
}

impl Domain {

    pub fn new(fabric: &crate::fabric::Fabric, info: &crate::InfoEntry) -> Self {
        let mut c_domain: *mut libfabric_sys::fid_domain = std::ptr::null_mut();
        let c_domain_ptr: *mut *mut libfabric_sys::fid_domain = &mut c_domain;
        let err = unsafe { libfabric_sys::inlined_fi_domain(fabric.c_fabric, info.c_info, c_domain_ptr, std::ptr::null_mut()) };

        if err != 0 {
            panic!("fi_domain failed {}: {}", err, crate::error_to_string(err.into()));
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

    pub fn bind(self, fid: &impl crate::FID, flags: u64)  /*[TODO] Change to Result*/ {
        let err = unsafe{ libfabric_sys::inlined_fi_domain_bind(self.c_domain, fid.fid(), flags)} ;

        if err != 0 {
            panic!("fi_domain_bind failed {}", err);
        }
    } 

    pub fn srx_context<T0>(&self, rx_attr: crate::RxAttr) -> crate::ep::Endpoint {
        crate::ep::Endpoint::from_attr(self, rx_attr)
    }
    
    pub fn srx_context_with_context<T0>(&self, rx_attr: crate::RxAttr, context: &mut T0) -> crate::ep::Endpoint {
        crate::ep::Endpoint::from_attr_with_context(self, rx_attr, context)
    }

    pub fn query_atomic(&self, datatype: crate::DataType, op: crate::enums::Op, mut attr: crate::AtomicAttr, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_query_atomic(self.c_domain, datatype, op.get_value(), attr.get_mut(), flags )};

        if err != 0 {
            panic!("fi_query_atomic failed {}", err);
        }
    }
    pub fn cq_open(&self, attr: crate::eq::CommandQueueAttr) -> crate::eq::CommandQueue {
        crate::eq::CommandQueue::new(self, attr)
    }

    pub fn cntr_open(&self, attr: crate::CounterAttr) -> crate::Counter {
        crate::Counter::new(self, attr)
    }


    pub fn poll_open(&self, attr: crate::sync::PollAttr) -> crate::sync::Poll {
        crate::sync::Poll::new(self, attr)
    }

    pub fn av_open(&self, attr: crate::av::AddressVectorAttr) -> crate::av::AddressVector {
        crate::av::AddressVector::new(self, attr)
    }

    pub fn mr_reg<T0>(&self, buf: &[T0], acs: u64, offset: u64, requested_key: u64, flags: u64) -> crate::mr::MemoryRegion {
        crate::mr::MemoryRegion::from_buffer(self, buf, acs, offset, requested_key, flags)
    }

    pub fn mr_regv<T0>(&self,  iov : &crate::IoVec, count: usize, acs: u64, offset: u64, requested_key: u64, flags: u64) -> crate::mr::MemoryRegion {
        crate::mr::MemoryRegion::from_iovec(self, iov, count, acs, offset, requested_key, flags)
    }

    pub fn mr_regattr<T0>(&self, attr: crate::mr::MemoryRegionAttr ,  flags: u64) -> crate::mr::MemoryRegion {
        crate::mr::MemoryRegion::from_attr(self, attr,  flags)
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

    pub fn query_collective(&self, coll: crate::enums::CollectiveOp, mut attr: crate::CollectiveAttr, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_query_collective(self.c_domain, coll.get_value(), attr.get_mut(), flags) };
    
        if err != 0 {
            panic!("query_collective failed {}", err);
        }
    }

}

impl crate::FID for Domain {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe {&mut (*self.c_domain).fid }
    }
}

//================== Domain attribute ==================//

#[derive(Clone)]
pub struct DomainAttr {
    pub(crate) c_attr : libfabric_sys::fi_domain_attr,
}

impl DomainAttr {

    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_domain_attr {
            domain: std::ptr::null_mut(),
            name: std::ptr::null_mut(),
            threading: crate::enums::Threading::UNSPEC.get_value(),
            control_progress: crate::enums::Progress::UNSPEC.get_value(),
            data_progress: crate::enums::Progress::UNSPEC.get_value(),
            resource_mgmt: crate::enums::ResourceMgmt::UNSPEC.get_value(),
            av_type: crate::enums::AddressVectorType::UNSPEC.get_value(),
            mr_mode: 0,
            mr_key_size: 0,
            cq_data_size: 0,
            cq_cnt: 0,
            ep_cnt: 0,
            tx_ctx_cnt: 0,
            rx_ctx_cnt: 0,
            max_ep_tx_ctx: 0,
            max_ep_rx_ctx: 0,
            max_ep_stx_ctx: 0,
            max_ep_srx_ctx: 0,
            cntr_cnt: 0,
            mr_iov_limit: 0,
            caps: 0,
            mode: 0,
            auth_key: std::ptr::null_mut(),
            auth_key_size: 0,
            max_err_data: 0,
            mr_cnt: 0,
            tclass: 0,         
        };
        Self { c_attr }
    }

    pub fn mode(&mut self, mode: u64) -> &mut Self {
        self.c_attr.mode = mode;
        
        self
    }

    pub fn get_av_type(&self) ->  crate::enums::AddressVectorType {
        crate::enums::AddressVectorType::from_value( self.c_attr.av_type)
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_domain_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_domain_attr {
        &mut self.c_attr
    }
}

//================== Domain tests ==================//

#[test]
fn domain_test() {
    let info = crate::Info::all();
    let entries = info.get();
    
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(crate::eq::EventQueueAttr::new());
    let count = 10;
    let mut doms = Vec::new();
    for _ in 0..count {
        let domain = fab.domain(&entries[0]);
        doms.push(domain);
    }

    for mut dom in doms {
        dom.close();
    }

    eq.close();
    fab.close();
}