use std::{ffi::CString, rc::Rc, cell::OnceCell};

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{enums::{DomainCaps, TClass}, fabric::FabricImpl, utils::{check_error, to_fi_datatype}, info::InfoEntry, fid::{OwnedFid, self, AsRawFid}, eq::{EventQueue, EventQueueImpl}, eqoptions::EqConfig};

//================== Domain (fi_domain) ==================//

pub(crate) struct DomainImpl {
    pub(crate) c_domain: *mut libfabric_sys::fid_domain,
    fid: OwnedFid,
    pub(crate) domain_attr: DomainAttr,
    _eq_rc: OnceCell<Rc<EventQueueImpl>>,
    _fabric_rc: Rc<FabricImpl>,
}

impl DomainImpl {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_domain {
        self.c_domain
    }

    pub(crate) fn new<T0, E>(fabric: &Rc<crate::fabric::FabricImpl>, info: &InfoEntry<E>, flags: u64, domain_attr: DomainAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_domain: *mut libfabric_sys::fid_domain = std::ptr::null_mut();
        let c_domain_ptr: *mut *mut libfabric_sys::fid_domain = &mut c_domain;
        let err =
            if let Some(ctx) = context {
                if flags == 0 {
                    unsafe { libfabric_sys::inlined_fi_domain(fabric.c_fabric, info.c_info, c_domain_ptr, (ctx as *mut T0).cast()) }
                    
                }
                else {
                    unsafe { libfabric_sys::inlined_fi_domain2(fabric.c_fabric, info.c_info, c_domain_ptr, flags, (ctx as *mut T0).cast()) }
                }
            }
            else {
                if flags == 0 {
                    unsafe { libfabric_sys::inlined_fi_domain(fabric.c_fabric, info.c_info, c_domain_ptr, std::ptr::null_mut()) }
                }
                else {
                    unsafe { libfabric_sys::inlined_fi_domain2(fabric.c_fabric, info.c_info, c_domain_ptr, flags, std::ptr::null_mut()) }
                }
            };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self { 
                    c_domain, 
                    domain_attr,
                    _fabric_rc: fabric.clone(), 
                    _eq_rc: OnceCell::new(),
                    fid: OwnedFid::from(unsafe { &mut (*c_domain).fid } ), 
                })
        }
    }
    
    pub(crate) fn bind(&self, eq: &Rc<EventQueueImpl>, async_mem_reg: bool) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_domain_bind(self.handle(), eq.as_fid().as_raw_fid(), if async_mem_reg {libfabric_sys::FI_REG_MR} else {0})} ;

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            if self._eq_rc.set(eq.clone()).is_err() {
                panic!("Domain is alread bound to an EventQueue");
            }
            Ok(())
        }
    } 

    // pub(crate) fn srx_context<T0>(&self, rx_attr: crate::RxAttr) -> Result<crate::ep::Endpoint, crate::error::Error> { //[TODO]
    //     crate::ep::Endpoint::from_attr(self, rx_attr)
    // }
    
    // pub(crate) fn srx_context_with_context<T0>(&self, rx_attr: crate::RxAttr, context: &mut T0) -> Result<crate::ep::Endpoint, crate::error::Error> { //[TODO]
    //     crate::ep::Endpoint::from_attr_with_context(self, rx_attr, context)
    // }

    pub(crate) fn query_atomic<T: 'static>(&self, op: crate::enums::Op, mut attr: crate::comm::atomic::AtomicAttr, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_atomic(self.handle(), to_fi_datatype::<T>(), op.get_value(), attr.get_mut(), flags )};

        check_error(err.try_into().unwrap())
    }

    pub(crate) fn map_raw(&self, mr_key: &mut crate::mr::MemoryRegionKey, flags: u64) -> Result<u64, crate::error::Error> {
        let mut mapped_key = 0;
        let err = match mr_key {
            crate::mr::MemoryRegionKey::Key(simple_key) => {
                return Ok(*simple_key)
                // unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.handle(), base_addr, simple_key as *mut u64 as *mut u8, std::mem::size_of::<u64>(), &mut mapped_key, flags) }
            }
            crate::mr::MemoryRegionKey::RawKey(raw_key) => {
                unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.handle(), raw_key.1 , raw_key.0.as_mut_ptr().cast(), raw_key.0.len(), &mut mapped_key, flags) }
            }
        };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(mapped_key)
        }
    }

    // pub fn map_raw(&self, base_addr: u64, raw_key: &mut u8, key_size: usize, key: &mut u64, flags: u64) -> Result<(), crate::error::Error> {
    //     let err = unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.handle(), base_addr, raw_key, key_size, key, flags) };
        
    //     check_error(err.try_into().unwrap())
    // }

    pub(crate) fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_unmap_key(self.handle(), key) };


        check_error(err.try_into().unwrap())
    }

    // pub fn stx_context<T0>(&self, attr: crate::TxAttr) -> Result<crate::Stx, crate::error::Error> { //[TODO]
    //     crate::Stx::new(self, attr, std::ptr::null_mut())
    // }

    // pub fn stx_context_with_context<T0>(&self, attr: crate::TxAttr , context: &mut T0) -> Result<crate::Stx, crate::error::Error> { //[TODO]
    //     crate::Stx::new(self, attr, context)
    // }

    pub(crate) fn query_collective<T: 'static>(&self, coll: crate::enums::CollectiveOp, attr: &mut crate::comm::collective::CollectiveAttr<T>) -> Result<bool, crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_collective(self.handle(), coll.get_value(), attr.get_mut(), 0) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(true)
        }
    }

    pub(crate) fn query_collective_scatter<T: 'static>(&self, coll: crate::enums::CollectiveOp, attr: &mut crate::comm::collective::CollectiveAttr<T>) -> Result<bool, crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_collective(self.handle(), coll.get_value(), attr.get_mut(), libfabric_sys::fi_collective_op_FI_SCATTER.into()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(true)
        }
    }

}

