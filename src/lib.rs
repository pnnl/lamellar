use core::panic;
use std::{marker::PhantomData, rc::Rc, path::Display};

use infocapsoptions::{Capabilities, NewInfoCaps};

// use ep::ActiveEndpoint;
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
pub mod comm;
pub mod error;
pub mod xcontext;
pub mod eqoptions;
pub mod cqoptions;
pub mod cntroptions;
pub mod infocapsoptions;
const FI_ADDR_NOTAVAIL : u64 = u64::MAX;




#[derive(Clone, Debug)]
pub struct InfoCaps {
    pub(crate) bitfield: u64,
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
    pub fn named_rx_ctx(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_NAMED_RX_CTX } }
    pub fn directed_recv(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_DIRECTED_RECV } }
    pub fn variable_msg(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_VARIABLE_MSG } }
    pub fn hmem(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_HMEM } }
    pub fn collective(self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_COLLECTIVE as u64 } }
    
    pub fn msg_if(self, cond: bool) -> Self  { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_MSG as u64} else { self.bitfield } } }
    pub fn tagged_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_TAGGED as u64} else { self.bitfield } } }
    pub fn rma_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_RMA as u64} else { self.bitfield } } }
    pub fn atomic_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_ATOMIC as u64} else { self.bitfield } } }
    pub fn multicast_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_MULTICAST as u64} else { self.bitfield } } }
    pub fn named_rx_ctx_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_NAMED_RX_CTX} else { self.bitfield } } }
    pub fn directed_recv_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_DIRECTED_RECV} else { self.bitfield } } }
    pub fn variable_msg_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_VARIABLE_MSG} else { self.bitfield } } }
    pub fn hmem_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_HMEM} else { self.bitfield } } }
    pub fn collective_if(self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_COLLECTIVE as u64} else { self.bitfield } } }

    pub fn read(&self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_READ as u64 }}
    pub fn write(&self) -> Self { Self { bitfield: self.bitfield |  libfabric_sys::FI_WRITE as u64 }}
    pub fn recv(&self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_RECV as u64 }}
    pub fn send(&self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_SEND as u64 }}
    pub fn transmit(&self) -> Self { Self { bitfield: self.bitfield |  libfabric_sys::FI_TRANSMIT as u64 }}
    pub fn remote_read(&self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_REMOTE_READ as u64 }}
    pub fn remote_write(&self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_REMOTE_WRITE as u64 }}

    pub fn rma_event(&self) -> Self { Self { bitfield: self.bitfield | libfabric_sys::FI_RMA_EVENT }}

    
    pub fn is_msg(&self) -> bool {self.bitfield & libfabric_sys::FI_MSG as u64 == libfabric_sys::FI_MSG as u64 }
    pub fn is_tagged(&self) -> bool {self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_TAGGED as u64 }
    pub fn is_rma(&self) -> bool {self.bitfield & libfabric_sys::FI_RMA as u64 == libfabric_sys::FI_RMA as u64 }
    pub fn is_atomic(&self) -> bool {self.bitfield & libfabric_sys::FI_ATOMIC as u64 == libfabric_sys::FI_ATOMIC as u64 }
    pub fn is_multicast(&self) -> bool {self.bitfield & libfabric_sys::FI_MULTICAST as u64 == libfabric_sys::FI_MULTICAST as u64 }
    pub fn is_named_rx_ctx(self) -> bool {self.bitfield & libfabric_sys::FI_NAMED_RX_CTX == libfabric_sys::FI_NAMED_RX_CTX} 
    pub fn is_directed_recv(self) -> bool {self.bitfield & libfabric_sys::FI_DIRECTED_RECV == libfabric_sys::FI_DIRECTED_RECV} 
    pub fn is_variable_msg(self) -> bool {self.bitfield & libfabric_sys::FI_VARIABLE_MSG == libfabric_sys::FI_VARIABLE_MSG} 
    pub fn is_hmem(self) -> bool {self.bitfield & libfabric_sys::FI_HMEM == libfabric_sys::FI_HMEM} 
    pub fn is_collective(&self) -> bool {self.bitfield & libfabric_sys::FI_COLLECTIVE as u64 == libfabric_sys::FI_COLLECTIVE as u64 }

    pub fn is_read(&self) -> bool {self.bitfield & libfabric_sys::FI_READ as u64 == libfabric_sys::FI_READ as u64 }
    pub fn is_write(&self) -> bool {self.bitfield & libfabric_sys::FI_WRITE as u64 == libfabric_sys::FI_WRITE as u64 }
    pub fn is_recv(&self) -> bool {self.bitfield & libfabric_sys::FI_RECV as u64 == libfabric_sys::FI_RECV as u64 }
    pub fn is_send(&self) -> bool {self.bitfield & libfabric_sys::FI_SEND as u64 == libfabric_sys::FI_SEND as u64 }
    pub fn is_transmit(&self) -> bool {self.bitfield & libfabric_sys::FI_TRANSMIT as u64 == libfabric_sys::FI_TRANSMIT as u64 }
    pub fn is_remote_read(&self) -> bool {self.bitfield & libfabric_sys::FI_REMOTE_READ as u64 == libfabric_sys::FI_REMOTE_READ as u64 }
    pub fn is_remote_write(&self) -> bool {self.bitfield & libfabric_sys::FI_REMOTE_WRITE as u64 == libfabric_sys::FI_REMOTE_WRITE as u64 }

    pub fn is_rma_event(&self) -> bool {self.bitfield & libfabric_sys::FI_RMA_EVENT == libfabric_sys::FI_RMA_EVENT }
}

