#[cfg(all(
    any(feature = "use-async-std", feature = "use-tokio"),
    any(
        feature = "threading-completion",
        feature = "threading-domain",
        feature = "threading-endpoint",
        feature = "threading-fid",
        feature = "threading-thread-safe"
    )
))]
pub mod parallel_async_ofi {
    use std::sync::Arc;

    use libfabric::async_::comm::message::AsyncRecvEp;
    use libfabric::async_::comm::message::AsyncSendEp;
    use libfabric::async_::comm::message::ConnectedAsyncRecvEp;
    use libfabric::async_::comm::message::ConnectedAsyncSendEp;
    use libfabric::async_::comm::rma::AsyncReadEp;
    use libfabric::async_::comm::rma::AsyncWriteEp;
    use libfabric::async_::comm::rma::ConnectedAsyncReadEp;
    use libfabric::async_::comm::rma::ConnectedAsyncWriteEp;
    use libfabric::async_::comm::tagged::AsyncTagRecvEp;
    use libfabric::async_::comm::tagged::AsyncTagSendEp;
    use libfabric::async_::comm::tagged::ConnectedAsyncTagRecvEp;
    use libfabric::async_::comm::tagged::ConnectedAsyncTagSendEp;
    use libfabric::async_::domain::Domain;
    use libfabric::ep::BaseEndpoint;
    use libfabric::info::Info;
    use libfabric::infocapsoptions::InfoCaps;
    use libfabric::mr::EpBindingMemoryRegion;
    use libfabric::mr::MemoryRegionDesc;
    use libfabric::mr::MemoryRegionKey;
    use libfabric::MyRc;
    use libfabric::{
        async_::{
            av::AddressVectorBuilder,
            conn_ep::ConnectedEndpoint,
            connless_ep::ConnectionlessEndpoint,
            cq::{CompletionQueue, CompletionQueueBuilder},
            ep::{Endpoint, EndpointBuilder},
            eq::EventQueueBuilder,
        },
        domain::DomainBuilder,
        enums::{AVOptions, CqFormat, EndpointType},
        ep::Address,
        error::{Error, ErrorKind},
        fabric::FabricBuilder,
        info::InfoEntry,
        infocapsoptions::{Caps, CollCap, MsgDefaultCap, RmaDefaultCap, TagDefaultCap},
        iovec::IoVec,
        mr::{MappedMemoryRegionKey, MemoryRegion, MemoryRegionBuilder},
        Context, EqCaps, MappedAddress,
    };

    pub type SpinCq = libfabric::async_cq_caps_type!(CqCaps::FD);
    pub type WaitableEq = libfabric::eq_caps_type!(EqCaps::FD);

    pub enum CqType {
        Separate((CompletionQueue<SpinCq>, CompletionQueue<SpinCq>)),
        Shared(CompletionQueue<SpinCq>),
    }

    pub enum Either<L, R> {
        Left(L),
        Right(R),
    }

    impl CqType {
        pub fn tx_cq(&self) -> &CompletionQueue<SpinCq> {
            match self {
                CqType::Separate((tx, _)) => tx,
                CqType::Shared(tx) => tx,
            }
        }

        pub fn rx_cq(&self) -> &CompletionQueue<SpinCq> {
            match self {
                CqType::Separate((_, rx)) => rx,
                CqType::Shared(rx) => rx,
            }
        }
    }

    // pub enum EpType<I> {
    //     Connected(Endpoint<I>, EventQueue<WaitableEq>),
    //     Connectionless(Endpoint<I>, MappedAddress),
    // }

    pub enum MyEndpoint<I> {
        Connected(Arc<ConnectedEndpoint<I>>),
        Connectionless(Arc<ConnectionlessEndpoint<I>>),
    }

    pub struct Ofi<I> {
        pub info_entry: InfoEntry<I>,
        pub mr: Option<MemoryRegion>,
        pub remote_key: Option<MappedMemoryRegionKey>,
        pub remote_mem_addr: Option<(u64, u64)>,
        pub domain: Domain,
        pub cq_type: CqType,
        pub ep: Arc<MyEndpoint<I>>,
        pub mapped_addr: Option<MyRc<MappedAddress>>,
        pub reg_mem: Vec<u8>,
        // pub tx_pending_cnt: AtomicUsize,
        // pub tx_complete_cnt: AtomicUsize,
        // pub rx_pending_cnt: AtomicUsize,
        // pub rx_complete_cnt: AtomicUsize,
    }

    #[cfg(feature = "threading-fid")]
    pub trait IsSyncSend: Send + Sync {}

    #[cfg(feature = "threading-fid")]
    impl<I> IsSyncSend for Ofi<I> {}

    impl<I> Drop for Ofi<I> {
        fn drop(&mut self) {
            match self.info_entry.ep_attr().type_() {
                EndpointType::Msg => match self.ep.as_ref() {
                    MyEndpoint::Connected(ep) => ep.shutdown().unwrap(),
                    MyEndpoint::Connectionless(_) => todo!(),
                },
                EndpointType::Unspec | EndpointType::Dgram | EndpointType::Rdm => {}
            }
        }
    }

