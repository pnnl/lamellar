use core::slice;
use std::ffi::CString;

use libfabric_sys::fi_domain_attr;

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{enums::{DomainCaps, MrMode, TClass}, eq::{EventQueue, EventQueueBase, ReadEq}, fabric::FabricImpl, fid::{self, AsRawFid, AsRawTypedFid, AsTypedFid, DomainRawFid, OwnedDomainFid}, info::InfoEntry, utils::{check_error, to_fi_datatype}, MyOnceCell, MyRc};

pub(crate) struct DomainImplBase<EQ: ?Sized> {
    pub(crate) c_domain: OwnedDomainFid,
    pub(crate) mr_mode: MrMode,
    pub(crate) mr_key_size: usize,
    pub(crate) _eq_rc: MyOnceCell<(MyRc<EQ>, bool)>,
    _fabric_rc: MyRc<FabricImpl>,
}



pub(crate) trait DomainImplT: AsRawFid + AsRawTypedFid<Output = DomainRawFid>{
    fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error>;
    #[allow(unused)]
    fn get_fabric_impl(&self) -> MyRc<FabricImpl>;
    fn get_mr_mode(&self) -> MrMode;
    fn get_mr_key_size(&self) -> usize;
}

impl<EQ: ?Sized> DomainImplT for DomainImplBase<EQ> {
    fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
       self.unmap_key(key) 
    }

    fn get_fabric_impl(&self) -> MyRc<FabricImpl> {
        self._fabric_rc.clone()
    }

    fn get_mr_mode(&self) -> MrMode {
        self.mr_mode
    }

    fn get_mr_key_size(&self) -> usize {
        self.mr_key_size
    }
}

//================== Domain (fi_domain) ==================//

impl<EQ: ?Sized > DomainImplBase<EQ> {

    pub(crate) fn new<T0, E>(fabric: &MyRc<crate::fabric::FabricImpl>, info: &InfoEntry<E>, flags: u64, domain_attr: DomainAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_domain: DomainRawFid = std::ptr::null_mut();
        let err =
            if let Some(ctx) = context {
                if flags == 0 {
                    unsafe { libfabric_sys::inlined_fi_domain(fabric.as_raw_typed_fid(), info.c_info, &mut c_domain, (ctx as *mut T0).cast()) }
                    
                }
                else {
                    unsafe { libfabric_sys::inlined_fi_domain2(fabric.as_raw_typed_fid(), info.c_info, &mut c_domain, flags, (ctx as *mut T0).cast()) }
                }
            }
            else if flags == 0 {
                unsafe { libfabric_sys::inlined_fi_domain(fabric.as_raw_typed_fid(), info.c_info, &mut c_domain, std::ptr::null_mut()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_domain2(fabric.as_raw_typed_fid(), info.c_info, &mut c_domain, flags, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self { 
                    c_domain: OwnedDomainFid::from(c_domain), 
                    mr_key_size: domain_attr.mr_key_size,
                    mr_mode: domain_attr.mr_mode,
                    _fabric_rc: fabric.clone(), 
                    _eq_rc: MyOnceCell::new(),
                })
        }
    }
}

impl DomainImplBase<dyn ReadEq+ Sync + Send> {

    
    pub(crate) fn bind(&self, eq: MyRc<dyn ReadEq+ Sync + Send>, async_mem_reg: bool) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_domain_bind(self.as_raw_typed_fid(), eq.as_raw_fid(), if async_mem_reg {libfabric_sys::FI_REG_MR} else {0})} ;

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            if self._eq_rc.set((eq, async_mem_reg)).is_err() {
                panic!("Domain is alread bound to an EventQueue");
            }
            Ok(())
        }
    } 
}

impl<EQ: ?Sized > DomainImplBase<EQ> {

    // pub(crate) fn srx_context<T0>(&self, rx_attr: crate::RxAttr) -> Result<crate::ep::Endpoint, crate::error::Error> { //[TODO]
    //     crate::ep::Endpoint::from_attr(self, rx_attr)
    // }
    
    // pub(crate) fn srx_context_with_context<T0>(&self, rx_attr: crate::RxAttr, context: &mut T0) -> Result<crate::ep::Endpoint, crate::error::Error> { //[TODO]
    //     crate::ep::Endpoint::from_attr_with_context(self, rx_attr, context)
    // }

