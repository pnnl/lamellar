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
pub mod cntr;
pub mod cq;
#[derive(Clone, Debug)]
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

    pub fn msg(self) -> Self  { Self { bitfield: self.bitfield | libfabric_sys::FI_MSG as u64 } }
    pub fn tagged(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_TAGGED as u64 } }
    pub fn rma(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_RMA as u64 } }
    pub fn atomic(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_ATOMIC as u64 } }
    pub fn collective(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_COLLECTIVE as u64 } }


    pub fn is_msg(&self) -> bool {self.bitfield & libfabric_sys::FI_MSG as u64 == libfabric_sys::FI_MSG as u64 }
    pub fn is_tagged(&self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_TAGGED as u64 }
    pub fn is_rma(&self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_RMA as u64 }
    pub fn is_atomic(&self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_ATOMIC as u64 }
    pub fn is_collective(&self) -> bool {self.bitfield & libfabric_sys::FI_COLLECTIVE as u64 == libfabric_sys::FI_COLLECTIVE as u64 }

    pub fn is_read(&self) -> bool {self.bitfield & libfabric_sys::FI_READ as u64 == libfabric_sys::FI_READ as u64 }
    pub fn is_write(&self) -> bool {self.bitfield & libfabric_sys::FI_WRITE as u64 == libfabric_sys::FI_WRITE as u64 }
    pub fn is_recv(&self) -> bool {self.bitfield & libfabric_sys::FI_RECV as u64 == libfabric_sys::FI_RECV as u64 }
    pub fn is_send(&self) -> bool {self.bitfield & libfabric_sys::FI_SEND as u64 == libfabric_sys::FI_SEND as u64 }
    pub fn is_transmit(&self) -> bool {self.bitfield & libfabric_sys::FI_TRANSMIT as u64 == libfabric_sys::FI_TRANSMIT as u64 }
    pub fn is_remote_read(&self) -> bool {self.bitfield & libfabric_sys::FI_REMOTE_READ as u64 == libfabric_sys::FI_REMOTE_READ as u64 }
    pub fn is_remote_write(&self) -> bool {self.bitfield & libfabric_sys::FI_REMOTE_WRITE as u64 == libfabric_sys::FI_REMOTE_WRITE as u64 }
}

pub struct Info {
    entries : std::vec::Vec<InfoEntry>,
    c_info: *mut  libfabric_sys::fi_info,
}

pub struct InfoBuilder {
    c_info_hints: *mut libfabric_sys::fi_info,
    c_node: std::ffi::CString,
    c_service: std::ffi::CString,
    flags: u64,
}

impl InfoBuilder {
    
    pub fn node(self, node: &str) -> Self {
        InfoBuilder {
            c_node: std::ffi::CString::new(node).unwrap(),
            ..self
        }
    }

    pub fn service(self, service: &str) -> Self {
        InfoBuilder {
            c_service: std::ffi::CString::new(service).unwrap(),
            ..self
        }
    }

    pub fn flags(self, flags: u64) -> Self {
        InfoBuilder {
            flags,
            ..self
        }
    }

    pub fn hints(self, hints: InfoHints) -> Self {
        InfoBuilder {
            c_info_hints: hints.c_info,
            ..self
        }
    }