/// Owned wrapper around a libfabric `fid_domain`.
/// 
/// This type wraps an instance of a `fid_domain`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_domain.3.html).
/// 
/// Note that other objects that rely on a Domain (e.g., [`Endpoint`](crate::ep::Endpoint)) will extend its lifetime until they
/// are also dropped.
pub struct Domain {
    pub(crate) inner: Rc<DomainImpl>,
}

impl Domain {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_domain {
        self.inner.handle()
    }
    
    pub(crate) fn new<T0, E>(fabric: &crate::fabric::Fabric, info: &InfoEntry<E>, flags: u64, domain_attr: DomainAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(
            Self{
                inner: 
                    Rc::new(DomainImpl::new(&fabric.inner, info, flags, domain_attr, context)?)
            }
        )    
    }
    
    /// Associates an [crate::eq::EventQueue] with the domain.
    /// 
    /// If `async_mem_reg` is true, the provider should perform all memory registration operations asynchronously, with the completion reported through the event queue
    /// 
    /// Corresponds to `fi_domain_bind`, with flag `FI_REG_MR` if `async_mem_reg` is true. 
    pub fn bind_eq<T: EqConfig>(&self, eq: &EventQueue<T>, async_mem_reg: bool) -> Result<(), crate::error::Error> {
        self.inner.bind(&eq.inner, async_mem_reg)
    }

    /// Indicates if a provider supports a specific atomic operation
    /// 
    /// Returns true if the provider supports operations `op` for datatype `T` and atomic ops as reflected in `flags`.
    /// 
    /// Corresponds to `fi_query_atomic` with `datatype` automatically inferred from `T`.
    pub fn query_atomic<T: 'static>(&self, op: crate::enums::Op, attr: crate::comm::atomic::AtomicAttr, flags: u64) -> Result<(), crate::error::Error> { //[TODO] Flags
        
        self.inner.query_atomic::<T>(op, attr, flags)
    }

    pub(crate) fn map_raw(&self, mr_key: &mut crate::mr::MemoryRegionKey, flags: u64) -> Result<u64, crate::error::Error> {
        self.inner.map_raw(mr_key, flags)
    }

    pub(crate) fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        self.inner.unmap_key(key)
    }

    /// Returns information about which collective operations are supported by a provider, and limitations on the collective.
    /// 
    /// Direclty corresponds to `fi_query_collective` with `flags` = 0
    pub fn query_collective<T: 'static>(&self, coll: crate::enums::CollectiveOp, attr: &mut crate::comm::collective::CollectiveAttr<T>) -> Result<bool, crate::error::Error> {
        self.inner.query_collective::<T>(coll, attr)
    }

    /// Requests attribute information on the reduce-scatter collective operation.
    /// 
    /// Direclty corresponds to `fi_query_collective` with `flags` = `FI_SCATTER`
    pub fn query_collective_scatter<T: 'static>(&self, coll: crate::enums::CollectiveOp, attr: &mut crate::comm::collective::CollectiveAttr<T>) -> Result<bool, crate::error::Error> {
        self.inner.query_collective_scatter::<T>(coll, attr)
    }
                                
}

