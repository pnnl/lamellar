use core::panic;
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
    pub fn multicast(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_MULTICAST as u64 } }
    pub fn collective(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_COLLECTIVE as u64 } }

    pub fn is_msg(&self) -> bool {self.bitfield & libfabric_sys::FI_MSG as u64 == libfabric_sys::FI_MSG as u64 }
    pub fn is_tagged(&self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_TAGGED as u64 }
    pub fn is_rma(&self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_RMA as u64 }
    pub fn is_atomic(&self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_ATOMIC as u64 }
    pub fn is_multicast(&self) -> bool {self.bitfield & libfabric_sys::FI_COLLECTIVE as u64 == libfabric_sys::FI_COLLECTIVE as u64 }
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

#[derive(Clone)]
pub struct InfoEntry { // [TODO] Make fields private
    pub caps: InfoCaps,
    pub fabric_attr: crate::fabric::FabricAttr,
    pub domain_attr: crate::domain::DomainAttr,
    pub tx_attr: TxAttr,
    pub rx_attr: RxAttr,
    pub ep_attr: crate::ep::EndpointAttr,
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
        let ep_attr = crate::ep::EndpointAttr::from(unsafe {(*c_info).ep_attr});
        let caps: u64 = unsafe {(*c_info).caps};
        Self { caps: InfoCaps::from(caps) , fabric_attr, domain_attr, tx_attr, rx_attr, ep_attr, c_info }
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

    pub fn get_ep_attr(&self) -> &crate::ep::EndpointAttr {
        &self.ep_attr
    }

    pub fn get_caps(&self) -> &InfoCaps {
        &self.caps
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
        // unsafe { (*c_info).mode = !0 };
        Self {  c_info }
    }

