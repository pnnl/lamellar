use std::marker::PhantomData;

use crate::{
    ep::{Address, Connected, EndpointBase, EndpointImplBase, Unconnected, UninitUnconnected},
    eq::{Event, EventQueueBase},
    fid::{AsRawFid, AsRawTypedFid, AsTypedFid, Fid}, utils::check_error,
};

use super::{cq::AsyncReadCq, eq::AsyncReadEq};

pub type UninitUnconnectedEndpointBase<EP> = EndpointBase<EP, UninitUnconnected>;

pub type UninitUnconnectedEndpoint<T> =
    UninitUnconnectedEndpointBase<EndpointImplBase<T, dyn AsyncReadEq, dyn AsyncReadCq>>;

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
            .async_event_wait(
                libfabric_sys::FI_CONNECTED,
                Fid(self.as_typed_fid().as_raw_fid() as usize),
                None,
                None,
            )
            .await?;

        match res {
            Event::Connected(event) => {
                assert_eq!(event.fid(), self.as_typed_fid().as_raw_fid())
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
            .async_event_wait(
                libfabric_sys::FI_CONNECTED,
                Fid(self.as_typed_fid().as_raw_fid() as usize),
                None,
                None,
            )
            .await?;

        match res {
            Event::Connected(event) => {
                assert_eq!(event.fid(), self.as_typed_fid().as_raw_fid())
            }
            _ => panic!("Unexpected Event Type"),
        }

        Ok(ConnectedEndpoint {
            inner: self.inner.clone(),
            phantom: PhantomData,
        })
    }
}


impl<E> UninitUnconnectedEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>> {
    pub fn enable<EQ: AsyncReadEq + 'static>(self, eq: &EventQueueBase<EQ>) -> Result<UnconnectedEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>>, crate::error::Error> {
        self.bind_eq(eq)?;
        let err =
            unsafe { libfabric_sys::inlined_fi_enable(self.as_typed_fid_mut().as_raw_typed_fid()) };
        check_error(err.try_into().unwrap())?;
        Ok(UnconnectedEndpointBase::<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>> {
            inner: self.inner.clone(),
            phantom: PhantomData,
        })
    }
}