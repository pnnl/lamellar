use core::panic;

use enums::AvType;

// use libfabric_sys;
pub mod ep;
pub mod domain;
pub mod eq;
pub mod fabric;
pub mod enums;
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
    pub fabric_attr: FabricAttr,
    pub domain_attr: DomainAttr,
    c_info: *mut  libfabric_sys::fi_info,
}

impl InfoEntry {
    
    pub(crate) fn new(c_info: *mut  libfabric_sys::fi_info) -> Self {
        let fabric_attr = unsafe { FabricAttr::new((*c_info).fabric_attr)};
        let domain_attr = unsafe { DomainAttr::new((*c_info).domain_attr)};
        let caps = unsafe {(*c_info).caps};
        Self { caps: InfoCaps::from(caps) , fabric_attr, domain_attr, c_info }
    }
}
// pub struct fi_info {
//     pub next: *mut libfabric_sys::fi_info,
//     pub caps: u64,
//     pub mode: u64,
//     pub addr_format: u32,
//     pub src_addrlen: usize,
//     pub dest_addrlen: usize,
//     pub src_addr: *mut ::std::os::raw::c_void,
//     pub dest_addr: *mut ::std::os::raw::c_void,
//     pub handle: libfabric_sys::fid_t,
//     pub tx_attr: *mut libfabric_sys::fi_tx_attr,
//     pub rx_attr: *mut libfabric_sys::fi_rx_attr,
//     pub ep_attr: *mut libfabric_sys::fi_ep_attr,
//     pub domain_attr: *mut libfabric_sys::fi_domain_attr,
//     pub fabric_attr: *mut libfabric_sys::fi_fabric_attr,
//     pub nic: *mut libfabric_sys::fid_nic,
// }
// pub fn fi_getinfo(
//     version: u32,
//     node: *const ::std::os::raw::c_char,
//     service: *const ::std::os::raw::c_char,
//     flags: u64,
//     hints: *const fi_info,
//     info: *mut *mut fi_info,
// ) -> ::std::os::raw::c_int;
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


#[derive(Clone)]
pub struct FabricAttr {
    pub name: String,
    pub prov_name: String,
    pub prov_version: (u32,u32),
    pub api_version: u32,
    c_attr : *mut libfabric_sys::fi_fabric_attr,
}

impl FabricAttr {

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_fabric_attr {
        self.c_attr as *const libfabric_sys::fi_fabric_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_fabric_attr {
        self.c_attr
    }    
}

#[derive(Clone)]
pub struct DomainAttr {
    pub name: String,
    #[allow(dead_code)]
    c_attr : *const libfabric_sys::fi_domain_attr,
}

impl DomainAttr {
    pub(crate) fn new(c_attr: *const libfabric_sys::fi_domain_attr) -> Self {
        let c_str_name = unsafe {(*c_attr).name};
        let name = unsafe {std::ffi::CStr::from_ptr(c_str_name)}.to_str().unwrap().to_owned();

        DomainAttr { name, c_attr }
    }

