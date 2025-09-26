
pub mod sync_;
#[cfg(test)]
pub mod sync_conn_ep {
    use libfabric::infocapsoptions::InfoCaps;

    use crate::sync_::{handshake, DEFAULT_BUF_SIZE};

    #[test]
    fn handshake_connected0() {
        handshake(None, true, "handshake_connected0", Some(InfoCaps::new().msg()), DEFAULT_BUF_SIZE);
    }

    #[test]
    fn handshake_connected1() {
        handshake(None, false, "handshake_connected0", Some(InfoCaps::new().msg()), DEFAULT_BUF_SIZE);
    }
}

pub mod async_;
#[cfg(test)]
pub mod async_conn_ep {
    use libfabric::infocapsoptions::InfoCaps;

    use crate::async_::handshake;

    #[test]
    fn async_handshake_connected0() {
        handshake(true, "handshake_connected0", Some(InfoCaps::new().msg()));
    }

    #[test]
    fn async_handshake_connected1() {
        handshake(false, "handshake_connected0", Some(InfoCaps::new().msg()));
    }
}