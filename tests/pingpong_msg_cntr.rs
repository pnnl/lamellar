use libfabric::infocapsoptions::InfoCaps;

use crate::{pp_sizes::{IP, TEST_SIZES}, sync_::tests::{get_ip, CntrsCompMeth, CqsCompMeth, Ofi, TestConfigBuilder}};
pub mod sync_;
pub mod pp_sizes;

/// Connected + Counter is not supported by provider
// #[test]
// fn connected_pp_server_msg_cntr() {

//     let caps = InfoCaps::new().msg();
//     let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Msg);
//     config.buf_size = 1 << 23;
//     config.use_cqs_for_completion = CqsCompMeth::None;
//     config.use_cntrs_for_completion = CntrsCompMeth::Spin;

//     let config = config.build(|_| true);
//     let info = Ofi::new(config).unwrap();


//     for size in TEST_SIZES {
//         info.pingpong(10, 100, true, size);
//     }
// }

// #[test]
// fn connected_pp_client_msg_cntr() {

//     let caps = InfoCaps::new().msg();
//     // let info = handshake(Some(&get_ip(IP)), false, "pp_msg", Some(caps), 1 << 23);
//     let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Msg);
//     config.buf_size = 1 << 23;
//     config.use_cqs_for_completion = CqsCompMeth::None;
//     config.use_cntrs_for_completion = CntrsCompMeth::Spin;

//     let config = config.build(|_| true);
//     let info = Ofi::new(config).unwrap();
    
//     for size in TEST_SIZES {
//         info.pingpong(10, 100, false, size);
//     }

// }


#[test]
fn pp_server_msg_cntr() {

    let caps = InfoCaps::new().msg();
    let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Rdm);
    config.buf_size = 1 << 23;
    config.use_cqs_for_completion = CqsCompMeth::None;
    config.use_cntrs_for_completion = CntrsCompMeth::Spin;

    let config = config.build(|_| true);
    let info = Ofi::new(config).unwrap();
    // let info = handshake_connectionless(Some(&get_ip(IP)), true, "pp_msg", Some(caps), 1 << 23);

    for size in TEST_SIZES {
        info.pingpong(10, 100, size);
    }

}

#[test]
fn pp_client_msg_cntr() {

    let caps = InfoCaps::new().msg();
    let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Rdm);
    config.buf_size = 1 << 23;
    config.use_cqs_for_completion = CqsCompMeth::None;
    config.use_cntrs_for_completion = CntrsCompMeth::Spin;

    let config = config.build(|_| true);
    let info = Ofi::new(config).unwrap();
    // let info = handshake_connectionless(Some(&get_ip(IP)),false, "pp_msg", Some(caps), 1 << 23);

    for size in TEST_SIZES {
        info.pingpong(10, 100, size);
    }
}

