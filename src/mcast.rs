use crate::{av::{AddressVectorSet, AddressVectorSetImpl}, comm::{collective::CollectiveEp, message::extract_raw_ctx}, enums::{AddressVectorType, JoinOptions}, ep::{EndpointBase, EpState}, eq::JoinCompleteEvent, error::Error, fid::{AsRawTypedFid, AsTypedFid, BorrowedTypedFid, EpRawFid, McRawFid, MutBorrowedTypedFid, OwnedMcFid}, Context, MappedAddress, MyOnceCell, MyRc, MyRefCell, RawMappedAddress, SyncSend};

pub(crate) enum MulticastAddressSource {
    MulticastGroup(MyRc<MulticastGroupImpl>),
    AVSet(MyRc<AddressVectorSetImpl>),
    #[allow(dead_code)]
    RawAddress(RawMappedAddress)
}

pub struct MultiCastGroup {
    pub(crate) inner: MyRc<MulticastGroupImpl>,
}

pub struct MulticastGroupCollectiveBuilder {
    pub(crate) avset: MyRc<AddressVectorSetImpl>,
    addr_source: MulticastAddressSource,
}

pub struct MulticastGroupBuilder {
    addr_source: MulticastAddressSource,
}

pub struct PendingMulticastGroup {
    _inner: MyRc<MulticastGroupImpl>,
}

pub struct PendingMulticastGroupCollective {
    pub(crate) inner: MyRc<MulticastGroupImpl>,
}

pub struct WaitingMulticastGroupCollective {
    inner: MyRc<MulticastGroupImpl>,
}

impl MulticastGroupBuilder {
    pub fn from_av_set(avset: &AddressVectorSet) -> MulticastGroupCollectiveBuilder {
        MulticastGroupCollectiveBuilder {
            addr_source: MulticastAddressSource::AVSet(avset.inner.clone()),
            avset: avset.inner.clone(),
        }
    }

    pub fn from_multicast_group(mc_group: &MultiCastGroup) -> Self {
        Self{
            addr_source: MulticastAddressSource::MulticastGroup(mc_group.inner.clone())
        }
    }

    pub fn from_raw_addr(raw_addr: &MappedAddress) -> Self {
        Self {
            addr_source: MulticastAddressSource::RawAddress(RawMappedAddress::Unspec(raw_addr.raw_addr())),
        }
    }

    pub fn collective(self, avset: &AddressVectorSet) -> MulticastGroupCollectiveBuilder{
        MulticastGroupCollectiveBuilder {
            avset: avset.inner.clone(),
            addr_source: self.addr_source,
        }
    }

    pub fn build(self) -> PendingMulticastGroup {
        todo!();
        // PendingMulticastGroup {
        //     inner: MyRc::new(MulticastGroupImpl::new(self.addr_source))
        // }
    }
}


impl MulticastGroupCollectiveBuilder {
    pub fn build(self) -> PendingMulticastGroupCollective {
        PendingMulticastGroupCollective {
            inner: MyRc::new(MulticastGroupImpl::new_collective(self.addr_source, &self.avset))
        }
    }
}


impl WaitingMulticastGroupCollective {

    pub fn join_complete(self, _event: JoinCompleteEvent) -> MultiCastGroup {
        MultiCastGroup {
            inner: self.inner
        }
    }
}


// impl PendingMulticastGroup {
//         pub fn join_with_context<
//         E: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
//         STATE: EpState,
//     >(
//         &self,
//         ep: &EndpointBase<E, STATE>,
//         options: JoinOptions,
//         context: &mut Context,
//     ) -> Result<WaitingMulticastGroup, Error> {
//         self.inner
//             .join_impl(&ep.inner, options, Some(context.inner_mut()))?;
//         Ok(WaitingMulticastGroup {
//             inner: self.inner.clone()
//         })
//     }

//     pub fn join_collective<
//         E: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
//         STATE: EpState,
//         const INIT: bool,
//     >(
//         &self,
//         ep: &EndpointBase<E, STATE>,
//         options: JoinOptions,
//     ) -> Result<WaitingMulticastGroup, Error> {
//         self.inner.join_impl(&ep.inner, options, None)?;
//         Ok(WaitingMulticastGroup {
//             inner: self.inner.clone()
//         })
//     }
// }


