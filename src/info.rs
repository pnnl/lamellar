use std::{collections::VecDeque, ffi::CString, marker::PhantomData, sync::atomic::AtomicUsize};

use libfabric_sys::FI_SOURCE;

use crate::{
    domain::{DomainAttr, DomainBase},
    enums::{
        AddressFormat, AddressVectorType, DomainCaps, EndpointType, Mode, MrMode, Progress,
        ResourceMgmt, Threading, TrafficClass, TransferOptions, TriggerEvent,
    },
    ep::Address,
    fabric::Fabric,
    fid::{AsRawTypedFid, AsTypedFid},
    infocapsoptions::Caps,
    nic::Nic,
    trigger::{TriggeredContext, TriggeredContext1, TriggeredContext2, TriggeredContextType},
    utils::check_error,
    xcontext::{MsgOrder, RxCaps, RxCompOrder, TxCaps, TxCompOrder},
    Context, Context1, Context2, ContextType, MappedAddress, RawMappedAddress, FI_ADDR_NOTAVAIL,
    FI_ADDR_UNSPEC,
};

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

    pub fn msg(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_MSG as u64,
        }
    }
    pub fn tagged(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_TAGGED as u64,
        }
    }
    pub fn rma(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_RMA as u64,
        }
    }
    pub fn atomic(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_ATOMIC as u64,
        }
    }
    pub fn multicast(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_MULTICAST as u64,
        }
    }
    pub fn named_rx_ctx(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_NAMED_RX_CTX,
        }
    }
    pub fn directed_recv(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_DIRECTED_RECV,
        }
    }
    pub fn variable_msg(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_VARIABLE_MSG,
        }
    }
    pub fn hmem(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_HMEM,
        }
    }
    pub fn collective(self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_COLLECTIVE as u64,
        }
    }

    pub fn msg_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_MSG as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn tagged_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_TAGGED as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn rma_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_RMA as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn atomic_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_ATOMIC as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn multicast_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_MULTICAST as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn named_rx_ctx_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_NAMED_RX_CTX
            } else {
                self.bitfield
            },
        }
    }
    pub fn directed_recv_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_DIRECTED_RECV
            } else {
                self.bitfield
            },
        }
    }
    pub fn variable_msg_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_VARIABLE_MSG
            } else {
                self.bitfield
            },
        }
    }
    pub fn hmem_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_HMEM
            } else {
                self.bitfield
            },
        }
    }
    pub fn collective_if(self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_COLLECTIVE as u64
            } else {
                self.bitfield
            },
        }
    }

    pub fn read(&self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_READ as u64,
        }
    }
    pub fn write(&self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_WRITE as u64,
        }
    }
    pub fn recv(&self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_RECV as u64,
        }
    }
    pub fn send(&self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_SEND as u64,
        }
    }
    pub fn transmit(&self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_TRANSMIT as u64,
        }
    }
    pub fn remote_read(&self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_REMOTE_READ as u64,
        }
    }
    pub fn remote_write(&self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_REMOTE_WRITE as u64,
        }
    }

    pub fn rma_event(&self) -> Self {
        Self {
            bitfield: self.bitfield | libfabric_sys::FI_RMA_EVENT,
        }
    }

    pub fn read_if(&self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_READ as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn write_if(&self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_WRITE as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn recv_if(&self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_RECV as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn send_if(&self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_SEND as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn transmit_if(&self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_TRANSMIT as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn remote_read_if(&self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_REMOTE_READ as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn remote_write_if(&self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_REMOTE_WRITE as u64
            } else {
                self.bitfield
            },
        }
    }
    pub fn rma_event_if(&self, cond: bool) -> Self {
        Self {
            bitfield: if cond {
                self.bitfield | libfabric_sys::FI_RMA_EVENT
            } else {
                self.bitfield
            },
        }
    }

    pub fn is_msg(&self) -> bool {
        self.bitfield & libfabric_sys::FI_MSG as u64 == libfabric_sys::FI_MSG as u64
    }
    pub fn is_tagged(&self) -> bool {
        self.bitfield & libfabric_sys::FI_TAGGED as u64 == libfabric_sys::FI_TAGGED as u64
    }
    pub fn is_rma(&self) -> bool {
        self.bitfield & libfabric_sys::FI_RMA as u64 == libfabric_sys::FI_RMA as u64
    }
    pub fn is_atomic(&self) -> bool {
        self.bitfield & libfabric_sys::FI_ATOMIC as u64 == libfabric_sys::FI_ATOMIC as u64
    }
    pub fn is_multicast(&self) -> bool {
        self.bitfield & libfabric_sys::FI_MULTICAST as u64 == libfabric_sys::FI_MULTICAST as u64
    }
    pub fn is_named_rx_ctx(self) -> bool {
        self.bitfield & libfabric_sys::FI_NAMED_RX_CTX == libfabric_sys::FI_NAMED_RX_CTX
    }
    pub fn is_directed_recv(self) -> bool {
        self.bitfield & libfabric_sys::FI_DIRECTED_RECV == libfabric_sys::FI_DIRECTED_RECV
    }
    pub fn is_variable_msg(self) -> bool {
        self.bitfield & libfabric_sys::FI_VARIABLE_MSG == libfabric_sys::FI_VARIABLE_MSG
    }
    pub fn is_hmem(self) -> bool {
        self.bitfield & libfabric_sys::FI_HMEM == libfabric_sys::FI_HMEM
    }
    pub fn is_collective(&self) -> bool {
        self.bitfield & libfabric_sys::FI_COLLECTIVE as u64 == libfabric_sys::FI_COLLECTIVE as u64
    }

    pub fn is_read(&self) -> bool {
        self.bitfield & libfabric_sys::FI_READ as u64 == libfabric_sys::FI_READ as u64
    }
    pub fn is_write(&self) -> bool {
        self.bitfield & libfabric_sys::FI_WRITE as u64 == libfabric_sys::FI_WRITE as u64
    }
    pub fn is_recv(&self) -> bool {
        self.bitfield & libfabric_sys::FI_RECV as u64 == libfabric_sys::FI_RECV as u64
    }
    pub fn is_send(&self) -> bool {
        self.bitfield & libfabric_sys::FI_SEND as u64 == libfabric_sys::FI_SEND as u64
    }
    pub fn is_transmit(&self) -> bool {
        self.bitfield & libfabric_sys::FI_TRANSMIT as u64 == libfabric_sys::FI_TRANSMIT as u64
    }
    pub fn is_remote_read(&self) -> bool {
        self.bitfield & libfabric_sys::FI_REMOTE_READ as u64 == libfabric_sys::FI_REMOTE_READ as u64
    }
    pub fn is_remote_write(&self) -> bool {
        self.bitfield & libfabric_sys::FI_REMOTE_WRITE as u64
            == libfabric_sys::FI_REMOTE_WRITE as u64
    }

    pub fn is_rma_event(&self) -> bool {
        self.bitfield & libfabric_sys::FI_RMA_EVENT == libfabric_sys::FI_RMA_EVENT
    }
}

impl Default for InfoCapsImpl {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Info<T> {
    entries: VecDeque<InfoEntry<T>>,
}

pub struct InfoBuilder<T> {
    hints_info: FabricInfo,
    c_version: u32,
    c_node: std::ffi::CString,
    c_service: std::ffi::CString,
    flags: u64,
    phantom: PhantomData<T>,
}

impl<T> InfoBuilder<T> {
    pub fn source(self, source: ServiceAddress) -> Self {
        let (c_node, c_service) = match source {
            ServiceAddress::String(fulladdress) => (
                std::ffi::CString::new(fulladdress).unwrap(),
                std::ffi::CString::new("").unwrap(),
            ),
            ServiceAddress::NodeAndService(node, service) => (
                std::ffi::CString::new(node).unwrap(),
                std::ffi::CString::new(service).unwrap(),
            ),
            ServiceAddress::Node(node) => (
                std::ffi::CString::new(node).unwrap(),
                std::ffi::CString::new("").unwrap(),
            ),
            ServiceAddress::Service(service) => (
                std::ffi::CString::new("").unwrap(),
                std::ffi::CString::new(service).unwrap(),
            ),
        };

        Self {
            flags: self.flags | FI_SOURCE,
            c_node,
            c_service,
            ..self
        }
    }

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

    pub fn numeric_host(mut self) -> Self {
        self.flags |= libfabric_sys::FI_NUMERICHOST;
        self
    }

    pub fn prov_attr_only(mut self) -> Self {
        self.flags |= libfabric_sys::FI_PROV_ATTR_ONLY;
        self
    }

    pub fn get(self) -> Result<Info<T>, crate::error::Error> {
        let mut c_info = std::ptr::null_mut();
        let node = if self.c_node.is_empty() {
            std::ptr::null_mut()
        } else {
            self.c_node.as_ptr()
        };
        let service = if self.c_service.is_empty() {
            std::ptr::null_mut()
        } else {
            self.c_service.as_ptr()
        };

        let err = unsafe {
            libfabric_sys::fi_getinfo(
                self.c_version,
                node,
                service,
                self.flags,
                self.hints_info.0,
                &mut c_info,
            )
        };

        if !self.hints_info.0.is_null() {
            let c_fabric_attr_ptr = unsafe { (*self.hints_info.0).fabric_attr };

            let fabric_name = unsafe { *c_fabric_attr_ptr }.name;
            if !fabric_name.is_null() {
                drop(unsafe { std::ffi::CString::from_raw(fabric_name) })
            }

            let prov_name = unsafe { *c_fabric_attr_ptr }.prov_name;
            if !prov_name.is_null() {
                drop(unsafe { std::ffi::CString::from_raw(prov_name) })
            }

            let c_domain_attr_ptr = unsafe { (*self.hints_info.0).domain_attr };
            let domain_name = unsafe { *c_domain_attr_ptr }.name;
            if !domain_name.is_null() {
                drop(unsafe { std::ffi::CString::from_raw(domain_name) })
            }
        }

        check_error(err.try_into().unwrap())?;

        let mut entries = VecDeque::new();
        if !c_info.is_null() {
            entries.push_back(InfoEntry::new(c_info));

            unsafe {
                let mut curr = (*c_info).next;
                while !curr.is_null() {
                    entries.push_back(InfoEntry::new(curr));
                    curr = (*curr).next;
                }
            }
        }

        Ok(Info::<T> { entries })
    }

    pub fn enter_hints(self) -> InfoHints<T> {
        InfoHints::new(self)
    }
}

// #[derive(Clone)]
pub struct InfoEntry<T> {
    ctx_id: AtomicUsize,
    caps: InfoCapsImpl,
    mode: crate::enums::Mode,
    src_address: Option<Address>,
    dest_address: Option<Address>,
    fabric_attr: crate::fabric::FabricAttr,
    domain_attr: crate::domain::DomainAttr,
    tx_attr: crate::xcontext::TxAttr,
    rx_attr: crate::xcontext::RxAttr,
    ep_attr: crate::ep::EndpointAttr,
    nic: Option<Nic>,
    pub(crate) info: FabricInfo,
    phantom: PhantomData<fn() -> T>, // fn() -> T because we only need to track the Endpoint capabilities requested but avoid requiring caps to implement Sync+Send
}

#[cfg(feature = "thread-safe")]
unsafe impl Send for FabricInfo {} // FabricInfo is send because we never copy the underlying pointer
#[cfg(feature = "thread-safe")]
unsafe impl Sync for FabricInfo {}

impl<T> InfoEntry<T> {
    pub(crate) fn new(c_info: *mut libfabric_sys::fi_info) -> Self {
        let c_info = unsafe { libfabric_sys::fi_dupinfo(c_info) };
        let fabric_attr = crate::fabric::FabricAttr::from_raw_ptr(unsafe { *c_info }.fabric_attr);
        let domain_attr = DomainAttr::from_raw_ptr(unsafe { *c_info }.domain_attr);
        let mode = Mode::from_raw(unsafe { *c_info }.mode);
        let dest_address = if unsafe { *c_info }.dest_addr.is_null() {
            None
        } else {
            Some(unsafe {
                Address::from_raw_parts((*c_info).dest_addr as *const u8, (*c_info).dest_addrlen)
            })
        };

        let src_address = if unsafe { *c_info }.src_addr.is_null() {
            None
        } else {
            Some(unsafe {
                Address::from_raw_parts((*c_info).src_addr as *const u8, (*c_info).src_addrlen)
            })
        };

        let tx_attr = crate::xcontext::TxAttr::from_raw_ptr(unsafe { *c_info }.tx_attr);
        let rx_attr = crate::xcontext::RxAttr::from_raw_ptr(unsafe { *c_info }.rx_attr);
        let ep_attr = crate::ep::EndpointAttr::from_raw_ptr(unsafe { *c_info }.ep_attr);

        let caps: u64 = unsafe { (*c_info).caps };
        let nic = if !unsafe { (*c_info).nic.is_null() } {
            Some(Nic::from_raw_ptr(unsafe { *c_info }.nic))
        } else {
            None
        };
        Self {
            ctx_id: AtomicUsize::new(0),
            caps: InfoCapsImpl::from(caps),
            mode,
            src_address,
            dest_address,
            fabric_attr,
            domain_attr,
            tx_attr,
            rx_attr,
            ep_attr,
            nic,
            info: FabricInfo(c_info),
            phantom: PhantomData,
        }
    }

    pub fn rx_addr(
        &self,
        rx_index: i32,
        rx_ctx_bits: i32,
    ) -> Result<MappedAddress, crate::error::Error> {
        let ret =
            unsafe { libfabric_sys::inlined_fi_rx_addr(FI_ADDR_NOTAVAIL, rx_index, rx_ctx_bits) };
        if ret == FI_ADDR_NOTAVAIL || ret == FI_ADDR_UNSPEC {
            return Err(crate::error::Error::from_err_code(
                libfabric_sys::FI_EADDRNOTAVAIL,
            ));
        }
        match self.domain_attr.av_type() {
            AddressVectorType::Unspec => Ok(MappedAddress::from_raw_addr_no_av(
                RawMappedAddress::Unspec(ret),
            )),
            AddressVectorType::Map => Ok(MappedAddress::from_raw_addr_no_av(
                RawMappedAddress::Map(ret),
            )),
            AddressVectorType::Table => Ok(MappedAddress::from_raw_addr_no_av(
                RawMappedAddress::Table(ret),
            )),
        }
    }

    pub fn allocate_context(&self) -> Context {
        let ctx_id = self
            .ctx_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.mode.is_context() {
            Context(ContextType::Context1(Box::new(Context1::new(ctx_id))))
        } else {
            Context(ContextType::Context2(Box::new(Context2::new(ctx_id))))
        }
    }

    pub fn allocate_triggered_context<'a, 'b>(
        &self,
        event: &'a mut TriggerEvent<'b>,
    ) -> TriggeredContext<'a, 'b> {
        if self.mode.is_context() {
            TriggeredContext(TriggeredContextType::TriggeredContext1(Box::new(
                TriggeredContext1::new(event),
            )))
        } else {
            TriggeredContext(TriggeredContextType::TriggeredContext2(Box::new(
                TriggeredContext2::new(event),
            )))
        }
    }

    pub fn dest_addr(&self) -> Option<&Address> {
        self.dest_address.as_ref()
    }

    pub fn src_addr(&self) -> Option<&Address> {
        self.src_address.as_ref()
    }

    pub fn mode(&self) -> &crate::enums::Mode {
        &self.mode
    }

    pub fn domain_attr(&self) -> &crate::domain::DomainAttr {
        &self.domain_attr
    }

    pub fn fabric_attr(&self) -> &crate::fabric::FabricAttr {
        &self.fabric_attr
    }

    pub fn tx_attr(&self) -> &crate::xcontext::TxAttr {
        &self.tx_attr
    }

    pub fn rx_attr(&self) -> &crate::xcontext::RxAttr {
        &self.rx_attr
    }

    pub fn ep_attr(&self) -> &crate::ep::EndpointAttr {
        &self.ep_attr
    }

    pub fn caps(&self) -> &InfoCapsImpl {
        &self.caps
    }

    pub fn nic(&self) -> Option<Nic> {
        self.nic.clone()
    }
}

pub struct InfoIterator<'a, T> {
    info: &'a Info<T>,
    index: usize,
}

impl<T> Info<T> {
    pub fn iter(&self) -> InfoIterator<T> {
        InfoIterator {
            info: self,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for InfoIterator<'a, T> {
    type Item = &'a InfoEntry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.info.entries.len() {
            let result = Some(&self.info.entries[self.index]);
            self.index += 1;
            result
        } else {
            None
        }
    }
}

pub struct InfoIntoIterator<T> {
    info: Info<T>,
}

impl<T> IntoIterator for Info<T> {
    type Item = InfoEntry<T>;

    type IntoIter = InfoIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        InfoIntoIterator { info: self }
    }
}

impl<T> Iterator for InfoIntoIterator<T> {
    type Item = InfoEntry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.info.entries.remove(0)
    }
}

impl<T> std::fmt::Debug for InfoEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c_str = unsafe {
            libfabric_sys::fi_tostr(self.info.0.cast(), libfabric_sys::fi_type_FI_TYPE_INFO)
        };
        if c_str.is_null() {
            panic!("String is null")
        }
        let val = unsafe { std::ffi::CStr::from_ptr(c_str) };

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

pub enum ServiceAddress {
    String(String),
    NodeAndService(String, String),
    Node(String),
    Service(String),
}

impl Info<()> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(version: &Version) -> InfoBuilder<()> {
        InfoBuilder::<()> {
            hints_info: FabricInfo(unsafe { libfabric_sys::inlined_fi_allocinfo() }),
            c_version: version.as_raw(),
            c_node: std::ffi::CString::new("").unwrap(),
            c_service: std::ffi::CString::new("").unwrap(),
            flags: 0,
            phantom: PhantomData,
        }
    }

    pub fn with_numeric_host(version: &Version, host: &str) -> InfoBuilder<()> {
        InfoBuilder::<()> {
            hints_info: FabricInfo(unsafe { libfabric_sys::inlined_fi_allocinfo() }),
            c_version: version.as_raw(),
            c_node: std::ffi::CString::new(host).unwrap(),
            c_service: std::ffi::CString::new("").unwrap(),
            flags: libfabric_sys::FI_NUMERICHOST,
            phantom: PhantomData,
        }
    }
}
// impl<T> Drop for Info<T> {
//     fn drop(&mut self) {
//         unsafe {
//             libfabric_sys::fi_freeinfo(self.c_info);
//         }
//     }
// }
impl Drop for FabricInfo {
    fn drop(&mut self) {
        unsafe {
            libfabric_sys::fi_freeinfo(self.0);
        }
    }
}

