use crate::{
    cq::ReadCq, ep::{EndpointBase, EndpointImplBase}, eq::ReadEq, info::InfoEntry, Context, MyRc
};

pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, false> ;

pub type ConnectionlessEndpoint<E> = ConnectionlessEndpointBase<EndpointImplBase<E, dyn ReadEq , dyn ReadCq>>;

pub trait ConnlessEp {}
impl<EP> ConnlessEp for ConnectionlessEndpointBase<EP> {}