impl Default for InfoCaps {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Info<T> {
    entries : std::vec::Vec<InfoEntry<T>>,
    c_info: *mut  libfabric_sys::fi_info,
}

pub struct InfoBuilder<T> {
    c_info_hints: *mut libfabric_sys::fi_info,
    c_node: std::ffi::CString,
    c_service: std::ffi::CString,
    flags: u64,
    phantom: PhantomData<T>,
}

impl<T> InfoBuilder<T> {
    
    pub fn node(self, node: &str) -> Self {
        Self {
            c_node: std::ffi::CString::new(node).unwrap(),
            ..self
        }
    }

    pub fn service(self, service: &str) -> Self {
        Self {
            c_service: std::ffi::CString::new(service).unwrap(),
            ..self
        }
    }

    pub fn flags(self, flags: u64) -> Self {
        Self {
            flags,
            ..self
        }
    }

    pub fn request(self) -> Result<Info<T>, crate::error::Error> {
        let mut c_info: *mut libfabric_sys::fi_info = std::ptr::null_mut();
        let c_info_ptr: *mut *mut libfabric_sys::fi_info = &mut c_info;
        let node = if self.c_node.is_empty() { std::ptr::null_mut() } else { self.c_node.as_ptr() };
        let service = if self.c_service.is_empty() { std::ptr::null_mut() } else { self.c_service.as_ptr() };
        
        let err = unsafe{
            libfabric_sys::fi_getinfo(libfabric_sys::fi_version(), node, service, self.flags, self.c_info_hints, c_info_ptr)
        };

        check_error(err.try_into().unwrap())?;


        let mut entries = std::vec::Vec::new();
        if !c_info.is_null() {
            entries.push(InfoEntry::new(c_info));
        }
        unsafe {
            let mut curr = (*c_info).next;
            while  !curr.is_null() {
                entries.push(InfoEntry::new(curr));
                curr = (*curr).next;
            }
        }
        
        Ok(Info::<T> {
            entries,
            c_info,
        })
    }
}

impl InfoBuilder<()> {
    
    pub fn hints<T>(self, hints: &InfoHints<T>) -> InfoBuilder<T> {
        InfoBuilder::<T> {
            c_info_hints: hints.c_info,
            phantom: PhantomData,
            c_node: self.c_node,
            c_service: self.c_service,
            flags: self.flags,
        }
    }
}

#[derive(Clone)]
pub struct InfoEntry<T> { 
    caps: InfoCaps,
    fabric_attr: crate::fabric::FabricAttr,
    domain_attr: crate::domain::DomainAttr,
    tx_attr: crate::xcontext::TxAttr,
    rx_attr: crate::xcontext::RxAttr,
    ep_attr: crate::ep::EndpointAttr,
    nic: Option<Nic>,
    c_info: *mut  libfabric_sys::fi_info,
    phantom: PhantomData<T>
}

impl<T> InfoEntry<T> {
    
