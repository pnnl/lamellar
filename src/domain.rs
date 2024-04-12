use std::{ffi::CString, rc::Rc};

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{enums::{DomainCaps, TClass}, fabric::FabricImpl, utils::check_error, info::InfoEntry, fid::{OwnedFid, self, AsRawFid}};

//================== Domain (fi_domain) ==================//

pub(crate) struct DomainImpl {
    pub(crate) c_domain: *mut libfabric_sys::fid_domain,
    fid: OwnedFid,
    _fabric_rc: Rc<FabricImpl>,
}

pub struct Domain {
    pub(crate) inner: Rc<DomainImpl>,
}

impl Domain {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_domain {
        self.inner.c_domain
    }

    pub(crate) fn new<T0, E>(fabric: &crate::fabric::Fabric, info: &InfoEntry<E>, flags: u64, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_domain: *mut libfabric_sys::fid_domain = std::ptr::null_mut();
        let c_domain_ptr: *mut *mut libfabric_sys::fid_domain = &mut c_domain;
        let err =
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_domain2(fabric.inner.c_fabric, info.c_info, c_domain_ptr, flags, (ctx as *mut T0).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_domain2(fabric.inner.c_fabric, info.c_info, c_domain_ptr, flags, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self { 
                    inner : Rc::new(
                        DomainImpl {
                            c_domain, 
                            _fabric_rc: fabric.inner.clone(), 
                            fid: OwnedFid::from(unsafe { &mut (*c_domain).fid } ), 
                    })
                })
        }
    }

    pub fn bind(self, fid: &impl AsFid, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_domain_bind(self.handle(), fid.as_fid().as_raw_fid(), flags)} ;

        check_error(err.try_into().unwrap())
    } 

    // pub fn srx_context<T0>(&self, rx_attr: crate::RxAttr) -> Result<crate::ep::Endpoint, crate::error::Error> { //[TODO]
    //     crate::ep::Endpoint::from_attr(self, rx_attr)
    // }
    
    // pub fn srx_context_with_context<T0>(&self, rx_attr: crate::RxAttr, context: &mut T0) -> Result<crate::ep::Endpoint, crate::error::Error> { //[TODO]
    //     crate::ep::Endpoint::from_attr_with_context(self, rx_attr, context)
    // }

    pub fn query_atomic(&self, datatype: crate::DataType, op: crate::enums::Op, mut attr: crate::comm::atomic::AtomicAttr, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_atomic(self.handle(), datatype, op.get_value(), attr.get_mut(), flags )};

        check_error(err.try_into().unwrap())
    }

    pub fn map_raw(&self, base_addr: u64, raw_key: &mut u8, key_size: usize, key: &mut u64, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.handle(), base_addr, raw_key as *mut u8, key_size, key as *mut u64, flags) };
        
        check_error(err.try_into().unwrap())
    }

    pub fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_unmap_key(self.handle(), key) };


        check_error(err.try_into().unwrap())
    }

    // pub fn stx_context<T0>(&self, attr: crate::TxAttr) -> Result<crate::Stx, crate::error::Error> { //[TODO]
    //     crate::Stx::new(self, attr, std::ptr::null_mut())
    // }

    // pub fn stx_context_with_context<T0>(&self, attr: crate::TxAttr , context: &mut T0) -> Result<crate::Stx, crate::error::Error> { //[TODO]
    //     crate::Stx::new(self, attr, context)
    // }

    pub fn query_collective<T: 'static>(&self, coll: crate::enums::CollectiveOp, mut attr: crate::comm::collective::CollectiveAttr<T>) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_collective(self.handle(), coll.get_value(), attr.get_mut(), 0) };
    
        check_error(err.try_into().unwrap())
    }

    pub fn query_collective_scatter<T: 'static>(&self, coll: crate::enums::CollectiveOp, mut attr: crate::comm::collective::CollectiveAttr<T>) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_collective(self.handle(), coll.get_value(), attr.get_mut(), libfabric_sys::fi_collective_op_FI_SCATTER.into()) };
    
        check_error(err.try_into().unwrap())
    }

}

