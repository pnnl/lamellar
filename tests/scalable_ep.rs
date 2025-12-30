use libfabric::{ep::EndpointBuilder, infocapsoptions::InfoCaps};

use crate::sync_::tests::{get_ip, Ofi, TestConfigBuilder};
pub mod sync_;

#[test]
fn enable_scalable_endpoint1() {
    let caps = InfoCaps::new().msg();
    let mut config = TestConfigBuilder::new(Some(&get_ip(None)), None, false, caps, libfabric::enums::EndpointType::Rdm);
    config.name = "sep0".to_string();
    let config = config.build(
        |entry| {
            println!("{}", entry.fabric_attr().prov_name());
            !entry.fabric_attr().prov_name().contains("verbs")
        });
    let info = Ofi::new(config).unwrap();
    let sep = EndpointBuilder::new(&info.info_entry)
        .build_scalable(&info.domain)
        .unwrap();
    sep.bind_av(&info.av.unwrap()).unwrap();
    sep.enable().unwrap();
}
#[test]
fn enable_scalable_endpoint0() {
    let caps = InfoCaps::new().msg();
    let mut config = TestConfigBuilder::new(Some(&get_ip(None)), None, true, caps, libfabric::enums::EndpointType::Rdm);
    config.name = "sep0".to_string();
    
    let config = config.build(
        |entry| {
            println!("{}", entry.fabric_attr().prov_name());
            !entry.fabric_attr().prov_name().contains("verbs")
        });
        let info = Ofi::new(config).unwrap();
        let sep = EndpointBuilder::new(&info.info_entry)
        .build_scalable(&info.domain)
        .unwrap();
    sep.bind_av(&info.av.unwrap()).unwrap();
    sep.enable().unwrap();
}
