use std::marker::PhantomData;

use crate::{
    av::{AVSyncMode, AddressVectorBase}, cq::ReadCq, ep::{Connectionless, EndpointBase, EndpointImplBase, UninitConnectionless}, eq::ReadEq, fid::{AsRawTypedFid, AsTypedFid}, utils::check_error
};

pub type UninitConnectionlessEndpointBase<EP> = EndpointBase<EP, UninitConnectionless>;
pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, Connectionless>;

pub type ConnectionlessEndpoint<E> =
    ConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>;

pub type UninitConnectionlessEndpoint<E> =
    UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>;

pub trait ConnlessEp {}
impl<EP> ConnlessEp for ConnectionlessEndpointBase<EP> {}
// impl<EP: AsRawTypedFid<Output = EpRawFid> + AsRawFid> UninitEndpoint
//     for UninitConnectionlessEndpointBase<EP>
// {
// }
    // pub fn bind_av<EQ: ?Sized + ReadEq + 'static>(
    //     &self,
    //     av: &AddressVectorBase<EQ>,
    // ) -> Result<(), crate::error::Error> {
    //     self.inner.bind_av(av)
    // }
impl<E> UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>> {
    pub fn enable< Mode: AVSyncMode, EQ: ?Sized + ReadEq + 'static>(self, av: &AddressVectorBase<Mode, EQ>) -> Result<ConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>, crate::error::Error> {
        // TODO: Move this into an UninitEp struct
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
