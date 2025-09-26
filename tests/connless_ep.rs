pub mod sync_;
pub mod sync_connless_ep {
    use libfabric::infocapsoptions::InfoCaps;
    use crate::sync_::{handshake_connectionless, DEFAULT_BUF_SIZE};

    #[test]
    fn handshake_connectionless0() {
        handshake_connectionless(
            None,
            true,
            "handshake_connectionless0",
            Some(InfoCaps::new().msg()),
            DEFAULT_BUF_SIZE
        );
    }

    #[test]
    fn handshake_connectionless1() {
        handshake_connectionless(
            None,
            false,
            "handshake_connectionless0",
            Some(InfoCaps::new().msg()),
            DEFAULT_BUF_SIZE
        );
    }

}

pub mod async_;
pub mod async_connless_ep {
    use libfabric::infocapsoptions::InfoCaps;
    use crate::async_::handshake_connectionless;

    #[test]
    fn async_handshake_connectionless0() {
        handshake_connectionless(
            true,
            "handshake_connectionless0",
            Some(InfoCaps::new().msg()),
        );
    }

    #[test]
    fn async_handshake_connectionless1() {
        handshake_connectionless(
            false,
            "handshake_connectionless0",
            Some(InfoCaps::new().msg()),
        );
    }

}
