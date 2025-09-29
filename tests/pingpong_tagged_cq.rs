
pub mod sync_;
mod pp_sizes;
pub mod sync_tagged {
    use libfabric::infocapsoptions::InfoCaps;

    use crate::{pp_sizes::{IP, TEST_SIZES}, sync_::tests::{get_ip, Ofi, TestConfigBuilder}};

    #[test]
    fn connected_pp_server_tagged_cq() {

        let caps = InfoCaps::new().msg().tagged();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();


        for size in TEST_SIZES {
            info.pingpong_tagged(10, 100, size);
        }
    }

    #[test]
    fn connected_pp_client_tagged_cq() {

        let ip = get_ip(IP);
        let caps = InfoCaps::new().msg().tagged();
        let mut config = TestConfigBuilder::new(Some(&ip), None, false, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();

        for size in TEST_SIZES {
            info.pingpong_tagged(10, 100, size);
        }

    }


    #[test]
    fn pp_server_tagged_cq() {

        let caps = InfoCaps::new().msg().tagged();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();

        for size in TEST_SIZES {
            info.pingpong_tagged(10, 100, size);
        }

    }

    #[test]
    fn pp_client_tagged_cq() {

        let caps = InfoCaps::new().msg().tagged();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();

        for size in TEST_SIZES {
            info.pingpong_tagged(10, 100, size);
        }
    }
}

pub mod async_;

pub mod async_tagged {
    use libfabric::infocapsoptions::InfoCaps;

    use crate::{async_::{Ofi, TestConfigBuilder}, pp_sizes::{IP, TEST_SIZES}, sync_::tests::get_ip};


    #[test]
    fn connected_pp_server_tagged_cq() {

        let caps = InfoCaps::new().msg().tagged();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();


        for size in TEST_SIZES {
            info.pingpong_tagged(10, 100, size);
        }
    }

    #[test]
    fn connected_pp_client_tagged_cq() {

        let caps = InfoCaps::new().msg().tagged();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();

        for size in TEST_SIZES {
            info.pingpong_tagged(10, 100, size);
        }

    }


    #[test]
    fn pp_server_tagged_cq() {

        let caps = InfoCaps::new().msg().tagged();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();

        for size in TEST_SIZES {
            info.pingpong_tagged(10, 100, size);
        }

    }

    #[test]
    fn pp_client_tagged_cq() {

        let caps = InfoCaps::new().msg().tagged();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();

        for size in TEST_SIZES {
            info.pingpong_tagged(10, 100, size);
        }
    }
}