    pub fn request(self) -> Info {
        let mut c_info: *mut libfabric_sys::fi_info = std::ptr::null_mut();
        let c_info_ptr: *mut *mut libfabric_sys::fi_info = &mut c_info;
        let node = if self.c_node.is_empty() { std::ptr::null_mut() } else { self.c_node.as_ptr() };
        let service = if self.c_service.is_empty() { std::ptr::null_mut() } else { self.c_service.as_ptr() };
        
        unsafe{
            let err = libfabric_sys::fi_getinfo(libfabric_sys::fi_version(), node, service, self.flags, self.c_info_hints, c_info_ptr);
            if err != 0 {
                panic!("fi_getinfo failed {} : {} \n", err, error_to_string(err.into()) ); // [TODO] Use Error()
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
}

#[derive(Clone, Debug)]
pub struct InfoEntry {
    pub caps: InfoCaps,
    pub fabric_attr: crate::fabric::FabricAttr,
    pub domain_attr: crate::domain::DomainAttr,
    pub tx_attr: TxAttr,
    pub rx_attr: RxAttr,
    c_info: *mut  libfabric_sys::fi_info,
}

impl InfoEntry {
    
    pub(crate) fn new(c_info: *mut  libfabric_sys::fi_info) -> Self {
        let mut fabric_attr = crate::fabric::FabricAttr::new();
            unsafe { *fabric_attr.get_mut() = *(*c_info).fabric_attr}
        let mut domain_attr = crate::domain::DomainAttr::new();
            unsafe { *domain_attr.get_mut() = *(*c_info).domain_attr}
        let tx_attr = TxAttr::from( unsafe {(*c_info).tx_attr } );
        let rx_attr = RxAttr::from( unsafe {(*c_info).rx_attr } );
        let caps: u64 = unsafe {(*c_info).caps};
        Self { caps: InfoCaps::from(caps) , fabric_attr, domain_attr, tx_attr, rx_attr, c_info }
    }

    pub fn get_dest_addr<T0>(&self) -> & T0 {
        unsafe { &*((*self.c_info).dest_addr as *const  usize as *const T0) as &T0}
    }

    pub fn get_src_addr<T0>(&self) -> & T0 {
        unsafe { &*((*self.c_info).src_addr as *const  usize as *const T0) as &T0}
    }

    pub fn get_mode(&self) -> u64 {
        unsafe { (*self.c_info).mode }
    }

    pub fn get_domain_attr(&self) -> &crate::domain::DomainAttr {
        &self.domain_attr
    }

    pub fn get_fabric_attr(&self) -> &crate::fabric::FabricAttr {
        &self.fabric_attr
    }

    pub fn get_tx_attr(&self) -> &TxAttr {
        &self.tx_attr
    }

    pub fn get_rx_attr(&self) -> &RxAttr {
        &self.rx_attr
    }

}

impl Info {

    pub fn new() -> InfoBuilder {
        InfoBuilder {
            c_info_hints: std::ptr::null_mut(),
            c_node: std::ffi::CString::new("").unwrap(),
            c_service: std::ffi::CString::new("").unwrap(),
            flags: 0,
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
        Self {  c_info }
    }

    pub fn addr_format(self, format: crate::enums::AddressFormat) -> Self {
        let c_info = self.c_info;
        unsafe { (*c_info).addr_format = format.get_value() };

        Self { c_info }
    }

    pub fn ep_attr(self, attr: crate::ep::EndpointAttr) -> Self {
        let c_info = self.c_info;
        unsafe { *(*c_info).ep_attr = *attr.get() };

        Self { c_info }
    }
    
    pub fn domain_attr(self, attr: crate::domain::DomainAttr) -> Self {
        let c_info = self.c_info;
        unsafe { *(*c_info).domain_attr = *attr.get() };
        
        Self { c_info }
    }

    pub fn tx_attr(self, attr: crate::TxAttr) -> Self {
        let c_info = self.c_info;
        unsafe { *(*c_info).tx_attr = *attr.get() };
        
        Self { c_info }

    }
    
    pub fn caps(self, caps: InfoCaps)  -> Self {
        let c_info = self.c_info;
        unsafe { (*self.c_info).caps = caps.bitfield };
        
        Self { c_info }
    }

    pub fn no_src_address(self) -> Self {
        let c_info = self.c_info;
        unsafe { (*self.c_info).src_addr = std::ptr::null_mut() };
        unsafe { (*self.c_info).src_addrlen = 0 };
        
        Self { c_info }
    }

    pub fn get_caps(&self) -> InfoCaps {
        InfoCaps::from(unsafe{ (*self.c_info).caps })
    }
}



pub type Address = libfabric_sys::fi_addr_t; 
pub type DataType = libfabric_sys::fi_datatype;
pub struct Msg {
    c_msg: libfabric_sys::fi_msg,
}

impl Msg {

    pub fn new<T0>(iov: &[IoVec], desc: &mut T0, addr: Address) -> Self {
        Msg {
            c_msg : libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: std::ptr::null_mut(),
                iov_count: iov.len(),
                addr,
                context: std::ptr::null_mut(),
                data: 0,
            }
        }
    }
}

pub struct MsgRma {
    c_msg_rma: libfabric_sys::fi_msg_rma,
}

impl MsgRma {
    pub fn new<T0,T1>(iov: &[IoVec], desc: &mut T0, addr: Address, rma_iov: &[RmaIoVec], context: &mut T1, data: u64) -> Self {
        Self {
            c_msg_rma : libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc as *mut T0 as *mut *mut std::ffi::c_void,
                iov_count: iov.len(),
                addr,
                rma_iov: rma_iov.as_ptr() as *const libfabric_sys::fi_rma_iov,
                rma_iov_count: rma_iov.len(),
                context: context as *mut T1 as *mut std::ffi::c_void,
                data,
            }
        }
    }
}

pub struct MsgTagged {
    c_msg_tagged: libfabric_sys::fi_msg_tagged,
}

impl MsgTagged {
    pub fn new<T0,T1>(iov: &[IoVec], desc: &mut T0, addr: Address, context: &mut T1, data: u64, tag: u64, ignore: u64) -> Self {
    
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc as *mut T0 as *mut *mut std::ffi::c_void,
                iov_count: iov.len(),
                addr,
                context: context as *mut T1 as *mut std::ffi::c_void,
                data,
                tag,
                ignore,
            }
        }
    }
}