pub struct InfoHints<T> {
    info_builder: InfoBuilder<T>,
}
pub(crate) struct FabricInfo(pub(crate) *mut libfabric_sys::fi_info);

impl FabricInfo {
    fn set_mode(&mut self, mode: Mode) {
        unsafe { (*self.0).mode = mode.as_raw() };
    }

    fn set_addr_format(&mut self, addr_format: AddressFormat) {
        unsafe { (*self.0).addr_format = addr_format.as_raw() };
    }

    fn set_eptype(&mut self, eptype: EndpointType) {
        unsafe { (*(*self.0).ep_attr).type_ = eptype.as_raw() };
    }

    fn set_ep_mem_tag_format(&mut self, tag: u64) {
        unsafe { (*(*self.0).ep_attr).mem_tag_format = tag };
    }

    fn set_ep_tx_ctx_cnt(&mut self, size: usize) {
        unsafe { (*(*self.0).ep_attr).tx_ctx_cnt = size };
    }

    fn set_ep_rx_ctx_cnt(&mut self, size: usize) {
        unsafe { (*(*self.0).ep_attr).rx_ctx_cnt = size };
    }

    fn set_ep_auth_key(&mut self, key: &[u8]) {
        unsafe { (*(*self.0).ep_attr).auth_key_size = key.len() };
        unsafe {
            (*(*self.0).ep_attr).auth_key = std::mem::transmute::<*const u8, *mut u8>(key.as_ptr())
        };
    }

