use std::marker::PhantomData;

use crate::{
    conn_ep::{ConnectionPendingEndpointBase, EnabledConnectionOrientedEndpoint},
    ep::{
        Address, Connected, EndpointBase, EndpointImplBase, PendingAccept, Unconnected,
        UninitUnconnected,
    },
    eq::{ConnectedEvent, Event, EventQueueBase},
    fid::{AsRawFid, AsRawTypedFid, AsTypedFid, Fid},
    utils::check_error,
};

use super::{cq::AsyncReadCq, eq::AsyncReadEq};

pub type UninitUnconnectedEndpointBase<EP> = EndpointBase<EP, UninitUnconnected>;

pub type UninitUnconnectedEndpoint<T> =
    UninitUnconnectedEndpointBase<EndpointImplBase<T, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub type UnconnectedEndpointBase<EP> = EndpointBase<EP, Unconnected>;

pub type UnconnectedEndpoint<T> =
    UnconnectedEndpointBase<EndpointImplBase<T, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub type AcceptPendingEndpointBase<EP> = EndpointBase<EP, PendingAccept>;

pub type AcceptPendingEndpoint<T> =
    AcceptPendingEndpointBase<EndpointImplBase<T, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub trait ConnectedEp {}

pub type ConnectedEndpointBase<EP> = EndpointBase<EP, Connected>;

pub type ConnectedEndpoint<T> =
    ConnectedEndpointBase<EndpointImplBase<T, dyn AsyncReadEq, dyn AsyncReadCq>>;

impl<EP> ConnectedEp for ConnectedEndpointBase<EP> {}

impl<EP> UnconnectedEndpoint<EP> {
    pub async fn connect_async(
        self,
        addr: &Address,
    ) -> Result<ConnectedEndpoint<EP>, crate::error::Error> {
        let inner = self.inner.clone();

        let eq = inner.eq.get().expect("Endpoint not bound to an EventQueue");

        let fid = Fid(self.as_typed_fid().as_raw_fid() as usize);
        let pending = self.connect(addr)?;

        let res = eq
            .async_event_wait(libfabric_sys::FI_CONNECTED, fid, None, None)
            .await?;

        let event = match res {
            Event::Connected(event) => event,
            _ => panic!("Unexpected Event Type"),
        };

        Ok(pending.connect_complete(event))
    }
}

impl<EP> AcceptPendingEndpoint<EP> {
    pub async fn accept_async(self) -> Result<ConnectedEndpoint<EP>, crate::error::Error> {
        let inner = self.inner.clone();

        let eq = inner.eq.get().expect("Endpoint not bound to an EventQueue");

        let fid = Fid(self.as_typed_fid().as_raw_fid() as usize);
        let pending = self.accept()?;

        let res = eq
            .async_event_wait(libfabric_sys::FI_CONNECTED, fid, None, None)
            .await?;

        let event = match res {
            Event::Connected(event) => event,
            _ => panic!("Unexpected Event Type"),
        };

        Ok(pending.connect_complete(event))
    }
}

impl<E> UninitUnconnectedEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>> {
    pub fn enable<EQ: AsyncReadEq + 'static>(
        self,
        eq: &EventQueueBase<EQ>,
    ) -> Result<
        EnabledConnectionOrientedEndpoint<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>>,
        crate::error::Error,
    > {
        self.bind_eq(eq)?;
        let err =
            unsafe { libfabric_sys::inlined_fi_enable(self.as_typed_fid_mut().as_raw_typed_fid()) };
        check_error(err.try_into().unwrap())?;
        if self.inner.has_conn_req {
            Ok(EnabledConnectionOrientedEndpoint::AcceptPending(
                AcceptPendingEndpoint {
                    inner: self.inner.clone(),
                    phantom: PhantomData,
                },
            ))
        } else {
            Ok(EnabledConnectionOrientedEndpoint::Unconnected(
                UnconnectedEndpoint {
                    inner: self.inner.clone(),
                    phantom: PhantomData,
                },
            ))
        }
    }
}

pub type ConnectionPendingEndpoint<T> =
    ConnectionPendingEndpointBase<EndpointImplBase<T, dyn AsyncReadEq, dyn AsyncReadCq>>;
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