    pub fn mode(self, mode: u64) -> Self {
        let c_info = self.c_info;
        unsafe { (*c_info).mode = mode };

        Self { c_info }
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

    pub fn get_ep_attr(&self) -> crate::ep::EndpointAttr {
        crate::ep::EndpointAttr::from(unsafe{ (*self.c_info).ep_attr })
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

    pub fn op_flags(self, tfer: crate::enums::TransferOptions) -> Self {
        let mut c_attr = self.c_attr;
        c_attr.op_flags = tfer.get_value().into();

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
#[derive(Clone)]
pub struct RmaIoVec {
    c_rma_iovec: libfabric_sys::fi_rma_iov,
}

impl RmaIoVec {
    pub fn new() -> Self {
        Self {
            c_rma_iovec: libfabric_sys::fi_rma_iov {
                addr: 0,
                len: 0,
                key: 0,
            }
        }
    }

    pub fn address(mut self, addr: u64) -> Self {
        self.c_rma_iovec.addr = addr;
        self
    }

    pub fn len(mut self, len: usize) -> Self {
        self.c_rma_iovec.len = len;
        self
    }

    pub fn key(mut self, key: u64) -> Self {
        self.c_rma_iovec.key = key;
        self
    }

    pub fn get_address(&self) -> u64 {
        self.c_rma_iovec.addr
    }
    
    pub fn get_len(&self) -> usize {
        self.c_rma_iovec.len
    }

    pub fn get_key(&self) -> u64 {
        self.c_rma_iovec.key
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

pub trait DataDescriptor {
    fn get_desc(&mut self) -> *mut std::ffi::c_void;
}

pub trait FID{
    fn fid(&self) -> *mut libfabric_sys::fid;
    
    fn setname<T>(&mut self, addr:&[T]) {
        let err = unsafe { libfabric_sys::inlined_fi_setname(self.fid(), addr.as_ptr() as *mut std::ffi::c_void, addr.len()) };
        
        if err != 0 {
            panic!("fi_setname failed {}", err);
        }
    }

    fn getname<T0>(&self, addr: &mut[T0]) -> usize {
        let mut len: usize = std::mem::size_of::<T0>() * addr.len();
        println!("Passing len = {} {} {}", len, addr.len(), std::mem::size_of::<T0>());
        let len_ptr: *mut usize = &mut len;
        let err = unsafe { libfabric_sys::inlined_fi_getname(self.fid(), addr.as_mut_ptr() as *mut std::ffi::c_void, len_ptr) };
        println!("Ret = {}", err);
        if err < 0 {
            panic!("fi_setname failed {}: {}", err, error_to_string(err.into()));
        }

        len
    }
    
    fn setopt<T0>(&mut self, level: i32, optname: i32, opt: &[T0]) {
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



// struct fi_param {
// 	const char *name;
// 	enum fi_param_type type;
// 	const char *help_string;
// 	const char *value;
// };

// int fi_getparams(struct fi_param **params, int *count);
// void fi_freeparams(struct fi_param *params);


// pub struct Param {
//     c_param : libfabric_sys::fi_param,
// }

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


pub fn error_to_string(errnum: i64) -> String {
    let ret = unsafe { libfabric_sys::fi_strerror(errnum as i32) };
    let str = unsafe { std::ffi::CStr::from_ptr(ret) };
    str.to_str().unwrap().to_string()
}

pub struct DefaultMemDesc {}

pub fn default_desc() -> DefaultMemDesc { DefaultMemDesc {  }}

impl DataDescriptor for DefaultMemDesc {
    fn get_desc(&mut self) -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }
}

struct TestsGlobalCtx {
    tx_size: usize,
    rx_size: usize,
    tx_mr_size: usize,
    rx_mr_size: usize,
    tx_seq: u64,
    rx_seq: u64,
    tx_cq_cntr: u64,
    rx_cq_cntr: u64,
    tx_buf_size: usize,
    rx_buf_size: usize,
    buf_size: usize,
    buf: Vec<u8>,
    tx_buf_index: usize,
    rx_buf_index: usize,
    max_msg_size: usize,
    remote_address: Address,
    ft_tag: u64, 
}

impl TestsGlobalCtx {
    fn new( ) -> Self {
        let mem = Vec::new();
        TestsGlobalCtx { tx_size: 0, rx_size: 0, tx_mr_size: 0, rx_mr_size: 0, tx_seq: 0, rx_seq: 0, tx_cq_cntr: 0, rx_cq_cntr: 0, tx_buf_size: 0, rx_buf_size: 0, buf_size: 0, buf: mem, tx_buf_index: 0, rx_buf_index: 0, max_msg_size: 0, remote_address: FI_ADDR_UNSPEC, ft_tag: 0 }
    }
}

#[allow(dead_code)]
fn ft_open_fabric_res(info: &InfoEntry) -> (crate::fabric::Fabric, crate::eq::EventQueue, crate::domain::Domain) {
    
    let fab = crate::fabric::Fabric::new(info.fabric_attr.clone());
    let eq = fab.eq_open(crate::eq::EventQueueAttr::new());
    let domain = fab.domain(&info);

    (fab, eq, domain)
}

#[allow(dead_code)]
fn ft_alloc_active_res(info: &InfoEntry, domain: &crate::domain::Domain) -> (crate::cq::CompletionQueue, crate::cq::CompletionQueue, crate::ep::Endpoint, Option<crate::av::AddressVector>) {
    
    let mut txcq_attr =  crate::cq::CompletionQueueAttr::new();
    let mut rxcq_attr =  crate::cq::CompletionQueueAttr::new();

    if info.get_caps().is_tagged() {

        txcq_attr.format(enums::CqFormat::TAGGED);
        rxcq_attr.format(enums::CqFormat::TAGGED);
    }
    else {
        txcq_attr.format(enums::CqFormat::CONTEXT);
        rxcq_attr.format(enums::CqFormat::CONTEXT);
    }
        // .wait_obj(crate::enums::WaitObj::NONE)
    txcq_attr.size(info.get_tx_attr().get_size() );//.wait_obj(crate::enums::WaitObj::NONE);
    rxcq_attr.size(info.get_tx_attr().get_size() );//.wait_obj(crate::enums::WaitObj::NONE);
    
    let tx_cq = domain.cq_open(txcq_attr);
    let rx_cq = domain.cq_open(rxcq_attr);


    let ep = domain.ep(&info);

    println!("{}", info.get_ep_attr().get_type().get_value());

    let av = match info.get_ep_attr().get_type() {
        crate::enums::EndpointType::RDM | crate::enums::EndpointType::DGRAM  => {
                let av_attr = match info.get_domain_attr().get_av_type() {
                    enums::AddressVectorType::UNSPEC => crate::av::AddressVectorAttr::new(),
                    _ => crate::av::AddressVectorAttr::new().avtype(info.get_domain_attr().get_av_type()),
                }.count(1);
                Option::Some(domain.av_open(av_attr))
            }
        _ => None,
    };

    (tx_cq, rx_cq, ep, av)
}

#[allow(dead_code)]
fn ft_enable_ep(info: &InfoEntry, ep: &crate::ep::Endpoint, tx_cq: &crate::cq::CompletionQueue, rx_cq: &crate::cq::CompletionQueue, eq: &crate::eq::EventQueue, av: &Option<crate::av::AddressVector>) {
    
    match info.get_ep_attr().get_type() {
        crate::enums::EndpointType::MSG => ep.bind(eq, 0),
        _ => if info.get_caps().is_collective() || info.get_caps().is_multicast() {
            ep.bind(eq, 0);
        }
    }

    match av {
        Some(av_val) => ep.bind(av_val, 0),
        _ => {}
    }

    ep.bind(tx_cq, libfabric_sys::FI_TRANSMIT.into());
    ep.bind(rx_cq, libfabric_sys::FI_RECV.into());
    ep.enable();
}

#[allow(dead_code)]
fn ft_complete_connect(eq: &crate::eq::EventQueue) { // [TODO] Do not panic, return errors
    
    let mut event = 0;
    let mut eq_cm_entry = [crate::eq::EventQueueCmEntry::new()];
    
    let ret = eq.sread(&mut event, &mut eq_cm_entry, -1, 0);

    if ret != std::mem::size_of::<crate::eq::EventQueueCmEntry>().try_into().unwrap() {
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::eq::EventQueueCmEntry>());
    }
    
    if event != libfabric_sys::FI_CONNECTED {
        panic!("Unexpected event value returned: {} vs {}", event, libfabric_sys::FI_CONNREQ);
    }
}

#[allow(dead_code)]
fn ft_accept_connection(ep: &crate::ep::Endpoint, eq: &crate::eq::EventQueue) {
    
    ep.accept();
    
    ft_complete_connect(eq);
}

#[allow(dead_code)]
fn ft_retrieve_conn_req(eq: &crate::eq::EventQueue) -> InfoEntry { // [TODO] Do not panic, return errors
    
    let mut event = 0;

    let mut eq_cm_entry = crate::eq::EventQueueCmEntry::new();
    let ret = eq.sread(&mut event, &mut eq_cm_entry, -1, 0);
    if ret != std::mem::size_of::<crate::eq::EventQueueCmEntry>().try_into().unwrap() {
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::eq::EventQueueCmEntry>());
    }

    if event != libfabric_sys::FI_CONNREQ {
        panic!("Unexpected event value returned: {} vs {}", event, libfabric_sys::FI_CONNREQ);
    }

    eq_cm_entry.get_info()
}

#[allow(dead_code)]
fn ft_server_connect(eq: &crate::eq::EventQueue, domain: &crate::domain::Domain) -> (crate::cq::CompletionQueue, crate::cq::CompletionQueue, crate::ep::Endpoint) {

    let new_info = ft_retrieve_conn_req(&eq);

    let (tx_cq, rx_cq, ep, _) = ft_alloc_active_res(&new_info, &domain);
    
    ft_enable_ep(&new_info, &ep, &tx_cq, &rx_cq, &eq, &None);

    ft_accept_connection(&ep, &eq);

    (tx_cq, rx_cq, ep)
}

#[allow(dead_code)]
fn ft_getinfo(hints: InfoHints, node: String, service: String, flags: u64) -> Info {
    let ep_attr = hints.get_ep_attr();

    let hints = match ep_attr.get_type() {
        crate::enums::EndpointType::UNSPEC => hints.ep_attr(ep_attr.ep_type(crate::enums::EndpointType::RDM)),
        _ => hints ,
    };

    crate::Info::new().node(node.as_str()).service(service.as_str()).flags(flags).hints(hints).request()
}

#[allow(dead_code)]
fn ft_connect_ep(ep: &crate::ep::Endpoint, eq: &crate::eq::EventQueue, addr: &Address) {
    
    ep.connect(addr);
    ft_complete_connect(eq);
}

// fn ft_av_insert<T0>(addr: T0, count: size, fi_addr: Address, flags: u64) {
//     a
// }


fn ft_rx_prefix_size(info: &InfoEntry) -> usize {

    if info.get_rx_attr().get_mode() & libfabric_sys::FI_MSG_PREFIX == libfabric_sys::FI_MSG_PREFIX{ // [TODO]
        info.get_ep_attr().get_max_msg_size()
    }
    else {
        0
    }
}

fn ft_tx_prefix_size(info: &InfoEntry) -> usize {

    if info.get_tx_attr().get_mode() & libfabric_sys::FI_MSG_PREFIX == libfabric_sys::FI_MSG_PREFIX{ // [TODO]
        info.get_ep_attr().get_max_msg_size()
    }
    else {
        0
    }
}
const WINDOW_SIZE : usize = 64;
const FT_MAX_CTRL_MSG : usize = 1024;
const FT_RMA_SYNC_MSG_BYTES : usize = 4;
const FI_ADDR_UNSPEC : Address = u64::MAX;

fn ft_set_tx_rx_sizes(info: &InfoEntry, tx_size: &mut usize, rx_size: &mut usize) {
    *tx_size = FT_MAX_CTRL_MSG * FT_MAX_CTRL_MSG;
    if *tx_size > info.get_ep_attr().get_max_msg_size() {
        *tx_size = info.get_ep_attr().get_max_msg_size();
    }
    println!("FT PREFIX = {}", ft_rx_prefix_size(&info));
    *rx_size = *tx_size + ft_rx_prefix_size(&info);
    *tx_size +=  ft_tx_prefix_size(&info);
}

fn ft_alloc_msgs(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, domain: &crate::domain::Domain, ep: &crate::ep::Endpoint) -> (crate::mr::MemoryRegion, crate::mr::MemoryRegionDesc) {

    let alignment: usize = 64;
    ft_set_tx_rx_sizes(info, &mut gl_ctx.tx_size, &mut gl_ctx.rx_size);
    gl_ctx.rx_buf_size = std::cmp::max(gl_ctx.rx_size, FT_MAX_CTRL_MSG) * WINDOW_SIZE;
    gl_ctx.tx_buf_size = std::cmp::max(gl_ctx.tx_size, FT_MAX_CTRL_MSG) * WINDOW_SIZE;


    let rma_resv_bytes = FT_RMA_SYNC_MSG_BYTES + std::cmp::max(ft_tx_prefix_size(info), ft_rx_prefix_size(info));
    gl_ctx.tx_buf_size += rma_resv_bytes;
    gl_ctx.rx_buf_size += rma_resv_bytes;

    gl_ctx.buf_size = gl_ctx.rx_buf_size + gl_ctx.tx_buf_size;

    gl_ctx.buf_size += alignment;
    gl_ctx.buf.resize(gl_ctx.buf_size, 0);
    println!("Buf size: {}", gl_ctx.buf_size);
    gl_ctx.max_msg_size = gl_ctx.tx_size;
    
    gl_ctx.rx_buf_index = 0;
    println!("rx_buf_index: {}", gl_ctx.rx_buf_index);
    gl_ctx.tx_buf_index = gl_ctx.rx_buf_size;
    println!("tx_buf_index: {}", gl_ctx.tx_buf_index);

    ft_reg_mr(info, domain, ep, &mut gl_ctx.buf, 0xC0DE)
}


fn ft_enable_ep_recv(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, ep: &crate::ep::Endpoint, domain: &crate::domain::Domain, tx_cq: &crate::cq::CompletionQueue, rx_cq: &crate::cq::CompletionQueue, eq: &crate::eq::EventQueue, av: &Option<crate::av::AddressVector>) -> (crate::mr::MemoryRegion, crate::mr::MemoryRegionDesc) {
    
    ft_enable_ep(info, ep, tx_cq, rx_cq, eq, av);

    let (mr, mut mr_desc) =  ft_alloc_msgs(info, gl_ctx, domain, ep);

    if info.get_caps().is_msg() || info.get_caps().is_tagged() {
        if info.get_caps().is_tagged() {
            ep.trecv(&mut gl_ctx.buf[gl_ctx.rx_buf_index.. gl_ctx.rx_buf_index + FT_MAX_CTRL_MSG], &mut mr_desc, gl_ctx.remote_address, gl_ctx.rx_seq, 0);
        }
        else {
            println!("Posting receive");
            let addr: u64 = (0 as u64).wrapping_sub(1);
            let ret = ep.recv(&mut gl_ctx.buf[gl_ctx.rx_buf_index.. gl_ctx.rx_buf_index + FT_MAX_CTRL_MSG], &mut mr_desc, gl_ctx.remote_address);
            // let  ret = ep.recv2(&mut gl_ctx.buf[gl_ctx.rx_buf_index.. gl_ctx.rx_buf_index + FT_MAX_CTRL_MSG], mr_desc, addr);
            println!("Returned {}", ret);
        }

        gl_ctx.rx_seq += 1;
    }

    (mr, mr_desc)
}

fn ft_init_fabric(hints: InfoHints, gl_ctx: &mut TestsGlobalCtx, node: String, service: String, flags: u64) -> (Info, crate::fabric::Fabric, crate::ep::Endpoint, crate::domain::Domain, crate::cq::CompletionQueue, crate::cq::CompletionQueue, crate::eq::EventQueue, crate::mr::MemoryRegion, crate::av::AddressVector, crate::mr::MemoryRegionDesc) {
    
    let info = ft_getinfo(hints, node, service.clone(), flags);
    let entries: Vec<crate::InfoEntry> = info.get();
    
    if entries.len() == 0 {
        panic!("No entires in fi_info");
    }
        
    let (fabric, eq, domain) = ft_open_fabric_res(&entries[0]);

    let (tx_cq, rx_cq, ep, av) =  ft_alloc_active_res(&entries[0], &domain);

    let (mr, mut mr_desc)  = ft_enable_ep_recv(&entries[0], gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &eq, &av);
    println!("Passed enable_ep_recv");
    let av = av.unwrap();
    ft_init_av(&entries[0], gl_ctx, &av , &ep, &tx_cq, &rx_cq, &mut mr_desc, service.is_empty());

    (info, fabric, ep, domain, tx_cq, rx_cq, eq, mr, av, mr_desc)
}

fn ft_av_insert<T>(av: &crate::av::AddressVector, addr: &T, fi_addr: &mut Address, flags: u64) {

    let ret = av.insert(std::slice::from_ref(addr), fi_addr, flags);
}

const NO_CQ_DATA: u64 = 0;


macro_rules!  ft_post{
    ($post_fn:ident, $prog_fn:ident, $cq:ident, $seq:expr, $cq_cntr:expr, $op_str:literal, $ep:ident, $( $x:ident),* ) => {
        loop {
            let ret = $ep.$post_fn($($x,)*);
            if ret == 0 {
                break;
            }
            else if -ret as u32 != libfabric_sys::FI_EAGAIN {
                panic!("{} returned error", stringify!(post_fn));
            }
            let rc = $prog_fn($cq, $seq, $cq_cntr);
            if rc != 0 && -rc as u32 != libfabric_sys::FI_EAGAIN {
                panic!("{} returned error", stringify!(prog_fn));
            }
        }
        $seq+=1;
    };
}
enum RmaOp {
    RMA_WRITE,
    RMA_WRITEDATA,
    RMA_READ,
}


fn ft_post_rma(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, rma_op: RmaOp, offset: usize, size: usize, remote: &RmaIoVec, ep: &crate::ep::Endpoint, fi_addr: Address, data: u64, data_desc: &mut impl crate::DataDescriptor, tx_cq: &crate::cq::CompletionQueue) {
    match rma_op {
        
        RmaOp::RMA_WRITE => {
            let addr = remote.get_address() + offset as u64;
            let key = remote.get_key();
            let buf = &gl_ctx.buf[gl_ctx.tx_buf_index+offset..gl_ctx.tx_buf_index+offset+size];
            ft_post!(write, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_write", ep, buf, data_desc, fi_addr, addr, key);
        }

        RmaOp::RMA_WRITEDATA => {
            todo!()
        }
        
        RmaOp::RMA_READ => {
            todo!()
        }
    }
}


fn ft_post_rma_inject(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, rma_op: RmaOp, offset: usize, size: usize, remote: &RmaIoVec, ep: &crate::ep::Endpoint, fi_addr: Address, data: u64, data_desc: &mut impl crate::DataDescriptor, tx_cq: &crate::cq::CompletionQueue) {
    match rma_op {
        
        RmaOp::RMA_WRITE => {
            let addr = remote.get_address() + offset as u64;
            let key = remote.get_key();
            let buf = &gl_ctx.buf[gl_ctx.tx_buf_index+offset..gl_ctx.tx_buf_index+offset+size];
            ft_post!(inject_write, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_inject_write", ep, buf, fi_addr, addr, key);
        }

        RmaOp::RMA_WRITEDATA => {
            todo!()
        }
        
        RmaOp::RMA_READ => {
            todo!()
        }
    }
}

fn ft_post_tx(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, ep: &crate::ep::Endpoint, fi_addr: Address, mut size: usize, data: u64, data_desc: &mut impl crate::DataDescriptor, tx_cq: &crate::cq::CompletionQueue) {
    
    size += ft_tx_prefix_size(info);
    let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index+size];
    if info.get_caps().is_tagged() {
        let op_tag = if gl_ctx.ft_tag != 0 {gl_ctx.ft_tag} else {gl_ctx.tx_seq};

        if data != NO_CQ_DATA {
            ft_post!(tsenddata, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "transmit", ep, buf, data_desc, data, fi_addr, op_tag);
        }
        else {
            ft_post!(tsend, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "transmit", ep, buf, data_desc, fi_addr, op_tag);
        }
    }
    else {
        if data != NO_CQ_DATA {
            ft_post!(senddata, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "transmit", ep, buf, data_desc, data, fi_addr);
        }
        else {
            ft_post!(send, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "transmit", ep, buf, data_desc, fi_addr);
        }
    }
}

fn ft_tx(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, ep: &crate::ep::Endpoint, fi_addr: Address, mut size: usize, data: u64, data_desc: &mut impl crate::DataDescriptor, tx_cq: &crate::cq::CompletionQueue) {

    ft_post_tx(info, gl_ctx, ep, fi_addr, size, data, data_desc, tx_cq);
    ft_get_tx_comp(&mut gl_ctx.tx_cq_cntr, tx_cq, gl_ctx.tx_seq);
}

fn ft_post_rx(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, ep: &crate::ep::Endpoint, fi_addr: Address, mut size: usize, data: u64, data_desc: &mut impl crate::DataDescriptor, rx_cq: &crate::cq::CompletionQueue) {
    size = std::cmp::max(size, FT_MAX_CTRL_MSG) +  ft_tx_prefix_size(info);
    let buf = &mut gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index+size];
    if info.get_caps().is_tagged() {
        let op_tag = if gl_ctx.ft_tag != 0 {gl_ctx.ft_tag} else {gl_ctx.rx_seq};
        let zero = 0;
        ft_post!(trecv, ft_progress, rx_cq, gl_ctx.rx_seq, &mut gl_ctx.rx_cq_cntr, "receive", ep, buf, data_desc, fi_addr, op_tag, zero);
    }
    else {
        ft_post!(recv, ft_progress, rx_cq, gl_ctx.rx_seq, &mut gl_ctx.rx_cq_cntr, "receive", ep, buf, data_desc, fi_addr);
    }
}

fn ft_rx(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, ep: &crate::ep::Endpoint, fi_addr: Address, mut size: usize, data: u64, data_desc: &mut impl crate::DataDescriptor, rx_cq: &crate::cq::CompletionQueue) {

    ft_get_rx_comp(&mut gl_ctx.rx_cq_cntr, rx_cq, gl_ctx.rx_seq);
    ft_post_rx(info, gl_ctx, ep, fi_addr, gl_ctx.rx_size, data, data_desc, rx_cq);
}

fn ft_progress(cq: &crate::cq::CompletionQueue, total: u64, cq_cntr: &mut u64) -> isize {
    let mut cq_err_entry = crate::cq::CqErrEntry::new();
    let ret = cq.read(std::slice::from_mut(&mut cq_err_entry), 1);
    if ret < 0 && -ret as u32 != libfabric_sys::FI_EAGAIN {
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }

    if ret > 0 {
        *cq_cntr += 1;
    }

    0
}

fn ft_init_av_dst_addr(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx,  av: &crate::av::AddressVector, ep: &crate::ep::Endpoint, tx_cq: &crate::cq::CompletionQueue, rx_cq: &crate::cq::CompletionQueue, mr_desc: &mut crate::mr::MemoryRegionDesc, server: bool) {
    let mut v = [0 as u8; FT_MAX_CTRL_MSG];
    if !server {
        ft_av_insert(av, info.get_dest_addr::<Address>(), &mut gl_ctx.remote_address, 0);
        let mut v2 = vec![0 as u8; 4];
        let len = ep.getname(&mut v);
        for el in v[0..len].iter() {
            print!("{}", el);
        }
        
        gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index+len].copy_from_slice(&v[0..len]);


        ft_tx(info, gl_ctx, ep, gl_ctx.remote_address, len, NO_CQ_DATA, mr_desc, tx_cq);
        ft_rx(info, gl_ctx, ep, gl_ctx.remote_address, 1, NO_CQ_DATA, mr_desc, rx_cq);
    }
    else {
        ft_get_rx_comp(&mut gl_ctx.rx_cq_cntr, rx_cq, gl_ctx.rx_seq);
        v.copy_from_slice(&gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index+FT_MAX_CTRL_MSG]);


        if matches!(info.get_domain_attr().get_av_type(), crate::enums::AddressVectorType::TABLE ) {
            let mut zero = 0;
            ft_av_insert(av, &v, &mut zero, 0);
        }
        else {
            ft_av_insert(av, &v, &mut gl_ctx.remote_address, 0);
        }

        ft_post_rx(info, gl_ctx, ep, gl_ctx.remote_address, 1, NO_CQ_DATA, mr_desc, rx_cq);
        
        if matches!(info.get_domain_attr().get_av_type(), crate::enums::AddressVectorType::TABLE) {
            gl_ctx.remote_address = 0;
        }
        ft_tx(info, gl_ctx, ep, gl_ctx.remote_address, 1, NO_CQ_DATA, mr_desc, tx_cq);
    }
}

fn ft_init_av(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, av: &crate::av::AddressVector, ep: &crate::ep::Endpoint, tx_cq: &crate::cq::CompletionQueue, rx_cq: &crate::cq::CompletionQueue, mr_desc: &mut crate::mr::MemoryRegionDesc, server: bool) {

    ft_init_av_dst_addr(info, gl_ctx, av, ep,  tx_cq, rx_cq, mr_desc, server);
}

fn ft_read_cq(cq: &crate::cq::CompletionQueue, curr: &mut u64, total: u64, timeout: i32, tag: u64) {

    let mut comp = crate::cq::CqErrEntry::new();

    while total - *curr > 0 {
        loop {

            let err = cq.read(std::slice::from_mut(&mut comp), 1);
            if err >= 0 {
                break;
            }
            else if -err as u32 != libfabric_sys::FI_EAGAIN { 
                let mut err_entry = crate::cq::CqErrEntry::new();
                let ret2 = cq.readerr(&mut err_entry, 0);
    
    
                println!("sread error: {} {}", ret2, cq.strerror(err_entry.get_prov_errno(), err_entry.get_err_data(), err_entry.get_err_data_size()));
                panic!("ERROR IN CQ_READ {}", err);
            }
        }
        *curr += 1;
    }
}

fn ft_get_rx_comp(rx_curr: &mut u64, rx_cq: &crate::cq::CompletionQueue, total: u64) {

    ft_read_cq(rx_cq, rx_curr, total, -1, 0);
}

fn ft_get_tx_comp(tx_curr: &mut u64, tx_cq: &crate::cq::CompletionQueue, total: u64) {

    ft_read_cq(tx_cq, tx_curr, total, -1, 0);
}

fn ft_need_mr_reg(info: &InfoEntry) -> bool {

    let res = if info.get_caps().is_rma() || info.get_caps().is_atomic() {
        true
    }
    else {
        if info.get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_LOCAL == libfabric_sys::FI_MR_LOCAL.try_into().unwrap() { // [TODO] Make this return enum
            true
        }
        else {
            false
        }
    };

    res
}

fn ft_reg_mr(info: &InfoEntry, domain: &crate::domain::Domain, ep: &crate::ep::Endpoint, buf: &mut [u8], key: u64) -> (crate::mr::MemoryRegion, crate::mr::MemoryRegionDesc) {

    let iov = IoVec::new(buf);
    let mut mr_attr = crate::mr::MemoryRegionAttr::new().iov(std::slice::from_ref(&iov)).requested_key(key).iface(crate::enums::HmemIface::SYSTEM);
    if (info.get_mode() & libfabric_sys::FI_LOCAL_MR)  == libfabric_sys::FI_LOCAL_MR  ||  
        (info.get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_LOCAL == libfabric_sys::FI_MR_LOCAL) { //[TODO]

        if info.get_caps().is_msg() || info.get_caps().is_tagged() {
            let mut temp = info.get_caps().is_send();
            if temp {
                mr_attr = mr_attr.access_send();
            }
            temp |= info.get_caps().is_recv();
            if temp {
                mr_attr = mr_attr.access_recv();
            }
            if !temp {
                mr_attr = mr_attr.access_send().access_recv();
            }
        }
    } 

    let mr = domain.mr_regattr(mr_attr, 0);
    let desc = mr.description();

    if info.get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_ENDPOINT == libfabric_sys::FI_MR_ENDPOINT {
        println!("MR ENDPOINT");
        mr.bind(ep, 0);
    }

    (mr,desc)

}

fn ft_sync(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, mr: &mut crate::mr::MemoryRegion, tx_cq: &crate::cq::CompletionQueue, rx_cq: &crate::cq::CompletionQueue, ep: &crate::ep::Endpoint, mr_desc: &mut crate::mr::MemoryRegionDesc) {
    ft_tx(info, gl_ctx, ep, gl_ctx.remote_address, 1, NO_CQ_DATA, mr_desc, tx_cq);
    ft_rx(info, gl_ctx, ep, gl_ctx.remote_address, 1, NO_CQ_DATA, mr_desc, rx_cq);
}

fn ft_exchange_keys(info: &InfoEntry, gl_ctx: &mut TestsGlobalCtx, mr: &mut crate::mr::MemoryRegion, tx_cq: &crate::cq::CompletionQueue, rx_cq: &crate::cq::CompletionQueue, domain: &crate::domain::Domain, ep: &crate::ep::Endpoint, mr_desc: &mut crate::mr::MemoryRegionDesc) -> RmaIoVec {
    println!("Exchangin keys");
    let mut addr = 0; 
    let mut key_size = 0;
    let mut len = 0;
    let mut rma_iov = RmaIoVec::new();
    
    if info.get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_RAW  == libfabric_sys::FI_MR_RAW { // [TODO] Use enums
        mr.raw_attr(&mut addr, &mut key_size, 0); // [TODO] Change this to return base_addr, key_size
        println!(" ======== Using RAW ===============");
    }
    len = std::mem::size_of::<RmaIoVec>();
    if key_size >= len - std::mem::size_of_val(&rma_iov.get_key()) {
        panic!("Key size does not fit");
    }
    // let mut addr;
    if info.get_domain_attr().get_mr_mode() as u32 == libfabric_sys::fi_mr_mode_FI_MR_BASIC  || info.get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_VIRT_ADDR == libfabric_sys::FI_MR_VIRT_ADDR { // [TODO]
        addr = gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + ft_rx_prefix_size(info)].as_mut_ptr() as u64;
        let addr_usize = gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + ft_rx_prefix_size(info)].as_mut_ptr() as usize;
        println!("ADDRESS = {:?}, {:x}, {:x}", gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + ft_rx_prefix_size(info)].as_mut_ptr(), addr, addr_usize);
        rma_iov = rma_iov.address(addr);
        println!("USING MR_BASIC || MR_VIRT_ADDR");
    }
    