    fn set_domain<EQ>(&mut self, name: &DomainBase<EQ>) {
        unsafe { (*(*self.0).domain_attr).domain = name.as_typed_fid().as_raw_typed_fid() };
    }

    fn set_domain_name(&mut self, name: &str) {
        let c_str = CString::new(name.to_string()).unwrap();
        unsafe { (*(*self.0).domain_attr).name = c_str.into_raw() };
    }

    fn set_domain_threading(&mut self, threading: Threading) {
        unsafe { (*(*self.0).domain_attr).threading = threading.as_raw() };
    }

    fn set_domain_control_progress(&mut self, cntrl_progress: Progress) {
        unsafe { (*(*self.0).domain_attr).control_progress = cntrl_progress.as_raw() };
    }

    fn set_domain_data_progress(&mut self, data_progress: Progress) {
        unsafe { (*(*self.0).domain_attr).data_progress = data_progress.as_raw() };
    }

    fn set_domain_resource_mgmt(&mut self, resource_mgmt: ResourceMgmt) {
        unsafe { (*(*self.0).domain_attr).resource_mgmt = resource_mgmt.as_raw() };
    }

    fn set_domain_av_type(&mut self, av_type: AddressVectorType) {
        unsafe { (*(*self.0).domain_attr).av_type = av_type.as_raw() };
    }

