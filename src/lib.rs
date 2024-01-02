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
pub mod error;
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

    pub fn is_rma_event(&self) -> bool {self.bitfield & libfabric_sys::FI_RMA_EVENT as u64 == libfabric_sys::FI_RMA_EVENT as u64 }
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

    pub fn hints(self, hints: &InfoHints) -> Self {
        InfoBuilder {
            c_info_hints: hints.c_info,
            ..self
        }
    }

    pub fn request(self) -> Result<Info, crate::error::Error> {
        let mut c_info: *mut libfabric_sys::fi_info = std::ptr::null_mut();
        let c_info_ptr: *mut *mut libfabric_sys::fi_info = &mut c_info;
        let node = if self.c_node.is_empty() { std::ptr::null_mut() } else { self.c_node.as_ptr() };
        let service = if self.c_service.is_empty() { std::ptr::null_mut() } else { self.c_service.as_ptr() };
        
        let err = unsafe{
            libfabric_sys::fi_getinfo(libfabric_sys::fi_version(), node, service, self.flags, self.c_info_hints, c_info_ptr)
        };

        if err != 0 {
            return Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) );
        }


        let mut entries = std::vec::Vec::new();
        entries.push(InfoEntry::new(c_info));
        unsafe {
            let mut curr = (*c_info).next;
            while  !curr.is_null() {
                entries.push(InfoEntry::new(curr));
                curr = (*curr).next;
            }
        }
        
        Ok(Info {
            entries,
            c_info,
        })
    }
}

#[derive(Clone)]
pub struct InfoEntry { 
    caps: InfoCaps,
    fabric_attr: crate::fabric::FabricAttr,
    domain_attr: crate::domain::DomainAttr,
    tx_attr: TxAttr,
    rx_attr: RxAttr,
    ep_attr: crate::ep::EndpointAttr,
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

    pub fn get_mode(&self) -> crate::enums::Mode {
        crate::enums::Mode::from_value(unsafe { (*self.c_info).mode })
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
        if c_info.is_null() {
            panic!("Failed to allocate memory");
        }
        Self { c_info }
    }

    // pub fn mode(mut self, mode: crate::enums::Mode) -> Self {
    //     unsafe { (*self.c_info).mode = mode.get_value() };

    //     self
    // }
    pub fn mode(mut self, mode: crate::enums::Mode) -> Self {
        unsafe { (*self.c_info).mode = mode.get_value()} ;

        self
    }

    pub fn addr_format(self, format: crate::enums::AddressFormat) -> Self {
        unsafe { (*self.c_info).addr_format = format.get_value() };

        self
    }

    pub fn ep_attr(self, attr: crate::ep::EndpointAttr) -> Self {
        unsafe { *(*self.c_info).ep_attr = *attr.get() };

        self
    }
    
    pub fn domain_attr(self, attr: crate::domain::DomainAttr) -> Self {
        unsafe { *(*self.c_info).domain_attr = *attr.get() };

        self
    }

    pub fn tx_attr(self, attr: crate::TxAttr) -> Self {
        unsafe { *(*self.c_info).tx_attr = *attr.get() };
        
        self
    }

    #[allow(unused_mut)]
    pub fn caps(mut self, caps: InfoCaps)  -> Self {
        unsafe { (*self.c_info).caps = caps.bitfield };
        
        self
    }

    pub fn no_src_address(mut self) -> Self { // [TODO]
        unsafe { (*self.c_info).src_addr = std::ptr::null_mut() };
        unsafe { (*self.c_info).src_addrlen = 0 };
        
        self
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

    pub fn new(iov: &[IoVec], desc: &mut impl DataDescriptor, addr: Address) -> Self {
        Msg {
            c_msg : libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.get_desc_ptr(),
                iov_count: iov.len(),
                addr,
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            }
        }
    }
}

pub struct MsgRma {
    c_msg_rma: libfabric_sys::fi_msg_rma,
}

impl MsgRma {
    pub fn new<T0>(iov: &[IoVec], desc: &mut impl DataDescriptor, addr: Address, rma_iov: &[RmaIoVec], context: &mut T0, data: u64) -> Self {
        Self {
            c_msg_rma : libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.get_desc_ptr(),
                iov_count: iov.len(),
                addr,
                rma_iov: rma_iov.as_ptr() as *const libfabric_sys::fi_rma_iov,
                rma_iov_count: rma_iov.len(),
                context: context as *mut T0 as *mut std::ffi::c_void,
                data,
            }
        }
    }
}

pub struct MsgTagged {
    c_msg_tagged: libfabric_sys::fi_msg_tagged,
}

impl MsgTagged {
    pub fn new(iov: &[IoVec], desc: &mut impl DataDescriptor, addr: Address, data: u64, tag: u64, ignore: u64) -> Self {
    
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.get_desc_ptr(),
                iov_count: iov.len(),
                addr,
                context: std::ptr::null_mut(), // [TODO]
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

    pub fn get_mode(&self) -> crate::enums::Mode {
        crate::enums::Mode::from_value(self.c_attr.mode)
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

#[derive(Clone)]
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

    pub fn get_mode(&self) -> crate::enums::Mode {
        crate::enums::Mode::from_value(self.c_attr.mode)
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
    pub(crate) fn new<T0>(domain: &crate::domain::Domain, mut attr: crate::TxAttr, context: &mut T0) -> Result<Stx, error::Error> {
        let mut c_stx: *mut libfabric_sys::fid_stx = std::ptr::null_mut();
        let c_stx_ptr: *mut *mut libfabric_sys::fid_stx = &mut c_stx;
        let err = unsafe { libfabric_sys::inlined_fi_stx_context(domain.c_domain, attr.get_mut(), c_stx_ptr, context as *mut T0 as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_stx }
            )
        }

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
            iov_len: std::mem::size_of_val(mem),
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
    pub(crate) fn new<T0>(ep: &crate::ep::Endpoint, addr: &T0, flags: u64) -> Result<Mc, error::Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join(ep.c_ep, addr as *const T0 as *const std::ffi::c_void, flags, c_mc_ptr, std::ptr::null_mut()) };

        if err != 0 {
            Err(error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_mc }
            )
        }

    }

