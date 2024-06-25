use crate::{domain::{DomainImplBase, DomainBase, PeerDomainCtx}, info::InfoEntry};

use super::eq::AsyncEventQueueImpl;

pub(crate) type AsyncDomainImpl = DomainImplBase<AsyncEventQueueImpl>;
pub type Domain = DomainBase<AsyncEventQueueImpl>;

/// Builder for the [Domain] type.
/// 
/// `DomainBuilder` is used to configure and build a new [Domain].
/// It encapsulates an incremental configuration of the address vector set, as provided by a `fi_domain_attr`,
/// followed by a call to `fi_domain_open`  
pub struct DomainBuilder<'a, T, E> {
    fabric: &'a crate::fabric::Fabric,
    info: &'a InfoEntry<E>,
    ctx: Option<&'a mut T>,
    flags: u64,
}

impl<'a> DomainBuilder<'a, (), ()> {


    /// Initiates the creation of new [Domain] on `fabric`, using the configuration found in `info`.
    /// 
    /// The initial configuration is what would be set if no `fi_domain_attr` or `context` was provided to 
    /// the `fi_domain` call. 
    pub fn new<E>(fabric: &'a crate::fabric::Fabric, info: &'a InfoEntry<E>) -> DomainBuilder<'a, (), E> {
        DomainBuilder::<(), E> {
            fabric,
            info,
            flags: 0,
            ctx: None,
        }
    }


    /// Initiates the creation of new [Domain] on `fabir`, using the configuration found in `info`.
    /// 
    /// The initial configuration is what would be set if no `fi_domain_attr` was provided to 
    /// the `fi_domain2` call and `context` was set to a `fi_peer_context`. 
    pub fn new_with_peer<E>(fabric: &'a crate::fabric::Fabric, info: &'a InfoEntry<E>, peer_ctx: &'a mut PeerDomainCtx) -> DomainBuilder<'a, PeerDomainCtx, E> {
        DomainBuilder::<PeerDomainCtx, E> {
            fabric,
            info,
            flags: libfabric_sys::FI_PEER,
            ctx: Some(peer_ctx),
        }
    }
}


impl<'a, E> DomainBuilder<'a, (), E> {
    
    /// Sets the context to be passed to the domain.
    /// 
    /// Corresponds to passing a non-NULL, non-`fi_peer_context` `context` value to `fi_domain`.
    pub fn context<T>(self, ctx: &'a mut T) -> DomainBuilder<'a, T, E> {
        DomainBuilder {
            fabric: self.fabric,
            info: self.info,
            flags: 0,
            ctx: Some(ctx),
        }
    }
}

impl<'a, T, E> DomainBuilder<'a, T, E> {

    // pub fn flags(mut self, flags: u64) -> Self {
    //     self.flags = flags;
    //     self
    // }


    /// Constructs a new [Domain] with the configurations requested so far.
    /// 
    /// Corresponds to creating a `fi_domain_attr`, setting its fields to the requested ones,
    /// and passing it to a `fi_domain` call with an optional `context` (set by [Self::context]).
    /// Or a call to `fi_domain2` with `context` of type `fi_peer_context` and `flags` equal to `FI_PEER`
    pub fn build(self) -> Result<Domain, crate::error::Error> {
        Domain::new(self.fabric, self.info, self.flags, self.info.get_domain_attr().clone(), self.ctx)
    }
}
