use std::rc::Rc;

use crate::{ep::{Address, ActiveEndpointImpl, PassiveEndpointBase, EndpointBase, EndpointAttr, EndpointImplBase, PassiveEndpointImplBase}, fid::{RawFid, AsRawFid, AsRawTypedFid}, eq::{Event, EventQueueBase}, info::InfoEntry, cq::ReadCq, domain::DomainBase};

use super::{eq::AsyncReadEq, cq::AsyncReadCq, domain::Domain};

pub struct ConnectionListener {
    eq:  Rc<dyn AsyncReadEq>,
    ep_fid: RawFid,
}

impl ConnectionListener {
    fn new(ep_fid: RawFid, eq: &Rc<dyn AsyncReadEq>) -> Self {
        
        Self {
            ep_fid,
            eq: eq.clone(),
        }
    }

    pub async fn next(&self) -> Result<Event<usize>, crate::error::Error> {
        
        // let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_CONNREQ}>::new(self.ep_fid, self.eq.clone(), Rc::strong_count(&self.eq)).await?;
        let res = self.eq.async_event_wait(libfabric_sys::FI_CONNREQ, self.ep_fid,  0).await?;
        Ok(res)
    }
}

pub type Endpoint<T> = EndpointBase<T, dyn AsyncReadEq, dyn AsyncReadCq>;
// pub struct AsyncEndpoint<T> {
//     pub(crate) inner: Rc<AsyncEndpointImpl>,
//     phantom: PhantomData<T>,
// }

impl<T> Endpoint<T> {

    pub async fn connect_async(&self, addr: &Address) -> Result<Event<usize>, crate::error::Error> {
        self.inner.connect(addr)?;
        
        let eq = self.inner.eq.get().expect("Endpoint not bound to an EventQueue");
        // let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_CONNECTED}>::new(self.as_raw_fid(), eq, 0).await?;
        let res = eq.async_event_wait(libfabric_sys::FI_CONNECTED, self.as_raw_fid(),  0).await?;

        
        Ok(res)
    }

    pub async fn accept_async(&self) -> Result<Event<usize>, crate::error::Error> {
        self.accept()?;

        let eq = self.inner.eq.get().expect("Endpoint not bound to an EventQueue");
        let res = eq.async_event_wait(libfabric_sys::FI_CONNECTED, self.as_raw_fid(),  0).await?;

        // let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_CONNECTED}>::new(self.as_raw_fid(), eq, 0).await?;
        Ok(res)
    }
}

pub struct IncompleteBindCq<'a> {
    pub(crate) ep: &'a EndpointImplBase<dyn AsyncReadEq, dyn AsyncReadCq>,
    pub(crate) flags: u64,
}

impl EndpointImplBase<dyn AsyncReadEq, dyn AsyncReadCq> {
    pub(crate) fn bind_cq_<T: AsyncReadCq + 'static>(&self, cq: &Rc<T>, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.as_raw_typed_fid(), cq.as_raw_fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {

            if (flags & libfabric_sys::FI_TRANSMIT as u64) != 0 && (flags & libfabric_sys::FI_RECV as u64) != 0{
                if self.tx_cq.set(cq.clone()).is_err() {
                    panic!("Endpoint is already bound to a CompletionQueue for this direction");
                }
                if self.rx_cq.set(cq.clone()).is_err() {
                    panic!("Endpoint is already bound to a CompletionQueue for this direction");
                }
            }
            else if flags & libfabric_sys::FI_TRANSMIT as u64 != 0 {
                if self.tx_cq.set(cq.clone()).is_err() {
                    panic!("Endpoint is already bound to a CompletionQueue for this direction");
                }
            }
            else if flags & libfabric_sys::FI_RECV as u64 != 0{
                if self.rx_cq.set(cq.clone()).is_err() {
                    panic!("Endpoint is already bound to a CompletionQueue for this direction");
                }
            }
            else {
                panic!("Binding to Endpoint without specifying direction");
            }

