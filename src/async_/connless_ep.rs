use std::marker::PhantomData;

use crate::{
    av::{AddressVectorBase, NoBlock},
    ep::{Connectionless, EndpointBase, EndpointImplBase, UninitConnectionless},
    eq::ReadEq,
    fid::{AsRawTypedFid, AsTypedFid},
    utils::check_error,
};

use super::{cq::AsyncReadCq, eq::AsyncReadEq};

pub type UninitConnectionlessEndpointBase<EP> = EndpointBase<EP, UninitConnectionless>;
pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, Connectionless>;

pub type ConnectionlessEndpoint<E> =
    ConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub type UninitConnectionlessEndpoint<E> =
    UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub trait ConnlessEp {}
impl<EP> ConnlessEp for ConnectionlessEndpointBase<EP> {}

impl<E> UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>> {
    pub fn enable<EQ: ?Sized + ReadEq + 'static>(
        self,
        av: &AddressVectorBase<NoBlock, EQ>,
    ) -> Result<
        ConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>>,
        crate::error::Error,
    > {
        self.bind_av(av)?;
        let err =
            unsafe { libfabric_sys::inlined_fi_enable(self.as_typed_fid_mut().as_raw_typed_fid()) };
        check_error(err.try_into().unwrap())?;
        Ok(
            ConnectionlessEndpointBase::<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>> {
                inner: self.inner.clone(),
                phantom: PhantomData,
            },
        )
    }
}