    if info.get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_RAW == libfabric_sys::FI_MR_RAW {
        mr.raw_attr_with_key(&mut addr, &mut (rma_iov.get_key() as u8), &mut key_size, 0);
        println!("USING RAW ATTR");
    }
    else {
        rma_iov = rma_iov.key(mr.get_key());
        println!("USING KEY");
    }
    
    gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index+len].copy_from_slice(unsafe{ std::slice::from_raw_parts(&rma_iov as *const RmaIoVec as *const u8, std::mem::size_of::<RmaIoVec>())});
    
    println!("TX IOV");
    println!("addr: {}", rma_iov.get_address());
    println!("len: {}", rma_iov.get_len());
    println!("key: {}", rma_iov.get_key());
    ft_tx(info, gl_ctx, ep, gl_ctx.remote_address, len + ft_tx_prefix_size(info), NO_CQ_DATA, mr_desc, tx_cq);
    println!("DONE TX");
    
    println!("DONE RX");
    ft_get_rx_comp(&mut gl_ctx.rx_cq_cntr, rx_cq, gl_ctx.rx_seq);
    
    unsafe{ std::slice::from_raw_parts_mut(&mut rma_iov as *mut RmaIoVec as *mut u8,std::mem::size_of::<RmaIoVec>())}.copy_from_slice(&gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index+len]);
    let mut peer_iov = RmaIoVec::new();
    if info.get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_RAW == libfabric_sys::FI_MR_RAW {
        peer_iov = peer_iov.address(rma_iov.get_address());
        peer_iov = peer_iov.len(rma_iov.get_len());
        let mut key = 0;
        domain.map_raw(rma_iov.get_address(), &mut (rma_iov.get_key() as u8), key_size, &mut key, 0);
        peer_iov = peer_iov.key(key);
    }
    else {
        peer_iov = rma_iov.clone();
    }
    println!("POST RX");
    
    ft_post_rx(info, gl_ctx, ep, gl_ctx.remote_address, gl_ctx.rx_size, NO_CQ_DATA, mr_desc, rx_cq);
    
    ft_sync(info, gl_ctx, mr, tx_cq, rx_cq, ep, mr_desc);
    println!("DONE SYNC");

    println!("PEER IOV: ");
    println!("addr: {}", peer_iov.get_address());
    println!("len: {}", peer_iov.get_len());
    println!("key: {}", peer_iov.get_key());
    peer_iov
}

