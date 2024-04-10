use std::marker::PhantomData;

use crate::{nic::Nic, utils::check_error, infocapsoptions::Caps};

#[derive(Clone, Debug)]
pub struct InfoCapsImpl {
    pub(crate) bitfield: u64,
}

impl InfoCapsImpl {
    pub const fn new() -> Self {
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
    
    
    pub fn read_if(&self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_READ as u64} else {self.bitfield} }}
    pub fn write_if(&self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield |  libfabric_sys::FI_WRITE as u64} else {self.bitfield} }}
    pub fn recv_if(&self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_RECV as u64} else {self.bitfield} }}
    pub fn send_if(&self, cond: bool) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_SEND as u64} else {self.bitfield} }}
    pub fn transmit_if(&self, cond: bool ) -> Self { Self { bitfield: if cond {self.bitfield |  libfabric_sys::FI_TRANSMIT as u64} else {self.bitfield} }}
    pub fn remote_read_if(&self, cond: bool ) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_REMOTE_READ as u64} else {self.bitfield} }}
    pub fn remote_write_if(&self, cond: bool ) -> Self { Self { bitfield: if cond {self.bitfield | libfabric_sys::FI_REMOTE_WRITE as u64} else {self.bitfield} }}
    pub fn rma_event_if(&self, cond: bool) -> Self { Self { bitfield: if cond{self.bitfield | libfabric_sys::FI_RMA_EVENT} else {self.bitfield} }}


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

impl Default for InfoCapsImpl {
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
    caps: InfoCapsImpl,
    fabric_attr: crate::fabric::FabricAttr,
    domain_attr: crate::domain::DomainAttr,
    tx_attr: crate::xcontext::TxAttr,
    rx_attr: crate::xcontext::RxAttr,
    ep_attr: crate::ep::EndpointAttr,
    nic: Option<Nic>,
    pub(crate) c_info: *mut  libfabric_sys::fi_info,
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
        Self { caps: InfoCapsImpl::from(caps) , fabric_attr, domain_attr, tx_attr, rx_attr, ep_attr, nic, c_info, phantom: PhantomData }
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

    pub fn get_caps(&self) -> &InfoCapsImpl {
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

impl<T: Caps> Caps for InfoEntry<T> {
    fn bitfield() -> u64 {
        T::bitfield()
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
    pub fn caps<T: Caps>(mut self, _caps: T)  -> InfoHints<T> {
        unsafe { (*self.c_info).caps = T::bitfield() };
        
        InfoHints::<T> {
            c_info: self.c_info,
            phantom: PhantomData,
        }
    }
}

impl<T: Caps> Caps for InfoHints<T> {
    fn bitfield() -> u64 {
        T::bitfield()
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

    pub fn get_caps(&self) -> InfoCapsImpl {
        InfoCapsImpl::from(unsafe{ (*self.c_info).caps })
    }

    pub fn get_ep_attr(&self) -> crate::ep::EndpointAttr {
        crate::ep::EndpointAttr::from(unsafe{ (*self.c_info).ep_attr })
    }
}