    pub fn get_av_type(&self) ->  enums::AvType {
        crate::enums::AvType::from_value(unsafe { (*self.c_attr).av_type })
    }
}

// pub struct fi_domain_attr {
//     pub domain: *mut fid_domain,
//     pub name: *mut ::std::os::raw::c_char,
//     pub threading: fi_threading,
//     pub control_progress: fi_progress,
//     pub data_progress: fi_progress,
//     pub resource_mgmt: fi_resource_mgmt,
//     pub av_type: fi_av_type,
//     pub mr_mode: ::std::os::raw::c_int,
//     pub mr_key_size: usize,
//     pub cq_data_size: usize,
//     pub cq_cnt: usize,
//     pub ep_cnt: usize,
//     pub tx_ctx_cnt: usize,
//     pub rx_ctx_cnt: usize,
//     pub max_ep_tx_ctx: usize,
//     pub max_ep_rx_ctx: usize,
//     pub max_ep_stx_ctx: usize,
//     pub max_ep_srx_ctx: usize,
//     pub cntr_cnt: usize,
//     pub mr_iov_limit: usize,
//     pub caps: u64,
//     pub mode: u64,
//     pub auth_key: *mut u8,
//     pub auth_key_size: usize,
//     pub max_err_data: usize,
//     pub mr_cnt: usize,
//     pub tclass: u32,
// }
impl FabricAttr {
    pub(crate) fn new(c_attr: *mut libfabric_sys::fi_fabric_attr) -> Self { 
        
        let c_str_name = unsafe {(*c_attr).name};
        let c_str_provider_name = unsafe {(*c_attr).prov_name};
        let name = unsafe {std::ffi::CStr::from_ptr(c_str_name)}.to_str().unwrap().to_owned();
        let prov_name = unsafe {std::ffi::CStr::from_ptr(c_str_provider_name)}.to_str().unwrap().to_owned();
        let prov_version =  unsafe {((*c_attr).prov_version >> 16, (*c_attr).prov_version  & 0xFFFF as u32)};
        let api_version =  unsafe {(*c_attr).api_version};

        Self { 
            name,
            prov_name,
            prov_version,
            api_version,
            c_attr, 
        } 
    }
}

// pub struct fi_fabric_attr {
//     pub fabric: *mut fid_fabric,
//     pub name: *mut ::std::os::raw::c_char,
//     pub prov_name: *mut ::std::os::raw::c_char,
//     pub prov_version: u32,
//     pub api_version: u32,
// }

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
pub struct Poll {
    pub(crate) c_poll: *mut libfabric_sys::fid_poll,
}

impl Poll {
    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: crate::PollAttr) -> Self {
        let mut c_poll: *mut libfabric_sys::fid_poll = std::ptr::null_mut();
        let c_poll_ptr: *mut *mut libfabric_sys::fid_poll = &mut c_poll;
        let err = unsafe { libfabric_sys::inlined_fi_poll_open(domain.c_domain, attr.get_mut(), c_poll_ptr) };
    
        if err != 0 {
            panic!("fi_poll_open failed {}", err);
        }
    
        Self { c_poll }
    }

    pub fn poll<T0>(&self, contexts: &mut [T0]) {
        let err = unsafe { libfabric_sys::inlined_fi_poll(self.c_poll, contexts.as_mut_ptr() as *mut *mut std::ffi::c_void,  contexts.len() as i32) };
        
        if err != 0{
            panic!("fi_poll failed {}", err);
        }
    }

    pub fn add(&self, fid: &impl FID, flags:u64) {
        let err = unsafe { libfabric_sys::inlined_fi_poll_add(self.c_poll, fid.fid(), flags) };

        if err != 0 {
            panic!("fi_poll_add failed {}", err);
        }
    }

    pub fn del(&self, fid: &impl FID, flags:u64) {
        let err = unsafe { libfabric_sys::inlined_fi_poll_del(self.c_poll, fid.fid(), flags) };

        if err != 0 {
            panic!("fi_poll_del failed {}", err);
        }
    }


}

pub struct PollAttr {
    pub(crate) c_attr: libfabric_sys::fi_poll_attr,
}

impl PollAttr {

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_poll_attr {
        &self.c_attr
    }   

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_poll_attr {
        &mut self.c_attr
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

    pub fn bind(&self, fid: &impl FID, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_bind(self.c_mr, fid.fid(), flags) } ;
        
        if err != 0 {
            panic!("fi_mr_bind failed {}", err);
        }
    }