    pub(crate) fn new_with_context<T0>(ep: &crate::ep::Endpoint, addr: &T0, flags: u64, ctx: &mut crate::Context) -> Result<Mc, error::Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join(ep.c_ep, addr as *const T0 as *const std::ffi::c_void, flags, c_mc_ptr, ctx.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_mc }
            )
        }

    }

    pub(crate) fn new_collective(ep: &crate::ep::Endpoint, addr: Address, set: &crate::av::AddressVectorSet, flags: u64) -> Result<Mc, crate::error::Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join_collective(ep.c_ep, addr, set.c_set, flags, c_mc_ptr, std::ptr::null_mut()) };

        if err != 0 {
            Err(error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_mc }
            )
        }
    }

    pub(crate) fn new_collective_with_context(ep: &crate::ep::Endpoint, addr: Address, set: &crate::av::AddressVectorSet, flags: u64, ctx: &mut crate::Context) -> Result<Mc, crate::error::Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join_collective(ep.c_ep, addr, set.c_set, flags, c_mc_ptr, ctx.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_mc }
            )
        }
    }

    pub fn addr(&self) -> Address {
        unsafe { libfabric_sys::inlined_fi_mc_addr(self.c_mc) }
    }
}

pub trait DataDescriptor {
    fn get_desc(&mut self) -> *mut std::ffi::c_void;
    fn get_desc_ptr(&mut self) -> *mut *mut std::ffi::c_void;
}

pub trait FID{
    fn fid(&self) -> *mut libfabric_sys::fid;
    
    fn setname<T>(&mut self, addr:&[T]) -> Result<(), error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_setname(self.fid(), addr.as_ptr() as *mut std::ffi::c_void, addr.len()) };
        
        if err != 0 {
            return Err(error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn getname<T0>(&self, addr: &mut[T0]) -> Result<usize, error::Error> {
        let mut len: usize = std::mem::size_of_val(addr);
        let len_ptr: *mut usize = &mut len;
        let err: i32 = unsafe { libfabric_sys::inlined_fi_getname(self.fid(), addr.as_mut_ptr() as *mut std::ffi::c_void, len_ptr) };

        if -err as u32  == libfabric_sys::FI_ETOOSMALL {
            Err(error::Error{ c_err: -err  as u32, kind: error::ErrorKind::TooSmall(len)} )
        }
        else if err < 0 {
            Err(error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(len)
        }
    }
    
    fn setopt<T0>(&mut self, level: i32, optname: i32, opt: &[T0]) -> Result<(), error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_setopt(self.fid(), level, optname, opt.as_ptr() as *const std::ffi::c_void, opt.len())};

        if err != 0 {
            return Err(error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn getopt<T0>(&self, level: i32, optname: i32, opt: &mut [T0]) -> Result<usize, error::Error> {
        let mut len = 0_usize;
        let len_ptr : *mut usize = &mut len;
        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.fid(), level, optname, opt.as_mut_ptr() as *mut std::ffi::c_void, len_ptr)};
        
        if -err as u32  == libfabric_sys::FI_ETOOSMALL {
            Err(error::Error{ c_err: -err  as u32, kind: error::ErrorKind::TooSmall(len)} )
        }
        else if err < 0 {
            Err(error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(len)
        }
    }

    fn close(self) -> Result<(), crate::error::Error> where Self: Sized {
        let err = unsafe { libfabric_sys::inlined_fi_close(self.fid()) };

        if err != 0 {
            return Err(error::Error::from_err_code((-err).try_into().unwrap()));
        }

        Ok(())
    }

    fn cancel(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cancel(self.fid(), std::ptr::null_mut()) };

        if err != 0 {
            return Err(error::Error::from_err_code((-err).try_into().unwrap()));
        }

        Ok(())
    }


    fn control<T0>(&self, opt: crate::enums::ControlOpt, arg: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_control(self.fid(), opt.get_value() as i32, arg as *mut T0 as *mut std::ffi::c_void) };
    
        if err != 0 {
            return Err(error::Error::from_err_code((-err).try_into().unwrap()));
        }

        Ok(())
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
    
    fn get_desc_ptr(&mut self) -> *mut *mut std::ffi::c_void {
        std::ptr::null_mut()
    }
}

pub struct Context {
    c_val: libfabric_sys::fi_context,
}

impl Context {
    pub fn new() -> Self {
        Self {
            c_val : {
                libfabric_sys::fi_context { internal: [std::ptr::null_mut(); 4] }
            }
        }
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_context {
        &mut self.c_val
    }
}

pub struct Context2 {
    c_val: libfabric_sys::fi_context2,
}

impl Context2 {
    pub fn new() -> Self {
        Self {
            c_val : {
                libfabric_sys::fi_context2 { internal: [std::ptr::null_mut(); 8] }
            }
        }
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_context2 {
        &mut self.c_val
    }
}