    fn set_domain_mr_mode(&mut self, mr_mode: MrMode) {
        unsafe { (*(*self.0).domain_attr).mr_mode = mr_mode.as_raw() as i32 };
    }

    fn set_domain_caps(&mut self, caps: DomainCaps) {
        unsafe { (*(*self.0).domain_attr).caps = caps.as_raw() };
    }

    fn set_domain_mode(&mut self, mode: Mode) {
        unsafe { (*(*self.0).domain_attr).mode = mode.as_raw() };
    }

    fn set_domain_auth_key(&mut self, auth_key: &[u8]) {
        unsafe { (*(*self.0).domain_attr).auth_key_size = auth_key.len() };
        unsafe {
            (*(*self.0).domain_attr).auth_key =
                std::mem::transmute::<*const u8, *mut u8>(auth_key.as_ptr())
        };
    }

    fn set_domain_mr_count(&mut self, mr_count: usize) {
        unsafe { (*(*self.0).domain_attr).mr_cnt = mr_count };
    }

    fn set_domain_traffic_class(&mut self, traffic_class: TrafficClass) {
        unsafe { (*(*self.0).domain_attr).tclass = traffic_class.as_raw() };
    }

    fn set_fabric(&mut self, fabric: &Fabric) {
        unsafe { (*(*self.0).fabric_attr).fabric = fabric.as_typed_fid().as_raw_typed_fid() };
    }

