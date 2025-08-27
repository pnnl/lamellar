use std::marker::PhantomData;

use crate::{
    cq::ReadCq,
    ep::{Address, Connected, EndpointBase, EndpointImplBase, PendingAccept, Unconnected, UninitUnconnected},
    eq::{ConnectedEvent, EventQueueBase, ReadEq},
    fid::{AsRawFid, AsRawTypedFid, AsTypedFid, EpRawFid},
    utils::check_error, MyRc,
};

/// A connection-oriented endpoint that is not yet enabled.
pub type UninitUnconnectedEndpointBase<EP> = EndpointBase<EP, UninitUnconnected>;

pub type UninitUnconnectedEndpoint<T> =
    UninitUnconnectedEndpointBase<EndpointImplBase<T, dyn ReadEq, dyn ReadCq>>;

/// A connection-oriented endpoint that is not yet connected, but is enabled.
pub type UnconnectedEndpointBase<EP> = EndpointBase<EP, Unconnected>;

/// A connection-oriented endpoint that is waiting to accept a remote connection.
pub type AcceptPendingEndpointBase<EP> = EndpointBase<EP, PendingAccept>;

pub type UnconnectedEndpoint<T> =
    UnconnectedEndpointBase<EndpointImplBase<T, dyn ReadEq, dyn ReadCq>>;

pub type AcceptPendingEndpoint<T> =
    AcceptPendingEndpointBase<EndpointImplBase<T, dyn ReadEq, dyn ReadCq>>;


pub enum EnabledConnectionOrientedEndpoint<EP> {
    Unconnected(UnconnectedEndpointBase<EP>),
    AcceptPending(AcceptPendingEndpointBase<EP>),
}

impl<EP> EnabledConnectionOrientedEndpoint<EP> {
    pub(crate) fn inner(&self) -> MyRc<EP> {
        match self {
            EnabledConnectionOrientedEndpoint::Unconnected(ep) => ep.inner.clone(),
            EnabledConnectionOrientedEndpoint::AcceptPending(ep) => ep.inner.clone(),
        }
    }
}

impl<EP: AsTypedFid<EpRawFid>> AsRawTypedFid for EnabledConnectionOrientedEndpoint<EP> {
    type Output = EpRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        match self {
            EnabledConnectionOrientedEndpoint::Unconnected(ep) => ep.as_typed_fid().as_raw_typed_fid(),
            EnabledConnectionOrientedEndpoint::AcceptPending(ep) => ep.as_typed_fid().as_raw_typed_fid(),
        }
    }
}

impl<E> UninitUnconnectedEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>> {

    /// Enables the endpoint and binds it to the specified event queue.
    /// After enabling, the endpoint can be used to initiate or accept connections.
    ///
    /// Corresponds to `fi_bind` followed by `fi_enable` in libfabric.
    pub fn enable<EQ: ReadEq + 'static>(self, eq: &EventQueueBase<EQ>) -> Result<EnabledConnectionOrientedEndpoint<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>, crate::error::Error> {
        self.bind_eq(eq)?;
        let err =
            unsafe { libfabric_sys::inlined_fi_enable(self.as_typed_fid_mut().as_raw_typed_fid()) };
        check_error(err.try_into().unwrap())?;

        if self.inner.has_conn_req {
            Ok(EnabledConnectionOrientedEndpoint::AcceptPending(AcceptPendingEndpointBase::<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>> {
                inner: self.inner.clone(),
                phantom: PhantomData,
            }))
        }
        else {
            Ok(EnabledConnectionOrientedEndpoint::Unconnected(UnconnectedEndpointBase::<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>> {
                inner: self.inner.clone(),
                phantom: PhantomData,
            }))
        }
    }
}



impl<EP: AsTypedFid<EpRawFid>> AcceptPendingEndpointBase<EP> {
    
    // [TODO] Should only be available when ep was created from a connection request Info
    /// Accepts an incoming connection request with additional parameters.
    /// 
    /// Corrsponds to `fi_accept` in libfabric.
    pub fn accept_with<T0>(self, param: &[T0]) -> Result<ConnectionPendingEndpointBase<EP>, crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_accept(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                param.as_ptr().cast(),
                param.len(),
            )
        };
        
        check_error(err.try_into().unwrap())?;

        Ok(ConnectionPendingEndpointBase {
            inner: EnabledConnectionOrientedEndpoint::AcceptPending(self)
        })
    }
    
    /// Accepts an incoming connection request.
    /// 
    /// Corrsponds to `fi_accept` in libfabric without the param argument.
    pub fn accept(self) -> Result<ConnectionPendingEndpointBase<EP>, crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_accept(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                std::ptr::null_mut(),
                0,
            )
        };
        check_error(err.try_into().unwrap())?;
        Ok(ConnectionPendingEndpointBase {
            inner: EnabledConnectionOrientedEndpoint::AcceptPending(self)
        })
    }
}


