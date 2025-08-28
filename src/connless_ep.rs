use std::marker::PhantomData;

use crate::{
    av::{AVSyncMode, AddressVectorBase}, cq::ReadCq, ep::{Connectionless, EndpointBase, EndpointImplBase, UninitConnectionless}, eq::ReadEq, fid::{AsRawTypedFid, AsTypedFid}, utils::check_error
};

/// A connectionless endpoint that is not yet enabled.
pub type UninitConnectionlessEndpointBase<EP> = EndpointBase<EP, UninitConnectionless>;
pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, Connectionless>;

/// A connectionless endpoint that is enabled and ready for use.
pub type ConnectionlessEndpoint<E> =
    ConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>;

pub type UninitConnectionlessEndpoint<E> =
    UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>;

pub trait ConnlessEp {}
impl<EP> ConnlessEp for ConnectionlessEndpointBase<EP> {}

impl<E> UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>> {
    /// Enables the endpoint and binds it to the specified address vector.
    /// 
    /// After enabling, the endpoint can be used to send and receive messages.
    ///
    /// Corresponds to `fi_bind` followed by `fi_enable` in libfabric.
    pub fn enable< Mode: AVSyncMode, EQ: ?Sized + ReadEq + 'static>(self, av: &AddressVectorBase<Mode, EQ>) -> Result<ConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>, crate::error::Error> {
        self.bind_av(av)?;
        let err =
            unsafe { libfabric_sys::inlined_fi_enable(self.as_typed_fid_mut().as_raw_typed_fid()) };
        check_error(err.try_into().unwrap())?;
        Ok(ConnectionlessEndpointBase::<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>> {
            inner: self.inner.clone(),
            phantom: PhantomData,
        })
    }
}