    fn set_fabric_name(&mut self, name: &str) {
        let c_str = std::ffi::CString::new(name.to_string()).unwrap();
        unsafe { (*(*self.0).fabric_attr).name = c_str.into_raw() };
    }

    fn set_fabric_prov_name(&mut self, prov_name: &str) {
        let c_str = std::ffi::CString::new(prov_name.to_string()).unwrap();
        unsafe { (*(*self.0).fabric_attr).prov_name = c_str.into_raw() };
    }

    fn set_fabric_api_version(&mut self, api_version: &Version) {
        unsafe { (*(*self.0).fabric_attr).api_version = api_version.as_raw() };
    }

    fn set_tx_caps(&mut self, tx_caps: TxCaps) {
        unsafe { (*(*self.0).tx_attr).caps = tx_caps.as_raw() };
    }

    fn set_tx_mode(&mut self, mode: Mode) {
        unsafe { (*(*self.0).tx_attr).mode = mode.as_raw() };
    }

    fn set_tx_op_flags(&mut self, op_flags: TransferOptions) {
        unsafe { (*(*self.0).tx_attr).op_flags = op_flags.as_raw() as u64 };
    }

    fn set_tx_msg_order(&mut self, msg_order: MsgOrder) {
        unsafe { (*(*self.0).tx_attr).msg_order = msg_order.as_raw() };
    }

