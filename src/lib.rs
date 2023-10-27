use core::panic;

// use libfabric_sys;
pub mod ep;
pub mod domain;
pub mod eq;
pub mod fabric;
pub mod enums;
pub mod av;
pub mod mr;
pub mod sync;
#[derive(Clone)]
pub struct InfoCaps {
    bitfield: u64,
}

impl InfoCaps {
    pub fn new() -> Self {
        Self { bitfield: 0 }
    }

    pub(crate) fn from(bitfield: u64) -> Self {
        Self { bitfield }
    }

    pub fn msg(&mut self) {self.bitfield |= libfabric_sys::FI_MSG as u64; }
    pub fn tagged(&mut self) {self.bitfield |= libfabric_sys::FI_TAGGED as u64; }
    pub fn rma(&mut self) {self.bitfield |= libfabric_sys::FI_RMA as u64; }
    pub fn atomic(&mut self) {self.bitfield |= libfabric_sys::FI_ATOMIC as u64; }
    pub fn collective(&mut self) {self.bitfield |= libfabric_sys::FI_COLLECTIVE as u64; }

    pub fn is_msg(&mut self) -> bool {self.bitfield & libfabric_sys::FI_MSG as u64 == libfabric_sys::FI_MSG as u64 }
    pub fn is_tagged(&mut self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_TAGGED as u64 }
    pub fn is_rma(&mut self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_RMA as u64 }
    pub fn is_atomic(&mut self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_ATOMIC as u64 }
    pub fn is_collective(&mut self) -> bool {self.bitfield & libfabric_sys::FI_COLLECTIVE as u64 == libfabric_sys::FI_COLLECTIVE as u64 }
}

pub struct Info {
    entries : std::vec::Vec<InfoEntry>,
    c_info: *mut  libfabric_sys::fi_info,
}

#[derive(Clone)]
pub struct InfoEntry {
    pub caps: InfoCaps,
    pub fabric_attr: crate::fabric::FabricAttr,
    pub domain_attr: crate::domain::DomainAttr,
    c_info: *mut  libfabric_sys::fi_info,
}

impl InfoEntry {
    
    pub(crate) fn new(c_info: *mut  libfabric_sys::fi_info) -> Self {
        let mut fabric_attr = crate::fabric::FabricAttr::new();
            unsafe { *fabric_attr.get_mut() = *(*c_info).fabric_attr}
        let mut domain_attr = crate::domain::DomainAttr::new();
            unsafe { *domain_attr.get_mut() = *(*c_info).domain_attr}
        let caps = unsafe {(*c_info).caps};
        Self { caps: InfoCaps::from(caps) , fabric_attr, domain_attr, c_info }
    }
}

impl Info {
    pub fn all() -> Info {
        let mut c_info: *mut libfabric_sys::fi_info = std::ptr::null_mut();
        let c_info_ptr: *mut *mut libfabric_sys::fi_info = &mut c_info;
        
        unsafe{
            let err = libfabric_sys::fi_getinfo(libfabric_sys::fi_version(), 0 as *const std::ffi::c_char, 0 as *const std::ffi::c_char, 0, std::ptr::null(), c_info_ptr);
            if err != 0 {
                panic!("Could not retrieve Info\n"); // [TODO] Use Error()
            }
        }

        let mut entries = std::vec::Vec::new();
        entries.push(InfoEntry::new(c_info));
        unsafe {
            let mut curr = (*c_info).next;
            while  curr!= std::ptr::null_mut() {
                entries.push(InfoEntry::new(curr));
                curr = (*curr).next;
            }
        }
        
        Info {
            entries,
            c_info,
        }
    }

