pub mod sync_;
pub mod pp_sizes;

pub mod sync_msg {
    
    use libfabric::infocapsoptions::InfoCaps;
    use crate::{pp_sizes::{IP, TEST_SIZES}, sync_::tests::{get_ip, Ofi, TestConfigBuilder}};

    #[test]
    fn connected_pp_server_msg_cq() {

        let caps = InfoCaps::new().msg();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;

        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();


        for size in TEST_SIZES {
            info.pingpong(10, 100, size);
        }
    }

    #[test]
    fn connected_pp_client_msg_cq() {

        let caps = InfoCaps::new().msg();
        // let info = handshake(Some("172.17.110.4"), false, "pp_msg", Some(caps), 1 << 23);
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        
        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();
        
        for size in TEST_SIZES {
            info.pingpong(10, 100, size);
        }

    }


    #[test]
    fn pp_server_msg_cq() {

        let caps = InfoCaps::new().msg();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;


        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();
        // let info = handshake_connectionless(Some("172.17.110.4"), true, "pp_msg", Some(caps), 1 << 23);

        for size in TEST_SIZES {
            info.pingpong(10, 100, size);
        }

    }

    #[test]
    fn pp_client_msg_cq() {

        let caps = InfoCaps::new().msg();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Rdm);
        config.buf_size = 1 << 23;


        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();
        // let info = handshake_connectionless(Some("172.17.110.4"),false, "pp_msg", Some(caps), 1 << 23);

        for size in TEST_SIZES {
            info.pingpong(10, 100, size);
        }
    }
}

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_;


#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_msg {
    use libfabric::infocapsoptions::InfoCaps;

    use crate::{async_::{Ofi, TestConfigBuilder}, pp_sizes::{IP, TEST_SIZES}, sync_::tests::get_ip};


    #[test]
    fn connected_pp_server_msg_cq() {

        let caps = InfoCaps::new().msg();
        let mut config = TestConfigBuilder::new(None, None, true, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        

        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();


        for size in TEST_SIZES {
            info.pingpong(10, 100, size);
        }
    }

    #[test]
    fn connected_pp_client_msg_cq() {

        let caps = InfoCaps::new().msg();
        let mut config = TestConfigBuilder::new(Some(&get_ip(IP)), None, false, caps, libfabric::enums::EndpointType::Msg);
        config.buf_size = 1 << 23;
        

        let config = config.build(|_| true);
        let info = Ofi::new(config).unwrap();


        for size in TEST_SIZES {
            info.pingpong(10, 100, size);
        }
    }
    
}