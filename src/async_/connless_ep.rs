use std::marker::PhantomData;

use crate::{
    av::{AddressVectorBase, AVSyncMode},
    ep::{Connectionless, EndpointBase, EndpointImplBase, UninitConnectionless},
    eq::ReadEq,
    fid::{AsRawTypedFid, AsTypedFid, EpRawFid},
    utils::check_error,
};

use super::{cq::AsyncCq, eq::AsyncReadEq};

pub type UninitConnectionlessEndpointBase<EP> = EndpointBase<EP, UninitConnectionless>;
pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, Connectionless>;

pub type ConnectionlessEndpoint<E> =
    ConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncCq>>;

pub type UninitConnectionlessEndpoint<E> =
    UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncCq>>;

pub trait ConnlessEp {}
impl<EP: AsTypedFid<EpRawFid>> ConnlessEp for ConnectionlessEndpointBase<EP> {}

impl<E> UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncCq>> {
    pub fn enable<Mode: AVSyncMode,EQ: ?Sized + ReadEq + 'static>(
        self,
        av: &AddressVectorBase<Mode, EQ>,
    ) -> Result<
        ConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncCq>>,
        crate::error::Error,
    > {
        self.bind_av(av)?;
        let err =
            unsafe { libfabric_sys::inlined_fi_enable(self.as_typed_fid_mut().as_raw_typed_fid()) };
        check_error(err.try_into().unwrap())?;
        Ok(
            ConnectionlessEndpointBase::<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncCq>> {
                inner: self.inner.clone(),
                phantom: PhantomData,
            },
        )
    }
}