    pub fn refresh(&self, iov: &IoVec, count: usize, flags: u64) {
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
#[derive(Debug)]
pub struct AvAttr {
    pub(crate) c_attr: libfabric_sys::fi_av_attr, 
}

// pub struct fi_av_attr {
//     pub type_: fi_av_type,
//     pub rx_ctx_bits: ::std::os::raw::c_int,
//     pub count: usize,
//     pub ep_per_node: usize,
//     pub name: *const ::std::os::raw::c_char,
//     pub map_addr: *mut ::std::os::raw::c_void,
//     pub flags: u64,
// }

impl AvAttr {
    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_av_attr{
            type_: AvType::UNSPEC.get_value(), 
            rx_ctx_bits: 0,
            count: 0,
            ep_per_node: 0,
            name: std::ptr::null(),
            map_addr: std::ptr::null_mut(),
            flags: 0
        };

        Self { c_attr }
    }

    pub fn avtype(&mut self, av_type: crate::enums::AvType) -> &mut Self{
        self.c_attr.type_ = av_type.get_value();

        self
    }

    pub fn count(&mut self, count: usize) -> &mut Self {
        self.c_attr.count = count;

        self
    }

    pub fn flags(&mut self, flags: u64) -> &mut Self {
        self.c_attr.flags = flags;

        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_av_attr {
        &self.c_attr
    }   

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_av_attr {
        &mut self.c_attr
    }  
}

pub struct Av {
    pub(crate) c_av: *mut libfabric_sys::fid_av, 
}

impl Av {
    pub fn new(domain: &crate::domain::Domain, mut attr: AvAttr) -> Self {
        let mut c_av:   *mut libfabric_sys::fid_av =  std::ptr::null_mut();
        let c_av_ptr: *mut *mut libfabric_sys::fid_av = &mut c_av;

        let err = unsafe { libfabric_sys::inlined_fi_av_open(domain.c_domain, attr.get_mut(), c_av_ptr, std::ptr::null_mut()) };

        if err != 0 {
            panic!("fi_av_open failed {}", err);
        }

        Self {
            c_av,
        }
    }

    pub fn bind(&self, fid: &impl FID, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_bind(self.c_av, fid.fid(), flags) };

        if err != 0 {
            panic!("fi_av_bind failed {}", err);
        }
    }

    pub fn insert<T0>(&self, buf: &[T0], addr: &mut Address, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_insert(self.c_av, buf.as_ptr() as *const std::ffi::c_void, buf.len(), addr as *mut Address, flags, std::ptr::null_mut())  };

        if err != 0 {
            panic!("fi_av_insert failed {}", err);
        }
    }

    pub fn insertsvc(&self, node: &str, service: &str, addr: &mut Address, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_insertsvc(self.c_av, node.as_bytes().as_ptr() as *const i8, service.as_bytes().as_ptr() as *const i8, addr as *mut Address, flags, std::ptr::null_mut())  };

        if err != 0 {
            panic!("fi_av_insertvc failed {}", err);
        }
    }

    pub fn insertsym(&self, node: &str, nodecnt :usize, service: &str, svccnt: usize, addr: &mut Address, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_insertsym(self.c_av, node.as_bytes().as_ptr() as *const i8, nodecnt, service.as_bytes().as_ptr() as *const i8, svccnt, addr as *mut Address, flags, std::ptr::null_mut())  };

        if err != 0 {
            panic!("fi_av_insertsym failed {}", err);
        }
    }

    pub fn remove(&self, addr: &mut Address, count: usize, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_remove(self.c_av, addr as *mut Address, count, flags) };

        if err != 0 {
            panic!("fi_av_remove failed {}", err);
        }
    }

    pub fn lookup<T0>(&self, addr: Address, address: &mut [T0] ) -> usize {
        let mut addrlen : usize = 0;
        let addrlen_ptr: *mut usize = &mut addrlen;
        let err = unsafe { libfabric_sys::inlined_fi_av_lookup(self.c_av, addr, address.as_mut_ptr() as *mut std::ffi::c_void, addrlen_ptr) };
        
        if err != 0 {
            panic!("fi_av_lookup failed {}", err);
        }

        addrlen 
    }

    //[TODO]
    pub fn straddr<T0,T1>(&self, addr: &[T0], buf: &mut [T1]) -> &str {
        let mut strlen = buf.len();
        let strlen_ptr: *mut usize = &mut strlen;
        let straddr: *const i8 = unsafe { libfabric_sys::inlined_fi_av_straddr(self.c_av, addr.as_ptr() as *const std::ffi::c_void, buf.as_mut_ptr() as *mut std::ffi::c_char, strlen_ptr) };
        let str_addr = unsafe {std::ffi::CStr::from_ptr(straddr)};
        str_addr.to_str().unwrap()
    }

    pub fn avset<T0>(&self, attr:AvSetAttr , context: &mut T0) -> AvSet {
        AvSet::new(self, attr, context)
    }

}

