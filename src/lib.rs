// use libfabric_sys;
mod ep;
mod domain;
mod eq;
mod fabric;
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

        let fabric_attr = unsafe { FabricAttr::new((*c_info).fabric_attr)};
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

// impl Iterator for Info {
//     type Item = InfoEntry;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.curr.next()
//     }
// }

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

#[derive(Clone)]
pub struct DomainAttr {
    pub name: String,
    c_attr : *const libfabric_sys::fi_domain_attr,
}

impl DomainAttr {
    pub(crate) fn new(c_attr: *const libfabric_sys::fi_domain_attr) -> Self {
        let c_str_name = unsafe {(*c_attr).name};
        let name = unsafe {std::ffi::CStr::from_ptr(c_str_name)}.to_str().unwrap().to_owned();

        DomainAttr { name, c_attr }
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

pub struct Msg {
    c_msg: *mut libfabric_sys::fi_msg,
}

pub struct MsgRma {
    c_msg_rma: *mut libfabric_sys::fi_msg_rma,
}

pub struct MsgTagged {
    c_msg_tagged: *mut libfabric_sys::fi_msg_tagged,
}

pub struct IoVec{
    c_iovec: *mut libfabric_sys::iovec,
}

pub struct TxAttr {
    c_tx_attr: *mut libfabric_sys::fi_tx_attr,
}

impl TxAttr {
    pub(crate) fn new(c_tx_attr: *mut libfabric_sys::fi_tx_attr) -> Self {
        Self { c_tx_attr }
    }
}

// pub struct StxAttr {
//     c_stx_attr: *mut libfabric_sys::fid_stx_attr,
// }

// impl StxAttr {
//     pub(crate) fn new(c_stx_attr: *mut libfabric_sys::fid_stx_attr) -> Self {
//         Self { c_stx_attr }
//     }
// }

pub struct RxAttr {
    c_rx_attr: *mut libfabric_sys::fi_rx_attr,
}


impl RxAttr {
    pub(crate) fn new(c_rx_attr: *mut libfabric_sys::fi_rx_attr) -> Self {
        Self { c_rx_attr }
    }
}

// pub struct SrxAttr {
//     c_srx_attr: *mut libfabric_sys::fid_srx_attr,
// }


// impl SrxAttr {
//     pub(crate) fn new(c_srx_attr: *mut libfabric_sys::fid_srx_attr) -> Self {
//         Self { c_srx_attr }
//     }
// }

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
        let mut len_ptr : *mut usize = &mut len;
        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.fid(), level, optname, opt.as_mut_ptr() as *mut std::ffi::c_void, len_ptr)};
        if err != 0 {
            panic!("fi_getopt failed {}", err);
        }

        len
    }

    fn cancel(&self) {
        let _ = unsafe { libfabric_sys::inlined_fi_cancel(self.fid(), std::ptr::null_mut()) };
    }
}

#[test]
fn get_info(){
    let info = Info::all();
    let entries = info.get();
    for e in entries.iter() {
        println!("provider: {}", e.fabric_attr.prov_name);
        println!("  fabric: {}", e.fabric_attr.name);
        println!("  domain: {}", e.domain_attr.name);
        println!("  version: {}.{}", e.fabric_attr.prov_version.0, e.fabric_attr.prov_version.1 );
    }

}