use crate::ep::{Connectionless, EndpointBase, EndpointImplBase};

use super::{cq::AsyncReadCq, eq::AsyncReadEq};

pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, Connectionless>;

pub type ConnectionlessEndpoint<E> =
    ConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub trait ConnlessEp {}
impl<EP> ConnlessEp for ConnectionlessEndpointBase<EP> {}