    pub(crate) fn new(c_info: *mut  libfabric_sys::fi_info) -> Self {
        let mut fabric_attr = crate::fabric::FabricAttr::new();
            unsafe { *fabric_attr.get_mut() = *(*c_info).fabric_attr}
        let mut domain_attr = crate::domain::DomainAttr::new();
            unsafe { *domain_attr.get_mut() = *(*c_info).domain_attr}
        let tx_attr = crate::xcontext::TxAttr::from( unsafe {(*c_info).tx_attr } );
        let rx_attr = crate::xcontext::RxAttr::from( unsafe {(*c_info).rx_attr } );
        let ep_attr = crate::ep::EndpointAttr::from(unsafe {(*c_info).ep_attr});
        let caps: u64 = unsafe {(*c_info).caps};
        let nic = if ! unsafe{ (*c_info).nic.is_null()} {Some(Nic::from_attr(unsafe{*(*c_info).nic})) } else {None};
        Self { caps: InfoCaps::from(caps) , fabric_attr, domain_attr, tx_attr, rx_attr, ep_attr, nic, c_info, phantom: PhantomData }
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

    pub fn get_tx_attr(&self) -> &crate::xcontext::TxAttr {
        &self.tx_attr
    }

    pub fn get_rx_attr(&self) -> &crate::xcontext::RxAttr {
        &self.rx_attr
    }

    pub fn get_ep_attr(&self) -> &crate::ep::EndpointAttr {
        &self.ep_attr
    }

    pub fn get_caps(&self) -> &InfoCaps {
        &self.caps
    }

    pub fn get_nic(&self) -> Option<Nic> {
        self.nic.clone()
    }

}

impl<T> std::fmt::Debug for InfoEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c_str = unsafe{libfabric_sys::fi_tostr(self.c_info.cast(), libfabric_sys::fi_type_FI_TYPE_INFO)};
        if c_str.is_null() {
            panic!("String is null")
        }
        let val = unsafe{std::ffi::CStr::from_ptr(c_str)};
        
        write!(f, "{}", val.to_str().unwrap())
    }
}


impl Info<()> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> InfoBuilder<()> {
        InfoBuilder::<()> {
            c_info_hints: std::ptr::null_mut(),
            c_node: std::ffi::CString::new("").unwrap(),
            c_service: std::ffi::CString::new("").unwrap(),
            flags: 0,
            phantom: PhantomData,
        }
    }

}
impl<T> Info<T> {

    pub fn get(&self) -> &Vec<InfoEntry<T>> {
        &self.entries
    }
}

impl<T> Drop for Info<T> {
    
    fn drop(&mut self) {
        unsafe {
            libfabric_sys::fi_freeinfo(self.c_info);
        }
    }
}

#[derive(Clone)]
pub  struct InfoHints<T> {
    c_info: *mut libfabric_sys::fi_info,
    phantom: PhantomData<T>
}

impl InfoHints<()> {
    pub fn new() -> Self {
        let c_info = unsafe { libfabric_sys::inlined_fi_allocinfo() };
        if c_info.is_null() {
            panic!("Failed to allocate memory");
        }
        Self { c_info, phantom: PhantomData }
    }


    #[allow(unused_mut)]
    pub fn caps<T: Capabilities>(mut self, _caps: T)  -> InfoHints<T> {
        unsafe { (*self.c_info).caps = T::get_bitfield() };
        
        InfoHints::<T> {
            c_info: self.c_info,
            phantom: PhantomData,
        }
    }
}

impl<T: Capabilities> Capabilities for InfoHints<T> {
    fn get_bitfield() -> u64 {
        T::get_bitfield()
    }
    