pub struct MsgAtomic {
    c_msg_atomic: *mut libfabric_sys::fi_msg_atomic,
}


#[derive(Clone, Debug)]
pub struct TxAttr {
    c_attr: libfabric_sys::fi_tx_attr,
}

impl TxAttr {

    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_tx_attr {
            caps: 0,
            mode: 0,
            op_flags: 0,
            msg_order: 0,
            comp_order: 0,
            inject_size: 0,
            size: 0,
            iov_limit: 0,
            rma_iov_limit: 0,
            tclass: 0,
        };

        Self { c_attr }        
    }

    pub(crate) fn from(c_tx_attr_ptr: *mut libfabric_sys::fi_tx_attr) -> Self {
        let c_attr = unsafe { *c_tx_attr_ptr };

        Self { c_attr }
    }

    pub fn tclass(self, class: crate::enums::TClass) -> Self {
        let mut c_attr = self.c_attr;
        c_attr.tclass = class.get_value();

        Self { c_attr }
    }

    pub fn get_caps(&self) -> u64 {
        self.c_attr.caps
    }

    pub fn get_mode(&self) -> u64 {
        self.c_attr.mode
    }

    pub fn get_op_flags(&self) -> u64 {
        self.c_attr.op_flags
    }

    pub fn get_msg_order(&self) -> u64 {
        self.c_attr.msg_order
    }

    pub fn get_comp_order(&self) -> u64 {
        self.c_attr.comp_order
    }

    pub fn get_inject_size(&self) -> usize {
        self.c_attr.inject_size
    }

    pub fn get_size(&self) -> usize {
        self.c_attr.size
    }
    
    pub fn get_iov_limit(&self) -> usize {
        self.c_attr.iov_limit
    }

    pub fn get_rma_iov_limit(&self) -> usize {
        self.c_attr.rma_iov_limit
    }

    pub fn get_tclass(&self) -> u32 {
        self.c_attr.tclass
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_tx_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_tx_attr {
        &mut self.c_attr
    }
}

// pub struct fi_rx_attr {
//     pub caps: u64,
//     pub mode: u64,
//     pub op_flags: u64,
//     pub msg_order: u64,
//     pub comp_order: u64,
//     pub total_buffered_recv: usize,
//     pub size: usize,
//     pub iov_limit: usize,
// }
#[derive(Clone, Debug)]
pub struct RxAttr {
    c_attr: libfabric_sys::fi_rx_attr,
}


impl RxAttr {
    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_rx_attr {
            caps: 0,
            mode: 0,
            op_flags: 0,
            msg_order: 0,
            comp_order: 0,
            total_buffered_recv: 0,
            size: 0,
            iov_limit: 0,
        };