            // self._sync_rcs.borrow_mut().push(cq.inner().clone()); //  [TODO] Do we need this for cq?
            Ok(())
        }
    } 

    pub(crate) fn bind_cq(&self) -> IncompleteBindCq {
        IncompleteBindCq { ep: self, flags: 0}
    }
}

impl<CQ: ?Sized + ReadCq> EndpointImplBase<dyn AsyncReadEq, CQ> {

    pub(crate) fn bind_eq<T: AsyncReadEq + 'static>(&self, eq: &Rc<T>) -> Result<(), crate::error::Error>  {
            
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.as_raw_typed_fid(), eq.as_raw_fid(), 0) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            if self.eq.set(eq.clone()).is_err() {
                panic!("Endpoint is already bound to another EventQueue"); // Should never reach this since inlined_fi_ep_bind will throw an error ealier
                                                                        // but keep it here to satisfy the compiler.
            }

            // self._sync_rcs.borrow_mut().push(cq.inner().clone()); //  [TODO] Do we need this for eq?
            Ok(())
        }
    }
}

impl<E> EndpointBase<E, dyn AsyncReadEq, dyn AsyncReadCq> {
    pub fn bind_cq(&self) -> IncompleteBindCq {
        self.inner.bind_cq()
    }
}

impl<E, CQ: ?Sized + ReadCq> EndpointBase<E, dyn AsyncReadEq, CQ> {
    pub fn bind_eq<T: AsyncReadEq + 'static>(&self, eq: &EventQueueBase<T>) -> Result<(), crate::error::Error>  {
        self.inner.bind_eq(&eq.inner)
    }
}

// impl<E, CQ: ?Sized + CompletionQueueImplT> EndpointBase<E, dyn EventQueueImplT, CQ> {
//     pub fn bind_eq<T: EventQueueImplT + 'static>(&self, eq: &EventQueueBase<T>) -> Result<(), crate::error::Error>  {
//         self.inner.bind_eq(&eq.inner)
//     }
// }

impl<'a> IncompleteBindCq<'a> {
    pub fn recv(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_RECV  as u64 ;
        
            self
        }
        else {
            self.flags |= libfabric_sys::FI_RECV as u64;

            self
        }
    }

    pub fn transmit(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_TRANSMIT as u64;

            self
        }
        else {
            self.flags |= libfabric_sys::FI_TRANSMIT as u64;

            self
        }
    }

    pub fn cq<T: AsyncReadCq + 'static>(&mut self, cq: &crate::cq::CompletionQueueBase<T>) -> Result<(), crate::error::Error> {
        self.ep.bind_cq_(&cq.inner, self.flags)
    }
}

// ============== Async stuff ======================= //
pub type PassiveEndpoint<T> = PassiveEndpointBase<T, dyn AsyncReadEq>;


impl<T> PassiveEndpoint<T> {

    pub fn listen_async(&self) -> Result<ConnectionListener, crate::error::Error> {
        self.listen()?;

        // let eq = self.inner.eq.get().unwrap().clone();
        Ok(ConnectionListener::new(self.as_raw_fid(), self.inner.eq.get().unwrap()))
    }
    
}

impl<E> PassiveEndpointImplBase<E, dyn AsyncReadEq> {


    pub(crate) fn bind<T: AsyncReadEq + 'static>(&self, res: &Rc<T>, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_pep_bind(self.as_raw_typed_fid(), res.as_raw_fid(), flags) };
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            // self._sync_rcs.borrow_mut().push(res.clone()); 
            if self.eq.set(res.clone()).is_err() {panic!("Could not set oncecell")}
            Ok(())
        }
    }
}


impl<E> PassiveEndpointBase<E, dyn AsyncReadEq> {

    pub fn bind<T: AsyncReadEq + 'static>(&self, res: &EventQueueBase<T>, flags: u64) -> Result<(), crate::error::Error> {
        self.inner.bind(&res.inner, flags)
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

    pub fn build<DEQ: 'static>(self, domain: &DomainBase<DEQ>) -> Result<Endpoint<E>, crate::error::Error> {
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