    fn is_msg() -> bool {
        T::is_msg()
    }
    
    fn is_rma() -> bool {
        T::is_rma()
    }
    
    fn is_tagged() -> bool {
        T::is_tagged()
    }
    
    fn is_atomic() -> bool {
        T::is_atomic()
    }
    
    fn is_mcast() -> bool {
        T::is_mcast()
    }
    
    fn is_named_rx_ctx() -> bool {
        T::is_named_rx_ctx()
    }
    
    fn is_directed_recv() -> bool {
        T::is_directed_recv()
    }
    
    fn is_hmem() -> bool {
        T::is_hmem()
    }
    
    fn is_collective() -> bool {
        T::is_collective()
    }
    
    fn is_xpu() -> bool {
        T::is_xpu()
    }
}

impl<T> InfoHints<T> {
    // pub fn mode(mut self, mode: crate::enums::Mode) -> Self {
    //     unsafe { (*self.c_info).mode = mode.get_value() };

    //     self
    // }
    #[allow(unused_mut)]
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

    pub fn tx_attr(self, attr: crate::xcontext::TxAttr) -> Self {
        unsafe { *(*self.c_info).tx_attr = *attr.get() };
        
        self
    }

    
    
    #[allow(unused_mut)]
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

// impl Default for InfoHints {
//     fn default() -> Self {
//         Self::new()
//     }
// }



pub type Address = libfabric_sys::fi_addr_t; 
pub type DataType = libfabric_sys::fi_datatype;
pub struct Msg {
    c_msg: libfabric_sys::fi_msg,
}

impl Msg {

    pub fn new<T>(iov: &[IoVec<T>], desc: &mut [impl DataDescriptor], addr: Address) -> Self {
        Msg {
            c_msg : libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr,
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            }
        }
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

// pub struct Stx {

//     #[allow(dead_code)]
//     c_stx: *mut libfabric_sys::fid_stx,
// }

// impl Stx {
//     pub(crate) fn new<T0>(domain: &crate::domain::Domain, mut attr: crate::TxAttr, context: &mut T0) -> Result<Stx, error::Error> {
//         let mut c_stx: *mut libfabric_sys::fid_stx = std::ptr::null_mut();
//         let c_stx_ptr: *mut *mut libfabric_sys::fid_stx = &mut c_stx;
//         let err = unsafe { libfabric_sys::inlined_fi_stx_context(domain.c_domain, attr.get_mut(), c_stx_ptr, context as *mut T0 as *mut std::ffi::c_void) };

//         if err != 0 {
//             Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
//         }
//         else {
//             Ok(
//                 Self { c_stx }
//             )
//         }

//     }
// }

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
pub struct IoVec<'a, T> {
    c_iovec: libfabric_sys::iovec,
    borrow: PhantomData<&'a T>,
}

impl<'a, T> IoVec<'a, T> {

    pub fn from(mem: &'a T) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  (mem as *const T as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self { c_iovec, borrow: PhantomData }
    }

    pub fn from_mut(mem: &'a mut T) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  (mem as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self { c_iovec, borrow: PhantomData }
    }

    pub fn from_slice(mem: &'a [T]) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  (mem.as_ptr() as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self { c_iovec, borrow: PhantomData }
    }

    pub fn from_slice_mut(mem: &'a mut [T]) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  mem.as_mut_ptr().cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self { c_iovec, borrow: PhantomData }
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
pub struct Ioc<'a, T>{
    c_ioc: libfabric_sys::fi_ioc,
    borrow: PhantomData<&'a T>,
}

impl<'a, T> Ioc<'a, T> {

    pub fn from(mem: &'a T) -> Self {
        let c_ioc = libfabric_sys::fi_ioc{
            addr:  (mem as *const T as *mut T).cast(),
            count: 1,
        };

        Self { c_ioc, borrow: PhantomData }
    }

    pub fn from_mut(mem: &'a mut T) -> Self {
        let c_ioc = libfabric_sys::fi_ioc{
            addr:  (mem as *mut T).cast(),
            count: 1,
        };

        Self { c_ioc, borrow: PhantomData }
    }