    pub fn with_hints(hints: InfoHints) -> Self {
        let mut c_info: *mut libfabric_sys::fi_info = std::ptr::null_mut();
        let c_info_ptr: *mut *mut libfabric_sys::fi_info = &mut c_info;
        
        unsafe{
            let err = libfabric_sys::fi_getinfo(libfabric_sys::fi_version(), 0 as *const std::ffi::c_char, 0 as *const std::ffi::c_char, libfabric_sys::FI_SOURCE, hints.c_info, c_info_ptr);
            if err != 0 {
                panic!("Could not retrieve Info\n"); // [TODO] Use Error()
            }
        }

        let mut entries = std::vec::Vec::new();
        entries.push(InfoEntry::new(c_info));
        unsafe {
            let mut curr = (*c_info).next;
            while  curr!= std::ptr::null_mut() {
                entries.push(InfoEntry::new(curr));
                curr = (*curr).next;
            }
        }
        
        Info {
            entries,
            c_info,
        }  
    }

    pub fn get(&self) -> Vec<InfoEntry> {
        self.entries.clone()
    }
}

impl Drop for Info {
    
    fn drop(&mut self) {
        unsafe {
            libfabric_sys::fi_freeinfo(self.c_info);
        }
    }
}

pub  struct InfoHints {
    c_info: *mut libfabric_sys::fi_info,
}

impl InfoHints {
    pub fn new() -> Self {
        let c_info = unsafe { libfabric_sys::inlined_fi_allocinfo() };
        unsafe { (*c_info).mode = !0 };
        unsafe { (*c_info).addr_format = libfabric_sys::FI_SOCKADDR };
        Self {  c_info }
    }

    pub fn ep_attr(&mut self, attr: crate::ep::EndpointAttr) -> &mut Self {
        unsafe { *(*self.c_info).ep_attr = *attr.get() };
        self
    }

    pub fn domain_attr(&mut self, attr: crate::domain::DomainAttr) -> &mut Self {
        unsafe { *(*self.c_info).domain_attr = *attr.get() };
        self
    }
}



pub type Address = libfabric_sys::fi_addr_t; 
pub type DataType = libfabric_sys::fi_datatype;
// pub type Op = libfabric_sys::fi_op;
pub struct Msg {
    c_msg: *mut libfabric_sys::fi_msg,
}

pub struct MsgRma {
    c_msg_rma: *mut libfabric_sys::fi_msg_rma,
}

pub struct MsgTagged {
    c_msg_tagged: *mut libfabric_sys::fi_msg_tagged,
}

pub struct MsgAtomic {
    c_msg_atomic: *mut libfabric_sys::fi_msg_atomic,
}



pub struct TxAttr {
    c_attr: libfabric_sys::fi_tx_attr,
}

impl TxAttr {
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_tx_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_tx_attr {
        &mut self.c_attr
    }
}

pub struct AtomicAttr {
    pub(crate) c_attr : libfabric_sys::fi_atomic_attr,
}

impl AtomicAttr {
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_atomic_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_atomic_attr {
        &mut self.c_attr
    }
}

// pub struct Mc {
//     pub(crate) c_mc: *mut libfabric_sys::fid_mc,
// }

// impl FID for Mc {
//     fn fid(&self) -> *mut libfabric_sys::fid {
//         unsafe { &mut (*self.c_mc).fid as *mut libfabric_sys::fid }

//     }
// }
pub struct Stx {

    #[allow(dead_code)]
    c_stx: *mut libfabric_sys::fid_stx,
}

impl Stx {
    pub(crate) fn new<T0>(domain: &crate::domain::Domain, mut attr: crate::TxAttr, context: &mut T0) -> Self {
        let mut c_stx: *mut libfabric_sys::fid_stx = std::ptr::null_mut();
        let c_stx_ptr: *mut *mut libfabric_sys::fid_stx = &mut c_stx;
        let err = unsafe { libfabric_sys::inlined_fi_stx_context(domain.c_domain, attr.get_mut(), c_stx_ptr, context as *mut T0 as *mut std::ffi::c_void) };

        if err != 0 {
            panic!("fi_stx_context failed {}", err);
        }

        Self { c_stx }
    }
}

// pub struct SrxAttr {
//     c_attr: libfabric_sys::fi_srx_attr,
// }

// impl SrxAttr {
//     pub(crate) fn get(&self) -> *const libfabric_sys::fi_srx_attr {
//         &self.c_attr
//     }