    pub(crate) fn query_atomic<T: 'static>(&self, op: crate::enums::Op, mut attr: crate::comm::atomic::AtomicAttr, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_atomic(self.as_raw_typed_fid(), to_fi_datatype::<T>(), op.get_value(), attr.get_mut(), flags )};

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
                unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.as_raw_typed_fid(), raw_key.1 , raw_key.0.as_mut_ptr().cast(), raw_key.0.len(), &mut mapped_key, flags) }
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
    //     let err = unsafe { libfabric_sys::inlined_fi_mr_map_raw(self.as_raw_typed_fid(), base_addr, raw_key, key_size, key, flags) };
        
    //     check_error(err.try_into().unwrap())
    // }

    pub(crate) fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_unmap_key(self.as_raw_typed_fid(), key) };


        check_error(err.try_into().unwrap())
    }

    // pub fn stx_context<T0>(&self, attr: crate::TxAttr) -> Result<crate::Stx, crate::error::Error> { //[TODO]
    //     crate::Stx::new(self, attr, std::ptr::null_mut())
    // }

    // pub fn stx_context_with_context<T0>(&self, attr: crate::TxAttr , context: &mut T0) -> Result<crate::Stx, crate::error::Error> { //[TODO]
    //     crate::Stx::new(self, attr, context)
    // }

    pub(crate) fn query_collective<T: 'static>(&self, coll: crate::enums::CollectiveOp, attr: &mut crate::comm::collective::CollectiveAttr<T>) -> Result<bool, crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_collective(self.as_raw_typed_fid(), coll.get_value(), attr.get_mut(), 0) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(true)
        }
    }

    pub(crate) fn query_collective_scatter<T: 'static>(&self, coll: crate::enums::CollectiveOp, attr: &mut crate::comm::collective::CollectiveAttr<T>) -> Result<bool, crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_query_collective(self.as_raw_typed_fid(), coll.get_value(), attr.get_mut(), libfabric_sys::fi_collective_op_FI_SCATTER.into()) };
    
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
// pub type Domain = DomainBase<dyn EventQueueImplT>;

pub struct DomainBase<EQ: ?Sized+ Sync + Send> {
    pub(crate) inner: MyRc<DomainImplBase<EQ>>,
}

impl<EQ: ?Sized+ Sync + Send> DomainImplT for DomainBase<EQ> {
    fn unmap_key(&self, key: u64) -> Result<(), crate::error::Error> {
        self.inner.unmap_key(key)
    }

    fn get_fabric_impl(&self) -> MyRc<FabricImpl> {
        self.inner.get_fabric_impl()
    }

    fn get_mr_mode(&self) -> MrMode {
        self.inner.get_mr_mode()
    }

    fn get_mr_key_size(&self) -> usize {
        self.inner.get_mr_key_size()
    }
}

impl<EQ: ?Sized+ Sync + Send> DomainBase<EQ> {
    
    pub(crate) fn new<T0, E>(fabric: &crate::fabric::Fabric, info: &InfoEntry<E>, flags: u64, domain_attr: DomainAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(
            Self{
                inner: 
                MyRc::new(DomainImplBase::new(&fabric.inner, info, flags, domain_attr, context)?)
            }
        )    
    }
}

impl DomainBase<dyn ReadEq+ Sync + Send> {
    /// Associates an [crate::eq::EventQueue] with the domain.
    /// 
    /// If `async_mem_reg` is true, the provider should perform all memory registration operations asynchronously, with the completion reported through the event queue
    /// 
    /// Corresponds to `fi_domain_bind`, with flag `FI_REG_MR` if `async_mem_reg` is true. 
    pub(crate) fn bind_eq<EQ: ReadEq + 'static+ Sync + Send>(&self, eq: &EventQueueBase<EQ>, async_mem_reg: bool) -> Result<(), crate::error::Error> {
        self.inner.bind(eq.inner.clone(), async_mem_reg)
    }
}

impl<EQ: ?Sized+ Sync + Send> DomainBase<EQ> {

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
    
    #[allow(dead_code)]
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

impl<EQ: ?Sized> AsFid for DomainImplBase<EQ> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.c_domain.as_fid()
    }
}

impl<EQ: ?Sized> AsTypedFid<DomainRawFid> for DomainImplBase<EQ> {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<'_, DomainRawFid> {
       self.c_domain.as_typed_fid()
    }
}

impl<EQ: ?Sized> AsRawTypedFid for DomainImplBase<EQ> {
    type Output = DomainRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_domain.as_raw_typed_fid()
    }
}

impl<EQ: Sync + Send> AsFid for DomainBase<EQ> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.inner.as_fid()
    }
}

impl<EQ: ?Sized> AsRawFid for DomainImplBase<EQ> {
    fn as_raw_fid(&self) -> fid::RawFid {
        self.c_domain.as_raw_fid()
    }
}

impl<EQ: ?Sized+ Sync + Send> AsRawFid for DomainBase<EQ> {
    fn as_raw_fid(&self) -> fid::RawFid {
        self.inner.as_raw_fid()
    }
}

impl<EQ: Sync + Send> AsTypedFid<DomainRawFid> for DomainBase<EQ> {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<'_, DomainRawFid> {
       self.inner.as_typed_fid()
    }
}