impl AsFid for Domain {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.inner.fid.as_fid()
    }
}


//================== Domain attribute ==================//

#[derive(Clone, Debug)]
pub struct DomainAttr {
    pub(crate) c_attr : libfabric_sys::fi_domain_attr,
    f_name: std::ffi::CString,
}

impl DomainAttr {

    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_domain_attr {
            domain: std::ptr::null_mut(),
            name: std::ptr::null_mut(),
            threading: crate::enums::Threading::Unspec.get_value(),
            control_progress: crate::enums::Progress::Unspec.get_value(),
            data_progress: crate::enums::Progress::Unspec.get_value(),
            resource_mgmt: crate::enums::ResourceMgmt::Unspec.get_value(),
            av_type: crate::enums::AddressVectorType::Unspec.get_value(),
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
        Self { c_attr , f_name: CString::new("").unwrap()}
    }

    pub fn domain(&mut self, domain: &Domain) -> &mut Self {
        self.c_attr.domain = domain.handle();
        self
    }

    pub fn name(&mut self, name: String) -> &mut Self { //[TODO] Possible memory leak
        let name = std::ffi::CString::new(name).unwrap();
        self.f_name = name;
        self.c_attr.name = unsafe{std::mem::transmute(self.f_name.as_ptr())};
        self
    }

    pub fn threading(&mut self, threading: crate::enums::Threading) -> &mut Self {
        self.c_attr.threading = threading.get_value();
        self
    }

    pub fn control_progress(&mut self, control_progress: crate::enums::Progress) -> &mut Self {
        self.c_attr.control_progress = control_progress.get_value();
        self
    }

    pub fn data_progress(&mut self, data_progress: crate::enums::Progress) -> &mut Self {
        self.c_attr.data_progress = data_progress.get_value();
        self
    }

    pub fn resource_mgmt(&mut self, res_mgmt: crate::enums::ResourceMgmt) -> &mut Self {
        self.c_attr.resource_mgmt = res_mgmt.get_value();
        self
    }

    pub fn av_type(&mut self, av_type: crate::enums::AddressVectorType) -> &mut Self {
        self.c_attr.av_type = av_type.get_value();
        self
    }

    pub fn mr_mode(&mut self, mr_mode: crate::enums::MrMode) -> &mut Self {
        self.c_attr.mr_mode = mr_mode.get_value() as i32;
        self
    }

    pub fn mr_key_size(&mut self, size: usize) -> &mut Self{
        self.c_attr.mr_key_size = size;
        self
    }
    

    pub fn cq_data_size(&mut self, size: usize) -> &mut Self{
        self.c_attr.cq_data_size = size;
        self
    }
    

    pub fn cq_cnt(&mut self, size: usize) -> &mut Self{
        self.c_attr.cq_cnt = size;
        self
    }

    pub fn ep_cnt(&mut self, size: usize) -> &mut Self{
        self.c_attr.ep_cnt = size;
        self
    }

    pub fn tx_ctx_cnt(&mut self, size: usize) -> &mut Self{
        self.c_attr.tx_ctx_cnt = size;
        self
    }

    pub fn rx_ctx_cnt(&mut self, size: usize) -> &mut Self{
        self.c_attr.rx_ctx_cnt = size;
        self
    }

    pub fn max_ep_tx_ctx(&mut self, size: usize) -> &mut Self{
        self.c_attr.max_ep_tx_ctx = size;
        self
    }

    pub fn max_ep_rx_ctx(&mut self, size: usize) -> &mut Self{
        self.c_attr.max_ep_rx_ctx = size;
        self
    }

    pub fn max_ep_stx_ctx(&mut self, size: usize) -> &mut Self{
        self.c_attr.max_ep_stx_ctx = size;
        self
    }

    pub fn max_ep_srx_ctx(&mut self, size: usize) -> &mut Self{
        self.c_attr.max_ep_srx_ctx = size;
        self
    }

    pub fn cntr_cnt(&mut self, size: usize) -> &mut Self{
        self.c_attr.cntr_cnt = size;
        self
    }

