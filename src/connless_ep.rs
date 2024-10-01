use crate::{
    cq::ReadCq,
    ep::{Connectionless, EndpointBase, EndpointImplBase},
    eq::ReadEq,
    info::InfoEntry,
    Context, MyRc,
};

pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, Connectionless>;

pub type ConnectionlessEndpoint<E> =
    ConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>>;

pub trait ConnlessEp {}
impl<EP> ConnlessEp for ConnectionlessEndpointBase<EP> {}
