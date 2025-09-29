pub mod sync_;
mod pp_sizes;


pub mod sync_rma {
    use libfabric::infocapsoptions::InfoCaps;
    
    use crate::{pp_sizes::{IP, TEST_SIZES, WINDOW_SIZE}, sync_::tests::{get_ip, Ofi, TestConfigBuilder}};
    #[test]
    fn connected_pp_server_rma_cq() {

        let caps = InfoCaps::new().msg().rma();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let mut info = Ofi::new(config).unwrap();
        // let mut info = handshake(Some(&get_ip(IP)), true, "connected_pp_rma", Some(caps), 1 << 23);
        info.exchange_keys();

        for size in TEST_SIZES {
            info.pingpong_rma(10, 100, size, WINDOW_SIZE);
        }
    }

    #[test]
    fn connected_pp_client_rma_cq() {

        let caps = InfoCaps::new().msg().rma();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let mut info = Ofi::new(config).unwrap();
        // let mut info = handshake(Some(&get_ip(IP)), false, "connected_pp_rma", Some(caps), 1 << 23);
        info.exchange_keys();

        for size in TEST_SIZES {
            info.pingpong_rma(10, 100, size, WINDOW_SIZE);
        }
    }


    #[test]
    fn pp_server_rma_cq() {

        let caps = InfoCaps::new().msg().rma();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let mut info = Ofi::new(config).unwrap();
        // let mut info = handshake_connectionless(Some(&get_ip(IP)), true, "pp_rma", Some(caps), 1 << 23);
        info.exchange_keys();

        for size in TEST_SIZES {
            info.pingpong_rma(10, 100, size, WINDOW_SIZE);
        }

    }

    #[test]
    fn pp_client_rma_cq() {

        let caps = InfoCaps::new().msg().rma();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let mut info = Ofi::new(config).unwrap();
        // let mut info = handshake_connectionless(Some(&get_ip(IP)), false, "pp_rma", Some(caps), 1 << 23);
        info.exchange_keys();

        for size in TEST_SIZES {
            info.pingpong_rma(10, 100, size, WINDOW_SIZE);
        }
    }
}

pub mod async_;

pub mod async_rma {
    use libfabric::infocapsoptions::InfoCaps;

    use crate::{async_::{Ofi, TestConfigBuilder}, pp_sizes::{IP, TEST_SIZES, WINDOW_SIZE}, sync_::tests::get_ip};
    
    #[test]
    fn connected_pp_server_rma_cq() {

        let caps = InfoCaps::new().msg().rma();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let mut info = Ofi::new(config).unwrap();
        // let mut info = handshake(Some(&get_ip(IP)), true, "connected_pp_rma", Some(caps), 1 << 23);
        info.exchange_keys();

        for size in TEST_SIZES {
            info.pingpong_rma(10, 100, size, WINDOW_SIZE);
        }
    }

    #[test]
    fn connected_pp_client_rma_cq() {

        let caps = InfoCaps::new().msg().rma();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let mut info = Ofi::new(config).unwrap();
        // let mut info = handshake(Some(&get_ip(IP)), false, "connected_pp_rma", Some(caps), 1 << 23);
        info.exchange_keys();

        for size in TEST_SIZES {
            info.pingpong_rma(10, 100, size, WINDOW_SIZE);
        }

    }


    #[test]
    fn pp_server_rma_cq() {

        let caps = InfoCaps::new().msg().rma();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let mut info = Ofi::new(config).unwrap();
        // let mut info = handshake_connectionless(Some(&get_ip(IP)), true, "pp_rma", Some(caps), 1 << 23);
        info.exchange_keys();

        for size in TEST_SIZES {
            info.pingpong_rma(10, 100, size, WINDOW_SIZE);
        }

    }

    #[test]
    fn pp_client_rma_cq() {

        let caps = InfoCaps::new().msg().rma();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;
        let config = config.build(|_| true);
        let mut info = Ofi::new(config).unwrap();
        // let mut info = handshake_connectionless(Some(&get_ip(IP)), false, "pp_rma", Some(caps), 1 << 23);
        info.exchange_keys();

        for size in TEST_SIZES {
            info.pingpong_rma(10, 100, size, WINDOW_SIZE);
        }
    }
}