#[allow(dead_code)]
fn start_server(hints: InfoHints) -> (Info, fabric::Fabric,  crate::domain::Domain, crate::eq::EventQueue, crate::ep::PassiveEndpoint) {
   
   let info = ft_getinfo(hints, "127.0.0.1".to_owned(), "42206".to_owned(), libfabric_sys::FI_SOURCE);
   let entries: Vec<crate::InfoEntry> = info.get();
    
    if entries.len() == 0 {
        panic!("No entires in fi_info");
    }

    let (fab, eq, domain) = ft_open_fabric_res(&entries[0]);


    let pep = fab.passive_ep(&entries[0]);
        pep.bind(&eq, 0);
        pep.listen();


    (info, fab, domain, eq, pep)
}

#[allow(dead_code)]
fn client_connect(hints: InfoHints, node: String, service: String) -> (Info, fabric::Fabric,  domain::Domain, crate::eq::EventQueue, crate::cq::CompletionQueue, crate::cq::CompletionQueue, crate::ep::Endpoint) {
    let info = ft_getinfo(hints, node, service, 0);

    let entries: Vec<crate::InfoEntry> = info.get();

    if entries.len() == 0 {
        panic!("No entires in fi_info");
    }

    let (fab, eq, domain) = ft_open_fabric_res(&entries[0]);
    let (tx_cq, rx_cq, ep, _) = ft_alloc_active_res(&entries[0], &domain);
    ft_enable_ep(&entries[0], &ep, &tx_cq, &rx_cq, &eq, &None);
    ft_connect_ep(&ep, &eq, entries[0].get_dest_addr::<Address>());

    
    (info, fab, domain, eq, rx_cq, tx_cq, ep)
}