    pub fn from_slice(mem: &'a [T]) -> Self {
        let c_ioc = libfabric_sys::fi_ioc{
            addr:  (mem.as_ptr() as *mut T).cast(),
            count: mem.len(),
        };

        Self { c_ioc, borrow: PhantomData }
    }

    pub fn from_slice_mut(mem: &'a mut [T]) -> Self {
        let c_ioc = libfabric_sys::fi_ioc{
            addr:  mem.as_mut_ptr().cast(),
            count: mem.len(),
        };

        Self { c_ioc, borrow: PhantomData }
    }

    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_ioc {
        &self.c_ioc
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_ioc {
        &mut self.c_ioc
    }
}

pub trait DataDescriptor {
    fn get_desc(&mut self) -> *mut std::ffi::c_void;
    fn get_desc_ptr(&mut self) -> *mut *mut std::ffi::c_void;
}

pub(crate) struct OwnedFid {
    fid: *mut libfabric_sys::fid,
}

impl Drop for OwnedFid {
    fn drop(&mut self) {
        let err = unsafe { libfabric_sys::inlined_fi_close(self.fid) };

        if err != 0 {
            panic!("{}", error::Error::from_err_code((-err).try_into().unwrap()));
        }
    }
}

impl AsFid for OwnedFid {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.fid
    }
}

pub trait AsFid {
    fn as_fid(&self) -> *mut libfabric_sys::fid;
}



// pub trait FID{
//     // fn fid(&self) -> *mut libfabric_sys::fid;
    
//     fn setname<T>(&mut self, addr:&[T]) -> Result<(), error::Error> {
//         let err = unsafe { libfabric_sys::inlined_fi_setname(self.as_fid(), addr.as_ptr() as *mut std::ffi::c_void, addr.len()) };
        
//         if err != 0 {
//             Err(error::Error::from_err_code((-err).try_into().unwrap()))
//         }
//         else {
//             Ok(())
//         }
//     }

//     fn getname<T0>(&self, addr: &mut[T0]) -> Result<usize, error::Error> {
//         let mut len: usize = std::mem::size_of_val(addr);
//         let len_ptr: *mut usize = &mut len;
//         let err: i32 = unsafe { libfabric_sys::inlined_fi_getname(self.as_fid(), addr.as_mut_ptr() as *mut std::ffi::c_void, len_ptr) };

//         if -err as u32  == libfabric_sys::FI_ETOOSMALL {
//             Err(error::Error{ c_err: -err  as u32, kind: error::ErrorKind::TooSmall(len)} )
//         }
//         else if err < 0 {
//             Err(error::Error::from_err_code((-err).try_into().unwrap()))
//         }
//         else {
//             Ok(len)
//         }
//     }


//     fn control<T0>(&self, opt: crate::enums::ControlOpt, arg: &mut T0) -> Result<(), crate::error::Error> {
//         let err = unsafe { libfabric_sys::inlined_fi_control(self.as_fid(), opt.get_value() as i32, arg as *mut T0 as *mut std::ffi::c_void) };
    
//         if err != 0 {
//             return Err(error::Error::from_err_code((-err).try_into().unwrap()));
//         }

//         Ok(())
//     }
// }


pub type CollectiveOp = libfabric_sys::fi_collective_op;
pub struct CollectiveAttr {
    pub(crate) c_attr: libfabric_sys::fi_collective_attr,
}

impl CollectiveAttr {

    //[TODO] CHECK INITIAL VALUES
    pub fn new() -> Self {

        Self {
            c_attr: libfabric_sys::fi_collective_attr {
                op: 0,
                datatype: libfabric_sys::fi_datatype_FI_UINT64,
                datatype_attr: libfabric_sys::fi_atomic_attr{count: 0, size: 0},
                max_members: 0,
                mode: 0,
            }
        }
    }