        Self { c_attr }
    }

    pub(crate) fn from(c_rx_attr: *mut libfabric_sys::fi_rx_attr) -> Self {
        let c_attr = unsafe { *c_rx_attr };

        Self { c_attr }
    }

    pub fn get_caps(&self) -> u64 {
        self.c_attr.caps
    }

    pub fn get_mode(&self) -> u64 {
        self.c_attr.mode
    }

    pub fn get_op_flags(&self) -> u64 {
        self.c_attr.op_flags
    }

    pub fn get_msg_order(&self) -> u64 {
        self.c_attr.msg_order
    }

    pub fn get_comp_order(&self) -> u64 {
        self.c_attr.comp_order
    }

    pub fn get_size(&self) -> usize {
        self.c_attr.size
    }

    pub fn get_iov_limit(&self) -> usize {
        self.c_attr.iov_limit
    }

    pub fn get_total_buffered_recv(&self) -> usize {
        self.c_attr.total_buffered_recv
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_rx_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_rx_attr {
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





pub fn rx_addr(addr: Address, rx_index: i32, rx_ctx_bits: i32) -> Address {
    unsafe { libfabric_sys::inlined_fi_rx_addr(addr, rx_index, rx_ctx_bits) }
}

#[repr(C)]
pub struct IoVec{
    c_iovec: libfabric_sys::iovec,
}

impl IoVec {

    pub fn new<T0>(mem: &mut [T0] ) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  mem.as_mut_ptr() as *mut std::ffi::c_void,
            iov_len: mem.len() * std::mem::size_of::<T0>(),
        };

        Self { c_iovec }
    }

    pub(crate) fn get(&self) ->  *const libfabric_sys::iovec {
        &self.c_iovec
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::iovec {
        &mut self.c_iovec
    }
}

#[repr(C)]
pub struct RmaIoVec {
    c_rma_iovec: libfabric_sys::fi_rma_iov,
}

impl RmaIoVec {
    pub fn new(addr: u64, len: usize, key: u64) -> Self {
        Self {
            c_rma_iovec: libfabric_sys::fi_rma_iov {
                addr,
                len,
                key,
            }
        }
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

    fn close(self) where Self: Sized {
        let err = unsafe { libfabric_sys::inlined_fi_close(self.fid()) };

        if err != 0 {
            panic!("fi_close failed {} : {}", err, error_to_string(err.into()));
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
    let info = Info::new().request();
    let _entries: Vec<InfoEntry> = info.get();
}

#[test]
fn ft_open_fabric_res() {
    let info = Info::new().request();
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

// To run the following tests do:
// 1. export FI_LOG="info" . 
// 2. Run the server (cargo test start_server -- --ignored --nocapture) 
//    There will be a large number of info printed. What we need is the last line with: listening on: fi_sockaddr_in:// <ip:port>
// 3. Copy the ip, port of the previous step
// 4. On the client_connect change  crate::Info::new().node("172.17.110.21").service("39426").hints(hints).request(). 
//    node() to the copied ip e.g. node(<ip>) and service() to port e.g. service(port)
// 5. Run client_connect
#[ignore]
#[test]
fn start_server() {

    let ep_attr = crate::ep::EndpointAttr::new()
        .ep_type(crate::enums::EndpointType::MSG);

    let dom_attr = crate::domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode((enums::MrType::PROV_KEY.get_value() | enums::MrType::ALLOCATED.get_value() | enums::MrType::VIRT_ADDR.get_value()  | enums::MrType::LOCAL.get_value() | enums::MrType::ENDPOINT.get_value()| enums::MrType::RAW.get_value()) as i32 );
    
    let caps = InfoCaps::new()
        .msg();
    

    let tx_attr = TxAttr::new()
        .tclass(crate::enums::TClass::LOW_LATENCY);

    let hints = crate::InfoHints::new()
        .ep_attr(ep_attr)
        .caps(caps)
        .domain_attr(dom_attr)
        .tx_attr(tx_attr)
        .addr_format(crate::enums::AddressFormat::UNSPEC);

    let info = crate::Info::new().node("127.0.0.1").service("42206").flags(libfabric_sys::FI_SOURCE).hints(hints).request();
    let entries: Vec<crate::InfoEntry> = info.get();
    
    if entries.len() == 0 {
        panic!("No entires in fi_info");
    }

    let fattr = entries[0].get_fabric_attr();
    
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(crate::eq::EventQueueAttr::new());
    
    let mut pep = fab.passive_ep(&entries[0]);
        pep.bind(&eq, 0);
        pep.listen();
    let mut event = 0;

    let mut eq_cm_entry = crate::eq::EventQueueCmEntry::new();
    let ret = eq.sread(&mut event, &mut eq_cm_entry, -1, 0);
    if ret != std::mem::size_of::<crate::eq::EventQueueCmEntry>().try_into().unwrap() {
        pep.close();
        eq.close();
        fab.close();
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::eq::EventQueueCmEntry>());
    }

    if event != libfabric_sys::FI_CONNREQ {
        panic!("Unexpected event value returned: {} vs {}", event, libfabric_sys::FI_CONNREQ);
    }

    let new_info = eq_cm_entry.get_info();
    let mut domain = fab.domain(&new_info);
    
    let mut txcq_attr =  crate::cq::CompletionQueueAttr::new();
        txcq_attr
        .format(enums::CqFormat::CONTEXT)
        // .wait_obj(crate::enums::WaitObj::NONE)
        .size(new_info.get_tx_attr().get_size() );
    
    let mut tx_cq = domain.cq_open(txcq_attr);
    
    let mut rxcq_attr =  crate::cq::CompletionQueueAttr::new();
        rxcq_attr
        .format(enums::CqFormat::CONTEXT)
        // .wait_obj(crate::enums::WaitObj::NONE)
        .size(new_info.get_rx_attr().get_size() );

    let mut rx_cq = domain.cq_open(rxcq_attr);
    let address2 = new_info.get_src_addr::<Address>();

    let mut ep = domain.ep(&new_info);
        ep.bind(&eq, 0);
        ep.bind(&tx_cq, libfabric_sys::FI_TRANSMIT.into());
        ep.bind(&rx_cq, libfabric_sys::FI_RECV.into());
        ep.enable();
        ep.accept();
    
    let mut event = 0;
    let mut eq_cm_entry = [crate::eq::EventQueueCmEntry::new()];
    
    let ret = eq.sread(&mut event, &mut eq_cm_entry, -1, 0);

    if ret != std::mem::size_of::<crate::eq::EventQueueCmEntry>().try_into().unwrap() {
        pep.close();
        ep.close();
        eq.close();
        fab.close();
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::eq::EventQueueCmEntry>());
    }
    
    if event != libfabric_sys::FI_CONNECTED {
        pep.close();
        ep.close();
        eq.close();
        fab.close();
        panic!("Unexpected event value returned: {} vs {}", event, libfabric_sys::FI_CONNREQ);
    }

    let mut buff: [usize; 4] = [0; 4];
    let mut buff2: [usize; 4] = [0; 4];

    let addr: u64 = (0 as u64).wrapping_sub(1);
    let mut buffer: [usize; 4] = [0; 4];
    let len = buff.len();
    ep.recv(std::slice::from_mut(&mut buff), len * std::mem::size_of::<usize>(), std::slice::from_mut(&mut buff2), addr);

    let flag: u64 = (0 as u64).wrapping_sub(1);
    let iov = IoVec::new(&mut buffer);
    let msg = Msg::new(std::slice::from_ref(&iov), &mut buffer, flag);
    ep.sendmsg(&msg, enums::TransferOptions::TRANSMIT_COMPLETE);
    let mut cq_err_entry = crate::cq::CqErrEntry::new();
    
    let ret = tx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    if ret < 0  {
        pep.close();
        eq.close();
        fab.close();
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }

    let ret = rx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    if ret < 0  {
        ep.close();
        eq.close();
        fab.close();
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }

    ep.shutdown(0);
    ep.close();
    pep.close();
    eq.close();
    tx_cq.close();
    rx_cq.close();
    domain.close();
    fab.close();
    
}

#[ignore]
#[test]
fn client_connect() {

    let ep_attr = crate::ep::EndpointAttr::new()
        .ep_type(crate::enums::EndpointType::MSG);

    let dom_attr = crate::domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode((enums::MrType::PROV_KEY.get_value() | enums::MrType::ALLOCATED.get_value() | enums::MrType::VIRT_ADDR.get_value()  | enums::MrType::LOCAL.get_value() | enums::MrType::ENDPOINT.get_value()| enums::MrType::RAW.get_value()) as i32 );

    let tx_attr = TxAttr::new()
        .tclass(crate::enums::TClass::LOW_LATENCY);

    let caps = InfoCaps::new()
        .msg();

    let hints = crate::InfoHints::new()
        .ep_attr(ep_attr)
        .domain_attr(dom_attr)
        .tx_attr(tx_attr)
        .caps(caps)
        .addr_format(crate::enums::AddressFormat::UNSPEC);

    let info: Info = crate::Info::new().node("172.17.110.21").service("35696").hints(hints).request();

    let entries: Vec<crate::InfoEntry> = info.get();

    if entries.len() == 0 {
        panic!("No entires in fi_info");
    }

    let fattr = entries[0].get_fabric_attr();

    let mut fab: fabric::Fabric = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let eq_attr = crate::eq::EventQueueAttr::new();
    let mut eq = fab.eq_open(eq_attr);

    let mut domain = fab.domain(&entries[0]);
    let mut txcq_attr =  crate::cq::CompletionQueueAttr::new();
        txcq_attr
        .format(enums::CqFormat::CONTEXT)
        .size(entries[0].get_tx_attr().get_size() );
    
    let mut tx_cq = domain.cq_open(txcq_attr);
    
    let mut rxcq_attr =  crate::cq::CompletionQueueAttr::new();
        rxcq_attr
        .format(enums::CqFormat::CONTEXT)
        .size(entries[0].get_rx_attr().get_size() );

    let mut rx_cq = domain.cq_open(rxcq_attr);
    let mut ep = domain.ep(&entries[0]);
        ep.bind(&eq, 0);
        ep.bind(&tx_cq, libfabric_sys::FI_TRANSMIT.into());
        ep.bind(&rx_cq, libfabric_sys::FI_RECV.into());
        ep.enable();

    let address2 = entries[0].get_dest_addr::<Address>();
    ep.connect(address2);
    
    
    let mut event = 0;
    let mut eq_cm_entry = eq::EventQueueCmEntry::new();
    
    let ret = eq.sread(&mut event, &mut eq_cm_entry, -1, 0);

    if ret != std::mem::size_of::<eq::EventQueueCmEntry>().try_into().unwrap() {
        ep.close();
        eq.close();
        fab.close();
        panic!("Size different {} vs {} {}", ret, std::mem::size_of::<eq::EventQueueCmEntry>(), error_to_string(ret as i64));
    }

    if event != libfabric_sys::FI_CONNECTED {
        ep.close();
        eq.close();
        fab.close();
        panic!("Unexpected event value returned: {} vs {}", event, libfabric_sys::FI_CONNREQ);
    }
    let mut buff: [usize; 4] = [0; 4];
    let mut buff2: [usize; 4] = [0; 4];

    let addr: u64 = (0 as u64).wrapping_sub(1);
    let len = buff.len();
    ep.recv(std::slice::from_mut(&mut buff), len * std::mem::size_of::<usize>(), std::slice::from_mut(&mut buff2), addr);
    let flag: u64 = (0 as u64).wrapping_sub(1);

    let mut buffer: [usize; 4] = [0; 4];
    let iov = IoVec::new(&mut buffer);
    let msg = Msg::new(std::slice::from_ref(&iov), &mut buffer, flag);
    ep.sendmsg(&msg, crate::enums::TransferOptions::TRANSMIT_COMPLETE);
    let mut cq_err_entry = crate::cq::CqErrEntry::new();
    
    let ret = tx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    if ret < 0 && -ret as u32 != libfabric_sys::FI_EAGAIN {
        ep.close();
        eq.close();
        fab.close();
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }
    
    let ret = rx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    if ret < 0 && -ret as u32 != libfabric_sys::FI_EAGAIN {
        ep.close();
        eq.close();
        fab.close();
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }

    ep.shutdown(0);
    ep.close();
    eq.close();
    tx_cq.close();
    rx_cq.close();
    domain.close();
    fab.close();
}