impl FID for Av {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_av).fid }
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

    pub(crate) fn new_collective<T0>(ep: &crate::ep::Endpoint, addr: Address, set: &AvSet, flags: u64, ctx: &mut T0) -> Self {
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


pub struct AvSetAttr {
    c_attr: libfabric_sys::fi_av_set_attr,
}

impl AvSetAttr {

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_av_set_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_av_set_attr {
        &mut self.c_attr
    }    
}

pub struct AvSet {
    c_set : *mut libfabric_sys::fid_av_set,
}

impl AvSet {
    pub(crate) fn new<T0>(av: &Av, mut attr: AvSetAttr, context: &mut T0) -> Self {
        let mut c_set: *mut libfabric_sys::fid_av_set = std::ptr::null_mut();
        let c_set_ptr: *mut *mut libfabric_sys::fid_av_set = &mut c_set;

        let err = unsafe { libfabric_sys::inlined_fi_av_set(av.c_av, attr.get_mut(), c_set_ptr, context as *mut T0 as *mut std::ffi::c_void) };
        if err != 0 {
            panic!("fi_av_set failed {}", err);
        }

        Self { c_set }
    }
    
    pub fn union(&mut self, other: &AvSet) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_union(self.c_set, other.c_set) };

        if err != 0 {
            panic!("fi_av_set_union failed {}", err);
        }
    }
    pub fn intersect(&mut self, other: &AvSet) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_intersect(self.c_set, other.c_set) };

        if err != 0 {
            panic!("fi_av_set_intersect failed {}", err);
        }
    }
    pub fn diff(&mut self, other: &AvSet) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_diff(self.c_set, other.c_set) };

        if err != 0 {
            panic!("fi_av_set_diff failed {}", err);
        }
    }
    
    pub fn insert(&mut self, addr: Address) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_insert(self.c_set, addr) };

        if err != 0 {
            panic!("fi_av_set_insert failed {}", err);
        }
    }

    pub fn remove(&mut self, addr: Address) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_remove(self.c_set, addr) };

        if err != 0 {
            panic!("fi_av_set_remove failed {}", err);
        }
    }

    pub fn addr(&mut self) -> Address {
        let mut addr: Address = 0;
        let addr_ptr: *mut Address = &mut addr;
        let err = unsafe { libfabric_sys::inlined_fi_av_set_addr(self.c_set, addr_ptr) };

        if err != 0 {
            panic!("fi_av_set_addr failed {}", err);
        }

        addr
    }

}


pub struct PollFd {
    c_poll: libfabric_sys::pollfd,
}

impl PollFd {
    pub fn new() -> Self {
        let c_poll = libfabric_sys::pollfd{ fd: 0, events: 0, revents: 0 };
        Self { c_poll }
    }

    pub fn fd(&mut self, fd: i32) -> &mut Self {
        
        self.c_poll.fd = fd;
        self
    }

    pub fn events(&mut self, events: i16) -> &mut Self {
        
        self.c_poll.events = events;
        self
    }

    pub fn revents(&mut self, revents: i16) -> &mut Self {
        
        self.c_poll.revents = revents;
        self
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
    // for e in entries.iter() {
    //     println!("provider: {}", e.fabric_attr.prov_name);
    //     println!("  fabric: {}", e.fabric_attr.name);
    //     println!("  domain: {}", e.domain_attr.name);
    //     println!("  version: {}.{}", e.fabric_attr.prov_version.0, e.fabric_attr.prov_version.1 );
    // }

}

#[test]
fn ft_open_fabric_res() {
    let info = Info::all();
    let entries = info.get();
    
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(crate::eq::EqAttr::new());
    let mut domain = fab.domain(&entries[0]);
    domain.close();
    eq.close();
    fab.close();
}