#[allow(dead_code)]
fn close_all_pep(fab: crate::fabric::Fabric, domain: crate::domain::Domain, eq :crate::eq::EventQueue, rx_cq: crate::cq::CompletionQueue, tx_cq: crate::cq::CompletionQueue, ep: crate::ep::Endpoint, pep: crate::ep::PassiveEndpoint) {
    ep.shutdown(0);
    ep.close();
    pep.close();
    eq.close();
    tx_cq.close();
    rx_cq.close();
    domain.close();
    fab.close();        
}

#[allow(dead_code)]
fn close_all(fab: crate::fabric::Fabric, domain: crate::domain::Domain, eq :crate::eq::EventQueue, rx_cq: crate::cq::CompletionQueue, tx_cq: crate::cq::CompletionQueue, ep: crate::ep::Endpoint, mr: Option<crate::mr::MemoryRegion>, av: Option<crate::av::AddressVector>) {
    
    println!("Closing Ep");
    ep.close();
    println!("Closing Eq");
    eq.close();
    println!("Closing tx_cq");
    tx_cq.close();
    println!("Closing rx_cq");
    rx_cq.close();
    match mr {
        Some(mr_val) => mr_val.close(),
        _ => {}
    }
    match av {
        Some(av_val) => av_val.close(),
        _ => {}
    }
    println!("Closing domain");
    domain.close();
    println!("Closing fabric");
    fab.close();    
}

