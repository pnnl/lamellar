use super::{
    eq::{AsyncReadEq, EventQueue},
};
use crate::{
    av::{AddressVectorAttr, AddressVectorBase, AddressVectorImplBase},
    domain::DomainBase,
    enums::AVOptions,
    ep::Address,
    eq::Event,
    fid::{AsRawFid, AsRawTypedFid, AsTypedFid, Fid},
    Context, MappedAddress, MyRc, RawMappedAddress, SyncSend,
};

pub(crate) type AsyncAddressVectorImpl = AddressVectorImplBase<dyn AsyncReadEq>;

impl AsyncAddressVectorImpl {
    pub(crate) async fn insert_async(
        &self,
        addr: &[Address],
        flags: u64,
        ctx: &mut Context,
    ) -> Result<(Event, Vec<u64>), crate::error::Error> {
        // [TODO] //[TODO] as_raw_typed_fid flags, as_raw_typed_fid context, as_raw_typed_fid async
        let mut fi_addresses = vec![0u64; addr.len()];
        let total_size = addr.iter().fold(0, |acc, addr| acc + addr.as_bytes().len());
        let mut serialized: Vec<u8> = Vec::with_capacity(total_size);
        for a in addr {
            serialized.extend(a.as_bytes().iter())
        }

        let err = unsafe {
            libfabric_sys::inlined_fi_av_insert(
                self.as_typed_fid().as_raw_typed_fid(),
                serialized.as_ptr().cast(),
                fi_addresses.len(),
                fi_addresses.as_mut_ptr().cast(),
                flags,
                ctx.inner_mut(),
            )
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            let eq = if let Some(eq) = self._eq_rc.get() {
                eq
            } else {
                panic!("Calling insert_async on unbound AV");
            };

            let res = eq
                .async_event_wait(
                    libfabric_sys::FI_AV_COMPLETE,
                    Fid(self.as_typed_fid().as_raw_fid() as usize),
                    ctx.inner_mut() as usize,
                )
                .await?;
            if let Event::AVComplete(ref entry) = res {
                fi_addresses.truncate(entry.data() as usize);
            }
            Ok((res, fi_addresses))
        }
    }
}

pub type AddressVector = AddressVectorBase<dyn AsyncReadEq>;