    macro_rules!  post_async{
    ($post_fn:ident, $prog_fn:ident, $cq:expr, $ep:ident, $( $x:expr),* ) => {
        loop {
            let ret = $ep.$post_fn($($x,)*).await;
            if ret.is_ok() {
                break;
            }
            else if let Err(ref err) = ret {
                if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
                    panic!("Unexpected error!")
                }

            }
        }
    };
}

    impl<I: MsgDefaultCap + Caps + 'static> Ofi<I> {
        pub fn new(
            info_entry: InfoEntry<I>,
            shared_cqs: bool,
            server: bool,
            name: &str,
        ) -> Result<Self, Error> {
            if server {
                unsafe { std::env::set_var(name, "1") };
            } else {
                while std::env::var(name).is_err() {
                    std::thread::yield_now();
                }
            }

            let format = if info_entry.caps().is_tagged() {
                CqFormat::Tagged
            } else {
                CqFormat::Data
            };

            let fabric = FabricBuilder::new().build(&info_entry).unwrap();
            let tx_cq_builder = CompletionQueueBuilder::new()
                .size(info_entry.tx_attr().size())
                .format(format);

            let rx_cq_builder = CompletionQueueBuilder::new()
                .size(info_entry.rx_attr().size())
                .format(format);

            let shared_cq_builder = CompletionQueueBuilder::new()
                .size(info_entry.rx_attr().size() + info_entry.tx_attr().size())
                .format(format);

            let ep_type = info_entry.ep_attr().type_();
            let domain;
            let cq_type;
            let mr;

            // let mut tx_pending_cnt: usize = 0;
            // let mut tx_complete_cnt: usize = 0;
            // let mut rx_pending_cnt: usize = 0;
            // let mut rx_complete_cnt: usize = 0;
            let mut reg_mem = vec![0u8; 1024 * 1024];
            let eq = EventQueueBuilder::new(&fabric).build().unwrap();

            let (info_entry, ep, mapped_addr) =  {
                    let info_entry = if matches!(ep_type, EndpointType::Msg) && server {                    
                        let pep = EndpointBuilder::new(&info_entry)
                            .build_passive(&fabric)
                            .unwrap();
                        pep.bind(&eq, 0).unwrap();
                        // println!("Listening!");
                        let listener = pep.listen_async().unwrap();
                        // println!("Awaiting!");
                        let event =
                            async_std::task::block_on(async { listener.next().await }).unwrap();
                        // println!("Done!");

                        match event {
                            libfabric::eq::Event::ConnReq(entry) => entry.info().unwrap(),
                            _ => panic!("Unexpected event"),
                        }
                    } else {
                        info_entry
                    };

                    domain = DomainBuilder::new(&fabric, &info_entry).build().unwrap();

                    cq_type = if shared_cqs {
                        CqType::Shared(shared_cq_builder.build(&domain).unwrap())
                    } else {
                        CqType::Separate((
                            tx_cq_builder.build(&domain).unwrap(),
                            rx_cq_builder.build(&domain).unwrap(),
                        ))
                    };

                    let ep_builder = EndpointBuilder::new(&info_entry);
                    let ep = match &cq_type {
                        CqType::Separate((tx_cq, rx_cq)) => ep_builder.build_with_separate_cqs(&domain, tx_cq, rx_cq),
                        CqType::Shared(scq) => ep_builder.build_with_shared_cq(&domain, scq),
                    }.unwrap();

                    match ep {
                        Endpoint::Connectionless(ep) => {
                            let av = match info_entry.domain_attr().av_type() {
                                libfabric::enums::AddressVectorType::Unspec => {
                                    AddressVectorBuilder::new(&eq)
                                }
                                _ => AddressVectorBuilder::new(&eq)
                                    .type_(*info_entry.domain_attr().av_type()),
                            }
                            .build(&domain)
                            .unwrap();
                            let ep = ep.enable(&av).unwrap();

                            mr = if info_entry.domain_attr().mr_mode().is_local()
                                || info_entry.caps().is_rma()
                            {
                                let mr = MemoryRegionBuilder::new(
                                    &mut reg_mem,
                                    libfabric::enums::HmemIface::System,
                                )
                                .access_read()
                                .access_write()
                                .access_send()
                                .access_recv()
                                .build(&domain)?;
                                let mr = match mr {
                                    libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                                    libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                                        match disabled_mr {
                                            libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => ep_binding_memory_region.enable(&ep).unwrap(),
                                            libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                                        }
                                    }
                                };
                                Some(mr)
                            } else {
                                None
                            };

                            let mapped_address = if let Some(dest_addr) = info_entry.dest_addr() {
                                let mapped_address = av
                                    .insert(std::slice::from_ref(dest_addr).into(), AVOptions::new())
                                    .unwrap()
                                    .pop()
                                    .unwrap()
                                    .unwrap();
                                let epname = ep.getname().unwrap();
                                let epname_bytes = epname.as_bytes();
                                let addrlen = epname_bytes.len();
                                reg_mem[..addrlen].copy_from_slice(epname_bytes);

                                let mut ctx = info_entry.allocate_context();
                                async_std::task::block_on(async {
                                    post_async!(
                                        send_to_async,
                                        ft_progress,
                                        cq_type.tx_cq(),
                                        ep,
                                        &reg_mem[..addrlen],
                                        None,
                                        &mapped_address,
                                        &mut ctx
                                    )
                                });

                                async_std::task::block_on(async {
                                    post_async!(
                                        recv_from_any_async,
                                        ft_progress,
                                        cq_type.rx_cq(),
                                        ep,
                                        std::slice::from_mut(&mut reg_mem[0]),
                                        None,
                                        &mut ctx
                                    )
                                });

                                MyRc::new(mapped_address)
                            } else {
                                let epname = ep.getname().unwrap();
                                let addrlen = epname.as_bytes().len();

                                let mr_desc = if let Some(ref mr) = mr {
                                    Some(mr.descriptor())
                                } else {
                                    None
                                };
                                let mut ctx = info_entry.allocate_context();

                                async_std::task::block_on(async {
                                    post_async!(
                                        recv_from_any_async,
                                        ft_progress,
                                        cq_type.rx_cq(),
                                        ep,
                                        &mut reg_mem[..addrlen],
                                        mr_desc.as_ref(),
                                        &mut ctx
                                    )
                                });

                                let remote_address = unsafe { Address::from_bytes(&reg_mem) };
                                let mapped_address = av
                                    .insert(
                                        std::slice::from_ref(&remote_address).into(),
                                        AVOptions::new(),
                                    )
                                    .unwrap()
                                    .pop()
                                    .unwrap()
                                    .unwrap();

                                async_std::task::block_on(async {
                                    post_async!(
                                        send_to_async,
                                        ft_progress,
                                        cq_type.tx_cq(),
                                        ep,
                                        &std::slice::from_ref(&reg_mem[0]),
                                        mr_desc.as_ref(),
                                        &mapped_address,
                                        &mut ctx
                                    )
                                });

                                MyRc::new(mapped_address)
                            };
                            (
                                info_entry,
                                Arc::new(MyEndpoint::Connectionless(Arc::new(ep))),
                                Some(mapped_address),
                            )
                        },
                        Endpoint::ConnectionOriented(ep) => {
                            let ep = ep.enable(&eq).unwrap();

                            let ep = if !server {
                                let err = async_std::task::block_on(async {
                                    ep.connect_async(info_entry.dest_addr().unwrap()).await
                                });
                                match err {
                                    Ok(ep) => ep,
                                    Err(error) => match error.kind {
                                        ErrorKind::ErrorInEventQueue(q_error) => {
                                            panic!("{:?}", q_error.error())
                                        }
                                        _ => panic!("Other error"),
                                    },
                                }
                            } else {
                                async_std::task::block_on(async { ep.accept_async().await }).unwrap()
                            };

                            mr = if info_entry.domain_attr().mr_mode().is_local()
                                || info_entry.caps().is_rma()
                            {
                                let mr = MemoryRegionBuilder::new(
                                    &mut reg_mem,
                                    libfabric::enums::HmemIface::System,
                                )
                                .access_read()
                                .access_write()
                                .access_send()
                                .access_recv()
                                .build(&domain)?;
                                let mr = match mr {
                                    libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                                    libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                                        match disabled_mr {
                                            libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => ep_binding_memory_region.enable(&ep).unwrap(),
                                            libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                                        }
                                    }
                                };
                                Some(mr)
                            } else {
                                None
                            };

                            (
                                info_entry,
                                Arc::new(MyEndpoint::Connected(Arc::new(ep))),
                                None,
                            )
                        }
                    }

                };
            if server {
                unsafe { std::env::remove_var(name) };
            }

            Ok(Self {
                info_entry,
                mapped_addr,
                mr,
                remote_key: None,
                remote_mem_addr: None,
                cq_type,
                domain,
                ep,
                reg_mem,
                // tx_pending_cnt,
                // tx_complete_cnt,
                // rx_pending_cnt,
                // rx_complete_cnt,
            })
        }
    }

    impl<I: TagDefaultCap + 'static> Ofi<I> {
        pub fn tsend(&self, buf: &[u8], tag: u64, data: Option<u64>) {
            let handles: Vec<_> = (0..100)
                .map(|_| {
                    let mut ctx = self.info_entry.allocate_context();

                    let reg_mem = buf.to_vec();

                    let mr =
                        MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
                            .access_recv()
                            .access_send()
                            .build(&self.domain)
                            .unwrap();

                    let mr = match mr {
                        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                            match disabled_mr {
                                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&self.ep, ep_binding_memory_region),
                                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                            }
                        }
                    };
                    let ep = self.ep.clone();
                    let inject_size = self.info_entry.tx_attr().inject_size();
                    let mapped_addr = Arc::new(self.mapped_addr.clone());

                    (async_std::task::spawn(async move {
                        let desc = Some(mr.descriptor());
                        match &ep.as_ref() {
                            MyEndpoint::Connectionless(ep) => {
                                if reg_mem.len() <= inject_size {
                                    if data.is_some() {
                                        ep.tinjectdata_to_async(
                                            &reg_mem,
                                            data.unwrap(),
                                            mapped_addr.as_ref().as_ref().unwrap(),
                                            tag,
                                        )
                                        .await
                                    } else {
                                        ep.tinject_to_async(
                                            &reg_mem,
                                            mapped_addr.as_ref().as_ref().unwrap(),
                                            tag,
                                        )
                                        .await
                                    }
                                } else {
                                    if data.is_some() {
                                        ep.tsenddata_to_async(
                                            &reg_mem,
                                            desc.as_ref(),
                                            data.unwrap(),
                                            mapped_addr.as_ref().as_ref().unwrap(),
                                            tag,
                                            &mut ctx,
                                        )
                                        .await
                                    } else {
                                        ep.tsend_to_async(
                                            &reg_mem,
                                            desc.as_ref(),
                                            mapped_addr.as_ref().as_ref().unwrap(),
                                            tag,
                                            &mut ctx,
                                        )
                                        .await
                                    }
                                    .map(|_| {})
                                }
                            }
                            MyEndpoint::Connected(ep) => {
                                if reg_mem.len() <= inject_size {
                                    if data.is_some() {
                                        ep.tinjectdata_async(&reg_mem, data.unwrap(), tag).await
                                    } else {
                                        ep.tinject_async(&reg_mem, tag).await
                                    }
                                } else {
                                    if data.is_some() {
                                        ep.tsenddata_async(
                                            &reg_mem,
                                            desc.as_ref(),
                                            data.unwrap(),
                                            tag,
                                            &mut ctx,
                                        )
                                        .await
                                    } else {
                                        ep.tsend_async(&reg_mem, desc.as_ref(), tag, &mut ctx).await
                                    }
                                    .map(|_| {})
                                }
                            }
                        }
                        .unwrap()
                    }),)
                })
                .collect();

            handles
                .into_iter()
                .for_each(|h| async_std::task::block_on(h.0));
        }

        pub fn tsendv(
            &mut self,
            iov: &[IoVec],
            desc: Option<&[MemoryRegionDesc]>,
            tag: u64,
            ctx: &mut Context,
        ) {
            loop {
                let err = match self.ep.as_ref() {
                    MyEndpoint::Connectionless(ep) => async_std::task::block_on(async {
                        ep.tsendv_to_async(iov, desc, self.mapped_addr.as_ref().unwrap(), tag, ctx)
                            .await
                    }),
                    MyEndpoint::Connected(ep) => async_std::task::block_on(async {
                        ep.tsendv_async(iov, desc, tag, ctx).await
                    }),
                };
                match err {
                    Ok(_) => break,
                    Err(err) => {
                        if !matches!(err.kind, ErrorKind::TryAgain) {
                            panic!("{:?}", err);
                        }
                    }
                }
            }
        }

        //     pub fn trecvv(&mut self, iov: &[IoVecMut], desc: &mut [MemoryRegionDesc], tag: u64, ctx: &mut Context) {
        //         loop {
        //             let err = match self.ep {
        //                 MyEndpoint::Connectionless(ep) => {
        //                     async_std::task::block_on(async {
        //                     ep.trecvv_from_async(iov, desc, self.mapped_addr.as_ref().unwrap(), tag, 0, ctx).await})
        //                 }
        //                 MyEndpoint::Connected(ep) => async_std::task::block_on(async {ep.trecvv_async(iov, desc, 0, tag, ctx).await}),
        //             };
        //             match err {
        //                 Ok(_) => break,
        //                 Err(err) => {
        //                     if !matches!(err.kind, ErrorKind::TryAgain) {
        //                         panic!("{:?}", err);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        pub fn trecv(&mut self, buf: &mut [u8], tag: u64) -> Vec<Vec<u8>> {
            let handles: Vec<_> = (0..100)
                .map(|_| {
                    let ep = self.ep.clone();
                    let mapped_addr = Arc::new(self.mapped_addr.clone());

                    let mut ctx = self.info_entry.allocate_context();

                    let mut reg_mem = buf.to_vec();

                    let mr =
                        MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
                            .access_recv()
                            .access_send()
                            .build(&self.domain)
                            .unwrap();

                    let mr = match mr {
                        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                            match disabled_mr {
                                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&self.ep, ep_binding_memory_region),
                                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                            }
                        }
                    };

                    (async_std::task::spawn(async move {
                        let desc = Some(mr.descriptor());
                        match ep.as_ref() {
                            MyEndpoint::Connected(ep) => {
                                ep.trecv_async(&mut reg_mem, desc.as_ref(), tag, None, &mut ctx)
                                    .await
                            }
                            MyEndpoint::Connectionless(ep) => {
                                ep.trecv_from_async(
                                    &mut reg_mem,
                                    desc.as_ref(),
                                    mapped_addr.as_ref().as_ref().unwrap(),
                                    tag,
                                    None,
                                    &mut ctx,
                                )
                                .await
                            }
                        }
                        .unwrap();
                        reg_mem
                    }),)
                })
                .collect();

            handles
                .into_iter()
                .map(|h| async_std::task::block_on(h.0))
                .collect()
        }

        //     pub fn tsendmsg(&mut self, msg: &mut Either<MsgTagged, MsgTaggedConnected>, ctx: &mut Context) {
        //         loop {
        //             let err = match &self.ep {
        //                 MyEndpoint::Connectionless(ep) => match msg {
        //                     Either::Left(msg) => {
        //                         async_std::task::block_on(async {
        //                             ep.tsendmsg_to_async(msg, TferOptions::new().remote_cq_data(), ctx).await
        //                         })
        //                     },
        //                     Either::Right(_) => panic!("Wrong message type used"),
        //                 },
        //                 MyEndpoint::Connected(ep) => match msg {
        //                     Either::Left(_) => panic!("Wrong message type used"),
        //                     Either::Right(msg) =>
        //                         async_std::task::block_on(async {
        //                             ep.tsendmsg_async(msg, TferOptions::new().remote_cq_data(), ctx).await
        //                         })
        //                     ,
        //                 },
        //             };

        //             match err {
        //                 Ok(_) => break,
        //                 Err(err) => {
        //                     if !matches!(err.kind, ErrorKind::TryAgain) {
        //                         panic!("{:?}", err);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     pub fn trecvmsg(&mut self, msg: &mut Either<MsgTaggedMut, MsgTaggedConnectedMut>, ctx: &mut Context) {
        //         loop {
        //             let err = match &self.ep {
        //                 MyEndpoint::Connectionless(ep) => match msg {
        //                     Either::Left(msg) => {
        //                         async_std::task::block_on(async {ep.trecvmsg_from_async(msg, TferOptions::new(), ctx).await})
        //                     },
        //                     Either::Right(_) => panic!("Wrong message type"),
        //                 },
        //                 MyEndpoint::Connected(ep) => match msg {
        //                     Either::Left(_) => panic!("Wrong message type"),
        //                     Either::Right(msg) => {
        //                         async_std::task::block_on(async {ep.trecvmsg_async(msg, TferOptions::new(), ctx).await})
        //                     },
        //                 },
        //             };

        //             match err {
        //                 Ok(_) => break,
        //                 Err(err) => {
        //                     if !matches!(err.kind, ErrorKind::TryAgain) {
        //                         panic!("{:?}", err);
        //                     }
        //                 }
        //             }
        //         }
        //     }
    }

    impl<I: MsgDefaultCap + 'static> Ofi<I> {
        pub fn send(&self, buf: &[u8], data: Option<u64>) {
            let handles: Vec<_> = (0..100)
                .map(|_| {
                    let mut ctx = self.info_entry.allocate_context();

                    let reg_mem = buf.to_vec();

                    let mr =
                        MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
                            .access_recv()
                            .access_send()
                            .build(&self.domain)
                            .unwrap();

                    let mr = match mr {
                        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                            match disabled_mr {
                                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&self.ep, ep_binding_memory_region),
                                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                            }
                        }
                    };
                    let ep = self.ep.clone();
                    let inject_size = self.info_entry.tx_attr().inject_size();
                    let mapped_addr = Arc::new(self.mapped_addr.clone());

                    (async_std::task::spawn(async move {
                        let desc = Some(mr.descriptor());
                        loop {
                            let err = match &ep.as_ref() {
                                MyEndpoint::Connectionless(ep) => {
                                    if reg_mem.len() <= inject_size {
                                        if data.is_some() {
                                            ep.injectdata_to_async(
                                                &reg_mem,
                                                data.unwrap(),
                                                mapped_addr.as_ref().as_ref().unwrap(),
                                            )
                                            .await
                                        } else {
                                            ep.inject_to_async(
                                                &reg_mem,
                                                mapped_addr.as_ref().as_ref().unwrap(),
                                            )
                                            .await
                                        }
                                    } else {
                                        if data.is_some() {
                                            ep.senddata_to_async(
                                                &reg_mem,
                                                desc.as_ref(),
                                                data.unwrap(),
                                                mapped_addr.as_ref().as_ref().unwrap(),
                                                &mut ctx,
                                            )
                                            .await
                                        } else {
                                            ep.send_to_async(
                                                &reg_mem,
                                                desc.as_ref(),
                                                mapped_addr.as_ref().as_ref().unwrap(),
                                                &mut ctx,
                                            )
                                            .await
                                        }
                                        .map(|_| {})
                                    }
                                }
                                MyEndpoint::Connected(ep) => {
                                    if reg_mem.len() <= inject_size {
                                        if data.is_some() {
                                            ep.injectdata_async(&reg_mem, data.unwrap()).await
                                        } else {
                                            ep.inject_async(&reg_mem).await
                                        }
                                    } else {
                                        if data.is_some() {
                                            ep.senddata_async(
                                                &reg_mem,
                                                desc.as_ref(),
                                                data.unwrap(),
                                                &mut ctx,
                                            )
                                            .await
                                        } else {
                                            ep.send_async(&reg_mem, desc.as_ref(), &mut ctx).await
                                        }
                                        .map(|_| {})
                                    }
                                }
                            };
                            match err {
                                Ok(_) => break,
                                Err(err) => {
                                    if !matches!(err.kind, ErrorKind::TryAgain) {
                                        panic!("{:?}", err);
                                    }
                                }
                            }
                        }
                    }),)
                })
                .collect();

            handles
                .into_iter()
                .for_each(|h| async_std::task::block_on(h.0));
        }

        //     pub fn sendv(&mut self, iov: &[IoVec], desc: &mut [MemoryRegionDesc], ctx: &mut Context) {
        //         loop {
        //             let err = match &self.ep {
        //                 MyEndpoint::Connectionless(ep) => {
        //                     async_std::task::block_on(async {
        //                         ep.sendv_to_async(iov, desc, self.mapped_addr.as_ref().unwrap(), ctx).await
        //                     })
        //                 }
        //                 MyEndpoint::Connected(ep) =>
        //                     async_std::task::block_on(async {
        //                         ep.sendv_async(iov, desc, ctx).await
        //                     })
        //             };
        //             match err {
        //                 Ok(_) => break,
        //                 Err(err) => {
        //                     if !matches!(err.kind, ErrorKind::TryAgain) {
        //                         panic!("{:?}", err);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     pub fn recvv(&mut self, iov: &[IoVecMut], desc: &mut [MemoryRegionDesc], ctx: &mut Context) {
        //         loop {
        //             let err = match &self.ep {
        //                 MyEndpoint::Connectionless(ep) => {
        //                     async_std::task::block_on(async {
        //                         ep.recvv_from_async(iov, desc, self.mapped_addr.as_ref().unwrap(), ctx).await
        //                     })
        //                 }
        //                 MyEndpoint::Connected(ep) => {
        //                     async_std::task::block_on(async {
        //                         ep.recvv_async(iov, desc, ctx).await
        //                     })
        //                 }
        //             };
        //             match err {
        //                 Ok(_) => break,
        //                 Err(err) => {
        //                     if !matches!(err.kind, ErrorKind::TryAgain) {
        //                         panic!("{:?}", err);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        pub fn recv(&mut self, buff: &[u8]) -> Vec<Vec<u8>> {
            let handles: Vec<_> = (0..100)
                .map(|_| {
                    let ep = self.ep.clone();
                    let mapped_addr = Arc::new(self.mapped_addr.clone());

                    let mut ctx = self.info_entry.allocate_context();

                    let mut reg_mem = buff.to_vec();

                    let mr =
                        MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
                            .access_recv()
                            .access_send()
                            .build(&self.domain)
                            .unwrap();

                    let mr = match mr {
                        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                            match disabled_mr {
                                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&self.ep, ep_binding_memory_region),
                                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                            }
                        }
                    };

                    (async_std::task::spawn(async move {
                        let desc = Some(mr.descriptor());
                        match ep.as_ref() {
                            MyEndpoint::Connected(ep) => {
                                ep.recv_async(&mut reg_mem, desc.as_ref(), &mut ctx).await
                            }
                            MyEndpoint::Connectionless(ep) => {
                                ep.recv_from_async(
                                    &mut reg_mem,
                                    desc.as_ref(),
                                    mapped_addr.as_ref().as_ref().unwrap(),
                                    &mut ctx,
                                )
                                .await
                            }
                        }
                        .unwrap();

                        reg_mem
                    }),)
                })
                .collect();

            handles
                .into_iter()
                .map(|h| async_std::task::block_on(h.0))
                .collect()
        }
        // }

        //     pub fn sendmsg(&mut self, msg: &mut Either<Msg, MsgConnected>, ctx: &mut Context) {
        //         loop {
        //             let err = match &self.ep {
        //                 MyEndpoint::Connectionless(ep) => match msg {
        //                     Either::Left(msg) => {
        //                         async_std::task::block_on(async {
        //                             ep.sendmsg_to_async(msg, TferOptions::new().remote_cq_data(), ctx).await
        //                         })
        //                     },
        //                     Either::Right(_) => panic!("Wrong msg type"),
        //                 },
        //                 MyEndpoint::Connected(ep) => match msg {
        //                     Either::Left(_) => panic!("Wrong msg type"),
        //                     Either::Right(msg) => {
        //                         async_std::task::block_on(async {

        //                             ep.sendmsg_async(msg, TferOptions::new().remote_cq_data(), ctx).await
        //                         })
        //                     },
        //                 },
        //             };

        //             match err {
        //                 Ok(_) => break,
        //                 Err(err) => {
        //                     if !matches!(err.kind, ErrorKind::TryAgain) {
        //                         panic!("{:?}", err);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     pub fn recvmsg(&mut self, msg: &mut Either<MsgMut, MsgConnectedMut>, ctx: &mut Context) {
        //         loop {
        //             let err = match &self.ep {
        //                 MyEndpoint::Connectionless(ep) => match msg {
        //                     Either::Left(msg) => {
        //                         async_std::task::block_on(async {

        //                             ep.recvmsg_from_async(msg, TferOptions::new(), ctx).await
        //                         })
        //                     },
        //                     Either::Right(_) => panic!("Wrong message type"),
        //                 },
        //                 MyEndpoint::Connected(ep) => match msg {
        //                     Either::Left(_) => panic!("Wrong message type"),
        //                     Either::Right(msg) => {
        //                         async_std::task::block_on(async {
        //                             ep.recvmsg_async(msg, TferOptions::new(), ctx).await
        //                         })
        //                     },
        //                 },
        //             };

        //             match err {
        //                 Ok(_) => break,
        //                 Err(err) => {
        //                     if !matches!(err.kind, ErrorKind::TryAgain) {
        //                         panic!("{:?}", err);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        pub fn exchange_keys(
            &mut self,
            server: bool,
            key: MemoryRegionKey,
            addr: usize,
            len: usize,
        ) {
            let mut len = unsafe {
                std::slice::from_raw_parts(
                    &len as *const usize as *const u8,
                    std::mem::size_of::<usize>(),
                )
            }
            .to_vec();
            let mut addr = unsafe {
                std::slice::from_raw_parts(
                    &addr as *const usize as *const u8,
                    std::mem::size_of::<usize>(),
                )
            }
            .to_vec();

            let key_bytes = key.to_bytes();
            let mut reg_mem = Vec::new();
            reg_mem.append(&mut key_bytes.clone());
            reg_mem.append(&mut len);
            reg_mem.append(&mut addr);
            let total_len = reg_mem.len();
            reg_mem.append(&mut vec![0; total_len]);

            let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
                .access_recv()
                .access_send()
                .build(&self.domain)
                .unwrap();

            let mr = match mr {
                libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                    match disabled_mr {
                        libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&self.ep, ep_binding_memory_region),
                        libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                    }
                }
            };
            let mut ctx = self.info_entry.allocate_context();

            let desc = Some(mr.descriptor());
            if server {
                let _res = match self.ep.as_ref() {
                    MyEndpoint::Connected(ep) => async_std::task::block_on(async {
                        ep.send_async(
                            &reg_mem[..key_bytes.len() + 2 * std::mem::size_of::<usize>()],
                            desc.as_ref(),
                            &mut ctx,
                        )
                        .await
                    })
                    .unwrap(),
                    MyEndpoint::Connectionless(ep) => async_std::task::block_on(async {
                        ep.send_to_async(
                            &reg_mem[..key_bytes.len() + 2 * std::mem::size_of::<usize>()],
                            desc.as_ref(),
                            self.mapped_addr.as_ref().unwrap(),
                            &mut ctx,
                        )
                        .await
                    })
                    .unwrap(),
                };

                let _res = match self.ep.as_ref() {
                    MyEndpoint::Connected(ep) => async_std::task::block_on(async {
                        ep.recv_async(
                            &mut reg_mem[key_bytes.len() + 2 * std::mem::size_of::<usize>()
                                ..2 * key_bytes.len() + 4 * std::mem::size_of::<usize>()],
                            desc.as_ref(),
                            &mut ctx,
                        )
                        .await
                    })
                    .unwrap(),
                    MyEndpoint::Connectionless(ep) => async_std::task::block_on(async {
                        ep.recv_from_async(
                            &mut reg_mem[key_bytes.len() + 2 * std::mem::size_of::<usize>()
                                ..2 * key_bytes.len() + 4 * std::mem::size_of::<usize>()],
                            desc.as_ref(),
                            self.mapped_addr.as_ref().unwrap(),
                            &mut ctx,
                        )
                        .await
                    })
                    .unwrap(),
                };
            } else {
                let _res = match self.ep.as_ref() {
                    MyEndpoint::Connected(ep) => async_std::task::block_on(async {
                        ep.recv_async(
                            &mut reg_mem[key_bytes.len() + 2 * std::mem::size_of::<usize>()
                                ..2 * key_bytes.len() + 4 * std::mem::size_of::<usize>()],
                            desc.as_ref(),
                            &mut ctx,
                        )
                        .await
                    })
                    .unwrap(),
                    MyEndpoint::Connectionless(ep) => async_std::task::block_on(async {
                        ep.recv_from_async(
                            &mut reg_mem[key_bytes.len() + 2 * std::mem::size_of::<usize>()
                                ..2 * key_bytes.len() + 4 * std::mem::size_of::<usize>()],
                            desc.as_ref(),
                            self.mapped_addr.as_ref().unwrap(),
                            &mut ctx,
                        )
                        .await
                    })
                    .unwrap(),
                };

                let _res = match self.ep.as_ref() {
                    MyEndpoint::Connected(ep) => async_std::task::block_on(async {
                        ep.send_async(
                            &reg_mem[..key_bytes.len() + 2 * std::mem::size_of::<usize>()],
                            desc.as_ref(),
                            &mut ctx,
                        )
                        .await
                    })
                    .unwrap(),
                    MyEndpoint::Connectionless(ep) => async_std::task::block_on(async {
                        ep.send_to_async(
                            &reg_mem[..key_bytes.len() + 2 * std::mem::size_of::<usize>()],
                            desc.as_ref(),
                            self.mapped_addr.as_ref().unwrap(),
                            &mut ctx,
                        )
                        .await
                    })
                    .unwrap(),
                };
            }
            let remote_key = unsafe {
                MappedMemoryRegionKey::from_raw(
                    &reg_mem[key_bytes.len() + 2 * std::mem::size_of::<usize>()
                        ..2 * key_bytes.len() + 2 * std::mem::size_of::<usize>()],
                    &self.domain,
                )
            }
            .unwrap();
            let len = unsafe {
                std::slice::from_raw_parts(
                    reg_mem[2 * key_bytes.len() + 2 * std::mem::size_of::<usize>()
                        ..2 * key_bytes.len() + 3 * std::mem::size_of::<usize>()]
                        .as_ptr() as *const u8 as *const u64,
                    1,
                )
            }[0];
            let addr = unsafe {
                std::slice::from_raw_parts(
                    reg_mem[2 * key_bytes.len() + 3 * std::mem::size_of::<usize>()
                        ..2 * key_bytes.len() + 4 * std::mem::size_of::<usize>()]
                        .as_ptr() as *const u8 as *const u64,
                    1,
                )
            }[0];
            self.remote_key = Some(remote_key);
            self.remote_mem_addr = Some((addr, addr + len));
        }
    }

    impl<I: MsgDefaultCap + RmaDefaultCap + 'static> Ofi<I> {
        pub fn write(&mut self, buf: &[u8], dest_addr: u64, data: Option<u64>) {
            let (start, _end) = self.remote_mem_addr.unwrap();
            let handles: Vec<_> = (0..100)
                .map(|_| {
                    let ep = self.ep.clone();
                    let mapped_addr = Arc::new(self.mapped_addr.clone());
                    let key = self.remote_key.clone();

                    let mut ctx = self.info_entry.allocate_context();

                    let reg_mem = buf.to_vec();

                    let mr =
                        MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
                            .access_recv()
                            .access_send()
                            .build(&self.domain)
                            .unwrap();

                    let mr = match mr {
                        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                            match disabled_mr {
                                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&self.ep, ep_binding_memory_region),
                                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                            }
                        }
                    };
                    let injec_size = self.info_entry.tx_attr().inject_size();

                    (async_std::task::spawn(async move {
                        let desc = Some(mr.descriptor());
                        loop {
                            let err = match &ep.as_ref() {
                                MyEndpoint::Connectionless(ep) => {
                                    if &reg_mem.len() <= &injec_size {
                                        if data.is_some() {
                                            unsafe {
                                                ep.inject_writedata_to_async(
                                                    &reg_mem,
                                                    data.unwrap(),
                                                    mapped_addr.as_ref().as_ref().unwrap(),
                                                    start + dest_addr,
                                                    key.as_ref().as_ref().unwrap(),
                                                )
                                                .await
                                            }
                                        } else {
                                            unsafe {
                                                ep.inject_write_to_async(
                                                    &reg_mem,
                                                    mapped_addr.as_ref().as_ref().unwrap(),
                                                    start + dest_addr,
                                                    key.as_ref().as_ref().unwrap(),
                                                )
                                                .await
                                            }
                                        }
                                    } else {
                                        if data.is_some() {
                                            unsafe {
                                                ep.writedata_to_async(
                                                    &reg_mem,
                                                    desc.as_ref(),
                                                    data.unwrap(),
                                                    mapped_addr.as_ref().as_ref().unwrap(),
                                                    start + dest_addr,
                                                    key.as_ref().as_ref().unwrap(),
                                                    &mut ctx,
                                                )
                                                .await
                                            }
                                        } else {
                                            unsafe {
                                                ep.write_to_async(
                                                    &reg_mem,
                                                    desc.as_ref(),
                                                    mapped_addr.as_ref().as_ref().unwrap(),
                                                    start + dest_addr,
                                                    key.as_ref().as_ref().unwrap(),
                                                    &mut ctx,
                                                )
                                                .await
                                            }
                                        }
                                        .map(|_| {})
                                    }
                                }
                                MyEndpoint::Connected(ep) => {
                                    if &reg_mem.len() <= &injec_size {
                                        if data.is_some() {
                                            unsafe {
                                                ep.inject_writedata_async(
                                                    &reg_mem,
                                                    data.unwrap(),
                                                    start + dest_addr,
                                                    key.as_ref().as_ref().unwrap(),
                                                )
                                                .await
                                            }
                                        } else {
                                            unsafe {
                                                ep.inject_write_async(
                                                    &reg_mem,
                                                    start + dest_addr,
                                                    key.as_ref().as_ref().unwrap(),
                                                )
                                                .await
                                            }
                                        }
                                    } else {
                                        if data.is_some() {
                                            unsafe {
                                                ep.writedata_async(
                                                    &reg_mem,
                                                    desc.as_ref(),
                                                    data.unwrap(),
                                                    start + dest_addr,
                                                    key.as_ref().as_ref().unwrap(),
                                                    &mut ctx,
                                                )
                                                .await
                                            }
                                        } else {
                                            unsafe {
                                                ep.write_async(
                                                    &reg_mem,
                                                    desc.as_ref(),
                                                    start + dest_addr,
                                                    key.as_ref().as_ref().unwrap(),
                                                    &mut ctx,
                                                )
                                                .await
                                            }
                                        }
                                        .map(|_| {})
                                    }
                                }
                            };
                            match err {
                                Ok(_) => break,
                                Err(err) => {
                                    if !matches!(err.kind, ErrorKind::TryAgain) {
                                        panic!("{:?}", err);
                                    }
                                }
                            }
                        }
                    }),)
                })
                .collect();

            handles
                .into_iter()
                .for_each(|h| async_std::task::block_on(h.0))
        }
        // }
        pub fn read(&mut self, buf: &mut [u8], dest_addr: u64) -> Vec<Vec<u8>> {
            let (start, _end) = self.remote_mem_addr.unwrap();
            let handles: Vec<_> = (0..100)
                .map(|_| {
                    let ep = self.ep.clone();
                    let mapped_addr = Arc::new(self.mapped_addr.clone());
                    let key = Arc::new(self.remote_key.clone());

                    let mut ctx = self.info_entry.allocate_context();

                    let mut reg_mem = buf.to_vec();

                    let mr =
                        MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
                            .access_recv()
                            .access_send()
                            .build(&self.domain)
                            .unwrap();

                    let mr = match mr {
                        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                            match disabled_mr {
                                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&self.ep, ep_binding_memory_region),
                                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                            }
                        }
                    };

                    (async_std::task::spawn(async move {
                        let desc = Some(mr.descriptor());
                        match ep.as_ref() {
                            MyEndpoint::Connectionless(ep) => unsafe {
                                async_std::task::block_on(async {
                                    ep.read_from_async(
                                        &mut reg_mem,
                                        desc.as_ref(),
                                        mapped_addr.as_ref().as_ref().unwrap(),
                                        start + dest_addr,
                                        key.as_ref().as_ref().unwrap(),
                                        &mut ctx,
                                    )
                                    .await
                                })
                            },
                            MyEndpoint::Connected(ep) => unsafe {
                                async_std::task::block_on(async {
                                    ep.read_async(
                                        &mut reg_mem,
                                        desc.as_ref(),
                                        start + dest_addr,
                                        key.as_ref().as_ref().unwrap(),
                                        &mut ctx,
                                    )
                                    .await
                                })
                            },
                        }
                        .unwrap();

                        reg_mem
                    }),)
                })
                .collect();
            handles
                .into_iter()
                .map(|h| async_std::task::block_on(h.0))
                .collect()
        }
    }

    //     pub fn writev(&mut self, iov: &[IoVec], dest_addr: u64, desc: &mut [MemoryRegionDesc], ctx: &mut Context) {
    //         let (start, _end) = self.remote_mem_addr.unwrap();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => unsafe {
    //                     async_std::task::block_on(async {

    //                         ep.writev_to_async(
    //                             iov,
    //                             desc,
    //                             self.mapped_addr.as_ref().unwrap(),
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //                 MyEndpoint::Connected(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.writev_async(
    //                             iov,
    //                             desc,
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn readv(&mut self, iov: &[IoVecMut], dest_addr: u64, desc: &mut [MemoryRegionDesc], ctx: &mut Context) {
    //         let (start, _end) = self.remote_mem_addr.unwrap();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => unsafe {
    //                     async_std::task::block_on(async {

    //                         ep.readv_from_async(
    //                             iov,
    //                             desc,
    //                             self.mapped_addr.as_ref().unwrap(),
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //                 MyEndpoint::Connected(ep) => unsafe {
    //                     async_std::task::block_on(async {

    //                         ep.readv_async(
    //                             iov,
    //                             desc,
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     // [TODO] Enabling .remote_cq_data causes the buffer not being written correctly
    //     // on the remote side.
    //     pub fn writemsg(&mut self, msg: &mut Either<MsgRma, MsgRmaConnected>, ctx: &mut Context) {
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => match msg {
    //                     Either::Left(msg) => unsafe {
    //                         async_std::task::block_on(async {
    //                             ep.writemsg_to_async(msg, WriteMsgOptions::new(), ctx).await
    //                         })
    //                     },
    //                     Either::Right(_) => panic!("Wrong message type"),
    //                 },
    //                 MyEndpoint::Connected(ep) => match msg {
    //                     Either::Left(_) => panic!("Wrong message type"),
    //                     Either::Right(msg) => unsafe {
    //                         async_std::task::block_on(async {
    //                             ep.writemsg_async(msg, WriteMsgOptions::new(), ctx).await
    //                         })
    //                     },
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn readmsg(&mut self, msg: &mut Either<MsgRmaMut, MsgRmaConnectedMut>, ctx: &mut Context) {
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => match msg {
    //                     Either::Left(msg) => unsafe {
    //                         async_std::task::block_on(async {
    //                             ep.readmsg_from_async(msg, ReadMsgOptions::new(), ctx).await
    //                         })
    //                     },
    //                     Either::Right(_) => todo!(),
    //                 },
    //                 MyEndpoint::Connected(ep) => match msg {
    //                     Either::Left(_) => panic!("Wrong message type"),
    //                     Either::Right(msg) => unsafe {
    //                         async_std::task::block_on(async {
    //                             ep.readmsg_async(msg, ReadMsgOptions::new(), ctx).await
    //                         })
    //                     },
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    // impl<I: AtomicDefaultCap> Ofi<I> {
    //     pub fn atomic<T: libfabric::AsFiType>(
    //         &mut self,
    //         buf: &[T],
    //         dest_addr: u64,
    //         desc: &mut MemoryRegionDesc,
    //         op: AtomicOp,
    //         ctx: &mut Context
    //     ) {
    //         let (start, _end) = self.remote_mem_addr.unwrap();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => {
    //                     if buf.len() <= self.info_entry.tx_attr().inject_size() {
    //                         unsafe {
    //                             ep.inject_atomic_to(
    //                                 buf,
    //                                 self.mapped_addr.as_ref().unwrap(),
    //                                 start + dest_addr,
    //                                 self.remote_key.as_ref().unwrap(),
    //                                 op,
    //                             )
    //                         }
    //                     } else {
    //                         unsafe {
    //                             async_std::task::block_on(async {
    //                             ep.atomic_to_async(
    //                                 buf,
    //                                 desc,
    //                                 self.mapped_addr.as_ref().unwrap(),
    //                                 start + dest_addr,
    //                                 self.remote_key.as_ref().unwrap(),
    //                                 op,
    //                                 ctx
    //                             ).await})
    //                         }.map(|_| {})
    //                     }
    //                 }
    //                 MyEndpoint::Connected(ep) => {
    //                     if buf.len() <= self.info_entry.tx_attr().inject_size() {
    //                         unsafe {
    //                             ep.inject_atomic(
    //                                 buf,
    //                                 start + dest_addr,
    //                                 self.remote_key.as_ref().unwrap(),
    //                                 op,
    //                             )
    //                         }
    //                     } else {
    //                         unsafe {
    //                             async_std::task::block_on(async {
    //                                 ep.atomic_async(
    //                                     buf,
    //                                     desc,
    //                                     start + dest_addr,
    //                                     self.remote_key.as_ref().unwrap(),
    //                                     op,
    //                                     ctx
    //                                 ).await
    //                             })
    //                         }.map(|_| {})
    //                     }
    //                 }
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn atomicv<T: libfabric::AsFiType>(
    //         &mut self,
    //         ioc: &[libfabric::iovec::Ioc<T>],
    //         dest_addr: u64,
    //         desc: &mut [MemoryRegionDesc],
    //         op: AtomicOp,
    //         ctx: &mut Context
    //     ) {
    //         let (start, _end) = self.remote_mem_addr.unwrap();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.atomicv_to_async(
    //                             ioc,
    //                             desc,
    //                             self.mapped_addr.as_ref().unwrap(),
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //                 MyEndpoint::Connected(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.atomicv_async(
    //                             ioc,
    //                             desc,
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn atomicmsg<T: libfabric::AsFiType + 'static>(
    //         &mut self,
    //         msg: &mut Either<MsgAtomic<T>, MsgAtomicConnected<T>>,
    //         ctx: &mut Context
    //     ) {
    //         let opts = AtomicMsgOptions::new();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => match msg {
    //                     Either::Left(msg) => unsafe { async_std::task::block_on(async { ep.atomicmsg_to_async(msg, opts, ctx).await}) },
    //                     Either::Right(_) => todo!(),
    //                 },
    //                 MyEndpoint::Connected(ep) => match msg {
    //                     Either::Left(_) => todo!(),
    //                     Either::Right(msg) => unsafe { async_std::task::block_on(async { ep.atomicmsg_async(msg, opts, ctx).await}) },
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn fetch_atomic<T: libfabric::AsFiType>(
    //         &mut self,
    //         buf: &[T],
    //         res: &mut [T],
    //         dest_addr: u64,
    //         desc: &mut MemoryRegionDesc,
    //         res_desc: &mut MemoryRegionDesc,
    //         op: FetchAtomicOp,
    //         ctx: &mut Context
    //     ) {
    //         let (start, _end) = self.remote_mem_addr.unwrap();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.fetch_atomic_from_async(
    //                             buf,
    //                             desc,
    //                             res,
    //                             res_desc,
    //                             self.mapped_addr.as_ref().unwrap(),
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //                 MyEndpoint::Connected(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.fetch_atomic_async(
    //                             buf,
    //                             desc,
    //                             res,
    //                             res_desc,
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }

    //         }
    //     }

    //     pub fn fetch_atomicv<T: libfabric::AsFiType>(
    //         &mut self,
    //         ioc: &[libfabric::iovec::Ioc<T>],
    //         res_ioc: &mut [libfabric::iovec::IocMut<T>],
    //         dest_addr: u64,
    //         desc: &mut [MemoryRegionDesc],
    //         res_desc: &mut [MemoryRegionDesc],
    //         op: FetchAtomicOp,
    //         ctx: &mut Context
    //     ) {
    //         let (start, _end) = self.remote_mem_addr.unwrap();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.fetch_atomicv_from_async(
    //                             ioc,
    //                             desc,
    //                             res_ioc,
    //                             res_desc,
    //                             self.mapped_addr.as_ref().unwrap(),
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //                 MyEndpoint::Connected(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.fetch_atomicv_async(
    //                             ioc,
    //                             desc,
    //                             res_ioc,
    //                             res_desc,
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn fetch_atomicmsg<T: libfabric::AsFiType + 'static>(
    //         &mut self,
    //         msg: &mut Either<MsgFetchAtomic<T>, MsgFetchAtomicConnected<T>>,
    //         res_ioc: &mut [libfabric::iovec::IocMut<T>],
    //         res_desc: &mut [MemoryRegionDesc],
    //         ctx: &mut Context
    //     ) {
    //         let opts = AtomicMsgOptions::new();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => match msg {
    //                     Either::Left(msg) => unsafe {
    //                         async_std::task::block_on(async {
    //                             ep.fetch_atomicmsg_from_async(msg, res_ioc, res_desc, opts,ctx).await
    //                         })
    //                     },
    //                     Either::Right(_) => todo!(),
    //                 },
    //                 MyEndpoint::Connected(ep) => match msg {
    //                     Either::Left(_) => todo!(),
    //                     Either::Right(msg) => unsafe {
    //                         async_std::task::block_on(async {
    //                             ep.fetch_atomicmsg_async(msg, res_ioc, res_desc, opts, ctx).await
    //                         })
    //                     },
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn compare_atomic<T: libfabric::AsFiType>(
    //         &mut self,
    //         buf: &[T],
    //         comp: &[T],
    //         res: &mut [T],
    //         dest_addr: u64,
    //         desc: &mut MemoryRegionDesc,
    //         comp_desc: &mut MemoryRegionDesc,
    //         res_desc: &mut MemoryRegionDesc,
    //         op: CompareAtomicOp,
    //         ctx: &mut Context
    //     ) {
    //         let (start, _end) = self.remote_mem_addr.unwrap();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.compare_atomic_to_async(
    //                             buf,
    //                             desc,
    //                             comp,
    //                             comp_desc,
    //                             res,
    //                             res_desc,
    //                             self.mapped_addr.as_ref().unwrap(),
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //                 MyEndpoint::Connected(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.compare_atomic_async(
    //                             buf,
    //                             desc,
    //                             comp,
    //                             comp_desc,
    //                             res,
    //                             res_desc,
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn compare_atomicv<T: libfabric::AsFiType>(
    //         &mut self,
    //         ioc: &[libfabric::iovec::Ioc<T>],
    //         comp_ioc: &[libfabric::iovec::Ioc<T>],
    //         res_ioc: &mut [libfabric::iovec::IocMut<T>],
    //         dest_addr: u64,
    //         desc: &mut [MemoryRegionDesc],
    //         comp_desc: &mut [MemoryRegionDesc],
    //         res_desc: &mut [MemoryRegionDesc],
    //         op: CompareAtomicOp,
    //         ctx: &mut Context
    //     ) {
    //         let (start, _end) = self.remote_mem_addr.unwrap();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.compare_atomicv_to_async(
    //                             ioc,
    //                             desc,
    //                             comp_ioc,
    //                             comp_desc,
    //                             res_ioc,
    //                             res_desc,
    //                             self.mapped_addr.as_ref().unwrap(),
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //                 MyEndpoint::Connected(ep) => unsafe {
    //                     async_std::task::block_on(async {
    //                         ep.compare_atomicv_async(
    //                             ioc,
    //                             desc,
    //                             comp_ioc,
    //                             comp_desc,
    //                             res_ioc,
    //                             res_desc,
    //                             start + dest_addr,
    //                             self.remote_key.as_ref().unwrap(),
    //                             op,
    //                             ctx
    //                         ).await
    //                     })
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     pub fn compare_atomicmsg<T: libfabric::AsFiType + 'static>(
    //         &mut self,
    //         msg: &mut Either<MsgCompareAtomic<T>, MsgCompareAtomicConnected<T>>,
    //         comp_ioc: &[libfabric::iovec::Ioc<T>],
    //         res_ioc: &mut [libfabric::iovec::IocMut<T>],
    //         comp_desc: &mut [MemoryRegionDesc],
    //         res_desc: &mut [MemoryRegionDesc],
    //         ctx: &mut Context
    //     ) {
    //         let opts = AtomicMsgOptions::new();
    //         loop {
    //             let err = match &self.ep {
    //                 MyEndpoint::Connectionless(ep) => match msg {
    //                     Either::Left(msg) => unsafe {
    //                         async_std::task::block_on(async {
    //                             ep.compare_atomicmsg_to_async(msg, comp_ioc, comp_desc, res_ioc, res_desc, opts, ctx).await
    //                         })
    //                     },
    //                     Either::Right(_) => todo!(),
    //                 },
    //                 MyEndpoint::Connected(ep) => match msg {
    //                     Either::Left(_) => todo!(),
    //                     Either::Right(msg) => unsafe {
    //                         async_std::task::block_on(async {
    //                             ep.compare_atomicmsg_async(msg, comp_ioc, comp_desc, res_ioc, res_desc, opts, ctx).await
    //                         })
    //                     },
    //                 },
    //             };
    //             match err {
    //                 Ok(_) => break,
    //                 Err(err) => {
    //                     if !matches!(err.kind, ErrorKind::TryAgain) {
    //                         panic!("{:?}", err);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    impl<I: CollCap> Ofi<I> {}

    macro_rules! gen_info {
        ($ep_type: ident, $caps: ident, $shared_cq: literal, $ip: expr, $server: ident, $name: ident) => {
            Ofi::new(
                {
                    let info = Info::new(&libfabric::info::libfabric_version())
                        .enter_hints()
                        .enter_ep_attr()
                        .type_($ep_type)
                        .leave_ep_attr()
                        .enter_domain_attr()
                        .threading(libfabric::enums::Threading::Safe)
                        .mr_mode(
                            libfabric::enums::MrMode::new()
                                .prov_key()
                                .allocated()
                                .virt_addr()
                                .local()
                                .endpoint()
                                .raw(),
                        )
                        .leave_domain_attr()
                        .enter_tx_attr()
                        .traffic_class(libfabric::enums::TrafficClass::LowLatency)
                        .leave_tx_attr()
                        .addr_format(libfabric::enums::AddressFormat::Unspec)
                        .caps($caps)
                        .leave_hints();
                    if $server {
                        info.source(libfabric::info::ServiceAddress::Service("9222".to_owned()))
                            .get()
                            .unwrap()
                            .into_iter()
                            .next()
                            .unwrap()
                    } else {
                        info.node($ip)
                            .service("9222")
                            .get()
                            .unwrap()
                            .into_iter()
                            .next()
                            .unwrap()
                    }
                },
                $shared_cq,
                $server,
                $name,
            )
            .unwrap()
        };
    }

    fn handshake<I: Caps + MsgDefaultCap + 'static>(
        server: bool,
        name: &str,
        caps: Option<I>,
    ) -> Ofi<I> {
        let caps = caps.unwrap();
        let ep_type = EndpointType::Msg;
        let hostname = std::process::Command::new("hostname")
            .output()
            .expect("Failed to execute hostname")
            .stdout;
        let hostname = String::from_utf8(hostname[2..].to_vec()).unwrap();
        let ip = "172.17.110.".to_string() + &hostname;

        gen_info!(
            ep_type,
            caps,
            false,
            ip.strip_suffix("\n").unwrap_or(&ip),
            server,
            name
        )
    }

    #[test]
    fn parallel_async_handshake_connected0() {
        handshake(true, "handshake_connected0", Some(InfoCaps::new().msg()));
    }

    #[test]
    fn parallel_async_handshake_connected1() {
        handshake(false, "handshake_connected0", Some(InfoCaps::new().msg()));
    }

    fn handshake_connectionless<I: MsgDefaultCap + Caps + 'static>(
        server: bool,
        name: &str,
        caps: Option<I>,
    ) -> Ofi<I> {
        let caps = caps.unwrap();
        let ep_type = EndpointType::Rdm;
        let hostname = std::process::Command::new("hostname")
            .output()
            .expect("Failed to execute hostname")
            .stdout;
        let hostname = String::from_utf8(hostname[2..].to_vec()).unwrap();
        let ip = "172.17.110.".to_string() + &hostname;

        gen_info!(
            ep_type,
            caps,
            false,
            ip.strip_suffix("\n").unwrap_or(&ip),
            server,
            name
        )
    }

    #[test]
    fn parallel_async_handshake_connectionless0() {
        handshake_connectionless(
            true,
            "handshake_connectionless0",
            Some(InfoCaps::new().msg()),
        );
    }

    #[test]
    fn parallel_async_handshake_connectionless1() {
        handshake_connectionless(
            false,
            "handshake_connectionless0",
            Some(InfoCaps::new().msg()),
        );
    }

    fn sendrecv(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
        };
        // println!("passed handshake");

        let reg_mem: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();

        let reg_mem = Arc::new(reg_mem);

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&ofi.domain)
            .unwrap();

        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                match disabled_mr {
                    libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&ofi.ep, ep_binding_memory_region),
                    libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                }
            }
        };
        let mr = Arc::new(mr);
        if server {
            ofi.send(&reg_mem[..128], None);
            // println!("Injects completed");

            ofi.send(&reg_mem[..512], None);
            // println!("Send asyncs completed");

            let handles: Vec<_> = (0..100)
                .map(|_| {
                    let ep = ofi.ep.clone();
                    let mapped_addr = Arc::new(ofi.mapped_addr.clone());
                    let reg_mem_0 = reg_mem.clone();
                    let mut ctx = ofi.info_entry.allocate_context();
                    let mr = mr.clone();

                    async_std::task::spawn(async move {
                        let desc = mr.descriptor();
                        let iov = [IoVec::from_slice(&reg_mem_0[..512])];
                        match ep.as_ref() {
                            MyEndpoint::Connected(ep) => {
                                ep.sendv_async(&iov, Some(std::slice::from_ref(&desc)), &mut ctx)
                                    .await
                            }
                            MyEndpoint::Connectionless(ep) => {
                                ep.send_to_async(
                                    &reg_mem_0[..512],
                                    Some(desc).as_ref(),
                                    mapped_addr.as_ref().as_ref().unwrap(),
                                    &mut ctx,
                                )
                                .await
                            }
                        }
                        .unwrap();
                    })
                })
                .collect();

            for hdl in handles {
                async_std::task::block_on(hdl);
            }
        } else {
            let expected: Vec<_> = (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect();

            let mem: Vec<_> = vec![0u8; 512];

            let results = ofi.recv(&mem[..128]);

            for res in results {
                assert_eq!(&res[..128], &expected[..128]);
            }

            // println!("Recv asyncs 1 completed");

            let results = ofi.recv(&mem[..512]);

            for res in results {
                assert_eq!(&res[..512], &expected[..512]);
            }

            // println!("Recv asyncs 2 completed");
            let results = ofi.recv(&mem[..512]);

            for res in results {
                assert_eq!(&res[..512], &expected[..512]);
            }

            // assert_eq!(mem0, &expected[..512]);
            // assert_eq!(mem1, &expected[512..1024]);
        }
    }

    #[test]
    fn parallel_async_sendrecv0() {
        sendrecv(true, "sendrecv0", false);
    }

    #[test]
    fn parallel_async_sendrecv1() {
        sendrecv(false, "sendrecv0", false);
    }

    #[test]
    fn parallel_async_conn_sendrecv0() {
        sendrecv(true, "conn_sendrecv0", true);
    }

    #[test]
    fn parallel_async_conn_sendrecv1() {
        sendrecv(false, "conn_sendrecv0", true);
    }

    // fn sendrecvdata(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
    //     };

    //     let mut reg_mem: Vec<_> = (0..1024 * 2)
    //         .into_iter()
    //         .map(|v: usize| (v % 256) as u8)
    //         .collect();
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };

    //     let mut desc = [mr.descriptor(), mr.descriptor()];
    //     let data = Some(128u64);
    //     let mut ctx = ofi.info_entry.allocate_context();
    //     if server {
    //         // Send a single buffer
    //         ofi.send(&reg_mem[..512], desc[0], data, &mut ctx);
    //     } else {
    //         let expected: Vec<_> = (0..1024 * 2)
    //             .into_iter()
    //             .map(|v: usize| (v % 256) as u8)
    //             .collect();
    //         reg_mem.iter_mut().for_each(|v| *v = 0);

    //         // Receive a single buffer
    //         ofi.recv(&mut reg_mem[..512], desc[0], &mut ctx);
    //         assert_eq!(reg_mem[..512], expected[..512]);
    //     }
    // }

    // #[test]
    // fn async_sendrecvdata0() {
    //     sendrecvdata(true, "sendrecvdata0", false);
    // }

    // #[test]
    // fn async_sendrecvdata1() {
    //     sendrecvdata(false, "sendrecvdata0", false);
    // }

    // #[test]
    // fn async_conn_sendrecvdata0() {
    //     sendrecvdata(true, "conn_sendrecvdata0", true);
    // }

    // #[test]
    // fn async_conn_sendrecvdata1() {
    //     sendrecvdata(false, "conn_sendrecvdata0", true);
    // }

    fn enable_ep_mr<E: 'static>(ep: &MyEndpoint<E>, mr: EpBindingMemoryRegion) -> MemoryRegion {
        match ep {
            MyEndpoint::Connected(ep) => mr.enable(ep).unwrap(),
            MyEndpoint::Connectionless(ep) => mr.enable(ep).unwrap(),
        }
    }

    fn tsendrecv(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().tagged()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().tagged()))
        };

        let mut reg_mem: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();

        let data = Some(128u64);

        if server {
            // Send a single buffer
            ofi.tsend(&reg_mem[..512], 10, data);

            // Inject a buffer
            ofi.tsend(&reg_mem[..128], 1, data);

            // Send a single buffer
            ofi.tsend(&reg_mem[..512], 10, data);

            // // // Send single Iov
            // let iov = [IoVec::from_slice(&reg_mem[..512])];
            // ofi.tsendv(&iov, desc[..1], 2, &mut ctx);

            // // Send multi Iov
            // let iov = [
            //     IoVec::from_slice(&reg_mem[..512]),
            //     IoVec::from_slice(&reg_mem[512..1024]),
            // ];
            // ofi.tsendv(&iov, desc, 3, &mut ctx);
        } else {
            let expected: Vec<_> = (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect();
            reg_mem.iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            let results = ofi.trecv(&mut reg_mem[..512], 10);

            for res in results {
                assert_eq!(res[..], expected[..512]);
            }

            // // Receive inject
            let results = ofi.trecv(&mut reg_mem[..128], 1);

            for res in results {
                assert_eq!(res[..], expected[..128]);
            }

            let results = ofi.trecv(&mut reg_mem[..512], 10);

            for res in results {
                assert_eq!(res[..], expected[..512]);
            }

            // // // Receive into a single Iov
            // let mut iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
            // ofi.trecvv(&mut iov, desc[..1], 2, &mut ctx);
            // assert_eq!(reg_mem[..512], expected[..512]);

            // reg_mem.iter_mut().for_each(|v| *v = 0);

            // // // Receive into multiple Iovs
            // let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
            // let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            // ofi.trecvv(&iov, desc, 3, &mut ctx);

            // assert_eq!(mem0, &expected[..512]);
            // assert_eq!(mem1, &expected[512..1024]);
        }
    }

    #[test]
    fn parallel_async_tsendrecv0() {
        tsendrecv(true, "tsendrecv0", false);
    }

    #[test]
    fn parallel_async_tsendrecv1() {
        tsendrecv(false, "tsendrecv0", false);
    }

    // #[test]
    // fn parallel_async_conn_tsendrecv0() {
    //     tsendrecv(true, "conn_tsendrecv0", true);
    // }

    // #[test]
    // fn parallel_async_conn_tsendrecv1() {
    //     tsendrecv(false, "conn_tsendrecv0", true);
    // }

    // fn sendrecvmsg(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
    //     };

    //     let mut reg_mem: Vec<_> = (0..1024 * 2)
    //         .into_iter()
    //         .map(|v: usize| (v % 256) as u8)
    //         .collect();
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };

    //     let desc = mr.descriptor();
    //     let mut descs = [desc.clone(), desc];
    //     let mapped_addr = ofi.mapped_addr.clone();
    //     let mut ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         // Single iov message
    //         let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
    //         let iov0 = IoVec::from_slice(mem0);
    //         let iov1 = IoVec::from_slice(mem1);
    //         let mut msg = if connected {
    //             Either::Right(MsgConnected::from_iov(&iov0, &mut descs[0], 128))
    //         } else {
    //             Either::Left(Msg::from_iov(
    //                 &iov0,
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //                 128,
    //             ))
    //         };
    //         ofi.sendmsg(&mut msg, &mut ctx);

    //         // let entry =
    //         // match entry {
    //         //     Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
    //         //     _ => panic!("Unexpected CQ entry format"),
    //         // }

    //         // Multi iov message with stride
    //         let iovs = [iov0, iov1];
    //         let mut msg = if connected {
    //             Either::Right(MsgConnected::from_iov_slice(&iovs, &mut descs, 128))
    //         } else {
    //             Either::Left(Msg::from_iov_slice(
    //                 &iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 128,
    //             ))
    //         };

    //         ofi.sendmsg(&mut msg, &mut ctx);

    //         // Single iov message
    //         let mut msg = if connected {
    //             Either::Right(MsgConnected::from_iov(&iovs[0], &mut descs[0], 0))
    //         } else {
    //             Either::Left(Msg::from_iov(
    //                 &iovs[0],
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //                 0,
    //             ))
    //         };

    //         ofi.sendmsg(&mut msg, &mut ctx);

    //         let mut msg = if connected {
    //             Either::Right(MsgConnected::from_iov_slice(&iovs, &mut descs, 0))
    //         } else {
    //             Either::Left(Msg::from_iov_slice(
    //                 &iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 0,
    //             ))
    //         };
    //         ofi.sendmsg(&mut msg, &mut ctx);
    //     } else {
    //         reg_mem.iter_mut().for_each(|v| *v = 0);
    //         let (mem0, mem1) = reg_mem.split_at_mut(512);
    //         let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

    //         // Receive a single message in a single buffer
    //         let mut iov = IoVecMut::from_slice(mem0);
    //         let mut msg = if connected {
    //             Either::Right(MsgConnectedMut::from_iov(&mut iov, &mut descs[0]))
    //         } else {
    //             Either::Left(MsgMut::from_iov(
    //                 &mut iov,
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //             ))
    //         };

    //         ofi.recvmsg(&mut msg, &mut ctx);

    //         assert_eq!(mem0.len(), expected[..512].len());
    //         assert_eq!(mem0, &expected[..512]);

    //         // Receive a multi iov message in a single buffer
    //         let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
    //         let mut msg = if connected {
    //             Either::Right(MsgConnectedMut::from_iov(&mut iov, &mut descs[0]))
    //         } else {
    //             Either::Left(MsgMut::from_iov(
    //                 &mut iov,
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //             ))
    //         };

    //         ofi.recvmsg(&mut msg, &mut ctx);
    //         assert_eq!(mem1[..1024], expected);

    //         // Receive a single iov message into two buffers
    //         reg_mem.iter_mut().for_each(|v| *v = 0);
    //         let (mem0, mem1) = reg_mem.split_at_mut(512);
    //         let iov = IoVecMut::from_slice(&mut mem0[..256]);
    //         let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
    //         let mut iovs = [iov, iov1];
    //         let mut msg = if connected {
    //             Either::Right(MsgConnectedMut::from_iov_slice(&mut iovs, &mut descs))
    //         } else {
    //             Either::Left(MsgMut::from_iov_slice(
    //                 &mut iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //             ))
    //         };

    //         ofi.recvmsg(&mut msg, &mut ctx);
    //         assert_eq!(mem0[..256], expected[..256]);
    //         assert_eq!(mem1[..256], expected[256..512]);

    //         // Receive a two iov message into two buffers
    //         reg_mem.iter_mut().for_each(|v| *v = 0);
    //         let (mem0, mem1) = reg_mem.split_at_mut(512);
    //         let iov = IoVecMut::from_slice(&mut mem0[..512]);
    //         let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
    //         let mut iovs = [iov, iov1];
    //         let mut msg = if connected {
    //             Either::Right(MsgConnectedMut::from_iov_slice(&mut iovs, &mut descs))
    //         } else {
    //             Either::Left(MsgMut::from_iov_slice(
    //                 &mut iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //             ))
    //         };

    //         ofi.recvmsg(&mut msg, &mut ctx);
    //         assert_eq!(mem0[..512], expected[..512]);
    //         assert_eq!(mem1[..512], expected[512..1024]);
    //     }
    // }

    // #[test]
    // fn async_sendrecvmsg0() {
    //     sendrecvmsg(true, "sendrecvmsg0", false);
    // }

    // #[test]
    // fn async_sendrecvmsg1() {
    //     sendrecvmsg(false, "sendrecvmsg0", false);
    // }

    // #[test]
    // fn async_conn_sendrecvmsg0() {
    //     sendrecvmsg(true, "conn_sendrecvmsg0", true);
    // }

    // #[test]
    // fn async_conn_sendrecvmsg1() {
    //     sendrecvmsg(false, "conn_sendrecvmsg0", true);
    // }

    // fn tsendrecvmsg(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg().tagged()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg().tagged()))
    //     };

    //     let mut reg_mem: Vec<_> = (0..1024 * 2)
    //         .into_iter()
    //         .map(|v: usize| (v % 256) as u8)
    //         .collect();
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .build(&ofi.domain)
    //         .unwrap();
    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };

    //     let desc = mr.descriptor();
    //     let mut descs = [desc.clone(), desc];
    //     let mapped_addr = ofi.mapped_addr.clone();
    //     let mut ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         // Single iov message
    //         let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
    //         let iov0 = IoVec::from_slice(mem0);
    //         let iov1 = IoVec::from_slice(mem1);
    //         let mut msg = if connected {
    //             Either::Right(MsgTaggedConnected::from_iov(
    //                 &iov0,
    //                 &mut descs[0],
    //                 128,
    //                 0,
    //                 0,
    //             ))
    //         } else {
    //             Either::Left(MsgTagged::from_iov(
    //                 &iov0,
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //                 128,
    //                 0,
    //                 0,
    //             ))
    //         };
    //         ofi.tsendmsg(&mut msg, &mut ctx);

    //         // Multi iov message with stride
    //         let iovs = [iov0, iov1];
    //         let mut msg = if connected {
    //             Either::Right(MsgTaggedConnected::from_iov_slice(
    //                 &iovs, &mut descs, 0, 1, 0,
    //             ))
    //         } else {
    //             Either::Left(MsgTagged::from_iov_slice(
    //                 &iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 0,
    //                 1,
    //                 0,
    //             ))
    //         };

    //         ofi.tsendmsg(&mut msg, &mut ctx);

    //         // Single iov message
    //         let mut msg = if connected {
    //             Either::Right(MsgTaggedConnected::from_iov(
    //                 &iovs[0],
    //                 &mut descs[0],
    //                 0,
    //                 2,
    //                 0,
    //             ))
    //         } else {
    //             Either::Left(MsgTagged::from_iov(
    //                 &iovs[0],
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //                 0,
    //                 2,
    //                 0,
    //             ))
    //         };

    //         ofi.tsendmsg(&mut msg, &mut ctx);

    //         let mut msg = if connected {
    //             Either::Right(MsgTaggedConnected::from_iov_slice(
    //                 &iovs, &mut descs, 0, 3, 0,
    //             ))
    //         } else {
    //             Either::Left(MsgTagged::from_iov_slice(
    //                 &iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 0,
    //                 3,
    //                 0,
    //             ))
    //         };
    //         ofi.tsendmsg(&mut msg, &mut ctx);
    //     } else {
    //         reg_mem.iter_mut().for_each(|v| *v = 0);
    //         let (mem0, mem1) = reg_mem.split_at_mut(512);
    //         let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

    //         // Receive a single message in a single buffer
    //         let mut iov = IoVecMut::from_slice(mem0);
    //         let mut msg = if connected {
    //             Either::Right(MsgTaggedConnectedMut::from_iov(
    //                 &mut iov,
    //                 &mut descs[0],
    //                 0,
    //                 0,
    //             ))
    //         } else {
    //             Either::Left(MsgTaggedMut::from_iov(
    //                 &mut iov,
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //                 0,
    //                 0,
    //             ))
    //         };

    //         ofi.trecvmsg(&mut msg, &mut ctx);
    //         assert_eq!(mem0.len(), expected[..512].len());
    //         assert_eq!(mem0, &expected[..512]);

    //         // Receive a multi iov message in a single buffer
    //         let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
    //         let mut msg = if connected {
    //             Either::Right(MsgTaggedConnectedMut::from_iov(
    //                 &mut iov,
    //                 &mut descs[0],
    //                 1,
    //                 0,
    //             ))
    //         } else {
    //             Either::Left(MsgTaggedMut::from_iov(
    //                 &mut iov,
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //                 1,
    //                 0,
    //             ))
    //         };

    //         ofi.trecvmsg(&mut msg, &mut ctx);
    //         assert_eq!(mem1[..1024], expected);

    //         // Receive a single iov message into two buffers
    //         reg_mem.iter_mut().for_each(|v| *v = 0);
    //         let (mem0, mem1) = reg_mem.split_at_mut(512);
    //         let iov = IoVecMut::from_slice(&mut mem0[..256]);
    //         let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
    //         let mut iovs = [iov, iov1];
    //         let mut msg = if connected {
    //             Either::Right(MsgTaggedConnectedMut::from_iov_slice(
    //                 &mut iovs, &mut descs, 2, 0,
    //             ))
    //         } else {
    //             Either::Left(MsgTaggedMut::from_iov_slice(
    //                 &mut iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 2,
    //                 0,
    //             ))
    //         };

    //         ofi.trecvmsg(&mut msg, &mut ctx);
    //         assert_eq!(mem0[..256], expected[..256]);
    //         assert_eq!(mem1[..256], expected[256..512]);

    //         // Receive a two iov message into two buffers
    //         reg_mem.iter_mut().for_each(|v| *v = 0);
    //         let (mem0, mem1) = reg_mem.split_at_mut(512);
    //         let iov = IoVecMut::from_slice(&mut mem0[..512]);
    //         let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
    //         let mut iovs = [iov, iov1];
    //         let mut msg = if connected {
    //             Either::Right(MsgTaggedConnectedMut::from_iov_slice(
    //                 &mut iovs, &mut descs, 3, 0,
    //             ))
    //         } else {
    //             Either::Left(MsgTaggedMut::from_iov_slice(
    //                 &mut iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 3,
    //                 0,
    //             ))
    //         };

    //         ofi.trecvmsg(&mut msg, &mut ctx);
    //         assert_eq!(mem0[..512], expected[..512]);
    //         assert_eq!(mem1[..512], expected[512..1024]);
    //     }
    // }

    // #[test]
    // fn async_tsendrecvmsg0() {
    //     tsendrecvmsg(true, "tsendrecvmsg0", false);
    // }

    // #[test]
    // fn async_tsendrecvmsg1() {
    //     tsendrecvmsg(false, "tsendrecvmsg0", false);
    // }

    // #[test]
    // fn async_conn_tsendrecvmsg0() {
    //     tsendrecvmsg(true, "conn_tsendrecvmsg0", true);
    // }

    // #[test]
    // fn async_conn_tsendrecvmsg1() {
    //     tsendrecvmsg(false, "conn_tsendrecvmsg0", true);
    // }

    fn writeread(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().rma()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().rma()))
        };

        let mut reg_mem: Vec<_> = if server {
            (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect()
        } else {
            vec![0; 1024 * 2]
        };
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .build(&ofi.domain)
            .unwrap();
        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                match disabled_mr {
                    libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&ofi.ep, ep_binding_memory_region),
                    libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
                }
            }
        };

        // let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(server, key, reg_mem.as_ptr() as usize, 1024 * 2);
        let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

        if server {
            // Write a single buffer
            ofi.write(&reg_mem[..512], 0, None);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], None);

            // Write inject a single buffer
            ofi.write(&reg_mem[..128], 0, None);
            // Send completion ack
            ofi.send(&reg_mem[512..1024], None);

            // // Write vector of buffers
            // let iovs = [
            //     IoVec::from_slice(&reg_mem[..512]),
            //     IoVec::from_slice(&reg_mem[512..1024]),
            // ];
            // ofi.writev(&iovs, 0, &mut descs, &mut ctx);

            // // Send completion ack
            // ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

            // // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024]);
        } else {
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024]);
            assert_eq!(&reg_mem[..128], &expected[..128]);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024]);
            assert_eq!(&reg_mem[..512], &expected[..512]);

            // // Recv a completion ack
            // ofi.recv(&mut reg_mem[1024..1536]);
            // assert_eq!(&reg_mem[..1024], &expected[..1024]);

            // reg_mem.iter_mut().for_each(|v| *v = 0);

            // // Read buffer from remote memory
            let results = ofi.read(&mut reg_mem[1024..1536], 0);
            for res in results {
                assert_eq!(&res[..512], &expected[512..1024]);
            }

            // // Read vector of buffers from remote memory
            // let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
            // let iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            // ofi.readv(&iovs, 0, &mut descs, &mut ctx);

            // assert_eq!(mem0, &expected[..256]);
            // assert_eq!(mem1, &expected[..256]);

            // // Send completion ack
            ofi.send(&reg_mem[512..1024], None);
        }
    }

    #[test]
    fn paralle_async_conn_writeread0() {
        writeread(true, "conn_writeread0", true);
    }

    #[test]
    fn paralle_async_conn_writeread1() {
        writeread(false, "conn_writeread0", true);
    }

    #[test]
    fn paralle_async_writeread0() {
        writeread(true, "writeread0", false);
    }

    #[test]
    fn paralle_async_writeread1() {
        writeread(false, "writeread0", false);
    }

    // fn writereadmsg(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg().rma()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg().rma()))
    //     };

    //     let mut reg_mem: Vec<_> = if server {
    //         (0..1024 * 2)
    //             .into_iter()
    //             .map(|v: usize| (v % 256) as u8)
    //             .collect()
    //     } else {
    //         vec![0; 1024 * 2]
    //     };
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .access_write()
    //         .access_read()
    //         .access_remote_write()
    //         .access_remote_read()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };
    //     let desc = mr.descriptor();
    //     let mut descs = [desc.clone(), desc];
    //     let mapped_addr = ofi.mapped_addr.clone();

    //     let key = mr.key().unwrap();
    //     ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    //     let expected: Vec<u8> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

    //     let (start, _end) = ofi.remote_mem_addr.unwrap();
    //     let mut ctx = ofi.info_entry.allocate_context();
    //     if server {
    //         let rma_iov = RmaIoVec::new()
    //             .address(start)
    //             .len(128)
    //             .mapped_key(ofi.remote_key.as_ref().unwrap());

    //         let iov = IoVec::from_slice(&reg_mem[..128]);
    //         let mut msg = if connected {
    //             Either::Right(MsgRmaConnected::from_iov(&iov, &mut descs[0], &rma_iov, 0))
    //         } else {
    //             Either::Left(MsgRma::from_iov(
    //                 &iov,
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //                 &rma_iov,
    //                 0,
    //             ))
    //         };

    //         // Write inject a single buffer
    //         ofi.writemsg(&mut msg, &mut ctx);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         let iov = IoVec::from_slice(&reg_mem[..512]);
    //         let rma_iov = RmaIoVec::new()
    //             .address(start)
    //             .len(512)
    //             .mapped_key(ofi.remote_key.as_ref().unwrap());

    //         let mut msg = if connected {
    //             Either::Right(MsgRmaConnected::from_iov(
    //                 &iov,
    //                 &mut descs[0],
    //                 &rma_iov,
    //                 128,
    //             ))
    //         } else {
    //             Either::Left(MsgRma::from_iov(
    //                 &iov,
    //                 &mut descs[0],
    //                 mapped_addr.as_ref().unwrap(),
    //                 &rma_iov,
    //                 128,
    //             ))
    //         };

    //         // Write a single buffer
    //         ofi.writemsg(&mut msg, &mut ctx);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         let iov0 = IoVec::from_slice(&reg_mem[..512]);
    //         let iov1 = IoVec::from_slice(&reg_mem[512..1024]);
    //         let iovs = [iov0, iov1];
    //         let rma_iov0 = RmaIoVec::new()
    //             .address(start)
    //             .len(512)
    //             .mapped_key(ofi.remote_key.as_ref().unwrap());

    //         let rma_iov1 = RmaIoVec::new()
    //             .address(start + 512)
    //             .len(512)
    //             .mapped_key(ofi.remote_key.as_ref().unwrap());
    //         let rma_iovs = [rma_iov0, rma_iov1];

    //         let mut msg = if connected {
    //             Either::Right(MsgRmaConnected::from_iov_slice(
    //                 &iovs, &mut descs, &rma_iovs, 0,
    //             ))
    //         } else {
    //             Either::Left(MsgRma::from_iov_slice(
    //                 &iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 &rma_iovs,
    //                 0,
    //             ))
    //         };

    //         ofi.writemsg(&mut msg, &mut ctx);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);
    //     } else {
    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);
    //         assert_eq!(&reg_mem[..128], &expected[..128]);

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);
    //         assert_eq!(&reg_mem[..512], &expected[..512]);

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[1024..1536], &mut descs[0], &mut ctx);
    //         assert_eq!(&reg_mem[..1024], &expected[..1024]);

    //         reg_mem.iter_mut().for_each(|v| *v = 0);

    //         {
    //             let mut iov = IoVecMut::from_slice(&mut reg_mem[1024..1536]);
    //             let rma_iov = RmaIoVec::new()
    //                 .address(start)
    //                 .len(512)
    //                 .mapped_key(ofi.remote_key.as_ref().unwrap());
    //             // Read buffer from remote memory
    //             let mut msg = if connected {
    //                 Either::Right(MsgRmaConnectedMut::from_iov(
    //                     &mut iov,
    //                     &mut descs[0],
    //                     &rma_iov,
    //                 ))
    //             } else {
    //                 Either::Left(MsgRmaMut::from_iov(
    //                     &mut iov,
    //                     &mut descs[0],
    //                     mapped_addr.as_ref().unwrap(),
    //                     &rma_iov,
    //                 ))
    //             };
    //             ofi.readmsg(&mut msg, &mut ctx);
    //             assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);
    //         }

    //         // // Read vector of buffers from remote memory
    //         let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
    //         let mut iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
    //         let rma_iov0 = RmaIoVec::new()
    //             .address(start)
    //             .len(256)
    //             .mapped_key(ofi.remote_key.as_ref().unwrap());
    //         let rma_iov1 = RmaIoVec::new()
    //             .address(start + 256)
    //             .len(256)
    //             .mapped_key(ofi.remote_key.as_ref().unwrap());
    //         let rma_iovs = [rma_iov0, rma_iov1];

    //         let mut msg = if connected {
    //             Either::Right(MsgRmaConnectedMut::from_iov_slice(
    //                 &mut iovs, &mut descs, &rma_iovs,
    //             ))
    //         } else {
    //             Either::Left(MsgRmaMut::from_iov_slice(
    //                 &mut iovs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 &rma_iovs,
    //             ))
    //         };
    //         ofi.readmsg(&mut msg, &mut ctx);

    //         assert_eq!(mem0, &expected[..256]);
    //         assert_eq!(mem1, &expected[..256]);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);
    //     }
    // }

    // #[test]
    // fn async_writereadmsg0() {
    //     writereadmsg(true, "writereadmsg0", false);
    // }

    // #[test]
    // fn async_writereadmsg1() {
    //     writereadmsg(false, "writereadmsg0", false);
    // }

    // #[test]
    // fn async_conn_writereadmsg0() {
    //     writereadmsg(true, "conn_writereadmsg0", true);
    // }

    // #[test]
    // fn async_conn_writereadmsg1() {
    //     writereadmsg(false, "conn_writereadmsg0", true);
    // }

    // fn atomic(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg().atomic()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
    //     };

    //     let mut reg_mem: Vec<_> = if server {
    //         vec![2; 1024 * 2]
    //     } else {
    //         vec![1; 1024 * 2]
    //     };
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .access_write()
    //         .access_read()
    //         .access_remote_write()
    //         .access_remote_read()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };
    //     let desc = mr.descriptor();
    //     let mut descs = [desc.clone(), desc];
    //     // let mapped_addr = ofi.mapped_addr.clone();
    //     let key = mr.key().unwrap();
    //     ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    //     let mut ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Min, &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Max, &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Sum, &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Prod, &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Bor, &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Band, &mut ctx);

    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Lor, &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Bxor, &mut ctx);

    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Land, &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Lxor, &mut ctx);

    //         ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::AtomicWrite, &mut ctx);
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         let iocs = [
    //             Ioc::from_slice(&reg_mem[..256]),
    //             Ioc::from_slice(&reg_mem[256..512]),
    //         ];

    //         ofi.atomicv(&iocs, 0, &mut descs, AtomicOp::Prod, &mut ctx);
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);
    //         // match err {
    //         //     Err(e) => {
    //         //         if matches!(e.kind, libfabric::error::ErrorKind::ErrorAvailable) {
    //         //             let realerr = ofi.cq_type.tx_cq().readerr(0).unwrap();
    //         //             panic!("{:?}", realerr.error());
    //         //         }
    //         //     }
    //         //     Ok(_) => {}
    //         // }

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);
    //     } else {
    //         let mut expected = vec![2u8; 1024 * 2];

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);

    //         assert_eq!(&reg_mem[..512], &expected[..512]);
    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         expected = vec![3; 1024 * 2];
    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);

    //         assert_eq!(&reg_mem[..512], &expected[..512]);
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         // expected = vec![2;1024*2];
    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);
    //         // assert_eq!(&reg_mem[..512], &expected[..512]);

    //         expected = vec![4; 1024 * 2];
    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);
    //         assert_eq!(&reg_mem[..512], &expected[..512]);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);
    //     }
    // }

    // // [TODO Not sure why, but connected endpoints fail with atomic ops
    // // #[test]
    // // fn async_conn_atomic0() {
    // //     atomic(true, "conn_atomic0", true);
    // // }

    // // #[test]
    // // fn async_conn_atomic1() {
    // //     atomic(false, "conn_atomic0", true);
    // // }

    // #[test]
    // fn async_atomic0() {
    //     atomic(true, "atomic0", false);
    // }

    // #[test]
    // fn async_atomic1() {
    //     atomic(false, "atomic0", false);
    // }

    // fn fetch_atomic(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg().atomic()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
    //     };

    //     let mut reg_mem: Vec<_> = if server {
    //         vec![2; 1024 * 2]
    //     } else {
    //         vec![1; 1024 * 2]
    //     };
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .access_write()
    //         .access_read()
    //         .access_remote_write()
    //         .access_remote_read()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };

    //     let mut desc0 = mr.descriptor();
    //     let mut desc1 = mr.descriptor();
    //     // let mapped_addr = ofi.mapped_addr.clone();
    //     let key = mr.key().unwrap();
    //     ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    //     let mut ctx = ofi.info_entry.allocate_context();
    //     if server {
    //         let mut expected: Vec<_> = vec![1; 256];
    //         let (op_mem, ack_mem) = reg_mem.split_at_mut(512);
    //         let (mem0, mem1) = op_mem.split_at_mut(256);
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Min, &mut ctx);

    //         assert_eq!(mem1, &expected[..256]);

    //         expected = vec![1; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Max, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         expected = vec![2; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Sum, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         expected = vec![4; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Prod, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         expected = vec![8; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Bor, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         expected = vec![10; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Band, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         // Send a done ack
    //         ofi.send(&ack_mem[..512], &mut desc0, None, &mut ctx);

    //         // Send a done ack

    //         ofi.recv(&mut ack_mem[..512], &mut desc0, &mut ctx);

    //         expected = vec![2; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Lor, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         expected = vec![1; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Bxor, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         // Send a done ack
    //         ofi.send(&ack_mem[..512], &mut desc0, None, &mut ctx);

    //         // Send a done ack

    //         ofi.recv(&mut ack_mem[..512], &mut desc0, &mut ctx);

    //         expected = vec![3; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Land, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         expected = vec![1; 256];
    //         ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Lxor, &mut ctx);

    //         assert_eq!(mem1, &expected);

    //         expected = vec![0; 256];
    //         ofi.fetch_atomic(
    //             &mem0,
    //             mem1,
    //             0,
    //             &mut desc0,
    //             &mut desc1,
    //             FetchAtomicOp::AtomicWrite,
    //             &mut ctx
    //         );

    //         assert_eq!(mem1, &expected);

    //         // Send a done ack
    //         ofi.send(&ack_mem[..512], &mut desc0, None, &mut ctx);

    //         ofi.recv(&mut ack_mem[..512], &mut desc0, &mut ctx);

    //         expected = vec![2; 256];
    //         ofi.fetch_atomic(
    //             &mem0,
    //             mem1,
    //             0,
    //             &mut desc0,
    //             &mut desc1,
    //             FetchAtomicOp::AtomicRead,
    //             &mut ctx
    //         );

    //         assert_eq!(mem1, &expected);

    //         expected = vec![2; 256];
    //         let (read_mem, write_mem) = op_mem.split_at_mut(256);
    //         let iocs = [
    //             Ioc::from_slice(&read_mem[..128]),
    //             Ioc::from_slice(&read_mem[128..256]),
    //         ];
    //         let write_mems = write_mem.split_at_mut(128);
    //         let mut res_iocs = [
    //             IocMut::from_slice(write_mems.0),
    //             IocMut::from_slice(write_mems.1),
    //         ];

    //         let desc0 = mr.descriptor();
    //         let desc1 = mr.descriptor();
    //         let desc2 = mr.descriptor();
    //         let desc3 = mr.descriptor();
    //         let mut descs = [desc0, desc1];
    //         let mut res_descs = [desc2, desc3];
    //         ofi.fetch_atomicv(
    //             &iocs,
    //             &mut res_iocs,
    //             0,
    //             &mut descs,
    //             &mut res_descs,
    //             FetchAtomicOp::Prod,
    //             &mut ctx
    //         );

    //         assert_eq!(write_mem, &expected);

    //         // Send a done ack
    //         ofi.send(&ack_mem[..512], &mut descs[0], None, &mut ctx);

    //         // Recv a completion ack
    //         ofi.recv(&mut ack_mem[..512], &mut descs[0], &mut ctx);

    //     } else {
    //         let mut expected = vec![2u8; 256];

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut desc0, &mut ctx);

    //         assert_eq!(&reg_mem[..256], &expected);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut desc0, None, &mut ctx);

    //         expected = vec![3; 256];
    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut desc0, &mut ctx);

    //         assert_eq!(&reg_mem[..256], &expected);
    //         ofi.send(&reg_mem[512..1024], &mut desc0, None, &mut ctx);

    //         expected = vec![2; 256];
    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut desc0, &mut ctx);

    //         assert_eq!(&reg_mem[..256], &expected);
    //         ofi.send(&reg_mem[512..1024], &mut desc0, None, &mut ctx);

    //         expected = vec![4; 256];
    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut desc0, &mut ctx);

    //         assert_eq!(&reg_mem[..256], &expected);
    //         ofi.send(&reg_mem[512..1024], &mut desc0, None, &mut ctx);
    //     }
    // }

    // #[test]
    // fn async_fetch_atomic0() {
    //     fetch_atomic(true, "fetch_atomic0", false);
    // }

    // #[test]
    // fn async_fetch_atomic1() {
    //     fetch_atomic(false, "fetch_atomic0", false);
    // }

    // // [TODO Not sure why, but connected endpoints fail with atomic ops
    // // #[test]
    // // fn async_conn_fetch_atomic0() {
    // //     fetch_atomic(true, "conn_fetch_atomic0", true);
    // // }

    // // #[test]
    // // fn async_conn_fetch_atomic1() {
    // //     fetch_atomic(false, "conn_fetch_atomic0", true);
    // // }

    // fn compare_atomic(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg().atomic()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
    //     };

    //     let mut reg_mem: Vec<_> = if server {
    //         vec![2; 1024 * 2]
    //     } else {
    //         vec![1; 1024 * 2]
    //     };
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .access_write()
    //         .access_read()
    //         .access_remote_write()
    //         .access_remote_read()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };
    //     let mut desc = mr.descriptor();
    //     let mut comp_desc = mr.descriptor();
    //     let mut res_desc = mr.descriptor();
    //     let key = mr.key().unwrap();
    //     ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    //     let mut ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         let mut expected: Vec<_> = vec![1; 256];
    //         let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
    //         let (buf, mem1) = op_mem.split_at_mut(256);
    //         let (comp, res) = mem1.split_at_mut(256);
    //         comp.iter_mut().for_each(|v| *v = 1);

    //         ofi.compare_atomic(
    //             &buf,
    //             comp,
    //             res,
    //             0,
    //             desc,
    //             &mut comp_desc,
    //             &mut res_desc,
    //             CompareAtomicOp::Cswap,
    //             &mut ctx
    //         );

    //         assert_eq!(res, &expected[..256]);

    //         expected = vec![2; 256];
    //         ofi.compare_atomic(
    //             &buf,
    //             comp,
    //             res,
    //             0,
    //             desc,
    //             &mut comp_desc,
    //             &mut res_desc,
    //             CompareAtomicOp::CswapNe,
    //             &mut ctx
    //         );

    //         assert_eq!(res, &expected);

    //         buf.iter_mut().for_each(|v| *v = 3);
    //         expected = vec![2; 256];
    //         ofi.compare_atomic(
    //             &buf,
    //             comp,
    //             res,
    //             0,
    //             desc,
    //             &mut comp_desc,
    //             &mut res_desc,
    //             CompareAtomicOp::CswapLe,
    //             &mut ctx
    //         );

    //         assert_eq!(res, &expected);

    //         buf.iter_mut().for_each(|v| *v = 2);
    //         expected = vec![3; 256];
    //         ofi.compare_atomic(
    //             &buf,
    //             comp,
    //             res,
    //             0,
    //             desc,
    //             &mut comp_desc,
    //             &mut res_desc,
    //             CompareAtomicOp::CswapLt,
    //             &mut ctx
    //         );

    //         assert_eq!(res, &expected);

    //         buf.iter_mut().for_each(|v| *v = 3);
    //         expected = vec![2; 256];
    //         ofi.compare_atomic(
    //             &buf,
    //             comp,
    //             res,
    //             0,
    //             desc,
    //             &mut comp_desc,
    //             &mut res_desc,
    //             CompareAtomicOp::CswapGe,
    //             &mut ctx
    //         );

    //         assert_eq!(res, &expected);

    //         expected = vec![2; 256];
    //         ofi.compare_atomic(
    //             &buf,
    //             comp,
    //             res,
    //             0,
    //             desc,
    //             &mut comp_desc,
    //             &mut res_desc,
    //             CompareAtomicOp::CswapGt,
    //             &mut ctx
    //         );

    //         assert_eq!(res, &expected);

    //         // Send a done ack
    //         ofi.send(&ack_mem[..512], desc, None, &mut ctx);

    //         // Send a done ack

    //         ofi.recv(&mut ack_mem[..512], desc, &mut ctx);

    //         // expected = vec![2; 256];
    //         let (buf0, buf1) = buf.split_at_mut(128);
    //         let (comp0, comp1) = comp.split_at_mut(128);
    //         let (res0, res1) = res.split_at_mut(128);

    //         let buf_iocs = [Ioc::from_slice(&buf0), Ioc::from_slice(&buf1)];
    //         let comp_iocs = [Ioc::from_slice(&comp0), Ioc::from_slice(&comp1)];
    //         let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
    //         let mut buf_descs = [mr.descriptor(), mr.descriptor()];
    //         let mut comp_descs = [mr.descriptor(), mr.descriptor()];
    //         let mut res_descs = [mr.descriptor(), mr.descriptor()];

    //         ofi.compare_atomicv(
    //             &buf_iocs,
    //             &comp_iocs,
    //             &mut res_iocs,
    //             0,
    //             &mut buf_descs,
    //             &mut comp_descs,
    //             &mut res_descs,
    //             CompareAtomicOp::CswapLe,
    //             &mut ctx
    //         );

    //         assert_eq!(res, &expected);

    //         // Send a done ack
    //         ofi.send(&ack_mem[..512], desc, None, &mut ctx);

    //         // Recv a completion ack
    //         ofi.recv(&mut ack_mem[..512], desc, &mut ctx);

    //     } else {
    //         let mut expected = vec![2u8; 256];

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], desc, &mut ctx);

    //         assert_eq!(&reg_mem[..256], &expected);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

    //         expected = vec![3; 256];
    //         // // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], desc, &mut ctx);

    //         assert_eq!(&reg_mem[..256], &expected);
    //         ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
    //     }
    // }

    // #[test]
    // fn async_compare_atomic0() {
    //     compare_atomic(true, "compare_atomic0", false);
    // }

    // #[test]
    // fn async_compare_atomic1() {
    //     compare_atomic(false, "compare_atomic0", false);
    // }

    // // [TODO Not sure why, but connected endpoints fail with atomic ops
    // // #[test]
    // // fn async_conn_compare_atomic0() {
    // //     compare_atomic(true, "conn_compare_atomic0", true);
    // // }

    // // #[test]
    // // fn async_conn_compare_atomic1() {
    // //     compare_atomic(false, "conn_compare_atomic0", true);
    // // }

    // fn atomicmsg(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg().atomic()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
    //     };

    //     let mut reg_mem: Vec<_> = if server {
    //         vec![2; 1024 * 2]
    //     } else {
    //         vec![1; 1024 * 2]
    //     };
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .access_write()
    //         .access_read()
    //         .access_remote_write()
    //         .access_remote_read()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };
    //     let desc = mr.descriptor();
    //     let mut descs = [desc.clone(), desc];
    //     let mapped_addr = ofi.mapped_addr.clone();
    //     let key = mr.key().unwrap();
    //     ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    //     let (start, _end) = ofi.remote_mem_addr.unwrap();

    //     let mut ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         let iocs = [
    //             Ioc::from_slice(&reg_mem[..256]),
    //             Ioc::from_slice(&reg_mem[256..512]),
    //         ];
    //         let rma_ioc0 = RmaIoc::new(start, 256, ofi.remote_key.as_ref().unwrap());
    //         let rma_ioc1 = RmaIoc::new(start + 256, 256, ofi.remote_key.as_ref().unwrap());
    //         let rma_iocs = [rma_ioc0, rma_ioc1];

    //         let mut msg = if connected {
    //             Either::Right(MsgAtomicConnected::from_ioc_slice(
    //                 &iocs,
    //                 &mut descs,
    //                 &rma_iocs,
    //                 AtomicOp::Bor,
    //                 128,
    //             ))
    //         } else {
    //             Either::Left(MsgAtomic::from_ioc_slice(
    //                 &iocs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 &rma_iocs,
    //                 AtomicOp::Bor,
    //                 128,
    //             ))
    //         };

    //         ofi.atomicmsg(&mut msg, &mut ctx);

    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);

    //     } else {
    //         let expected = vec![3u8; 1024 * 2];

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut descs[0], &mut ctx);

    //         assert_eq!(&reg_mem[..512], &expected[..512]);
    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut descs[0], None, &mut ctx);
    //     }
    // }

    // // [TODO Not sure why, but connected endpoints fail with atomic ops
    // // #[test]
    // // fn async_conn_atomic0() {
    // //     atomic(true, "conn_atomic0", true);
    // // }

    // // #[test]
    // // fn async_conn_atomic1() {
    // //     atomic(false, "conn_atomic0", true);
    // // }

    // #[test]
    // fn async_atomicmsg0() {
    //     atomicmsg(true, "atomicmsg0", false);
    // }

    // #[test]
    // fn async_atomicmsg1() {
    //     atomicmsg(false, "atomicmsg0", false);
    // }

    // fn fetch_atomicmsg(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg().atomic()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
    //     };

    //     let mut reg_mem: Vec<_> = if server {
    //         vec![2; 1024 * 2]
    //     } else {
    //         vec![1; 1024 * 2]
    //     };
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .access_write()
    //         .access_read()
    //         .access_remote_write()
    //         .access_remote_read()
    //         .build(&ofi.domain)
    //         .unwrap();
    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };
    //     let mapped_addr = ofi.mapped_addr.clone();
    //     let key = mr.key().unwrap();
    //     ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    //     let (start, _end) = ofi.remote_mem_addr.unwrap();
    //     let mut ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         let expected = vec![1u8; 256];
    //         let (op_mem, ack_mem) = reg_mem.split_at_mut(512);

    //         let (read_mem, write_mem) = op_mem.split_at_mut(256);
    //         let iocs = [
    //             Ioc::from_slice(&read_mem[..128]),
    //             Ioc::from_slice(&read_mem[128..256]),
    //         ];
    //         let write_mems = write_mem.split_at_mut(128);
    //         let mut res_iocs = [
    //             IocMut::from_slice(write_mems.0),
    //             IocMut::from_slice(write_mems.1),
    //         ];

    //         let desc0 = mr.descriptor();
    //         let desc1 = mr.descriptor();
    //         let desc2 = mr.descriptor();
    //         let desc3 = mr.descriptor();
    //         let mut descs = [desc0, desc1];
    //         let mut res_descs = [desc2, desc3];
    //         let rma_ioc0 = RmaIoc::new(start, 128, ofi.remote_key.as_ref().unwrap());
    //         let rma_ioc1 = RmaIoc::new(start + 128, 128, ofi.remote_key.as_ref().unwrap());
    //         let rma_iocs = [rma_ioc0, rma_ioc1];

    //         let mut msg = if connected {
    //             Either::Right(MsgFetchAtomicConnected::from_ioc_slice(
    //                 &iocs,
    //                 &mut descs,
    //                 &rma_iocs,
    //                 FetchAtomicOp::Prod,
    //                 0,
    //             ))
    //         } else {
    //             Either::Left(MsgFetchAtomic::from_ioc_slice(
    //                 &iocs,
    //                 &mut descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 &rma_iocs,
    //                 FetchAtomicOp::Prod,
    //                 0,
    //             ))
    //         };

    //         ofi.fetch_atomicmsg(&mut msg, &mut res_iocs, &mut res_descs, &mut ctx);

    //         assert_eq!(write_mem, &expected);

    //         // Send a done ack
    //         ofi.send(&ack_mem[..512], &mut descs[0], None, &mut ctx);

    //         // Recv a completion ack
    //         ofi.recv(&mut ack_mem[..512], &mut descs[0], &mut ctx);

    //     } else {
    //         let mut desc0 = mr.descriptor();
    //         let expected = vec![2u8; 256];

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], &mut desc0, &mut ctx);

    //         assert_eq!(&reg_mem[..256], &expected);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], &mut desc0, None, &mut ctx);
    //     }
    // }

    // #[test]
    // fn async_fetch_atomicmsg0() {
    //     fetch_atomicmsg(true, "fetch_atomicmsg0", false);
    // }

    // #[test]
    // fn async_fetch_atomicmsg1() {
    //     fetch_atomicmsg(false, "fetch_atomicmsg0", false);
    // }

    // // [TODO Not sure why, but connected endpoints fail with atomic ops
    // // #[test]
    // // fn async_conn_fetch_atomic0() {
    // //     fetch_atomic(true, "conn_fetch_atomic0", true);
    // // }

    // // #[test]
    // // fn async_conn_fetch_atomic1() {
    // //     fetch_atomic(false, "conn_fetch_atomic0", true);
    // // }

    // fn compare_atomicmsg(server: bool, name: &str, connected: bool) {
    //     let mut ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg().atomic()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
    //     };

    //     let mut reg_mem: Vec<_> = if server {
    //         vec![2; 1024 * 2]
    //     } else {
    //         vec![1; 1024 * 2]
    //     };
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .access_write()
    //         .access_read()
    //         .access_remote_write()
    //         .access_remote_read()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
    //             bind_mr(&ofi.ep, &mr);
    //             mr.enable().unwrap()
    //         }
    //     };

    //     let mut desc = mr.descriptor();
    //     let mapped_addr = ofi.mapped_addr.clone();
    //     let key = mr.key().unwrap();
    //     ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    //     let (start, _end) = ofi.remote_mem_addr.unwrap();
    //     let mut ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         let expected = vec![1u8; 256];
    //         let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
    //         let (buf, mem1) = op_mem.split_at_mut(256);
    //         let (comp, res) = mem1.split_at_mut(256);
    //         comp.iter_mut().for_each(|v| *v = 1);

    //         // expected = vec![2; 256];
    //         let (buf0, buf1) = buf.split_at_mut(128);
    //         let (comp0, comp1) = comp.split_at_mut(128);
    //         let (res0, res1) = res.split_at_mut(128);

    //         let buf_iocs = [Ioc::from_slice(&buf0), Ioc::from_slice(&buf1)];
    //         let comp_iocs = [Ioc::from_slice(&comp0), Ioc::from_slice(&comp1)];
    //         let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
    //         let mut buf_descs = [mr.descriptor(), mr.descriptor()];
    //         let mut comp_descs = [mr.descriptor(), mr.descriptor()];
    //         let mut res_descs = [mr.descriptor(), mr.descriptor()];
    //         let rma_ioc0 = RmaIoc::new(start, 128, ofi.remote_key.as_ref().unwrap());
    //         let rma_ioc1 = RmaIoc::new(start + 128, 128, ofi.remote_key.as_ref().unwrap());
    //         let rma_iocs = [rma_ioc0, rma_ioc1];

    //         let mut msg = if connected {
    //             Either::Right(MsgCompareAtomicConnected::from_ioc_slice(
    //                 &buf_iocs,
    //                 &mut buf_descs,
    //                 &rma_iocs,
    //                 CompareAtomicOp::CswapGe,
    //                 0,
    //             ))
    //         } else {
    //             Either::Left(MsgCompareAtomic::from_ioc_slice(
    //                 &buf_iocs,
    //                 &mut buf_descs,
    //                 mapped_addr.as_ref().unwrap(),
    //                 &rma_iocs,
    //                 CompareAtomicOp::CswapGe,
    //                 0,
    //             ))
    //         };

    //         ofi.compare_atomicmsg(
    //             &mut msg,
    //             &comp_iocs,
    //             &mut res_iocs,
    //             &mut comp_descs,
    //             &mut res_descs,
    //             &mut ctx
    //         );

    //         assert_eq!(res, &expected);

    //         // Send a done ack
    //         ofi.send(&ack_mem[..512], desc, None, &mut ctx);

    //         // Recv a completion ack
    //         ofi.recv(&mut ack_mem[..512], desc, &mut ctx);
    //     } else {
    //         let expected = vec![2u8; 256];

    //         // Recv a completion ack
    //         ofi.recv(&mut reg_mem[512..1024], desc, &mut ctx);

    //         assert_eq!(&reg_mem[..256], &expected);

    //         // Send completion ack
    //         ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
    //     }
    // }

    // #[test]
    // fn async_compare_atomicmsg0() {
    //     compare_atomicmsg(true, "compare_atomicmsg0", false);
    // }

    // #[test]
    // fn async_compare_atomicmsg1() {
    //     compare_atomicmsg(false, "compare_atomicmsg0", false);
    // }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_compare_atomic0() {
    //     compare_atomic(true, "conn_compare_atomic0", true);
    // }

    // #[test]
    // fn async_conn_compare_atomic1() {
    //     compare_atomic(false, "conn_compare_atomic0", true);
    // }
}