// To run the following tests do:
// 1. export FI_LOG_LEVEL="info" . 
// 2. Run the server (e.g. cargo test pp_server_msg -- --ignored --nocapture) 
//    There will be a large number of info printed. What we need is the last line with: listening on: fi_sockaddr_in:// <ip:port>
// 3. Copy the ip, port of the previous step
// 4. On the client (e.g. pp_client_msg) change  client_connect node(<ip>) and service(<port>) to service and port of the copied ones
// 5. Run client (e.g. cargo test pp_client_msg -- --ignored --nocapture) 

#[ignore]
#[test]
fn pp_server_msg() {

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


    let (_info, fab, domain, eq, pep)    = start_server(hints);
    let (tx_cq, rx_cq, ep) = ft_server_connect(&eq, &domain);

    let mut buff: [usize; 4] = [0; 4];
    let mut buff2: [usize; 4] = [0; 4];

    let addr: u64 = (0 as u64).wrapping_sub(1);
    let mut buffer: [usize; 4] = [255; 4];
    let len = buff.len();
    ep.recv(&mut buff, &mut default_desc(), addr);

    let addr: u64 = (0 as u64).wrapping_sub(1);
    let iov = IoVec::new(&mut buffer);
    let msg = Msg::new(std::slice::from_ref(&iov), &mut buffer, addr);
    ep.sendmsg(&msg, enums::TransferOptions::TRANSMIT_COMPLETE);
    let mut cq_err_entry = crate::cq::CqErrEntry::new();

    let ret = tx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    if ret < 0  {
        close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }
    
    let ret = rx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    if ret < 0  {
        close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }

    println!("Server Received {:?}", buff);
    
    ep.shutdown(0);

    close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
}



