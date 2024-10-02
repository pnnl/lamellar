use std::marker::PhantomData;

use crate::{
    ep::{Address, Connected, EndpointBase, EndpointImplBase, Unconnected},
    eq::Event,
    fid::{AsRawFid, Fid},
};

use super::{cq::AsyncReadCq, eq::AsyncReadEq};

pub type UnconnectedEndpointBase<EP> = EndpointBase<EP, Unconnected>;

pub type UnconnectedEndpoint<T> =
    UnconnectedEndpointBase<EndpointImplBase<T, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub trait ConnectedEp {}

pub type ConnectedEndpointBase<EP> = EndpointBase<EP, Connected>;

pub type ConnectedEndpoint<T> =
    ConnectedEndpointBase<EndpointImplBase<T, dyn AsyncReadEq, dyn AsyncReadCq>>;

impl<EP> ConnectedEp for ConnectedEndpointBase<EP> {}

impl<EP> UnconnectedEndpoint<EP> {
    pub async fn connect_async(
        &self,
        addr: &Address,
    ) -> Result<ConnectedEndpoint<EP>, crate::error::Error> {
        self.connect(addr)?;

        let eq = self
            .inner
            .eq
            .get()
            .expect("Endpoint not bound to an EventQueue");
        let res = eq
            .async_event_wait(libfabric_sys::FI_CONNECTED, Fid(self.as_raw_fid()), 0)
            .await?;

        match res {
            Event::Connected(event) => {
                assert_eq!(event.get_fid(), self.as_raw_fid())
            }
            _ => panic!("Unexpected Event Type"),
        }

        Ok(ConnectedEndpoint {
            inner: self.inner.clone(),
            phantom: PhantomData,
        })
    }

    pub async fn accept_async(&self) -> Result<ConnectedEndpoint<EP>, crate::error::Error> {
        self.accept()?;

        let eq = self
            .inner
            .eq
            .get()
            .expect("Endpoint not bound to an EventQueue");
        let res = eq
            .async_event_wait(libfabric_sys::FI_CONNECTED, Fid(self.as_raw_fid()), 0)
            .await?;

        match res {
            Event::Connected(event) => {
                assert_eq!(event.get_fid(), self.as_raw_fid())
            }
            _ => panic!("Unexpected Event Type"),
        }

        Ok(ConnectedEndpoint {
            inner: self.inner.clone(),
            phantom: PhantomData,
        })
    }
}