//     pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_srx_attr {
//         &mut self.c_attr
//     }
// }


pub struct RxAttr {
    c_rx_attr: libfabric_sys::fi_rx_attr,
}


impl RxAttr {
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_rx_attr {
        &self.c_rx_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_rx_attr {
        &mut self.c_rx_attr
    }
}

pub struct Counter {
    pub(crate) c_cntr: *mut libfabric_sys::fid_cntr,
}

impl Counter {
    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: CounterAttr) -> Self {
        let mut c_cntr: *mut libfabric_sys::fid_cntr = std::ptr::null_mut();
        let c_cntr_ptr: *mut *mut libfabric_sys::fid_cntr = &mut c_cntr;
        let err = unsafe { libfabric_sys::inlined_fi_cntr_open(domain.c_domain, attr.get_mut(), c_cntr_ptr, std::ptr::null_mut()) };

        if err != 0 {
            panic!("fi_cntr_open failed {}", err);
        }

        Self { c_cntr }
    }

    pub fn read(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_read(self.c_cntr) }
    }

    pub fn readerr(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_readerr(self.c_cntr) }
    }

    pub fn add(&self, val: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_add(self.c_cntr, val) };
    
        if err != 0 {
            panic!("fi_cntr_add failed {}", err);
        }
    }

    pub fn adderr(&self, val: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_adderr(self.c_cntr, val) };
            
        if err != 0 {
            panic!("fi_cntr_adderr failed {}", err);
        }
    }

    pub fn set(&self, val: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_set(self.c_cntr, val) };
            
        if err != 0 {
            panic!("fi_cntr_set failed {}", err);
        }
    }

    pub fn seterr(&self, val: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_seterr(self.c_cntr, val) };
            
        if err != 0 {
            panic!("fi_cntr_seterr failed {}", err);
        }
    }

    pub fn wait(&self, threshold: u64, timeout: i32) -> i32 { // [TODO]
        unsafe { libfabric_sys::inlined_fi_cntr_wait(self.c_cntr, threshold, timeout) }
    }
}


pub struct CounterAttr {
    pub(crate) c_attr: libfabric_sys::fi_cntr_attr,
}

impl CounterAttr {

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_cntr_attr {
        &self.c_attr
    }   

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_cntr_attr {
        &mut self.c_attr
    }   
}    



pub fn rx_addr(addr: Address, rx_index: i32, rx_ctx_bits: i32) -> Address {
    unsafe { libfabric_sys::inlined_fi_rx_addr(addr, rx_index, rx_ctx_bits) }
}

pub struct IoVec{
    c_attr: libfabric_sys::iovec,
}

impl IoVec {
    pub(crate) fn get(&self) ->  *const libfabric_sys::iovec {
        &self.c_attr
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::iovec {
        &mut self.c_attr
    }
}

pub struct Ioc {
    c_attr: libfabric_sys::fi_ioc,
}

impl Ioc {
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_ioc {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_ioc {
        &mut self.c_attr
    }
}


pub struct Mc {
    c_mc: *mut libfabric_sys::fid_mc,
}

impl Mc {
    pub(crate) fn new<T0,T1>(ep: &crate::ep::Endpoint, addr: &T0, flags: u64, ctx: &mut T1) -> Self {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join(ep.c_ep, addr as *const T0 as *const std::ffi::c_void, flags, c_mc_ptr, ctx as *mut T1 as *mut std::ffi::c_void) };

        if err != 0 {
            panic!("fi_join failed {}", err);
        }

        Self { c_mc }
    }

    pub(crate) fn new_collective<T0>(ep: &crate::ep::Endpoint, addr: Address, set: &crate::av::AddressVectorSet, flags: u64, ctx: &mut T0) -> Self {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join_collective(ep.c_ep, addr, set.c_set, flags, c_mc_ptr, ctx as *mut T0 as *mut std::ffi::c_void) };

        if err != 0 {
            panic!("fi_join_collective failed {}", err);
        }

        Self { c_mc }
    }

    pub fn addr(&self) -> Address {
        unsafe { libfabric_sys::inlined_fi_mc_addr(self.c_mc) }
    }
}

