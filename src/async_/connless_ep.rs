use crate::ep::{Connectionless, EndpointBase, EndpointImplBase, UninitConnectionless};

use super::{cq::AsyncReadCq, eq::AsyncReadEq};

pub type UninitConnectionlessEndpointBase<EP> = EndpointBase<EP, UninitConnectionless>;
pub type ConnectionlessEndpointBase<EP> = EndpointBase<EP, Connectionless>;

pub type ConnectionlessEndpoint<E> =
    ConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub type UninitConnectionlessEndpoint<E> =
    UninitConnectionlessEndpointBase<EndpointImplBase<E, dyn AsyncReadEq, dyn AsyncReadCq>>;

pub trait ConnlessEp {}
impl<EP> ConnlessEp for ConnectionlessEndpointBase<EP> {}