impl PendingMulticastGroupCollective {

    pub fn join_collective_with_context<
        E: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
        STATE: EpState,
    >(
        &self,
        ep: &EndpointBase<E, STATE>,
        options: JoinOptions,
        context: &mut Context,
    ) -> Result<WaitingMulticastGroupCollective, Error> {
        self.inner
            .join_collective_impl(&ep.inner, options, Some(context.inner_mut()))?;
        Ok(WaitingMulticastGroupCollective {
            inner: self.inner.clone()
        })
    }

    pub fn join_collective<
        E: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
        STATE: EpState,
        const INIT: bool,
    >(
        self,
        ep: &EndpointBase<E, STATE>,
        options: JoinOptions,
    ) -> Result<WaitingMulticastGroupCollective, Error> {
        self.inner.join_collective_impl(&ep.inner, options, None)?;
        Ok(WaitingMulticastGroupCollective {
            inner: self.inner.clone()
        })
    }

}

// pub struct MultiCastGroup {
//     pub(crate) inner: MyRc<MulticastGroupImpl>,
// }

pub(crate) struct MulticastGroupImpl {
    c_mc: MyOnceCell<OwnedMcFid>,
    eps: MyRefCell<Vec<MyRc<dyn CollectiveValidEp>>>,
    addr: MyOnceCell<RawMappedAddress>,
    addr_source: MulticastAddressSource,
    avset: MyOnceCell<MyRc<AddressVectorSetImpl>>,
}

pub(crate) trait CollectiveValidEp: SyncSend {}
impl<EP: CollectiveEp + SyncSend> CollectiveValidEp for EP {}

impl MulticastGroupImpl {
    // pub(crate) fn new(addr: MulticastAddressSource) -> Self {
    //     Self {
    //         c_mc: MyOnceCell::new(),
    //         addr: MyOnceCell::new(),
    //         eps: MyRefCell::new(Vec::new()),
    //         addr_source: addr,
    //         avset: MyOnceCell::new(),
    //     }
    // }


    pub(crate) fn new_collective(addr: MulticastAddressSource, avset: &MyRc<AddressVectorSetImpl>) -> Self {
        let oc_set: std::cell::OnceCell<std::rc::Rc<AddressVectorSetImpl>> = MyOnceCell::new();
        if let Err(_err) = oc_set.set(avset.clone()) {
            panic!("Could not set multicast group")
        }
        Self {
            c_mc: MyOnceCell::new(),
            addr: MyOnceCell::new(),
            eps: MyRefCell::new(Vec::new()),
            addr_source: addr,
            avset: oc_set,
        }
    }

    // pub(crate) fn join_impl<
    //     EP: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
    // >(
    //     &self,
    //     ep: &MyRc<EP>,
    //     options: JoinOptions,
    //     context: Option<*mut std::ffi::c_void>,
    // ) -> Result<(), Error> {
    //     let mut c_mc: McRawFid = std::ptr::null_mut();
    //     let ctx = extract_raw_ctx(context);
    //     let addr = self.addr.get();
    //     let raw_addr = if let Some(addr) = addr {
    //         addr.clone()
    //     } else {
    //         match &self.addr_source {
    //             MulticastAddressSource::MulticastGroup(mc) => mc.addr.get().unwrap().clone(),
    //             MulticastAddressSource::AVSet(avset) => {panic!("Cannot call join with a AV set address");},
    //             MulticastAddressSource::RawAddress(addr) => {*addr}
    //         }
    //     };
    //     let err =
    //         if let Some(ctx) = context {
    //             unsafe { libfabric_sys::inlined_fi_join(
    //                 ep.as_typed_fid_mut().as_raw_typed_fid(), 
    //                 raw_addr.get(), options.as_raw(), &mut c_mc, (ctx as *mut T).cast()
    //             ) }
    //         }
    //         else {
    //             unsafe { libfabric_sys::inlined_fi_join(
    //                 ep.as_typed_fid_mut().as_raw_typed_fid(), 
    //                 raw_addr.get(), options.as_raw(), &mut c_mc, std::ptr::null_mut()
    //             ) }
    //         };

