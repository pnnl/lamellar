use std::marker::PhantomData;

use crate::{
    cq::ReadCq,
    ep::{Connectionless, EndpointBase, EndpointImplBase, UninitConnectionless, UninitEndpoint},
    eq::ReadEq,
    fid::{AsRawFid, AsRawTypedFid, EpRawFid},
    utils::check_error,
};

pub type UninitConnectionlessEndpointBase<EP> = EndpointBase<EP, UninitConnectionless>;
pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, Connectionless>;

pub type ConnectionlessEndpoint<E> =
    ConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>;

pub type UninitConnectionlessEndpoint<E> =
    UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>;

pub trait ConnlessEp {}
impl<EP> ConnlessEp for ConnectionlessEndpointBase<EP> {}
impl<EP: AsRawTypedFid<Output = EpRawFid> + AsRawFid> UninitEndpoint
    for UninitConnectionlessEndpointBase<EP>
{
}

impl<EP: AsRawTypedFid<Output = EpRawFid>> UninitConnectionlessEndpointBase<EP> {
    pub fn enable(self) -> Result<ConnectionlessEndpointBase<EP>, crate::error::Error> {
        // TODO: Move this into an UninitEp struct
        let err = unsafe { libfabric_sys::inlined_fi_enable(self.as_raw_typed_fid()) };
        check_error(err.try_into().unwrap())?;
        Ok(ConnectionlessEndpointBase::<EP> {
            inner: self.inner.clone(),
            phantom: PhantomData,
        })
    }
}
