use libfabric::infocapsoptions::InfoCaps;

use crate::{pp_sizes::{IP, TEST_SIZES}, sync_::tests::{get_ip, CntrsCompMeth, CqsCompMeth, Ofi, TestConfigBuilder}};
pub mod sync_;
mod pp_sizes;

// #[test]
// fn connected_pp_server_tagged_cntr() {

//     let caps = InfoCaps::new().msg().tagged();
//     let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Msg);
//     config.buf_size = 1 << 23;
//     config.use_cntrs_for_completion = CntrsCompMeth::Spin;
//     config.use_cqs_for_completion = CqsCompMeth::None;

//     let config = config.build(|_| true);

//     let info = Ofi::new(config).unwrap();


//     for size in TEST_SIZES {
//         info.pingpong_tagged(10, 100, size);
//     }
// }

// #[test]
// fn connected_pp_client_tagged_cntr() {

//     let caps = InfoCaps::new().msg().tagged();
//     let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Msg);
//     config.buf_size = 1 << 23;
//     config.use_cntrs_for_completion = CntrsCompMeth::Spin;
//     config.use_cqs_for_completion = CqsCompMeth::None;

//     let config = config.build(|_| true);

//     let info = Ofi::new(config).unwrap();

//     for size in TEST_SIZES {
//         info.pingpong_tagged(10, 100, size);
//     }

// }


#[test]
fn pp_server_tagged_cntr() {

    let caps = InfoCaps::new().msg().tagged();
    let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Rdm);
    config.buf_size = 1 << 23;
    config.use_cntrs_for_completion = CntrsCompMeth::Spin;
    config.use_cqs_for_completion = CqsCompMeth::None;

    let config = config.build(|_| true);
    let info = Ofi::new(config).unwrap();

    for size in TEST_SIZES {
        info.pingpong_tagged(10, 100, size);
    }

}

#[test]
fn pp_client_tagged_cntr() {

    let caps = InfoCaps::new().msg().tagged();
    let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Rdm);
    config.buf_size = 1 << 23;
    config.use_cntrs_for_completion = CntrsCompMeth::Spin;
    config.use_cqs_for_completion = CqsCompMeth::None;

    let config = config.build(|_| true);
    let info = Ofi::new(config).unwrap();

    for size in TEST_SIZES {
        info.pingpong_tagged(10, 100, size);
    }
}