    fn set_tx_comp_order(&mut self, comp_order: TxCompOrder) {
        unsafe { (*(*self.0).tx_attr).comp_order = comp_order.as_raw() };
    }

    fn set_tx_inject_size(&mut self, inject_size: usize) {
        unsafe { (*(*self.0).tx_attr).inject_size = inject_size };
    }

    fn set_tx_size(&mut self, size: usize) {
        unsafe { (*(*self.0).tx_attr).size = size };
    }

    fn set_tx_iov_limit(&mut self, iov_limit: usize) {
        unsafe { (*(*self.0).tx_attr).iov_limit = iov_limit };
    }

    fn set_tx_rma_iov_limit(&mut self, rma_iov_limit: usize) {
        unsafe { (*(*self.0).tx_attr).rma_iov_limit = rma_iov_limit };
    }

    fn set_tx_traffic_class(&mut self, traffic_class: TrafficClass) {
        unsafe { (*(*self.0).tx_attr).tclass = traffic_class.as_raw() };
    }

    fn set_rx_caps(&mut self, rx_caps: RxCaps) {
        unsafe { (*(*self.0).rx_attr).caps = rx_caps.as_raw() };
    }

    fn set_rx_mode(&mut self, mode: Mode) {
        unsafe { (*(*self.0).rx_attr).mode = mode.as_raw() };
    }

    fn set_rx_op_flags(&mut self, op_flags: TransferOptions) {
        unsafe { (*(*self.0).rx_attr).op_flags = op_flags.as_raw() as u64 };
    }

    fn set_rx_msg_order(&mut self, msg_order: MsgOrder) {
        unsafe { (*(*self.0).rx_attr).msg_order = msg_order.as_raw() };
    }

    fn set_rx_comp_order(&mut self, comp_order: RxCompOrder) {
        unsafe { (*(*self.0).rx_attr).comp_order = comp_order.as_raw() };
    }

    fn set_rx_total_buffered_recv(&mut self, total_buffered_recv: usize) {
        unsafe { (*(*self.0).rx_attr).total_buffered_recv = total_buffered_recv };
    }

    fn set_rx_size(&mut self, size: usize) {
        unsafe { (*(*self.0).rx_attr).size = size };
    }

    fn set_rx_iov_limit(&mut self, iov_limit: usize) {
        unsafe { (*(*self.0).rx_attr).iov_limit = iov_limit };
    }
}

pub struct EndpointAttrIn<T> {
    hints: InfoHints<T>,
}

impl<T> EndpointAttrIn<T> {
    pub fn leave_ep_attr(self) -> InfoHints<T> {
        self.hints
    }

    pub fn type_(mut self, eptype: EndpointType) -> Self {
        self.hints.info_builder.hints_info.set_eptype(eptype);
        self
    }