impl<EQ: ?Sized+ Sync + Send> AsRawTypedFid for DomainBase<EQ> {
    type Output = DomainRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

//================== Domain attribute ==================//

#[derive(Clone)]
pub struct DomainAttr {
    _domain: usize, // [TODO] Not supported
    pub name: String, 
    pub threading: crate::enums::Threading,
    pub control_progress: crate::enums::Progress,
    pub data_progress: crate::enums::Progress,
    pub resource_mgmt: crate::enums::ResourceMgmt,
    pub av_type: crate::enums::AddressVectorType,
    pub mr_mode: crate::enums::MrMode,
    pub mr_key_size: usize,
    pub cq_data_size: usize,
    pub cq_cnt: usize,
    pub ep_cnt: usize,
    pub tx_ctx_cnt: usize,
    pub rx_ctx_cnt: usize,
    pub max_ep_tx_ctx: usize,
    pub max_ep_rx_ctx: usize,
    pub max_ep_stx_ctx: usize,
    pub max_ep_srx_ctx: usize,
    pub cntr_cnt: usize,
    pub mr_iov_limit: usize,
    pub caps: crate::enums::DomainCaps,
    pub mode: crate::enums::Mode,
    _auth_key: Option<Vec<u8>>, // [TODO] Not supported
    pub max_err_data: usize,
    pub mr_cnt: usize,
    pub tclass: crate::enums::TClass,
}

impl DomainAttr {
     
    pub fn new() -> Self {
        Self {
            _domain: 0,
            name: String::new(),
            threading: crate::enums::Threading::Unspec,
            control_progress: crate::enums::Progress::Unspec,
            data_progress: crate::enums::Progress::Unspec,
            resource_mgmt: crate::enums::ResourceMgmt::Unspec,
            av_type: crate::enums::AddressVectorType::Unspec,
            mr_mode: crate::enums::MrMode::new(),
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
            caps: crate::enums::DomainCaps::new(),
            mode: crate::enums::Mode::new(),
            _auth_key: None,
            max_err_data: 0,
            mr_cnt: 0,
            tclass: crate::enums::TClass::Unspec,
        }
    }
}

impl From<libfabric_sys::fi_domain_attr> for DomainAttr {
    fn from(value: libfabric_sys::fi_domain_attr) -> Self {
        Self {
            _domain: value.domain as usize,
            name: {if !value.name.is_null() {unsafe {std::ffi::CStr::from_ptr(value.name)}.to_str().unwrap().to_string()} else {String::new()}} ,
            threading: crate::enums::Threading::from_value(value.threading),
            control_progress: crate::enums::Progress::from_value(value.control_progress),
            data_progress: crate::enums::Progress::from_value(value.data_progress),
            resource_mgmt: crate::enums::ResourceMgmt::from_value(value.resource_mgmt),
            av_type: crate::enums::AddressVectorType::from_value(value.av_type),
            mr_mode: crate::enums::MrMode::from_value(value.mr_mode as u32),
            mr_key_size:value.mr_key_size,
            cq_data_size: value.cq_data_size,
            cq_cnt: value.cq_cnt,
            ep_cnt: value.ep_cnt,
            tx_ctx_cnt: value.tx_ctx_cnt,
            rx_ctx_cnt: value.rx_ctx_cnt,
            max_ep_tx_ctx: value.max_ep_tx_ctx,
            max_ep_rx_ctx: value.max_ep_rx_ctx,
            max_ep_stx_ctx: value.max_ep_stx_ctx,
            max_ep_srx_ctx: value.max_ep_srx_ctx,
            cntr_cnt: value.cntr_cnt,
            mr_iov_limit: value.mr_iov_limit,
            caps: crate::enums::DomainCaps::from_value(value.caps),
            mode: crate::enums::Mode::from_value(value.mode),
            _auth_key: {if !value.auth_key.is_null() {Some(unsafe{slice::from_raw_parts(value.auth_key, value.auth_key_size)}.to_vec())} else {None} },
            max_err_data: value.max_err_data,
            mr_cnt: value.mr_cnt,
            tclass: crate::enums::TClass::from_value(value.tclass),
       }  
    }
}

impl Into<libfabric_sys::fi_domain_attr> for DomainAttr {
    fn into(self) -> libfabric_sys::fi_domain_attr {
        // let (key, key_size) = if let Some(val) = self.auth_key {
        //     (val.as_ptr(), val.len())
        // }
        // else {
        //     (std::ptr::null(), 0)
        // };
        libfabric_sys::fi_domain_attr {
            domain: std::ptr::null_mut(),
            name: std::ptr::null_mut(), // Only used as output so it's fine
            threading: self.threading.get_value(),
            control_progress: self.control_progress.get_value(),
            data_progress: self.data_progress.get_value(),
            resource_mgmt: self.resource_mgmt.into(),
            av_type: self.av_type.get_value(),
            mr_mode: self.mr_mode.get_value() as i32,
            mr_key_size: self.mr_key_size,
            cq_data_size: self.cq_data_size,
            cq_cnt: self.cq_cnt,
            ep_cnt: self.ep_cnt,
            tx_ctx_cnt: self.tx_ctx_cnt,
            rx_ctx_cnt: self.rx_ctx_cnt,
            max_ep_tx_ctx: self.max_ep_tx_ctx,
            max_ep_rx_ctx: self.max_ep_rx_ctx,
            max_ep_stx_ctx: self.max_ep_stx_ctx,
            max_ep_srx_ctx: self.max_ep_srx_ctx,
            cntr_cnt: self.cntr_cnt,
            mr_iov_limit: self.mr_iov_limit,
            caps: self.caps.into(),
            mode: self.mode.into(),
            auth_key: std::ptr::null_mut(),
            auth_key_size: 0,
            max_err_data: self.max_err_data,
            mr_cnt: self.mr_cnt,
            tclass: self.tclass.get_value(),
       }  
    }
}

/// Builder for the [Domain] type.
/// 
/// `DomainBuilder` is used to configure and build a new [Domain].
/// It encapsulates an incremental configuration of the address vector set, as provided by a `fi_domain_attr`,
/// followed by a call to `fi_domain_open`  
pub struct DomainBuilder<'a, T, E> {
    pub(crate) fabric: &'a crate::fabric::Fabric,
    pub(crate) info: &'a InfoEntry<E>,
    pub(crate) ctx: Option<&'a mut T>,
    pub(crate) flags: u64,
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
            flags: self.flags,
            ctx: Some(ctx),
        }
    }
}