#[ignore]
#[test]
fn pp_client_msg() {
    let mut gl_ctx = TestsGlobalCtx::new();

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

    let (_info, fab, domain, eq, rx_cq, tx_cq, ep) = client_connect(hints, "172.17.110.19".to_owned(), "44836".to_owned());
    let mut buff: [usize; 4] = [0; 4];
    let mut buff2: [usize; 4] = [0; 4];

    let addr: u64 = (0 as u64).wrapping_sub(1); // Address Unspecified
    let len = buff.len();
    ep.recv(&mut buff, &mut default_desc(), addr);
    let flag: u64 = (0 as u64).wrapping_sub(1);

    let mut buffer: [usize; 4] = [166; 4];
    let iov = IoVec::new(&mut buffer);
    let msg = Msg::new(std::slice::from_ref(&iov), &mut buffer, flag);
    ep.sendmsg(&msg, crate::enums::TransferOptions::TRANSMIT_COMPLETE);
    let mut cq_err_entry = crate::cq::CqErrEntry::new();

    let ret = tx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    if ret < 0 && -ret as u32 != libfabric_sys::FI_EAGAIN {
        close_all(fab, domain, eq, rx_cq, tx_cq, ep, None, None);
        
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }
    
    let ret = rx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    if ret < 0 && -ret as u32 != libfabric_sys::FI_EAGAIN {
        close_all(fab, domain, eq, rx_cq, tx_cq, ep, None, None);
        
        panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
    }
    println!("Client Received {:?}", buff);

    close_all(fab, domain, eq, rx_cq, tx_cq, ep, None, None);

}