impl AsFid for DomainImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.fid.as_fid()
    }
}

impl AsFid for Domain {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.inner.as_fid()
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

    pub fn get_mr_key_size(&self) -> usize {
        self.c_attr.mr_key_size
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

/// Builder for the [Domain] type.
/// 
/// `DomainBuilder` is used to configure and build a new [Domain].
/// It encapsulates an incremental configuration of the address vector set, as provided by a `fi_domain_attr`,
/// followed by a call to `fi_domain_open`  
pub struct DomainBuilder<'a, T, E> {
    fabric: &'a crate::fabric::Fabric,
    info: &'a InfoEntry<E>,
    ctx: Option<&'a mut T>,
    flags: u64,
}

impl<'a> DomainBuilder<'a, (), ()> {


    /// Initiates the creation of new [Domain] on `fabric`, using the configuration found in `info`.
    /// 
    /// The initial configuration is what would be set if no `fi_domain_attr` or `context` was provided to 
    /// the `fi_domain` call. 
    pub fn new<E>(fabric: &'a crate::fabric::Fabric, info: &'a InfoEntry<E>) -> DomainBuilder<'a, (), E> {
        DomainBuilder::<(), E> {
            fabric,
            info,
            flags: 0,
            ctx: None,
        }
    }


    /// Initiates the creation of new [Domain] on `fabir`, using the configuration found in `info`.
    /// 
    /// The initial configuration is what would be set if no `fi_domain_attr` was provided to 
    /// the `fi_domain2` call and `context` was set to a `fi_peer_context`. 
    pub fn new_with_peer<E>(fabric: &'a crate::fabric::Fabric, info: &'a InfoEntry<E>, peer_ctx: &'a mut PeerDomainCtx) -> DomainBuilder<'a, PeerDomainCtx, E> {
        DomainBuilder::<PeerDomainCtx, E> {
            fabric,
            info,
            flags: libfabric_sys::FI_PEER,
            ctx: Some(peer_ctx),
        }
    }
}


impl<'a, E> DomainBuilder<'a, (), E> {
    
    /// Sets the context to be passed to the domain.
    /// 
    /// Corresponds to passing a non-NULL, non-`fi_peer_context` `context` value to `fi_domain`.
    pub fn context<T>(self, ctx: &'a mut T) -> DomainBuilder<'a, T, E> {
        DomainBuilder {
            fabric: self.fabric,
            info: self.info,
            flags: 0,
            ctx: Some(ctx),
        }
    }
}

impl<'a, T, E> DomainBuilder<'a, T, E> {

    // pub fn flags(mut self, flags: u64) -> Self {
    //     self.flags = flags;
    //     self
    // }


    /// Constructs a new [Domain] with the configurations requested so far.
    /// 
    /// Corresponds to creating a `fi_domain_attr`, setting its fields to the requested ones,
    /// and passing it to a `fi_domain` call with an optional `context` (set by [Self::context]).
    /// Or a call to `fi_domain2` with `context` of type `fi_peer_context` and `flags` equal to `FI_PEER`
    pub fn build(self) -> Result<Domain, crate::error::Error> {
        Domain::new(self.fabric, self.info, self.flags, self.info.get_domain_attr().clone(), self.ctx)
    }
}

#[repr(C)]
pub struct PeerDomainCtx {
    c_ctx: libfabric_sys::fi_peer_domain_context,
}

impl PeerDomainCtx {
    pub fn new(size: usize, domain: &Domain) -> Self {
        Self {
            c_ctx : {
                libfabric_sys::fi_peer_domain_context {
                    domain: domain.handle(),
                    size,
                }
            }
        }
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