    pub fn op(mut self, op: &enums::Op) -> Self {
        self.c_attr.op = op.get_value();
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_collective_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_collective_attr {
        &mut self.c_attr
    }
}

impl Default for CollectiveAttr {
    fn default() -> Self {
        Self::new()
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

#[repr(C)]
pub struct DefaultMemDesc {
    c_desc: *mut std::ffi::c_void,
}

pub fn default_desc() -> DefaultMemDesc { DefaultMemDesc { c_desc: std::ptr::null_mut() }}

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

impl Default for Context {
    fn default() -> Self {
        Self::new()
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

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_context2 {
        &mut self.c_val
    }
}

impl Default for Context2 {
    fn default() -> Self {
        Self::new()
    }
}

pub trait BindImpl{}
pub trait Bind {
    fn inner(&self) -> Rc<dyn BindImpl>;
}

#[repr(C)]
#[derive(Clone, Debug)]
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

impl Default for RmaIoVec {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MsgTagged {
    c_msg_tagged: libfabric_sys::fi_msg_tagged,
}

impl MsgTagged {
    pub fn new<T>(iov: &[IoVec<T>], desc: &mut [impl DataDescriptor], addr: Address, data: u64, tag: u64, ignore: u64) -> Self {
    
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.as_mut_ptr().cast(),
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

pub struct MsgRma {
    c_msg_rma: libfabric_sys::fi_msg_rma,
}

impl MsgRma {
    pub fn new<T, T0>(iov: &[IoVec<T>], desc: &mut [impl DataDescriptor], addr: Address, rma_iov: &[RmaIoVec], context: &mut T0, data: u64) -> Self {
        Self {
            c_msg_rma : libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.as_mut_ptr().cast(),
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

#[derive(Clone)]
pub struct Nic {
    pub device_attr: Option<DeviceAttr>,
    pub bus_attr: Option<BusAttr>,
    pub link_attr: Option<LinkAttr>,
}

impl Nic {
    pub(crate) fn from_attr(fid: libfabric_sys::fid_nic) -> Self {
        let device_attr = if ! fid.device_attr.is_null() {
            Some(DeviceAttr::from_attr(unsafe{*fid.device_attr}))
        }
        else {
            None
        };

        let bus_attr = if ! fid.bus_attr.is_null() {
            Some(BusAttr::from_attr(unsafe{*fid.bus_attr}))
        }
        else {
            None
        };

        let link_attr = if ! fid.link_attr.is_null() {
            Some(LinkAttr::from_attr(unsafe{*fid.link_attr}))
        }
        else {
            None
        };

        Self {
            device_attr,
            bus_attr,
            link_attr,
        }
    }
}

#[derive(Clone)]
pub struct DeviceAttr {
    pub name: Option<String>,
    pub device_id: Option<String>,
    pub device_version: Option<String>,
    pub driver: Option<String>,
    pub firmware: Option<String>,
}

impl DeviceAttr {
    pub(crate) fn from_attr(attr: libfabric_sys::fi_device_attr) -> Self {
        Self {
            name: if attr.name.is_null() {None} else {unsafe{std::ffi::CStr::from_ptr(attr.name).to_str().unwrap_or("").to_owned().into()}},
            device_id: if attr.device_id.is_null() {None} else {unsafe{std::ffi::CStr::from_ptr(attr.device_id).to_str().unwrap_or("").to_owned().into()}},
            device_version: if attr.device_version.is_null() {None} else {unsafe{std::ffi::CStr::from_ptr(attr.device_version).to_str().unwrap_or("").to_owned().into()}},
            driver: if attr.driver.is_null() {None} else {unsafe{std::ffi::CStr::from_ptr(attr.driver).to_str().unwrap_or("").to_owned().into()}},
            firmware: if attr.firmware.is_null() {None} else {unsafe{std::ffi::CStr::from_ptr(attr.firmware).to_str().unwrap_or("").to_owned().into()}},
        }
    }
}

#[derive(Clone)]
pub struct LinkAttr {
    pub address: Option<String>,
    pub mtu: usize,
    pub speed: usize,
    pub state: LinkState,
    pub network_type: Option<String>,
}

impl LinkAttr {
    pub(crate) fn from_attr(attr: libfabric_sys::fi_link_attr) -> Self {
        Self {
            address: if attr.address.is_null() {None} else {unsafe{std::ffi::CStr::from_ptr(attr.address).to_str().unwrap_or("").to_owned().into()}},
            mtu: attr.mtu,
            speed: attr.speed,
            state: LinkState::from_value(attr.state),
            network_type: if attr.network_type.is_null() {None} else {unsafe{std::ffi::CStr::from_ptr(attr.network_type).to_str().unwrap_or("").to_owned().into()}},
        }
    }
}


#[derive(Clone)]
pub enum LinkState {
    Unknown,
    Down,
    Up,
}

impl LinkState {
    pub(crate) fn from_value(val: libfabric_sys::fi_link_state) -> Self {
        if val == libfabric_sys::fi_link_state_FI_LINK_UNKNOWN {
            LinkState::Unknown
        }
        else if val == libfabric_sys::fi_link_state_FI_LINK_DOWN {
            LinkState::Down
        }
        else if val == libfabric_sys::fi_link_state_FI_LINK_UP {
            LinkState::Up
        }
        else {
            panic!("Unexpected link state");
        }
    }
}

#[derive(Clone)]
pub struct BusAttr {
    pub bus_type: BusType,
    pub pci: PciAttr,
}

impl BusAttr {
    pub(crate) fn from_attr(attr: libfabric_sys::fi_bus_attr) -> Self {
        Self {
            bus_type: BusType::from_value(attr.bus_type),
            pci: PciAttr::from_attr(unsafe{attr.attr.pci})
        }
    }
}

#[derive(Clone)]
pub enum BusType {
    Pci,
    Unknown,
    Unspec,
}

impl BusType {
    pub(crate) fn from_value(val: libfabric_sys::fi_bus_type) -> Self {
        if val == libfabric_sys::fi_bus_type_FI_BUS_UNKNOWN {
            BusType::Unknown
        }
        else if val == libfabric_sys::fi_bus_type_FI_BUS_PCI {
            BusType::Pci
        }
        else if val == libfabric_sys::fi_bus_type_FI_BUS_UNSPEC {
            BusType::Unspec
        }
        else {
            panic!("Unexpected link state");
        }
    }
}

#[derive(Clone)]
pub struct PciAttr {
    pub domain_id: u16,
    pub bus_id: u8,
    pub device_id: u8,
    pub function_id: u8,
}

impl PciAttr {
    pub(crate) fn from_attr(attr: libfabric_sys::fi_pci_attr) -> Self {
        Self {
            domain_id: attr.domain_id,
            bus_id: attr.bus_id,
            device_id: attr.device_id,
            function_id: attr.function_id,
        }
    }
}
fn check_error(err: isize) -> Result<(), crate::error::Error> {
    if err != 0 {
        Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
    }
    else {
        Ok(())
    }
}

pub trait FdRetrievable{}
pub trait Waitable{}
pub trait Writable{}
pub trait WaitRetrievable{}

#[cfg(test)]
#[cfg(ignore)]
mod rust_lifetime_tests {
    use crate::IoVec;

    fn foo(data: &mut [usize]) {}
    fn foo_ref(data: & [usize]) {}
    fn foo2<T>(data: & IoVec<T>) {}

    #[test]
    fn iovec_lifetime() {
        let mut  data: Vec<usize> = Vec::new();
        let iov = IoVec::from_slice(&data);
        drop(data);
        iov.get();
    }
    
    #[test]
    fn iovec_borrow_mut() {
        let mut  data: Vec<usize> = Vec::new();
        let iov = IoVec::from_slice(&data);
        foo(&mut data);
        drop(data);
        iov.get();
    }
    

    #[test]
    fn iovec_mut_mut() {
        let mut  data: Vec<usize> = Vec::new();
        let iov = IoVec::from_slice_mut(&mut data);
        foo(&mut data);
        iov.get();
    }
    
    #[test]
    fn iovec_mut_borrow() {
        let mut  data: Vec<usize> = Vec::new();
        let iov = IoVec::from_slice_mut(&mut data);
        foo_ref(&data);
        iov.get();
    }
}