impl AddressVector {
    pub async fn insert_async(
        &self,
        addr: &[Address],
        options: AVOptions,
        ctx: &mut Context,
    ) -> Result<(Event, Vec<MappedAddress>), crate::error::Error> {
        // [TODO] as_raw_typed_fid async
        let (event, fi_addresses) = self
            .inner
            .insert_async(addr, options.as_raw(), ctx)
            .await?;
        Ok((
            event,
            fi_addresses
                .into_iter()
                .map(|fi_addr| {
                    MappedAddress::from_raw_addr(
                        RawMappedAddress::from_raw(self.inner.type_, fi_addr),
                        crate::AddressSource::Av(self.inner.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        ))
    }

}

pub struct AddressVectorBuilder<'a> {
    av_attr: AddressVectorAttr,
    eq: MyRc<dyn AsyncReadEq>,
    ctx: Option<&'a mut Context>,
}

impl<'a> AddressVectorBuilder<'a> {
    /// Initiates the creation of a new [AddressVector] on `domain`.
    ///
    /// The initial configuration is what would be set if no `fi_av_attr` or `context` was provided to
    /// the `fi_av_open` call.
    pub fn new<EQ: AsyncReadEq + 'static>(eq: &EventQueue<EQ>) -> AddressVectorBuilder<'a> {
        let mut av_attr = AddressVectorAttr::new();
        av_attr.async_();
        AddressVectorBuilder {
            av_attr,
            eq: eq.inner.clone(),
            ctx: None,
        }
    }
}

impl<'a> AddressVectorBuilder<'a> {
    /// Sets the type of the [AddressVector].
    ///
    /// Corresponds to setting field `fi_av_attr::type`
    pub fn type_(mut self, av_type: crate::enums::AddressVectorType) -> Self {
        self.av_attr.type_(av_type);
        self
    }

    /// Sets address bits to identify rx ctx of the [AddressVector].
    ///
    /// Corresponds to setting field `fi_av_attr::rx_ctx_bits`
    pub fn rx_ctx_bits(mut self, rx_ctx_bits: i32) -> Self {
        //[TODO] Maybe wrap bitfield
        self.av_attr.rx_ctx_bits(rx_ctx_bits);
        self
    }

    /// Sets the number of [Address]es that will be inserted into the [AddressVector]
    ///
    /// Corresponds to setting field `fi_av_attr::count`
    pub fn count(mut self, count: usize) -> Self {
        self.av_attr.count(count);
        self
    }

    /// Sets the number of [Endpoint][crate::ep::Endpoint]s that will be inserted into the [AddressVector]
    ///
    /// Corresponds to setting field `fi_av_attr::ep_per_node`
    pub fn ep_per_node(mut self, count: usize) -> Self {
        self.av_attr.ep_per_node(count);
        self
    }

    /// Sets the system name of the [AddressVector] to `name`.
    ///
    /// Corresponds to setting field `fi_av_attr::name`
    pub fn name(mut self, name: String) -> Self {
        self.av_attr.name(name);
        self
    }

    /// Sets the base mmap address of the [AddressVector] to `addr`.
    ///
    /// Corresponds to setting field `fi_av_attr::map_addr`
    pub fn map_addr(mut self, addr: usize) -> Self {
        self.av_attr.map_addr(addr);
        self
    }

    /// Sets the [AddressVector] to read-only mode.
    ///
    /// Corresponds to setting the corresponding bit (`FI_READ`) of the field `fi_av_attr::flags`
    pub fn read_only(mut self) -> Self {
        self.av_attr.read_only();
        self
    }

    /// Requests that insertions to [AddressVector] be done asynchronously.
    ///
    /// An asynchronous address vector is required to be bound to an [EventQueue] before any insertions take place.
    /// Thus, setting this option requires the user to specify the queue that will be used to report the completion
    /// of address insertions.
    ///
    /// Corresponds to setting the corresponding bit (`FI_EVENT`) of the field `fi_av_attr::flags` and calling
    /// `fi_av_bind(eq)`, once the address vector has been constructed.
    // pub fn async_<EQ: EqConfig>(mut self, eq: &'a EventQueue<EQ>) -> Self {
    //     self.av_attr.async_();
    //     self.eq = Some(&eq.inner);
    //     self
    // }

    /// Indicates that each node will be associated with the same number of endpoints.
    ///
    /// Corresponds to setting the corresponding bit (`FI_SYMMETRIC`) of the field `fi_av_attr::flags`.
    pub fn symmetric(mut self) -> Self {
        self.av_attr.symmetric();
        self
    }

    /// Sets the context to be passed to the [AddressVector].
    ///
    /// Corresponds to passing a non-NULL `context` value to `fi_av_open`.
    pub fn context(self, ctx: &'a mut Context) -> AddressVectorBuilder<'a> {
        AddressVectorBuilder {
            av_attr: self.av_attr,
            eq: self.eq,
            ctx: Some(ctx),
        }
    }

    /// Constructs a new [AddressVector] with the configurations requested so far.
    ///
    /// Corresponds to creating an `fi_av_attr`, setting its fields to the requested ones,
    /// calling `fi_av_open` with an optional `context`, and, if asynchronous, binding with
    /// the selected [EventQueue].
    pub fn build<EQ: 'static + ?Sized + SyncSend>(
        self,
        domain: &DomainBase<EQ>,
    ) -> Result<AddressVector, crate::error::Error> {
        let av = AddressVector::new(domain, self.av_attr, self.ctx)?;
        av.inner.bind(&self.eq)?;
        Ok(av)
    }
}

#[cfg(test)]
mod tests {
    use crate::async_::eq::EventQueueBuilder;
    use crate::domain::DomainBuilder;
    use crate::info::{Info, Version};

    use super::AddressVectorBuilder;

    #[test]
    fn av_open_close() {
        let info = Info::new(&Version {
            major: 1,
            minor: 19,
        })
        .enter_hints()
        .enter_ep_attr()
        .type_(crate::enums::EndpointType::Rdm)
        .leave_ep_attr()
        .enter_domain_attr()
        .mode(crate::enums::Mode::all())
        .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse())
        .leave_domain_attr()
        .leave_hints()
        .get()
        .unwrap();

        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        let eq = EventQueueBuilder::new(&fab).write().build().unwrap();

        for i in 0..17 {
            let count = 1 << i;
            let _av = AddressVectorBuilder::new(&eq)
                .type_(crate::enums::AddressVectorType::Map)
                .count(count)
                .build(&domain)
                .unwrap();
        }
    }

    #[test]
    fn av_good_sync() {
        let info = Info::new(&Version {
            major: 1,
            minor: 19,
        })
        .enter_hints()
        .enter_ep_attr()
        .type_(crate::enums::EndpointType::Rdm)
        .leave_ep_attr()
        .enter_domain_attr()
        .mode(crate::enums::Mode::all())
        .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse())
        .leave_domain_attr()
        .leave_hints()
        .get()
        .unwrap();

        let entry = info.into_iter().next().unwrap();

        let fab: crate::fabric::Fabric = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        let eq = EventQueueBuilder::new(&fab).write().build().unwrap();
        let _av = AddressVectorBuilder::new(&eq)
            .type_(crate::enums::AddressVectorType::Map)
            .count(32)
            .build(&domain)
            .unwrap();
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::{
        async_::eq::EventQueueBuilder,
        domain::DomainBuilder,
        info::{Info, Version},
    };

    use super::AddressVectorBuilder;

    #[test]
    fn av_drops_before_domain() {
        let info = Info::new(&Version {
            major: 1,
            minor: 19,
        })
        .enter_hints()
        .enter_ep_attr()
        .type_(crate::enums::EndpointType::Rdm)
        .leave_ep_attr()
        .enter_domain_attr()
        .mode(crate::enums::Mode::all())
        .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse())
        .leave_domain_attr()
        .leave_hints()
        .get()
        .unwrap();

        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        let eq = EventQueueBuilder::new(&fab).write().build().unwrap();

        let mut avs = Vec::new();
        for i in 0..17 {
            let count = 1 << i;
            let av = AddressVectorBuilder::new(&eq)
                .type_(crate::enums::AddressVectorType::Map)
                .count(count)
                .build(&domain)
                .unwrap();
            avs.push(av);
        }
        drop(domain);
    }
}