impl<EP: AsTypedFid<EpRawFid>> UnconnectedEndpointBase<EP> {
    
    /// Initiates a connection to a remote endpoint specified by `addr`, with additional parameters.
    /// 
    /// Corrsponds to `fi_connect` in libfabric.
    pub fn connect_with<T>(self, addr: &Address, param: &[T]) -> Result<ConnectionPendingEndpointBase<EP>, crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_connect(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                addr.as_bytes().as_ptr().cast(),
                param.as_ptr().cast(),
                param.len(),
            )
        };

        check_error(err.try_into().unwrap())?;

        Ok(ConnectionPendingEndpointBase {
            inner: EnabledConnectionOrientedEndpoint::Unconnected(self)
        })
    }

    /// Initiates a connection to a remote endpoint specified by `addr`.
    /// 
    /// Corrsponds to `fi_connect` in libfabric without the param argument.
    pub fn connect(self, addr: &Address) -> Result<ConnectionPendingEndpointBase<EP>, crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_connect(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                addr.as_bytes().as_ptr().cast(),
                std::ptr::null_mut(),
                0,
            )
        };

        check_error(err.try_into().unwrap())?;


        Ok(ConnectionPendingEndpointBase {
            inner: EnabledConnectionOrientedEndpoint::Unconnected(self)
        })
    }

}

// pub struct ConnectionPendingEndpoint<EP> {
//     inner: EnabledConnectionOrientedEndpoint<EP>,
// }


pub struct ConnectionPendingEndpointBase<EP> {
    pub(crate) inner: EnabledConnectionOrientedEndpoint<EP>,
}

pub type ConnectionPendingEndpoint<T> = ConnectionPendingEndpointBase<EndpointImplBase<T, dyn ReadEq, dyn ReadCq>>;
impl<E> ConnectionPendingEndpoint<E> {

    /// Completes the connection process using the provided `ConnectedEvent` and returns a [ConnectedEndpoint] ready for use.
    /// This method asserts that the event's fid matches the endpoint's fid.
    /// 
    /// # Panics
    /// Panics if the event's fid does not match the endpoint's fid.
    pub fn connect_complete(self, event: ConnectedEvent) -> ConnectedEndpoint<E> {
        
        assert_eq!(event.fid(), self.inner.as_raw_typed_fid().as_raw_fid());

        ConnectedEndpoint {
            inner: self.inner.inner(),
            phantom: PhantomData,
        }
    }

    /// Same as [connect_complete](Self::connect_complete) but does not check that the event's fid matches the endpoint's fid.
    pub unsafe fn connect_complete_unchecked(self, _event: ConnectedEvent) -> ConnectedEndpoint<E> {

        ConnectedEndpoint {
            inner: self.inner.inner(),
            phantom: PhantomData,
        }
    }
}

/// A trait for connection-oriented endpoints that are in the connected state.
pub trait ConnectedEp {}

/// A connection-oriented endpoint that is in the connected state.
pub type ConnectedEndpointBase<EP> = EndpointBase<EP, Connected>;

pub type ConnectedEndpoint<T> = ConnectedEndpointBase<EndpointImplBase<T, dyn ReadEq, dyn ReadCq>>;

impl<EP> ConnectedEp for ConnectedEndpointBase<EP> {}

impl<EP: AsTypedFid<EpRawFid>> ConnectedEndpointBase<EP> {

    // [TODO]: Should this consume self and return an UnconnectedEndpoint?
    /// Shuts down the connection associated with the endpoint.
    /// 
    /// After calling this method, the endpoint will no longer be able to send or receive data.
    /// Corresponds to `fi_shutdown` in libfabric.
    pub fn shutdown(&self) -> Result<UnconnectedEndpointBase<EP>, crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_shutdown(self.as_typed_fid_mut().as_raw_typed_fid(), 0)
        };

        check_error(err.try_into().unwrap())?;

        Ok(UnconnectedEndpointBase {
            inner: self.inner.clone(),
            phantom: PhantomData,
        })
    }

    /// Retrieves the address of the remote peer connected to this endpoint.
    /// 
    /// Corresponds to `fi_getpeer` in libfabric.
    pub fn peer(&self) -> Result<Address, crate::error::Error> {
        let mut len = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_getpeer(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                std::ptr::null_mut(),
                &mut len,
            )
        };

        if -err as u32 == libfabric_sys::FI_ETOOSMALL {
            let mut address = vec![0; len];
            let err = unsafe {
                libfabric_sys::inlined_fi_getpeer(
                    self.as_typed_fid_mut().as_raw_typed_fid(),
                    address.as_mut_ptr().cast(),
                    &mut len,
                )
            };
            if err != 0 {
                Err(crate::error::Error::from_err_code(
                    (-err).try_into().unwrap(),
                ))
            } else {
                Ok(Address { address })
            }
        } else {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        }
    }
}