#[test]
fn pp_server_rma() {
    let mut gl_ctx = TestsGlobalCtx::new();

    let dom_attr = crate::domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode((enums::MrType::PROV_KEY.get_value() | enums::MrType::ALLOCATED.get_value() | enums::MrType::VIRT_ADDR.get_value()  | enums::MrType::LOCAL.get_value() | enums::MrType::ENDPOINT.get_value()| enums::MrType::RAW.get_value()) as i32 )
        .resource_mgmt(enums::ResourceMgmt::ENABLED);
    
    let caps = InfoCaps::new().msg().rma();
    

    let tx_attr = TxAttr::new().tclass(crate::enums::TClass::BULK_DATA); //.op_flags(enums::TransferOptions::DELIVERY_COMPLETE);

    let hints = crate::InfoHints::new()
        .caps(caps)
        .tx_attr(tx_attr)
        .mode(libfabric_sys::FI_CONTEXT) // [TODO]
        .domain_attr(dom_attr)
        .addr_format(crate::enums::AddressFormat::UNSPEC);
    
    
    let (info, fabric, ep, domain, tx_cq, rx_cq, eq, mut mr, av, mut mr_desc) = ft_init_fabric(hints, &mut gl_ctx, "127.0.0.1".to_owned(), "".to_owned(), libfabric_sys::FI_SOURCE);

    let entries: Vec<crate::InfoEntry> = info.get();
    
    if entries.len() == 0 {
        panic!("No entires in fi_info");
    }
    let remote = ft_exchange_keys(&entries[0], &mut gl_ctx, &mut mr, &tx_cq, &rx_cq, &domain, &ep, &mut mr_desc);
    let offset = FT_RMA_SYNC_MSG_BYTES + std::cmp::max(ft_tx_prefix_size(&entries[0]), ft_rx_prefix_size(&entries[0]));
    let to_send = [1 as u8; 512];
    gl_ctx.buf[gl_ctx.tx_buf_index+ offset..gl_ctx.tx_buf_index+ offset+to_send.len()].copy_from_slice(&to_send);
    let buff = & gl_ctx.buf[gl_ctx.tx_buf_index+ offset..];
    let fi_remote_address = gl_ctx.remote_address;
    ft_post_rma(&entries[0], &mut gl_ctx, RmaOp::RMA_WRITE, offset, to_send.len(), &remote, &ep, fi_remote_address, NO_CQ_DATA, &mut mr_desc, &tx_cq);
    ft_get_tx_comp(&mut gl_ctx.rx_cq_cntr, &tx_cq, gl_ctx.tx_seq);
    println!("{:?}", &gl_ctx.buf[gl_ctx.rx_buf_index+offset..gl_ctx.rx_buf_index+offset+to_send.len()]);

    // let mut buff: [usize; 4] = [0; 4];
    // let mut buff2: [usize; 4] = [0; 4];
    
    // let addr: u64 = (0 as u64).wrapping_sub(1);
    // let mut buffer: [usize; 4] = [255; 4];
    // let len = buff.len();
    // ep.recv(std::slice::from_mut(&mut buff), len * std::mem::size_of::<usize>(), std::slice::from_mut(&mut buff2), addr);
    
    // let addr: u64 = (0 as u64).wrapping_sub(1);
    // // let iov = IoVec::new(&mut buffer);
    // // let msg = Msg::new(std::slice::from_ref(&iov), &mut buffer, addr);
    // // ep.sendmsg(&msg, enums::TransferOptions::TRANSMIT_COMPLETE);
    // ep.senddata(&mut buffer, std::mem::size_of::<usize>() * 4, &mut buff2, 0, addr);
    // let mut cq_err_entry = crate::cq::CqErrEntry::new();
    
    // let ret = tx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    // if ret < 0  {
        //     close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
        //     panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
        // }
        
        // let ret = rx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
        // if ret < 0  {
            //     close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
            //     panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
            // }
            
            // println!("Server Received {:?}", buff);
            
    close_all(fabric, domain, eq, rx_cq, tx_cq, ep, mr.into(),av.into());
    // close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
}

#[test]
fn pp_client_rma() {
    let mut gl_ctx = TestsGlobalCtx::new();
    let dom_attr = crate::domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode((enums::MrType::PROV_KEY.get_value() | enums::MrType::ALLOCATED.get_value() | enums::MrType::VIRT_ADDR.get_value()  | enums::MrType::LOCAL.get_value() | enums::MrType::ENDPOINT.get_value()| enums::MrType::RAW.get_value()) as i32 )
        .resource_mgmt(enums::ResourceMgmt::ENABLED);
    
    let caps = InfoCaps::new().msg().rma();
    

    let tx_attr = TxAttr::new().tclass(crate::enums::TClass::BULK_DATA);//.op_flags(enums::TransferOptions::DELIVERY_COMPLETE);

    let hints = crate::InfoHints::new()
        .caps(caps)
        .tx_attr(tx_attr)
        .mode(libfabric_sys::FI_CONTEXT) // [TODO]
        .domain_attr(dom_attr)
        .addr_format(crate::enums::AddressFormat::UNSPEC);
    
    
    let (info, fabric, ep, domain, tx_cq, rx_cq, eq, mut mr, av, mut mr_desc) = 
        ft_init_fabric(hints, &mut gl_ctx, "172.17.110.6".to_owned(), "39822".to_owned(), 0);
    let entries: Vec<crate::InfoEntry> = info.get();
    
    if entries.len() == 0 {
        panic!("No entires in fi_info");
    }
    let remote = ft_exchange_keys(&entries[0], &mut gl_ctx, &mut mr, &tx_cq, &rx_cq, &domain, &ep, &mut mr_desc);
    let offset = FT_RMA_SYNC_MSG_BYTES + std::cmp::max(ft_tx_prefix_size(&entries[0]), ft_rx_prefix_size(&entries[0]));
    let to_send = [2 as u8; 512];

    gl_ctx.buf[gl_ctx.tx_buf_index+ offset..gl_ctx.tx_buf_index+ offset+to_send.len()].copy_from_slice(&to_send);
    let buff = & gl_ctx.buf[gl_ctx.tx_buf_index+ offset..];
    let fi_remote_address = gl_ctx.remote_address;
    ft_post_rma(&entries[0], &mut gl_ctx, RmaOp::RMA_WRITE, offset, to_send.len(), &remote, &ep, fi_remote_address, NO_CQ_DATA, &mut mr_desc, &tx_cq);
    ft_get_tx_comp(&mut gl_ctx.rx_cq_cntr, &tx_cq, gl_ctx.tx_seq);
    println!("{:?}", &gl_ctx.buf[gl_ctx.rx_buf_index+offset..gl_ctx.rx_buf_index+offset+to_send.len()]);
    // let mut buff: [usize; 4] = [0; 4];
    // let mut buff2: [usize; 4] = [0; 4];
    
    // let addr: u64 = (0 as u64).wrapping_sub(1);
    // let mut buffer: [usize; 4] = [255; 4];
    // let len = buff.len();
    // ep.recv(std::slice::from_mut(&mut buff), len * std::mem::size_of::<usize>(), std::slice::from_mut(&mut buff2), addr);
    
    // let addr: u64 = (0 as u64).wrapping_sub(1);
    // // let iov = IoVec::new(&mut buffer);
    // // let msg = Msg::new(std::slice::from_ref(&iov), &mut buffer, addr);
    // // ep.sendmsg(&msg, enums::TransferOptions::TRANSMIT_COMPLETE);
    // ep.senddata(&mut buffer, std::mem::size_of::<usize>() * 4, &mut buff2, 0, addr);
    // let mut cq_err_entry = crate::cq::CqErrEntry::new();
    
    // let ret = tx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
    // if ret < 0  {
        //     close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
        //     panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
        // }
        
        // let ret = rx_cq.sread(std::slice::from_mut(&mut cq_err_entry), 1, -1);
        // if ret < 0  {
            //     close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
            //     panic!("Size different {} vs {}", ret, std::mem::size_of::<crate::cq::CqErrEntry>());
            // }
            
            // println!("Server Received {:?}", buff);
            
    close_all(fabric, domain, eq, rx_cq, tx_cq, ep, mr.into(), av.into());
    // close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep);
}