    pub fn mem_tag_format(mut self, tag: u64) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_ep_mem_tag_format(tag);
        self
    }

    pub fn tx_ctx_cnt(mut self, size: usize) -> Self {
        self.hints.info_builder.hints_info.set_ep_tx_ctx_cnt(size);
        self
    }

    pub fn rx_ctx_cnt(mut self, size: usize) -> Self {
        self.hints.info_builder.hints_info.set_ep_rx_ctx_cnt(size);
        self
    }

    pub fn auth_key(mut self, key: &[u8]) -> Self {
        self.hints.info_builder.hints_info.set_ep_auth_key(key);
        self
    }
}

pub struct DomainAttrIn<T> {
    hints: InfoHints<T>,
}

impl<T> DomainAttrIn<T> {
    pub fn leave_domain_attr(self) -> InfoHints<T> {
        self.hints
    }

    pub fn domain<EQ>(mut self, name: &DomainBase<EQ>) -> Self {
        self.hints.info_builder.hints_info.set_domain(name);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.hints.info_builder.hints_info.set_domain_name(name);
        self
    }

    pub fn threading(mut self, threading: Threading) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_threading(threading);
        self
    }

    pub fn control_progress(mut self, cntrl_progress: Progress) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_control_progress(cntrl_progress);
        self
    }

    pub fn data_progress(mut self, data_progress: Progress) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_data_progress(data_progress);
        self
    }

    pub fn resource_mgmt(mut self, resource_mgmt: ResourceMgmt) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_resource_mgmt(resource_mgmt);
        self
    }

    pub fn av_type(mut self, av_type: AddressVectorType) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_av_type(av_type);
        self
    }

    pub fn mr_mode(mut self, mr_mode: MrMode) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_mr_mode(mr_mode);
        self
    }

    pub fn caps(mut self, caps: DomainCaps) -> Self {
        self.hints.info_builder.hints_info.set_domain_caps(caps);
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.hints.info_builder.hints_info.set_domain_mode(mode);
        self
    }

    pub fn auth_key(mut self, auth_key: &[u8]) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_auth_key(auth_key);
        self
    }

    pub fn mr_count(mut self, mr_count: usize) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_mr_count(mr_count);
        self
    }

    pub fn traffic_class(mut self, traffic_class: TrafficClass) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_domain_traffic_class(traffic_class);
        self
    }
}

pub struct FabricAttrIn<T> {
    hints: InfoHints<T>,
}

impl<T> FabricAttrIn<T> {
    pub fn leave_fab_attr(self) -> InfoHints<T> {
        self.hints
    }

    pub fn fabric(mut self, fabric: &Fabric) -> Self {
        self.hints.info_builder.hints_info.set_fabric(fabric);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.hints.info_builder.hints_info.set_fabric_name(name);
        self
    }

    pub fn prov_name(mut self, prov_name: &str) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_fabric_prov_name(prov_name);
        self
    }

    pub fn api_version(mut self, api_version: &Version) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_fabric_api_version(api_version);
        self
    }
}

pub struct TxAttrIn<T> {
    hints: InfoHints<T>,
}

impl<T> TxAttrIn<T> {
    pub fn leave_tx_attr(self) -> InfoHints<T> {
        self.hints
    }

    pub fn caps(mut self, tx_caps: TxCaps) -> Self {
        self.hints.info_builder.hints_info.set_tx_caps(tx_caps);
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.hints.info_builder.hints_info.set_tx_mode(mode);
        self
    }

    pub fn op_flags(mut self, op_flags: TransferOptions) -> Self {
        self.hints.info_builder.hints_info.set_tx_op_flags(op_flags);
        self
    }

    pub fn msg_order(mut self, msg_order: MsgOrder) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_tx_msg_order(msg_order);
        self
    }

    pub fn comp_order(mut self, comp_order: TxCompOrder) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_tx_comp_order(comp_order);
        self
    }

    pub fn inject_size(mut self, inject_size: usize) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_tx_inject_size(inject_size);
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.hints.info_builder.hints_info.set_tx_size(size);
        self
    }

    pub fn iov_limit(mut self, iov_limit: usize) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_tx_iov_limit(iov_limit);
        self
    }

    pub fn rma_iov_limit(mut self, rma_iov_limit: usize) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_tx_rma_iov_limit(rma_iov_limit);
        self
    }

    pub fn traffic_class(mut self, traffic_class: TrafficClass) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_tx_traffic_class(traffic_class);
        self
    }
}

pub struct RxAttrIn<T> {
    hints: InfoHints<T>,
}

impl<T> RxAttrIn<T> {
    pub fn leave_rx_attr(self) -> InfoHints<T> {
        self.hints
    }

