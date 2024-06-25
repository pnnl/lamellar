use std::rc::Rc;

use crate::{ep::{EndpointImplBase, Address, ActiveEndpointImpl, PassiveEndpointImplBase, PassiveEndpointBase, EndpointBase, EndpointAttr}, fid::{Fid, AsRawFid}, eq::Event, info::InfoEntry};

use super::{eq::AsyncEventQueueImpl, cq::AsyncCompletionQueueImpl, domain::Domain};


pub(crate) type AsyncEndpointImpl = EndpointImplBase<AsyncEventQueueImpl, AsyncCompletionQueueImpl>;

pub struct ConnectionListener {
    eq:  Rc<AsyncEventQueueImpl>,
    ep_fid: Fid,
}

impl ConnectionListener {
    fn new(ep_fid: Fid, eq: &Rc<AsyncEventQueueImpl>) -> Self {
        
        Self {
            ep_fid,
            eq: eq.clone(),
        }
    }

    pub async fn next(&self) -> Result<Event<usize>, crate::error::Error> {
        
        let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_CONNREQ}>{eq: self.eq.clone(), req_fid: self.ep_fid, ctx: Rc::strong_count(&self.eq)}.await?;

        Ok(res)
    }
}

pub type Endpoint<T> = EndpointBase<T, AsyncEventQueueImpl, AsyncCompletionQueueImpl>;
// pub struct AsyncEndpoint<T> {
//     pub(crate) inner: Rc<AsyncEndpointImpl>,
//     phantom: PhantomData<T>,
// }

impl<T> Endpoint<T> {

    pub async fn connect_async(&self, addr: &Address) -> Result<Event<usize>, crate::error::Error> {
        ActiveEndpointImpl::connect(self, addr)?;
        
        let eq = self.inner.eq.get().expect("Endpoint not bound to an EventQueue").clone();
        let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_CONNECTED}>{eq, req_fid: self.as_raw_fid(), ctx: 0}.await?;
        Ok(res)
    }

    pub async fn accept_async(&self) -> Result<Event<usize>, crate::error::Error> {
        self.accept()?;

        let eq = self.inner.eq.get().expect("Endpoint not bound to an EventQueue").clone();
        let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_CONNECTED}>{eq, req_fid: self.as_raw_fid(), ctx: 0}.await?;
        Ok(res)
    }
}


// ============== Async stuff ======================= //
pub(crate) type AsyncPassiveEndpointImpl<E> = PassiveEndpointImplBase<E, AsyncEventQueueImpl>;
pub type PassiveEndpoint<T> = PassiveEndpointBase<T, AsyncEventQueueImpl>;


impl<T> PassiveEndpoint<T> {

    pub fn listen_async(&self) -> Result<ConnectionListener, crate::error::Error> {
        self.listen()?;

        // let eq = self.inner.eq.get().unwrap().clone();
        Ok(ConnectionListener::new(self.as_raw_fid(), self.inner.eq.get().unwrap()))
    }
}

pub struct EndpointBuilder<'a, T, E> {
    ep_attr: EndpointAttr,
    flags: u64,
    info: &'a InfoEntry<E>,
    ctx: Option<&'a mut T>,
}

impl<'a> EndpointBuilder<'a, (), ()> {

    pub fn new<E>(info: &'a InfoEntry<E>, ) -> EndpointBuilder<'a, (), E> {
        EndpointBuilder::<(), E> {
            ep_attr: EndpointAttr::new(),
            flags: 0,
            info,
            ctx: None,
        }
    }
}

impl<'a, E> EndpointBuilder<'a, (), E> {

    pub fn build(self, domain: &Domain) -> Result<Endpoint<E>, crate::error::Error> {
        Endpoint::new(domain, self.info, self.flags, self.ctx)
    }

    // pub fn build_scalable(self, domain: &crate::domain::Domain) -> Result<ScalableEndpoint<E>, crate::error::Error> {
    //     ScalableEndpoint::new(domain, self.info, self.ctx)
    // }

    pub fn build_passive(self, fabric: &crate::fabric::Fabric) -> Result<PassiveEndpoint<E>, crate::error::Error> {
        PassiveEndpoint::new(fabric, self.info, self.ctx)
    }

    pub fn flags(mut self, flags: u64) -> Self {
        self.flags = flags;
        self
    }

    pub fn ep_type(mut self, type_: crate::enums::EndpointType) -> Self {

        self.ep_attr.ep_type(type_);
        self
    }

    pub fn protocol(mut self, proto: crate::enums::Protocol) -> Self{
        
        self.ep_attr.protocol(proto);
        self
    }

    pub fn max_msg_size(mut self, size: usize) -> Self {

        self.ep_attr.max_msg_size(size);
        self
    }

    pub fn msg_prefix_size(mut self, size: usize) -> Self {

        self.ep_attr.msg_prefix_size(size);
        self
    }

    pub fn max_order_raw_size(mut self, size: usize) -> Self {

        self.ep_attr.max_order_raw_size(size);
        self
    }

    pub fn max_order_war_size(mut self, size: usize) -> Self {

        self.ep_attr.max_order_war_size(size);
        self
    }

    pub fn max_order_waw_size(mut self, size: usize) -> Self {

        self.ep_attr.max_order_waw_size(size);
        self
    }

    pub fn mem_tag_format(mut self, tag: u64) -> Self {

        self.ep_attr.mem_tag_format(tag);
        self
    }

    pub fn tx_ctx_cnt(mut self, size: usize) -> Self {

        self.ep_attr.tx_ctx_cnt(size);
        self
    }

    pub fn rx_ctx_cnt(mut self, size: usize) -> Self {

        self.ep_attr.rx_ctx_cnt(size);
        self
    }

    pub fn auth_key(mut self, key: &mut [u8]) -> Self {

        self.ep_attr.auth_key(key);
        self
    }
}




