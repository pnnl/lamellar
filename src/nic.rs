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