    pub fn mr_iov_limit(&mut self, size: usize) -> &mut Self{
        self.c_attr.mr_iov_limit = size;
        self
    }

    pub fn caps(&mut self, caps: DomainCaps) -> &mut Self {
        self.c_attr.caps = caps.get_value();
        self
    }

    pub fn mode(&mut self, mode: crate::enums::Mode) -> &mut Self {
        self.c_attr.mode = mode.get_value();
        self
    }

    pub fn auth_key(&mut self, key: &mut [u8]) -> &mut Self {
        self.c_attr.auth_key_size = key.len();
        self.c_attr.auth_key = key.as_mut_ptr();
        self
    }

    pub fn max_err_data(&mut self, size: usize) -> &mut Self{
        self.c_attr.max_err_data = size;
        self
    }

    pub fn mr_cnt(&mut self, size: usize) -> &mut Self{
        self.c_attr.mr_cnt = size;
        self
    }

    pub fn tclass(&mut self, class: TClass) -> &mut Self {
        self.c_attr.tclass = class.get_value();
        self
    }
    
    pub fn get_mode(&self) -> u64 {
        self.c_attr.mode 
    }

    pub fn get_mr_mode(&self) -> crate::enums::MrMode {
        crate::enums::MrMode::from_value(self.c_attr.mr_mode.try_into().unwrap())
    }

    pub fn get_av_type(&self) ->  crate::enums::AddressVectorType {
        crate::enums::AddressVectorType::from_value( self.c_attr.av_type)
    }

    pub fn get_data_progress(&self) -> crate::enums::Progress {
        
        crate::enums::Progress::from_value(self.c_attr.data_progress)
    }

    pub fn get_mr_iov_limit(&self) -> usize {
        self.c_attr.mr_iov_limit
    }

    pub fn get_cntr_cnt(&self) -> usize {
        self.c_attr.cntr_cnt
    }

    pub fn get_cq_data_size(&self) -> u64 {
        self.c_attr.cq_data_size as u64
    }

    pub(crate) fn get(&self) -> *const libfabric_sys::fi_domain_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_domain_attr {
        &mut self.c_attr
    }
}

impl Default for DomainAttr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DomainBuilder<'a, T, E> {
    fabric: &'a crate::fabric::Fabric,
    info: &'a InfoEntry<E>,
    ctx: Option<&'a mut T>,
    flags: u64,
}

impl<'a> DomainBuilder<'a, (), ()> {
    pub fn new<E>(fabric: &'a crate::fabric::Fabric, info: &'a InfoEntry<E>) -> DomainBuilder<'a, (), E> {
        DomainBuilder::<(), E> {
            fabric,
            info,
            flags: 0,
            ctx: None,
        }
    }
}


impl<'a, T, E> DomainBuilder<'a, T, E> {
    pub fn context(self, ctx: &'a mut T) -> DomainBuilder<'a, T, E> {
        DomainBuilder {
            fabric: self.fabric,
            info: self.info,
            flags: 0,
            ctx: Some(ctx),
        }
    }

    pub fn flags(mut self, flags: u64) -> Self {
        self.flags = flags;
        self
    }

    pub fn build(self) -> Result<Domain, crate::error::Error> {
        Domain::new(self.fabric, self.info, self.flags, self.ctx)
    }
}


//================== Domain tests ==================//

#[cfg(test)]
mod tests {
    use crate::info::Info;

    #[test]
    fn domain_test() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let count = 10;
        let mut doms = Vec::new();
        for _ in 0..count {
            let domain = crate::domain::DomainBuilder::new(&fab, &entries[0]).build().unwrap();
            doms.push(domain);
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::info::Info;

    #[test]

    fn domain_drops_before_fabric() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let count = 10;
        let mut doms = Vec::new();
        for _ in 0..count {
            let domain = crate::domain::DomainBuilder::new(&fab, &entries[0]).build().unwrap();
            println!("Count = {}", std::rc::Rc::strong_count(&fab.inner));
            doms.push(domain);
        }
        drop(fab);
    }
}