pub trait FID{
    fn fid(&self) -> *mut libfabric_sys::fid;
    
    fn setname<T>(&mut self, addr:&[T]) {
        let err = unsafe { libfabric_sys::inlined_fi_setname(self.fid(), addr.as_ptr() as *mut std::ffi::c_void, addr.len()) };
        
        if err != 0 {
            panic!("fi_setname failed {}", err);
        }
    }

    fn getname<T0>(&mut self, addr: &mut[T0]) -> usize {
        let mut len: usize = 0;
        let len_ptr: *mut usize = &mut len;
        let err = unsafe { libfabric_sys::inlined_fi_getname(self.fid(), addr.as_mut_ptr() as *mut std::ffi::c_void, len_ptr) };
        
        if err != 0 {
            panic!("fi_setname failed {}", err);
        }

        len
    }
    
    fn setopt<T0>(&self, level: i32, optname: i32, opt: &[T0]) {
        let err = unsafe { libfabric_sys::inlined_fi_setopt(self.fid(), level, optname, opt.as_ptr() as *const std::ffi::c_void, opt.len())};
        if err != 0 {
            panic!("fi_setopt failed {}", err);
        }
    }

    fn getopt<T0>(&self, level: i32, optname: i32, opt: &mut [T0]) -> usize{
        let mut len = 0 as usize;
        let len_ptr : *mut usize = &mut len;
        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.fid(), level, optname, opt.as_mut_ptr() as *mut std::ffi::c_void, len_ptr)};
        if err != 0 {
            panic!("fi_getopt failed {}", err);
        }

        len
    }

    fn close(&mut self) {
        let err = unsafe { libfabric_sys::inlined_fi_close(self.fid()) };

        if err != 0 {
            panic!("fi_close failed {}", err);
        }
    }

    fn cancel(&self) {
        let _ = unsafe { libfabric_sys::inlined_fi_cancel(self.fid(), std::ptr::null_mut()) };
    }


    fn control<T0>(&self, opt: crate::enums::ControlOpt, arg: &mut T0) {
        let err = unsafe { libfabric_sys::inlined_fi_control(self.fid(), opt.get_value() as i32, arg as *mut T0 as *mut std::ffi::c_void) };
    
        if err != 0 {
            panic!("fi_control failed {}", err);
        }
    }
}


pub type CollectiveOp = libfabric_sys::fi_collective_op;
pub struct CollectiveAttr {
    pub(crate) c_attr: libfabric_sys::fi_collective_attr,
}

impl CollectiveAttr {

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_collective_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_collective_attr {
        &mut self.c_attr
    }
}

// pub fn get_params() -> Vec<Param> {
//     let mut len = 0 as i32;
//     let len_ptr : *mut i32 = &mut len;
//     let mut c_params: *mut libfabric_sys::fi_param = std::ptr::null_mut();
//     let mut c_params_ptr: *mut *mut libfabric_sys::fi_param = &mut c_params;
    
//     let err = libfabric_sys::fi_getparams(c_params_ptr, len_ptr);
//     if err != 0 {
//         panic!("fi_getparams failed {}", err);
//     }

//     let mut params = Vec::<Param>::new();
//     for i  in 0..len {
//         params.push(Param { c_param: unsafe{ c_params.add(i as usize) } });
//     }

//     params
// }


// pub struct Param {
//     c_param: libfabric_sys::fi_param,
// }

#[test]
fn get_info(){
    let info = Info::all();
    let _entries = info.get();
}

#[test]
fn ft_open_fabric_res() {
    let info = Info::all();
    let entries = info.get();
    
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(crate::eq::EventQueueAttr::new());
    let mut domain = fab.domain(&entries[0]);
    domain.close();
    eq.close();
    fab.close();
}

pub fn error_to_string(errnum: i64) -> String {
    let ret = unsafe { libfabric_sys::fi_strerror(errnum as i32) };
    let str = unsafe { std::ffi::CStr::from_ptr(ret) };
    str.to_str().unwrap().to_string()
}