impl<'a, T, E> DomainBuilder<'a, T, E> {
    /// Constructs a new [Domain] with the configurations requested so far.
    /// 
    /// Corresponds to creating a `fi_domain_attr`, setting its fields to the requested ones,
    /// and passing it to a `fi_domain` call with an optional `context` (set by [Self::context]).
    /// Or a call to `fi_domain2` with `context` of type `fi_peer_context` and `flags` equal to `FI_PEER`
    pub fn build_and_bind<EQ: ReadEq + 'static+ Sync + Send>(self, eq: &EventQueue<EQ>, async_mem_reg: bool) -> Result<DomainBase<dyn ReadEq+ Sync + Send>, crate::error::Error> {
        let domain = DomainBase::<dyn ReadEq+ Sync + Send>::new(self.fabric, self.info, self.flags, self.info.get_domain_attr().clone(), self.ctx)?;
        domain.bind_eq(eq, async_mem_reg)?;
        Ok(domain)
    }

}

impl<'a, T, E> DomainBuilder<'a, T, E> {
    /// Constructs a new [Domain] with the configurations requested so far.
    /// 
    /// Corresponds to creating a `fi_domain_attr`, setting its fields to the requested ones,
    /// and passing it to a `fi_domain` call with an optional `context` (set by [Self::context]).
    /// Or a call to `fi_domain2` with `context` of type `fi_peer_context` and `flags` equal to `FI_PEER`
    pub fn build(self) -> Result<Domain, crate::error::Error> {
        let domain = DomainBase::new(self.fabric, self.info, self.flags, self.info.get_domain_attr().clone(), self.ctx)?;
        Ok(domain)
    }
}

pub type Domain = DomainBase<()>;
pub type BoundDomain = DomainBase<dyn ReadEq+ Sync + Send>;


#[repr(C)]
pub struct PeerDomainCtx {
    c_ctx: libfabric_sys::fi_peer_domain_context,
}

impl PeerDomainCtx {
    pub fn new<EQ: Sync + Send>(size: usize, domain: &DomainBase<EQ>) -> Self {
        Self {
            c_ctx : {
                libfabric_sys::fi_peer_domain_context {
                    domain: domain.as_raw_typed_fid(),
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
        let info = Info::new().build().unwrap();
        let entry = info.into_iter().next().unwrap();
        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let count = 10;
        let mut doms = Vec::new();
        for _ in 0..count {
            let domain = crate::domain::DomainBuilder::new(&fab, &entry).build().unwrap();
            doms.push(domain);
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::info::Info;

    #[test]

    fn domain_drops_before_fabric() {
        let info = Info::new().build().unwrap();
        let entry = info.into_iter().next().unwrap();
        
        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let count = 10;
        let mut doms = Vec::new();
        for _ in 0..count {
            let domain = crate::domain::DomainBuilder::new(&fab, &entry).build().unwrap();
            doms.push(domain);
        }
        drop(fab);
    }
}