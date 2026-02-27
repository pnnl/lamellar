#[derive(Clone)]
/// Represents a Network Interface Card (NIC) in the system.
///
/// Corresponds to a `fi_nic` struct.
pub struct Nic {
    pub device_attr: Option<DeviceAttr>,
    pub bus_attr: Option<BusAttr>,
    pub link_attr: Option<LinkAttr>,
}

impl Nic {
    pub(crate) fn from_raw_ptr(fid: *const libfabric_sys::fid_nic) -> Self {
        assert!(!fid.is_null());
        let device_attr = if !unsafe { *fid }.device_attr.is_null() {
            Some(DeviceAttr::from_raw_ptr(unsafe { *fid }.device_attr))
        } else {
            None
        };

        let bus_attr = if !unsafe { *fid }.bus_attr.is_null() {
            Some(BusAttr::from_raw_ptr(unsafe { *fid }.bus_attr))
        } else {
            None
        };

        let link_attr = if !unsafe { *fid }.link_attr.is_null() {
            Some(LinkAttr::from_raw_ptr(unsafe { *fid }.link_attr))
        } else {
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
/// Represents the device attributes of a Network Interface Card (NIC).
///
/// Corresponds to a `fi_device_attr` struct.
pub struct DeviceAttr {
    pub name: Option<String>,
    pub device_id: Option<String>,
    pub device_version: Option<String>,
    pub driver: Option<String>,
    pub firmware: Option<String>,
}

impl DeviceAttr {
    pub(crate) fn from_raw_ptr(attr: *const libfabric_sys::fi_device_attr) -> Self {
        assert!(!attr.is_null());
        Self {
            name: if unsafe { *attr }.name.is_null() {
                None
            } else {
                unsafe {
                    std::ffi::CStr::from_ptr((*attr).name)
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                        .into()
                }
            },
            device_id: if unsafe { *attr }.device_id.is_null() {
                None
            } else {
                unsafe {
                    std::ffi::CStr::from_ptr((*attr).device_id)
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                        .into()
                }
            },
            device_version: if unsafe { *attr }.device_version.is_null() {
                None
            } else {
                unsafe {
                    std::ffi::CStr::from_ptr((*attr).device_version)
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                        .into()
                }
            },
            driver: if unsafe { *attr }.driver.is_null() {
                None
            } else {
                unsafe {
                    std::ffi::CStr::from_ptr((*attr).driver)
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                        .into()
                }
            },
            firmware: if unsafe { *attr }.firmware.is_null() {
                None
            } else {
                unsafe {
                    std::ffi::CStr::from_ptr((*attr).firmware)
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                        .into()
                }
            },
        }
    }
}

#[derive(Clone)]
/// Represents the link attributes of a Network Interface Card (NIC).
///
/// Corresponds to a `fi_link_attr` struct.
pub struct LinkAttr {
    pub address: Option<String>,
    pub mtu: usize,
    pub speed: usize,
    pub state: LinkState,
    pub network_type: Option<String>,
}

impl LinkAttr {
    pub(crate) fn from_raw_ptr(attr: *const libfabric_sys::fi_link_attr) -> Self {
        assert!(!attr.is_null());

        Self {
            address: if unsafe { *attr }.address.is_null() {
                None
            } else {
                unsafe {
                    std::ffi::CStr::from_ptr((*attr).address)
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                        .into()
                }
            },
            mtu: unsafe { *attr }.mtu,
            speed: unsafe { *attr }.speed,
            state: LinkState::from_raw(unsafe { *attr }.state),
            network_type: if unsafe { *attr }.network_type.is_null() {
                None
            } else {
                unsafe {
                    std::ffi::CStr::from_ptr((*attr).network_type)
                        .to_str()
                        .unwrap_or("")
                        .to_owned()
                        .into()
                }
            },
        }
    }
}

#[derive(Clone)]
/// Represents the link status of a Network Interface Card (NIC).
///
/// Corresponds to a `fi_link_state` enum.
pub enum LinkState {
    Unknown,
    Down,
    Up,
}

impl LinkState {
    pub(crate) fn from_raw(val: libfabric_sys::fi_link_state) -> Self {
        if val == libfabric_sys::fi_link_state_FI_LINK_UNKNOWN {
            LinkState::Unknown
        } else if val == libfabric_sys::fi_link_state_FI_LINK_DOWN {
            LinkState::Down
        } else if val == libfabric_sys::fi_link_state_FI_LINK_UP {
            LinkState::Up
        } else {
            panic!("Unexpected link state");
        }
    }
}

#[derive(Clone)]
/// Represents the bus attributes of a Network Interface Card (NIC).
///
/// Corresponds to a `fi_bus_attr` struct.
pub struct BusAttr {
    pub bus_type: BusType,
    pub pci: PciAttr,
}

impl BusAttr {
    pub(crate) fn from_raw_ptr(attr: *const libfabric_sys::fi_bus_attr) -> Self {
        assert!(!attr.is_null());
        Self {
            bus_type: BusType::from_raw(unsafe { *attr }.bus_type),
            pci: PciAttr::from_attr(unsafe { (*attr).attr.pci }),
        }
    }
}

#[derive(Clone)]
/// Represents the bus type of the Network Interface Card (NIC) bus.
///
/// Corresponds to a `fi_bus_type` enum.
pub enum BusType {
    Pci,
    Unknown,
    Unspec,
}

impl BusType {
    pub(crate) fn from_raw(val: libfabric_sys::fi_bus_type) -> Self {
        if val == libfabric_sys::fi_bus_type_FI_BUS_UNKNOWN {
            BusType::Unknown
        } else if val == libfabric_sys::fi_bus_type_FI_BUS_PCI {
            BusType::Pci
        } else if val == libfabric_sys::fi_bus_type_FI_BUS_UNSPEC {
            BusType::Unspec
        } else {
            panic!("Unexpected link state");
        }
    }
}

#[derive(Clone)]
/// Represents the PCI attributes of a Network Interface Card (NIC).
///
/// Corresponds to a `fi_pci_attr` struct.
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
