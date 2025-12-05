
pub mod sync_;
#[cfg(test)]
pub mod sync_conn_ep {
    use libfabric::infocapsoptions::InfoCaps;

    use crate::sync_::tests::handshake;

    #[test]
    fn handshake_connected0() {
        handshake(None, true, "handshake_connected0", Some(InfoCaps::new().msg()));
    }

    #[test]
    fn handshake_connected1() {
        handshake(None, false, "handshake_connected0", Some(InfoCaps::new().msg()));
    }
}

pub mod async_;
#[cfg(test)]
pub mod async_conn_ep {
    use libfabric::infocapsoptions::InfoCaps;

    use crate::async_::handshake;

    #[test]
    fn async_handshake_connected0() {
        handshake(None, true, "handshake_connected0", Some(InfoCaps::new().msg()));
    }

    #[test]
    fn async_handshake_connected1() {
        handshake(None, false, "handshake_connected0", Some(InfoCaps::new().msg()));
    }
}