    pub fn caps(mut self, rx_caps: RxCaps) -> Self {
        self.hints.info_builder.hints_info.set_rx_caps(rx_caps);
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.hints.info_builder.hints_info.set_rx_mode(mode);
        self
    }

    pub fn op_flags(mut self, op_flags: TransferOptions) -> Self {
        self.hints.info_builder.hints_info.set_rx_op_flags(op_flags);
        self
    }

    pub fn msg_order(mut self, msg_order: MsgOrder) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_rx_msg_order(msg_order);
        self
    }

    pub fn comp_order(mut self, comp_order: RxCompOrder) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_rx_comp_order(comp_order);
        self
    }

    pub fn total_buffered_recv(mut self, total_buffered_recv: usize) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_rx_total_buffered_recv(total_buffered_recv);
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.hints.info_builder.hints_info.set_rx_size(size);
        self
    }

    pub fn iov_limit(mut self, iov_limit: usize) -> Self {
        self.hints
            .info_builder
            .hints_info
            .set_rx_iov_limit(iov_limit);
        self
    }
}

impl<T> InfoHints<T> {
    pub fn new(info_builder: InfoBuilder<T>) -> Self {
        Self { info_builder }
    }

    pub fn caps<N: Caps>(self, _caps: N) -> InfoHints<N> {
        unsafe { (*self.info_builder.hints_info.0).caps = N::bitfield() };

        InfoHints::<N> {
            info_builder: InfoBuilder::<N> {
                hints_info: self.info_builder.hints_info,
                c_version: self.info_builder.c_version,
                c_node: self.info_builder.c_node,
                c_service: self.info_builder.c_service,
                flags: self.info_builder.flags,
                phantom: PhantomData,
            },
        }
    }
}

impl<T> InfoHints<T> {
    pub fn leave_hints(self) -> InfoBuilder<T> {
        InfoBuilder::<T> {
            hints_info: self.info_builder.hints_info,
            c_version: self.info_builder.c_version,
            c_node: self.info_builder.c_node,
            c_service: self.info_builder.c_service,
            flags: self.info_builder.flags,
            phantom: PhantomData,
        }
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.info_builder.hints_info.set_mode(mode);
        self
    }

    pub fn addr_format(mut self, addr_format: AddressFormat) -> Self {
        self.info_builder.hints_info.set_addr_format(addr_format);
        self
    }

    pub fn enter_ep_attr(self) -> EndpointAttrIn<T> {
        assert!(!unsafe { (*self.info_builder.hints_info.0).ep_attr.is_null() });
        EndpointAttrIn { hints: self }
    }

    pub fn enter_domain_attr(self) -> DomainAttrIn<T> {
        assert!(!unsafe { (*self.info_builder.hints_info.0).domain_attr.is_null() });
        DomainAttrIn { hints: self }
    }

    pub fn enter_fabric_attr(self) -> FabricAttrIn<T> {
        assert!(!unsafe { (*self.info_builder.hints_info.0).fabric_attr.is_null() });
        FabricAttrIn { hints: self }
    }

    pub fn enter_tx_attr(self) -> TxAttrIn<T> {
        assert!(!unsafe { (*self.info_builder.hints_info.0).tx_attr.is_null() });
        TxAttrIn { hints: self }
    }

    pub fn enter_rx_attr(self) -> RxAttrIn<T> {
        assert!(!unsafe { (*self.info_builder.hints_info.0).rx_attr.is_null() });
        RxAttrIn { hints: self }
    }
}

// #[derive(Clone)]
// pub struct InfoHints<T> {
//     c_info: *mut libfabric_sys::fi_info,
//     phantom: PhantomData<T>
// }

// impl InfoHints<()> {
//     pub fn new() -> Self {
//         let c_info = unsafe { libfabric_sys::inlined_fi_allocinfo() };
//         if c_info.is_null() {
//             panic!("Failed to allocate memory");
//         }
//         Self { c_info, phantom: PhantomData }
//     }

//     #[allow(unused_mut)]
//     pub fn caps<T: Caps>(mut self, _caps: T)  -> InfoHints<T> {
//         unsafe { (*self.c_info).caps = T::bitfield() };

//         InfoHints::<T> {
//             c_info: self.c_info,
//             phantom: PhantomData,
//         }
//     }
// }

// impl Default for InfoHints<()> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

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

#[derive(Clone, Copy, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl Version {
    pub(crate) fn as_raw(&self) -> u32 {
        self.major << 16 | self.minor
    }

    pub(crate) fn from_raw(raw_version: u32) -> Self {
        Self {
            major: raw_version >> 16,
            minor: raw_version & 0xffff,
        }
    }
}