    //     if err != 0 {
    //         Err(Error::from_err_code((-err).try_into().unwrap()))
    //     }
    //     else {
    //         if let Err(old_mc)  = self.c_mc.set(OwnedMcFid::from(c_mc)) {
    //             assert!(old_mc.as_raw_typed_fid() == c_mc);
    //         }
    //         else {
    //             self.addr.set(unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc)}).unwrap()
    //         }
    //         self.eps.write().push(ep.clone());
    //         Ok(())
    //     }
    // }

    pub(crate) fn join_collective_impl<
        EP: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
    >(
        &self,
        ep: &MyRc<EP>,
        options: JoinOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), Error> {
        let mut c_mc: McRawFid = std::ptr::null_mut();
        let ctx = extract_raw_ctx(context);
        let addr = self.addr.get();
        let raw_addr = if let Some(addr) = addr {
            addr.clone()
        } else {
            match &self.addr_source {
                MulticastAddressSource::MulticastGroup(mc) => mc.addr.get().unwrap().clone(),
                MulticastAddressSource::AVSet(avset) => avset.address()?,
                MulticastAddressSource::RawAddress(_) => {panic!("Cannot call join_collective without a AV set or MC group");}
            }
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_join_collective(
                ep.as_typed_fid_mut().as_raw_typed_fid(),
                raw_addr.get(),
                self.avset.get().unwrap().as_typed_fid().as_raw_typed_fid(),
                options.as_raw(),
                &mut c_mc,
                ctx,
            )
        };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        } else {
            if let Err(old_mc) = self.c_mc.set(OwnedMcFid::from(c_mc)) {
                assert!(old_mc.as_typed_fid().as_raw_typed_fid() == c_mc);
            } else {
                if let Some(avset) = self.avset.get() {
                    self.addr
                        .set(RawMappedAddress::from_raw(
                            avset._av_rc.type_(),
                            unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc) },
                        ))
                        .unwrap()
                }
                else {
                  self.addr
                        .set(RawMappedAddress::from_raw(
                            AddressVectorType::Unspec,
                            unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc) },
                        ))
                        .unwrap()  
                }
            }
            #[cfg(feature = "thread-safe")]
            self.eps.write().push(ep.clone());
            #[cfg(not(feature = "thread-safe"))]
            self.eps.borrow_mut().push(ep.clone());
            Ok(())
        }
    }
}

impl MultiCastGroup {
    #[allow(dead_code)]
    pub(crate) fn from_impl(mc_impl: &MyRc<MulticastGroupImpl>) -> Self {
        Self {
            inner: mc_impl.clone(),
        }
    }

    pub(crate) fn raw_addr(&self) -> &RawMappedAddress {
        self.inner.addr.get().unwrap()
    }

    // pub fn join<E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static>(&self, ep: &EndpointBase<E>, addr: &Address, options: JoinOptions) -> Result<(), Error> {
    //     self.inner.join_impl::<(), E>(&ep.inner, addr, options, None)
    // }

    // pub fn join_with_context<E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static,T>(&self, ep: &EndpointBase<E>, addr: &Address, options: JoinOptions, context: &mut Context) -> Result<(), Error> {
    //     self.inner.join_impl(&ep.inner, addr, options, Some(context.inner_mut()))
    // }

}

impl AsTypedFid<McRawFid> for MultiCastGroup {
    fn as_typed_fid(&self) -> BorrowedTypedFid<McRawFid> {
        self.inner.as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> MutBorrowedTypedFid<McRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

impl AsTypedFid<McRawFid> for MulticastGroupImpl {
    fn as_typed_fid(&self) -> BorrowedTypedFid<McRawFid> {
        self.c_mc.get().unwrap().as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> MutBorrowedTypedFid<McRawFid> {
        self.c_mc.get().unwrap().as_typed_fid_mut()
    }
}