#[cfg(test)]
#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_ofi {
    use std::cell::RefCell;
    use std::net::Shutdown;
    pub type EqOptions = libfabric::async_eq_caps_type!(EqCaps::WAIT);

    use libfabric::async_::av::AddressVector;
    use libfabric::async_::comm::atomic::AsyncAtomicCASEp;
    use libfabric::async_::comm::atomic::AsyncAtomicCASRemoteMemAddrSliceEp;
    use libfabric::async_::comm::atomic::AsyncAtomicFetchEp;
    use libfabric::async_::comm::atomic::AsyncAtomicFetchRemoteMemAddrSliceEp;
    use libfabric::async_::comm::atomic::AsyncAtomicWriteEp;
    use libfabric::async_::comm::atomic::AsyncAtomicWriteRemoteMemAddrSliceEp;
    use libfabric::async_::comm::atomic::ConnectedAsyncAtomicCASEp;
    use libfabric::async_::comm::atomic::ConnectedAsyncAtomicCASRemoteMemAddrSliceEp;
    use libfabric::async_::comm::atomic::ConnectedAsyncAtomicFetchEp;
    use libfabric::async_::comm::atomic::ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp;
    use libfabric::async_::comm::atomic::ConnectedAsyncAtomicWriteEp;
    use libfabric::async_::comm::atomic::ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp;
    use libfabric::async_::comm::collective::AsyncCollectiveEp;
    use libfabric::async_::comm::message::AsyncRecvEp;
    use libfabric::async_::comm::message::AsyncSendEp;
    use libfabric::async_::comm::message::ConnectedAsyncRecvEp;
    use libfabric::async_::comm::message::ConnectedAsyncSendEp;
    use libfabric::async_::comm::rma::AsyncReadEp;
    use libfabric::async_::comm::rma::AsyncReadRemoteMemAddrSliceEp;
    use libfabric::async_::comm::rma::AsyncWriteEp;
    use libfabric::async_::comm::rma::AsyncWriteRemoteMemAddrSliceEp;
    use libfabric::async_::comm::rma::ConnectedAsyncReadEp;
    use libfabric::async_::comm::rma::ConnectedAsyncReadRemoteMemAddrSliceEp;
    use libfabric::async_::comm::rma::ConnectedAsyncWriteEp;
    use libfabric::async_::comm::rma::ConnectedAsyncWriteRemoteMemAddrSliceEp;
    use libfabric::async_::comm::tagged::AsyncTagRecvEp;
    use libfabric::async_::comm::tagged::AsyncTagSendEp;
    use libfabric::async_::comm::tagged::ConnectedAsyncTagRecvEp;
    use libfabric::async_::comm::tagged::ConnectedAsyncTagSendEp;
    use libfabric::async_::domain::Domain;
    use libfabric::async_::eq::EventQueue;
    use libfabric::av_set::AddressVectorSetBuilder;
    use libfabric::cq::SingleCompletion;
    use libfabric::enums::CollectiveOptions;
    use libfabric::ep::ActiveEndpoint;
    use libfabric::ep::BaseEndpoint;
    use libfabric::info::Info;
    use libfabric::infocapsoptions::InfoCaps;
    use libfabric::iovec::Ioc;
    use libfabric::iovec::IocMut;
    use libfabric::iovec::RemoteMemAddrAtomicVec;
    use libfabric::iovec::RemoteMemAddrVec;
    use libfabric::iovec::RemoteMemAddrVecMut;
    use libfabric::mcast::MultiCastGroup;
    use libfabric::mr::EpBindingMemoryRegion;
    use libfabric::mr::MemoryRegionDesc;
    use libfabric::mr::MemoryRegionKey;
    use libfabric::AsFiType;
    use libfabric::MemAddressInfo;
    use libfabric::MyRc;
    use libfabric::RemoteMemAddrSlice;
    use libfabric::RemoteMemAddrSliceMut;
    use libfabric::RemoteMemAddressInfo;
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
        enums::{
            AVOptions, AtomicMsgOptions, AtomicOp, CompareAtomicOp, CqFormat, EndpointType,
            FetchAtomicOp, ReadMsgOptions, TferOptions, WriteMsgOptions,
        },
        ep::Address,
        error::Error,
        fabric::FabricBuilder,
        info::InfoEntry,
        infocapsoptions::{
            AtomicDefaultCap, Caps, CollCap, MsgDefaultCap, RmaDefaultCap, TagDefaultCap,
        },
        iovec::{IoVec, IoVecMut},
        mr::{MemoryRegion, MemoryRegionBuilder},
        msg::{
            Msg, MsgAtomic, MsgAtomicConnected, MsgCompareAtomic, MsgCompareAtomicConnected,
            MsgConnected, MsgConnectedMut, MsgFetchAtomic, MsgFetchAtomicConnected, MsgMut, MsgRma,
            MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut, MsgTagged, MsgTaggedConnected,
            MsgTaggedConnectedMut, MsgTaggedMut,
        },
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
        Connected(ConnectedEndpoint<I>),
        Connectionless(ConnectionlessEndpoint<I>),
    }

    pub struct Ofi<I> {
        pub info_entry: InfoEntry<I>,
        pub mr: Option<MemoryRegion>,
        pub remote_mem_info: Option<RefCell<RemoteMemAddressInfo>>,
        // pub remote_mem_addr: Option<(u64, u64)>,
        pub domain: Domain,
        pub cq_type: CqType,
        pub ep: MyEndpoint<I>,
        pub reg_mem: Vec<u8>,
        pub mapped_addr: Option<Vec<MyRc<MappedAddress>>>,
        pub av: Option<AddressVector>,
        pub eq: EventQueue<EqOptions>,
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
                EndpointType::Msg => match &self.ep {
                    MyEndpoint::Connected(ep) => {
                        ep.shutdown().unwrap();
                    }
                    MyEndpoint::Connectionless(_) => todo!(),
                },
                EndpointType::Unspec | EndpointType::Dgram | EndpointType::Rdm => {}
            }
        }
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

            let (info_entry, ep, mapped_addr, av, eq) = {
                let eq = EventQueueBuilder::new(&fabric).build().unwrap();
                let info_entry = if matches!(ep_type, EndpointType::Msg) {
                    if server {
                        let pep = EndpointBuilder::new(&info_entry)
                            .build_passive(&fabric)
                            .unwrap();
                        pep.bind(&eq, 0).unwrap();
                        let event = async_std::task::block_on(async {
                            pep.listen_async().unwrap().next().await
                        })
                        .unwrap();
                        match event {
                            libfabric::eq::Event::ConnReq(entry) => entry.info().unwrap(),
                            _ => panic!("Unexpected event"),
                        }
                    } else {
                        info_entry
                    }
                } else {
                    info_entry
                };

                domain = DomainBuilder::new(&fabric, &info_entry).build().unwrap();

                let ep_builder = EndpointBuilder::new(&info_entry);
                cq_type = if shared_cqs {
                    CqType::Shared(shared_cq_builder.build(&domain).unwrap())
                } else {
                    CqType::Separate((
                        tx_cq_builder.build(&domain).unwrap(),
                        rx_cq_builder.build(&domain).unwrap(),
                    ))
                };

                let ep = match &cq_type {
                    CqType::Separate((tx_cq, rx_cq)) => ep_builder
                        .build_with_separate_cqs(&domain, tx_cq, rx_cq)
                        .unwrap(),
                    CqType::Shared(scq) => ep_builder.build_with_shared_cq(&domain, scq).unwrap(),
                };
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
                        // ep.bind_av(&av).unwrap();
                        let eq = EventQueueBuilder::new(&fabric).build()?;
                        ep.bind_eq(&eq)?;
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
                                        libfabric::mr::DisabledMemoryRegion::EpBind(
                                            ep_binding_memory_region,
                                        ) => ep_binding_memory_region.enable(&ep).unwrap(),
                                        libfabric::mr::DisabledMemoryRegion::RmaEvent(
                                            rma_event_memory_region,
                                        ) => rma_event_memory_region.enable().unwrap(),
                                    }
                                }
                            };
                            Some(mr)
                        } else {
                            None
                        };

                        let mapped_addresses = if let Some(dest_addr) = info_entry.dest_addr() {
                            let all_addresses = [ep.getname().unwrap(), dest_addr.clone()];
                            let mut ctx = info_entry.allocate_context();
                            let mapped_addresses: Vec<std::rc::Rc<MappedAddress>> =
                                async_std::task::block_on(async {
                                    av.insert_async(
                                        all_addresses.as_ref().into(),
                                        AVOptions::new(),
                                        &mut ctx,
                                    )
                                    .await
                                })
                                .unwrap()
                                .1
                                .into_iter()
                                .map(|x| std::rc::Rc::new(x))
                                .collect();

                            let epname = ep.getname().unwrap();
                            let epname_bytes = epname.as_bytes();
                            let addrlen = epname_bytes.len();
                            reg_mem[..addrlen].copy_from_slice(epname_bytes);

                            let mut ctx = info_entry.allocate_context();
                            async_std::task::block_on(ep.send_to_async(
                                &reg_mem[..addrlen],
                                None,
                                &mapped_addresses[1],
                                &mut ctx,
                            ))
                            .unwrap();

                            async_std::task::block_on(ep.recv_from_any_async(
                                std::slice::from_mut(&mut reg_mem[0]),
                                None,
                                &mut ctx,
                            ))
                            .unwrap();

                            mapped_addresses
                        } else {
                            let epname = ep.getname().unwrap();
                            let addrlen = epname.as_bytes().len();

                            let mr_desc = if let Some(ref mr) = mr {
                                Some(mr.descriptor())
                            } else {
                                None
                            };
                            let mut ctx = info_entry.allocate_context();

                            async_std::task::block_on(ep.recv_from_any_async(
                                &mut reg_mem[..addrlen],
                                mr_desc.clone(),
                                &mut ctx,
                            ))
                            .unwrap();

                            let remote_address = unsafe { Address::from_bytes(&reg_mem) };
                            let all_addresses = [epname, remote_address];
                            let mut ctx = info_entry.allocate_context();
                            let mapped_addresses: Vec<std::rc::Rc<MappedAddress>> =
                                async_std::task::block_on(async {
                                    av.insert_async(
                                        all_addresses.as_ref().into(),
                                        AVOptions::new(),
                                        &mut ctx,
                                    )
                                    .await
                                })
                                .unwrap()
                                .1
                                .into_iter()
                                .map(|x| std::rc::Rc::new(x))
                                .collect();

                            async_std::task::block_on(ep.send_to_async(
                                &std::slice::from_ref(&reg_mem[0]),
                                mr_desc,
                                &mapped_addresses[1],
                                &mut ctx,
                            ))
                            .unwrap();

                            mapped_addresses
                        };
                        (
                            info_entry,
                            MyEndpoint::Connectionless(ep),
                            Some(mapped_addresses),
                            Some(av),
                            eq,
                        )
                    }
                    Endpoint::ConnectionOriented(ep) => {
                        let ep = ep.enable(&eq).unwrap();

                        let ep = match ep {
                            libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(ep) => {
                                async_std::task::block_on(async {
                                    ep.connect_async(info_entry.dest_addr().unwrap()).await
                                })
                                .unwrap()
                            },
                            libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(ep) => {
                                async_std::task::block_on(async { ep.accept_async().await }).unwrap()
                            },
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
                                        libfabric::mr::DisabledMemoryRegion::EpBind(
                                            ep_binding_memory_region,
                                        ) => ep_binding_memory_region.enable(&ep).unwrap(),
                                        libfabric::mr::DisabledMemoryRegion::RmaEvent(
                                            rma_event_memory_region,
                                        ) => rma_event_memory_region.enable().unwrap(),
                                    }
                                }
                            };
                            Some(mr)
                        } else {
                            None
                        };

                        (info_entry, MyEndpoint::Connected(ep), None, None, eq)
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
                remote_mem_info: None,
                cq_type,
                domain,
                ep,
                reg_mem,
                av,
                eq, // tx_pending_cnt,
                    // tx_complete_cnt,
                    // rx_pending_cnt,
                    // rx_complete_cnt,
            })
        }
    }

    impl<I: TagDefaultCap> Ofi<I> {
        pub fn tsend<T: Copy>(
            &self,
            buf: &[T],
            desc: Option<MemoryRegionDesc>,
            tag: u64,
            data: Option<u64>,
            ctx: &mut Context,
        ) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            if data.is_some() {
                                ep.tinjectdata_to_async(
                                    &buf,
                                    data.unwrap(),
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                    tag,
                                )
                                .await
                            } else {
                                ep.tinject_to_async(
                                    &buf,
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                    tag,
                                )
                                .await
                            }
                        } else {
                            if data.is_some() {
                                ep.tsenddata_to_async(
                                    &buf,
                                    desc,
                                    data.unwrap(),
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                    tag,
                                    ctx,
                                )
                                .await
                            } else {
                                ep.tsend_to_async(
                                    &buf,
                                    desc,
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                    tag,
                                    ctx,
                                )
                                .await
                            }
                            .map(|_| {})
                        }
                    }
                    MyEndpoint::Connected(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            if data.is_some() {
                                ep.tinjectdata_async(&buf, data.unwrap(), tag).await
                            } else {
                                ep.tinject_async(&buf, tag).await
                            }
                        } else {
                            if data.is_some() {
                                ep.tsenddata_async(&buf, desc, data.unwrap(), tag, ctx)
                                    .await
                            } else {
                                ep.tsend_async(&buf, desc, tag, ctx).await
                            }
                            .map(|_| {})
                        }
                    }
                }
            })
            .unwrap()
        }

        pub fn tsendv(
            &self,
            iov: &[IoVec],
            desc: Option<&[MemoryRegionDesc]>,
            tag: u64,
            ctx: &mut Context,
        ) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        ep.tsendv_to_async(
                            iov,
                            desc,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            tag,
                            ctx,
                        )
                        .await
                    }
                    MyEndpoint::Connected(ep) => ep.tsendv_async(iov, desc, tag, ctx).await,
                }
            })
            .unwrap();
        }

        pub fn trecvv(
            &self,
            iov: &[IoVecMut],
            desc: Option<&[MemoryRegionDesc]>,
            tag: u64,
            ctx: &mut Context,
        ) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        ep.trecvv_from_async(
                            iov,
                            desc,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            tag,
                            None,
                            ctx,
                        )
                        .await
                    }
                    MyEndpoint::Connected(ep) => ep.trecvv_async(iov, desc, tag, None, ctx).await,
                }
            })
            .unwrap();
        }

        pub fn trecv<T: Copy>(
            &self,
            buf: &mut [T],
            desc: Option<MemoryRegionDesc>,
            tag: u64,
            ctx: &mut Context,
        ) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        ep.trecv_from_async(
                            buf,
                            desc,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            tag,
                            None,
                            ctx,
                        )
                        .await
                    }
                    MyEndpoint::Connected(ep) => ep.trecv_async(buf, desc, tag, None, ctx).await,
                }
            })
            .unwrap();
        }

        pub fn tsendmsg(&self, msg: &mut Either<MsgTagged, MsgTaggedConnected>) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => {
                            ep.tsendmsg_to_async(msg, TferOptions::new().remote_cq_data())
                                .await
                        }
                        Either::Right(_) => panic!("Wrong message type used"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong message type used"),
                        Either::Right(msg) => {
                            ep.tsendmsg_async(msg, TferOptions::new().remote_cq_data())
                                .await
                        }
                    },
                }
            })
            .unwrap();
        }

        pub fn trecvmsg(&self, msg: &mut Either<MsgTaggedMut, MsgTaggedConnectedMut>) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => ep.trecvmsg_from_async(msg, TferOptions::new()).await,
                        Either::Right(_) => panic!("Wrong message type"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong message type"),
                        Either::Right(msg) => ep.trecvmsg_async(msg, TferOptions::new()).await,
                    },
                }
            })
            .unwrap();
        }
    }

    impl<I: MsgDefaultCap + 'static> Ofi<I> {
        pub fn send<T: Copy>(
            &self,
            buf: &[T],
            desc: Option<MemoryRegionDesc>,
            data: Option<u64>,
            ctx: &mut Context,
        ) {
            async_std::task::block_on(async {
                let err = match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            if data.is_some() {
                                ep.injectdata_to_async(
                                    buf,
                                    data.unwrap(),
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                )
                                .await
                            } else {
                                ep.inject_to_async(&buf, &self.mapped_addr.as_ref().unwrap()[1])
                                    .await
                            }
                        } else {
                            if data.is_some() {
                                ep.senddata_to_async(
                                    &buf,
                                    desc,
                                    data.unwrap(),
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                    ctx,
                                )
                                .await
                            } else {
                                ep.send_to_async(
                                    &buf,
                                    desc,
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                    ctx,
                                )
                                .await
                            }
                            .map(|_| {})
                        }
                    }
                    MyEndpoint::Connected(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            if data.is_some() {
                                ep.injectdata_async(&buf, data.unwrap()).await
                            } else {
                                ep.inject_async(&buf).await
                            }
                        } else {
                            if data.is_some() {
                                ep.senddata_async(&buf, desc, data.unwrap(), ctx).await
                            } else {
                                ep.send_async(&buf, desc, ctx).await
                            }
                            .map(|_| {})
                        }
                    }
                };
            })
            
        }

        // pub fn sendrecv_deadlock<T:Copy>(
        //     &self,
        //     buf: &mut [T],
        //     desc: Option<&MemoryRegionDesc>,
        //     data: Option<u64>,
        //     send_ctx: &mut Context,
        //     recv_ctx: &mut Context,
        // ) {
        //     println!("sendrecv_deadlock called");
        //     let send_fut = async {
        //         match &self.ep {
        //             MyEndpoint::Connectionless(ep) => {
        //                 if buf.len() <= self.info_entry.tx_attr().inject_size() {
        //                     if data.is_some() {
        //                         ep.injectdata_to_async(
        //                             buf,
        //                             data.unwrap(),
        //                             &self.mapped_addr.as_ref().unwrap()[1],
        //                         )
        //                         .await
        //                     } else {
        //                         ep.inject_to_async(&buf, &self.mapped_addr.as_ref().unwrap()[1])
        //                             .await
        //                     }
        //                 } else {
        //                     if data.is_some() {
        //                         ep.senddata_to_async(
        //                             &buf,
        //                             desc,
        //                             data.unwrap(),
        //                             &self.mapped_addr.as_ref().unwrap()[1],
        //                             send_ctx,
        //                         )
        //                         .await
        //                     } else {
        //                         ep.send_to_async(
        //                             &buf,
        //                             desc,
        //                             &self.mapped_addr.as_ref().unwrap()[1],
        //                             send_ctx,
        //                         )
        //                         .await
        //                     }
        //                     .map(|_| {})
        //                 }
        //             }
        //             MyEndpoint::Connected(ep) => {
        //                 if buf.len() <= self.info_entry.tx_attr().inject_size() {
        //                     if data.is_some() {
        //                         ep.injectdata_async(&buf, data.unwrap()).await
        //                     } else {
        //                         ep.inject_async(&buf).await
        //                     }
        //                 } else {
        //                     if data.is_some() {
        //                         ep.senddata_async(&buf, desc, data.unwrap(), send_ctx).await
        //                     } else {
        //                         ep.send_async(&buf, desc, send_ctx).await
        //                     }
        //                     .map(|_| {})
        //                 }
        //             }
        //         }.unwrap();
        //     };
        //     let mut buf = buf.to_vec();

        //     let recv_fu =
        //     async {
        //         match &self.ep {
        //             MyEndpoint::Connectionless(ep) => {
        //                 ep.recv_from_async(&mut buf, desc, &self.mapped_addr.as_ref().unwrap()[1], recv_ctx)
        //                     .await
        //             }
        //             MyEndpoint::Connected(ep) => {ep.recv_async(&mut buf, desc, recv_ctx).await}
        //         }.unwrap();
        //     };

        //     async_std::task::block_on(async {
        //         join!(recv_fu, send_fut);
        //     });
        // }

        pub fn sendv(&self, iov: &[IoVec], desc: Option<&[MemoryRegionDesc]>, ctx: &mut Context) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        ep.sendv_to_async(iov, desc, &self.mapped_addr.as_ref().unwrap()[1], ctx)
                            .await
                    }
                    MyEndpoint::Connected(ep) => ep.sendv_async(iov, desc, ctx).await,
                }
            })
            .unwrap();
        }

        pub fn recvv(
            &self,
            iov: &[IoVecMut],
            desc: Option<&[MemoryRegionDesc]>,
            ctx: &mut Context,
        ) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        ep.recvv_from_async(iov, desc, &self.mapped_addr.as_ref().unwrap()[1], ctx)
                            .await
                    }
                    MyEndpoint::Connected(ep) => ep.recvv_async(iov, desc, ctx).await,
                }
            })
            .unwrap();
        }

        pub fn recv<T: Copy>(
            &self,
            buf: &mut [T],
            desc: Option<MemoryRegionDesc>,
            ctx: &mut Context,
        ) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        ep.recv_from_async(buf, desc, &self.mapped_addr.as_ref().unwrap()[1], ctx)
                            .await
                    }
                    MyEndpoint::Connected(ep) => ep.recv_async(buf, desc, ctx).await,
                }
            })
            .unwrap();
        }

        pub fn sendmsg(&self, msg: &mut Either<Msg, MsgConnected>) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => {
                            ep.sendmsg_to_async(msg, TferOptions::new().remote_cq_data())
                                .await
                        }
                        Either::Right(_) => panic!("Wrong msg type"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong msg type"),
                        Either::Right(msg) => {
                            ep.sendmsg_async(msg, TferOptions::new().remote_cq_data())
                                .await
                        }
                    },
                }
            })
            .unwrap();
        }

        pub fn recvmsg(&self, msg: &mut Either<MsgMut, MsgConnectedMut>) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => ep.recvmsg_from_async(msg, TferOptions::new()).await,
                        Either::Right(_) => panic!("Wrong message type"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong message type"),
                        Either::Right(msg) => ep.recvmsg_async(msg, TferOptions::new()).await,
                    },
                }
            })
            .unwrap();
        }

        pub fn exchange_keys<T: Copy>(&mut self, key: &MemoryRegionKey, mem_slice: &[T]) {
            let mem_info =
                libfabric::MemAddressInfo::from_slice(mem_slice, 0, key, &self.info_entry);
            let mut mem_bytes = mem_info.to_bytes().to_vec();
            // let mut len = unsafe {
            //     std::slice::from_raw_parts(
            //         &len as *const usize as *const u8,
            //         std::mem::size_of::<usize>(),
            //     )
            // }
            // .to_vec();
            // let mut addr = unsafe {
            //     std::slice::from_raw_parts(
            //         &addr as *const usize as *const u8,
            //         std::mem::size_of::<usize>(),
            //     )
            // }
            // .to_vec();

            // let key_bytes = key.to_bytes();
            // let mut reg_mem = Vec::new();
            // reg_mem.append(&mut key_bytes.clone());
            // reg_mem.append(&mut len);
            // reg_mem.append(&mut addr);
            // let total_len = reg_mem.len();
            // reg_mem.append(&mut vec![0; total_len]);

            let mr = MemoryRegionBuilder::new(&mem_bytes, libfabric::enums::HmemIface::System)
                .access_recv()
                .access_send()
                .build(&self.domain)
                .unwrap();

            let mr = match mr {
                libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
                    match disabled_mr {
                        libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                            enable_ep_mr(&self.ep, ep_binding_memory_region)
                        }
                        libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                            rma_event_memory_region.enable().unwrap()
                        }
                    }
                }
            };
            let mut ctx = self.info_entry.allocate_context();

            let desc = Some(mr.descriptor());
            self.send(
                &mem_bytes,
                desc,
                None,
                &mut ctx,
            );
            self.recv(
                &mut mem_bytes,
                desc,
                &mut ctx,
            );

            let mem_info = unsafe { MemAddressInfo::from_bytes(&mem_bytes) };
            let remote_mem_info = mem_info.into_remote_info(&self.domain).unwrap();
            self.remote_mem_info = Some(RefCell::new(remote_mem_info));
        }
    }

    impl<I: MsgDefaultCap + RmaDefaultCap> Ofi<I> {
        pub fn write<T: Copy>(
            &self,
            buf: &[T],
            dest_addr: usize,
            desc: Option<MemoryRegionDesc>,
            data: Option<u64>,
            ctx: &mut Context,
        ) {
            let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
            let dest_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            if data.is_some() {
                                unsafe {
                                    ep.inject_writedata_slice_to_async(
                                        buf,
                                        data.unwrap(),
                                        &self.mapped_addr.as_ref().unwrap()[1],
                                        &dest_slice,
                                    )
                                    .await
                                }
                            } else {
                                unsafe {
                                    ep.inject_write_slice_to_async(
                                        buf,
                                        &self.mapped_addr.as_ref().unwrap()[1],
                                        &dest_slice,
                                    )
                                    .await
                                }
                            }
                        } else {
                            if data.is_some() {
                                unsafe {
                                    ep.writedata_slice_to_async(
                                        buf,
                                        desc,
                                        data.unwrap(),
                                        &self.mapped_addr.as_ref().unwrap()[1],
                                        &dest_slice,
                                        ctx,
                                    )
                                    .await
                                }
                            } else {
                                unsafe {
                                    ep.write_slice_to_async(
                                        buf,
                                        desc,
                                        &self.mapped_addr.as_ref().unwrap()[1],
                                        &dest_slice,
                                        ctx,
                                    )
                                    .await
                                }
                            }
                            .map(|_| {})
                        }
                    }
                    MyEndpoint::Connected(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            if data.is_some() {
                                unsafe {
                                    ep.inject_writedata_slice_async(buf, data.unwrap(), &dest_slice)
                                        .await
                                }
                            } else {
                                unsafe { ep.inject_write_slice_async(buf, &dest_slice).await }
                            }
                        } else {
                            if data.is_some() {
                                unsafe {
                                    ep.writedata_slice_async(
                                        buf,
                                        desc,
                                        data.unwrap(),
                                        &dest_slice,
                                        ctx,
                                    )
                                    .await
                                }
                            } else {
                                unsafe { ep.write_slice_async(buf, desc, &dest_slice, ctx).await }
                            }
                            .map(|_| {})
                        }
                    }
                }
            })
            .unwrap();
        }

        pub fn read<T: Copy>(
            &self,
            buf: &mut [T],
            dest_addr: usize,
            desc: Option<MemoryRegionDesc>,
            ctx: &mut Context,
        ) {
            let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
            let src_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());

            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        {
                            ep.read_slice_from_async(
                                buf,
                                desc,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &src_slice,
                                ctx,
                            )
                            .await
                        }
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        {
                            ep.read_slice_async(buf, desc, &src_slice, ctx).await
                        }
                    },
                }
            })
            .unwrap();
        }

        pub fn writev(
            &self,
            iov: &[IoVec],
            dest_addr: usize,
            desc: Option<&[MemoryRegionDesc]>,
            ctx: &mut Context,
        ) {
            let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
            let dst_slice = remote_mem_info
                .slice_mut::<u8>(dest_addr..dest_addr + iov.iter().fold(0, |acc, x| acc + x.len()));
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        {
                            ep.writev_slice_to_async(
                                iov,
                                desc,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &dst_slice,
                                ctx,
                            )
                            .await
                        }
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        {
                            ep.writev_slice_async(iov, desc, &dst_slice, ctx).await
                        }
                    },
                }
            })
            .unwrap();
        }

        pub fn readv(
            &self,
            iov: &[IoVecMut],
            dest_addr: usize,
            desc: Option<&[MemoryRegionDesc]>,
            ctx: &mut Context,
        ) {
            let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
            let src_slice = remote_mem_info
                .slice::<u8>(dest_addr..dest_addr + iov.iter().fold(0, |acc, x| acc + x.len()));

            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        {
                            ep.readv_slice_from_async(
                                iov,
                                desc,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &src_slice,
                                ctx,
                            )
                            .await
                        }
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        {
                            ep.readv_slice_async(iov, desc, &src_slice, ctx).await
                        }
                    },
                }
            })
            .unwrap();
        }

        // [TODO] Enabling .remote_cq_data causes the buffer not being written correctly
        // on the remote side.
        pub fn writemsg(&self, msg: &mut Either<MsgRma, MsgRmaConnected>) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => unsafe {
                            {
                                ep.writemsg_to_async(msg, WriteMsgOptions::new()).await
                            }
                        },
                        Either::Right(_) => panic!("Wrong message type"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong message type"),
                        Either::Right(msg) => unsafe {
                            {
                                ep.writemsg_async(msg, WriteMsgOptions::new()).await
                            }
                        },
                    },
                }
            })
            .unwrap();
        }

        pub fn readmsg(&self, msg: &mut Either<MsgRmaMut, MsgRmaConnectedMut>) {
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => unsafe {
                            {
                                ep.readmsg_from_async(msg, ReadMsgOptions::new()).await
                            }
                        },
                        Either::Right(_) => todo!(),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong message type"),
                        Either::Right(msg) => unsafe {
                            {
                                ep.readmsg_async(msg, ReadMsgOptions::new()).await
                            }
                        },
                    },
                }
            })
            .unwrap();
        }
    }

    async unsafe fn atomic_op<T, A>(op: libfabric::enums::AtomicOp, ep: &A, buf: &[T], desc: Option<MemoryRegionDesc<'_>>, dest: &MappedAddress, slice: &RemoteMemAddrSliceMut<'_, T>, context: &mut Context) -> Result<SingleCompletion, libfabric::error::Error>
    where
        T: AsFiType,
        A: AsyncAtomicWriteEp, 
    {

        match op {
            AtomicOp::Min => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_min_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            AtomicOp::Max => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_max_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            AtomicOp::Sum => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_sum_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            AtomicOp::Prod => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_prod_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            // AtomicOp::Lor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_lor_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            // AtomicOp::Land => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_land_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            AtomicOp::Bor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_bor_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            AtomicOp::Band => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_band_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            // AtomicOp::Lxor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_lxor_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            AtomicOp::Bxor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_bxor_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            AtomicOp::AtomicWrite => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_write_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            _ => todo!(),
        }
    }

    async unsafe fn atomic_bool_op<A>(op: libfabric::enums::AtomicOp, ep: &A, buf: &[bool], desc: Option<MemoryRegionDesc<'_>>, dest: &MappedAddress, slice: &RemoteMemAddrSliceMut<'_, bool>, context: &mut Context) -> Result<SingleCompletion, libfabric::error::Error>
    where
        A: AsyncAtomicWriteEp, 
    {

        match op {
            AtomicOp::Lor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_lor_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            AtomicOp::Land => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_land_mr_slice_to_async(ep, buf, desc, dest, slice, context).await,
            _ => todo!(),
        }
    }

    async unsafe fn atomicv_op<T, A>(op: libfabric::enums::AtomicOp, ep: &A, ioc: &[libfabric::iovec::Ioc<'_, T>], desc: Option<&[MemoryRegionDesc<'_>]>, dest: &MappedAddress, slice: &RemoteMemAddrSliceMut<'_, T>, context: &mut Context) -> Result<SingleCompletion, libfabric::error::Error>
    where
        T: AsFiType,
        A: AsyncAtomicWriteEp, 
    {

        match op {
            AtomicOp::Min => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_min_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::Max => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_max_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::Sum => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_sum_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::Prod => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_prod_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            // AtomicOp::Lor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_lor_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            // AtomicOp::Land => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_land_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::Bor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_bor_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::Band => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_band_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            // AtomicOp::Lxor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_lxor_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::Bxor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_bxor_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::AtomicWrite => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_write_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            _ => todo!(),
        }
    }

    async unsafe fn atomicv_bool_op<A>(op: libfabric::enums::AtomicOp, ep: &A, ioc: &[libfabric::iovec::Ioc<'_, bool>], desc: Option<&[MemoryRegionDesc<'_>]>, dest: &MappedAddress, slice: &RemoteMemAddrSliceMut<'_, bool>, context: &mut Context) -> Result<SingleCompletion, libfabric::error::Error>
    where
        A: AsyncAtomicWriteEp, 
    {

        match op {
            AtomicOp::Lor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_lor_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::Land => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_land_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            AtomicOp::Lxor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_lxor_mr_slice_to_async(ep, ioc, desc, dest, slice, context).await,
            _ => todo!(),
        }
    }

    async unsafe fn conn_atomicv_op<T, A>(op: libfabric::enums::AtomicOp, ep: &A, ioc: &[libfabric::iovec::Ioc<'_, T>], desc: Option<&[MemoryRegionDesc<'_>]>, slice: &RemoteMemAddrSliceMut<'_, T>, context: &mut Context) -> Result<SingleCompletion, libfabric::error::Error>
    where
        T: AsFiType,
        A: ConnectedAsyncAtomicWriteEp, 
    {

        match op {
            AtomicOp::Min => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_min_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::Max => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_max_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::Sum => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_sum_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::Prod => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_prod_mr_slice_async(ep, ioc, desc, slice, context).await,
            // AtomicOp::Lor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_lor_mr_slice_async(ep, ioc, desc, slice, context).await,
            // AtomicOp::Land => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_land_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::Bor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_bor_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::Band => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_band_mr_slice_async(ep, ioc, desc, slice, context).await,
            // AtomicOp::Lxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_lxor_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::Bxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_bxor_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::AtomicWrite => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_write_mr_slice_async(ep, ioc, desc, slice, context).await,
            _ => todo!(),
        }
    }

    async unsafe fn conn_atomicv_bool_op<A>(op: libfabric::enums::AtomicOp, ep: &A, ioc: &[libfabric::iovec::Ioc<'_, bool>], desc: Option<&[MemoryRegionDesc<'_>]>, slice: &RemoteMemAddrSliceMut<'_, bool>, context: &mut Context) -> Result<SingleCompletion, libfabric::error::Error>
    where
        A: ConnectedAsyncAtomicWriteEp, 
    {

        match op {
            AtomicOp::Lor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_lor_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::Land => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_land_mr_slice_async(ep, ioc, desc, slice, context).await,
            AtomicOp::Lxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomicv_lxor_mr_slice_async(ep, ioc, desc, slice, context).await,
            _ => todo!(),
        }
    }

    async unsafe fn conn_atomic_op<T, A>(op: libfabric::enums::AtomicOp, ep: &A, buf: &[T], desc: Option<MemoryRegionDesc<'_>>, slice: &RemoteMemAddrSliceMut<'_, T>, context: &mut Context) -> Result<SingleCompletion, libfabric::error::Error>
    where
        T: AsFiType,
        A: ConnectedAsyncAtomicWriteEp, 
    {

        match op {
            AtomicOp::Min => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_min_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::Max => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_max_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::Sum => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_sum_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::Prod => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_prod_mr_slice_async(ep, buf, desc, slice, context).await,
            // AtomicOp::Lor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_lor_mr_slice_async(ep, buf, desc, slice, context).await,
            // AtomicOp::Land => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_land_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::Bor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_bor_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::Band => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_band_mr_slice_async(ep, buf, desc, slice, context).await,
            // AtomicOp::Lxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_lxor_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::Bxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_bxor_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::AtomicWrite => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_write_mr_slice_async(ep, buf, desc, slice, context).await,
            _ => todo!(),
        }
    }

    async unsafe fn conn_atomic_bool_op<A>(op: libfabric::enums::AtomicOp, ep: &A, buf: &[bool], desc: Option<MemoryRegionDesc<'_>>, slice: &RemoteMemAddrSliceMut<'_, bool>, context: &mut Context) -> Result<SingleCompletion, libfabric::error::Error>
    where
        A: ConnectedAsyncAtomicWriteEp, 
    {

        match op {
            AtomicOp::Lor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_lor_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::Land => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_land_mr_slice_async(ep, buf, desc, slice, context).await,
            AtomicOp::Lxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_lxor_mr_slice_async(ep, buf, desc, slice, context).await,
            _ => todo!(),
        }
    }

    async unsafe fn atomic_inject_op<T, A>(op: libfabric::enums::AtomicOp, ep: &A, buf: &[T], dest: &MappedAddress, slice: &RemoteMemAddrSliceMut<'_, T>) -> Result<(), libfabric::error::Error>
    where
        T: AsFiType,
        A: AsyncAtomicWriteEp,
    {
        match op {
            AtomicOp::Min => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_min_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::Max => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_max_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::Sum => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_sum_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::Prod => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_prod_mr_slice_to_async(ep, buf, dest, slice).await,
            // AtomicOp::Lor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lor_mr_slice_to_async(ep, buf, dest, slice).await,
            // AtomicOp::Land => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_land_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::Bor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_bor_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::Band => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_band_mr_slice_to_async(ep, buf, dest, slice).await,
            // AtomicOp::Lxor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lxor_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::Bxor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_bxor_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::AtomicWrite => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_write_mr_slice_to_async(ep, buf, dest, slice).await,
            _ => todo!(),
        }
    }

    async unsafe fn atomic_inject_bool_op<A>(op: libfabric::enums::AtomicOp, ep: &A, buf: &[bool], dest: &MappedAddress, slice: &RemoteMemAddrSliceMut<'_, bool>) -> Result<(), libfabric::error::Error>
    where
        A: AsyncAtomicWriteEp,
    {
        match op {
            AtomicOp::Lor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lor_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::Land => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_land_mr_slice_to_async(ep, buf, dest, slice).await,
            AtomicOp::Lxor => AsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lxor_mr_slice_to_async(ep, buf, dest, slice).await,
            _ => todo!(),
        }
    }

    async unsafe fn conn_atomic_inject_op<T, A>(op: libfabric::enums::AtomicOp, ep: &A, buf: &[T], slice: &RemoteMemAddrSliceMut<'_, T>) -> Result<(), libfabric::error::Error>
    where
        T: AsFiType,
        A: ConnectedAsyncAtomicWriteEp,
    {
        match op {
            AtomicOp::Min => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_min_mr_slice_async(ep, buf,slice).await,
            AtomicOp::Max => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_max_mr_slice_async(ep, buf,slice).await,
            AtomicOp::Sum => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_sum_mr_slice_async(ep, buf,slice).await,
            AtomicOp::Prod => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_prod_mr_slice_async(ep, buf,slice).await,
            // AtomicOp::Lor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lor_mr_slice_async(ep, buf,slice).await,
            // AtomicOp::Land => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_land_mr_slice_async(ep, buf,slice).await,
            AtomicOp::Bor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_bor_mr_slice_async(ep, buf,slice).await,
            AtomicOp::Band => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_band_mr_slice_async(ep, buf,slice).await,
            // AtomicOp::Lxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lxor_mr_slice_async(ep, buf,slice).await,
            AtomicOp::Bxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_bxor_mr_slice_async(ep, buf,slice).await,
            AtomicOp::AtomicWrite => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_write_mr_slice_async(ep, buf,slice).await,
            _ => todo!(),
        }
    }

    async unsafe fn conn_atomic_inject_bool_op<A>(op: libfabric::enums::AtomicOp, ep: &A, buf: &[bool], slice: &RemoteMemAddrSliceMut<'_, bool>) -> Result<(), libfabric::error::Error>
    where
        A: ConnectedAsyncAtomicWriteEp,
    {
        match op {
            AtomicOp::Lor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lor_mr_slice_async(ep, buf,slice).await,
            AtomicOp::Land => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_land_mr_slice_async(ep, buf,slice).await,
            AtomicOp::Lxor => ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lxor_mr_slice_async(ep, buf,slice).await,
            _ => todo!(),
        }
    }


    async unsafe fn get_atomic_fetch_op<T, A>(
        op: libfabric::enums::FetchAtomicOp,
        ep: &A,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest: &MappedAddress,
        slice: &RemoteMemAddrSlice<'_, T>,
        ctx: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        T: AsFiType,
        A: AsyncAtomicFetchEp, 
    {

        match op {
            FetchAtomicOp::Min => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_min_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Max => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_max_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Sum => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_sum_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Prod => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_prod_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Bor => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_bor_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Band => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_band_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            // FetchAtomicOp::Lxor => AsyncAtomicFetchRemoteMemAddrSliceEp::atommr_slice_ic_lxor_to(ep, buf, desc, res, res_desc, dest, slice),
            FetchAtomicOp::Bxor => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_bxor_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::AtomicWrite => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_write_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::AtomicRead => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_read_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            _ => todo!(),
        }
    }

    async unsafe fn get_atomic_fetch_bool_op<A>(
        op: libfabric::enums::FetchAtomicOp,
        ep: &A,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [bool],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest: &MappedAddress,
        slice: &RemoteMemAddrSlice<'_, bool>,
        ctx: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        A: AsyncAtomicFetchEp, 
    {

        match op {
            FetchAtomicOp::Lor => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_lor_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Land => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_land_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Lxor => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_lxor_mr_slice_from_async(ep, buf, desc, res, res_desc, dest, slice, ctx).await,
            _ => todo!(),
        }
    }

    async unsafe fn get_atomicv_fetch_op<T, A>(
        op: libfabric::enums::FetchAtomicOp,
        ep: &A,
        ioc: &[libfabric::iovec::Ioc<'_, T>], 
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resioc: &mut [libfabric::iovec::IocMut<'_,T>], 
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest: &MappedAddress,
        slice: &RemoteMemAddrSlice<'_, T>,
        ctx: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        T: AsFiType,
        A: AsyncAtomicFetchEp, 
    {

        match op {
            FetchAtomicOp::Min => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_min_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Max => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_max_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Sum => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_sum_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Prod => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_prod_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Bor => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_bor_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Band => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_band_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Bxor => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_bxor_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::AtomicWrite => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_write_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::AtomicRead => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_read_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            _ => todo!(),
        }
    }

    async unsafe fn get_conn_atomic_fetch_op<T, A>(
        op: libfabric::enums::FetchAtomicOp,
        ep: &A,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        slice: &RemoteMemAddrSlice<'_, T>,
        ctx: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        T: AsFiType,
        A: ConnectedAsyncAtomicFetchEp, 
    {

        match op {
            FetchAtomicOp::Min => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_min_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::Max => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_max_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::Sum => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_sum_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::Prod => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_prod_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::Bor => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_bor_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::Band => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_band_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::Bxor => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_bxor_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::AtomicWrite => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_write_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::AtomicRead => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_read_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            _ => todo!(),
        }
    }

    async unsafe fn get_conn_atomicv_fetch_op<T, A>(
        op: libfabric::enums::FetchAtomicOp,
        ep: &A,
        ioc: &[libfabric::iovec::Ioc<'_, T>], 
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resioc: &mut [libfabric::iovec::IocMut<'_, T>], 
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        slice: &RemoteMemAddrSlice<'_, T>,
        ctx: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        T: AsFiType,
        A: ConnectedAsyncAtomicFetchEp, 
    {

        match op {
            FetchAtomicOp::Min => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_min_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::Max => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_max_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::Sum => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_sum_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::Prod => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_prod_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::Bor => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_bor_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::Band => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_band_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::Bxor => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_bxor_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::AtomicWrite => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_write_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::AtomicRead => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_read_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            _ => todo!(),
        }
    }

    async unsafe fn get_atomicv_fetch_bool_op<A>(
        op: libfabric::enums::FetchAtomicOp,
        ep: &A,
        ioc: &[libfabric::iovec::Ioc<'_, bool>], 
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resioc: &mut [libfabric::iovec::IocMut<'_,bool>], 
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest: &MappedAddress,
        slice: &RemoteMemAddrSlice<'_, bool>,
        ctx: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        A: AsyncAtomicFetchEp, 
    {

        match op {
            FetchAtomicOp::Lor => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_lor_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Land => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_land_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            FetchAtomicOp::Lxor => AsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_lxor_mr_slice_from_async(ep, ioc, desc, resioc, res_desc, dest, slice, ctx).await,
            _ => todo!(),
        }
    }

    async unsafe fn get_conn_atomic_fetch_bool_op<A>(
        op: libfabric::enums::FetchAtomicOp,
        ep: &A,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [bool],
        res_desc: Option<MemoryRegionDesc<'_>>,
        slice: &RemoteMemAddrSlice<'_, bool>,
        ctx: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        A: ConnectedAsyncAtomicFetchEp, 
    {

        match op {
            FetchAtomicOp::Lor => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_lor_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::Land => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_land_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            FetchAtomicOp::Lxor => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_lxor_mr_slice_async(ep, buf, desc, res, res_desc, slice, ctx).await,
            _ => todo!(),
        }
    }

    async unsafe fn get_conn_atomicv_fetch_bool_op<A>(
        op: libfabric::enums::FetchAtomicOp,
        ep: &A,
        ioc: &[libfabric::iovec::Ioc<'_, bool>], 
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resioc: &mut [libfabric::iovec::IocMut<'_, bool>], 
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        slice: &RemoteMemAddrSlice<'_, bool>,
        ctx: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        A: ConnectedAsyncAtomicFetchEp, 
    {

        match op {
            FetchAtomicOp::Lor => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_lor_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::Land => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_land_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            FetchAtomicOp::Lxor => ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_lxor_mr_slice_async(ep, ioc, desc, resioc, res_desc, slice, ctx).await,
            _ => todo!(),
        }
    }

    async unsafe fn get_atomic_compare_op<T, A>(
        op: libfabric::enums::CompareAtomicOp,
        ep: &A,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &MappedAddress,
        dst_slice: &RemoteMemAddrSliceMut<'_, T>, 
        context: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        T: AsFiType,
        A: AsyncAtomicCASEp, 
    {

        match op {
            CompareAtomicOp::Cswap => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_mr_slice_to_async(ep, buf, desc, compare, compare_desc, result, result_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapGe => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_ge_mr_slice_to_async(ep, buf, desc, compare, compare_desc, result, result_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapGt => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_gt_mr_slice_to_async(ep, buf, desc, compare, compare_desc, result, result_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapLe => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_le_mr_slice_to_async(ep, buf, desc, compare, compare_desc, result, result_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapLt => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_lt_mr_slice_to_async(ep, buf, desc, compare, compare_desc, result, result_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapNe => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_ne_mr_slice_to_async(ep, buf, desc, compare, compare_desc, result, result_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::Mswap => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_mswap_mr_slice_to_async(ep, buf, desc, compare, compare_desc, result, result_desc, dest_addr, dst_slice, context).await,
        }
    }

    async unsafe fn get_atomicv_compare_op<T, A>(
        op: libfabric::enums::CompareAtomicOp,
        ep: &A,
        ioc: &[libfabric::iovec::Ioc<'_, T>], 
        desc: Option<&[MemoryRegionDesc<'_>]>, 
        comparetv: &[libfabric::iovec::Ioc<'_, T>], 
        compare_desc: Option<&[MemoryRegionDesc<'_>]>, 
        resultv: &mut [libfabric::iovec::IocMut<'_, T>], 
        res_desc: Option<&[MemoryRegionDesc<'_>]>, 
        dest_addr: &MappedAddress, 
        dst_slice: &RemoteMemAddrSliceMut<'_, T>, 
        context: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        T: AsFiType,
        A: AsyncAtomicCASEp, 
    {

        match op {
            CompareAtomicOp::Cswap => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_mr_slice_to_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapGe => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_ge_mr_slice_to_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapGt => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_gt_mr_slice_to_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapLe => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_le_mr_slice_to_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapLt => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_lt_mr_slice_to_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::CswapNe => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_ne_mr_slice_to_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dest_addr, dst_slice, context).await,
            CompareAtomicOp::Mswap => AsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_mswap_mr_slice_to_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dest_addr, dst_slice, context).await,
        }
    }

    async unsafe fn get_conn_atomic_compare_op<T, A>(
        op: libfabric::enums::CompareAtomicOp,
        ep: &A,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dst_slice: &RemoteMemAddrSliceMut<'_, T>, 
        context: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        T: AsFiType,
        A: ConnectedAsyncAtomicCASEp, 
    {

        match op {
            CompareAtomicOp::Cswap => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_mr_slice_async(ep, buf, desc, compare, compare_desc, result, result_desc, dst_slice, context).await,
            CompareAtomicOp::CswapGe => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_ge_mr_slice_async(ep, buf, desc, compare, compare_desc, result, result_desc, dst_slice, context).await,
            CompareAtomicOp::CswapGt => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_gt_mr_slice_async(ep, buf, desc, compare, compare_desc, result, result_desc, dst_slice, context).await,
            CompareAtomicOp::CswapLe => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_le_mr_slice_async(ep, buf, desc, compare, compare_desc, result, result_desc, dst_slice, context).await,
            CompareAtomicOp::CswapLt => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_lt_mr_slice_async(ep, buf, desc, compare, compare_desc, result, result_desc, dst_slice, context).await,
            CompareAtomicOp::CswapNe => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_ne_mr_slice_async(ep, buf, desc, compare, compare_desc, result, result_desc, dst_slice, context).await,
            CompareAtomicOp::Mswap => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomic_mswap_mr_slice_async(ep, buf, desc, compare, compare_desc, result, result_desc, dst_slice, context).await,
        }
    }

    async unsafe fn get_conn_atomicv_compare_op<T, A>(
        op: libfabric::enums::CompareAtomicOp,
        ep: &A,
        ioc: &[libfabric::iovec::Ioc<'_, T>], 
        desc: Option<&[MemoryRegionDesc<'_>]>, 
        comparetv: &[libfabric::iovec::Ioc<'_, T>], 
        compare_desc: Option<&[MemoryRegionDesc<'_>]>, 
        resultv: &mut [libfabric::iovec::IocMut<'_, T>], 
        res_desc: Option<&[MemoryRegionDesc<'_>]>, 
        dst_slice: &RemoteMemAddrSliceMut<'_, T>, 
        context: &mut Context
    ) 
    -> Result<SingleCompletion, Error>
    where
        T: AsFiType,
        A: ConnectedAsyncAtomicCASEp, 
    {

        match op {
            CompareAtomicOp::Cswap => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_mr_slice_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dst_slice, context).await,
            CompareAtomicOp::CswapGe => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_ge_mr_slice_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dst_slice, context).await,
            CompareAtomicOp::CswapGt => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_gt_mr_slice_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dst_slice, context).await,
            CompareAtomicOp::CswapLe => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_le_mr_slice_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dst_slice, context).await,
            CompareAtomicOp::CswapLt => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_lt_mr_slice_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dst_slice, context).await,
            CompareAtomicOp::CswapNe => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_ne_mr_slice_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dst_slice, context).await,
            CompareAtomicOp::Mswap => ConnectedAsyncAtomicCASRemoteMemAddrSliceEp::compare_atomicv_mswap_mr_slice_async(ep, ioc, desc, comparetv, compare_desc, resultv, res_desc, dst_slice, context).await,
        }
    }

    impl<I: AtomicDefaultCap> Ofi<I> {
        pub fn atomic<T: libfabric::AsFiType>(
            &self,
            buf: &[T],
            dest_addr: usize,
            desc: Option<MemoryRegionDesc>,
            op: AtomicOp,
            ctx: &mut Context,
        ) {
            let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
            let dst_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());

            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            unsafe {
                                atomic_inject_op(op, ep, buf, &self.mapped_addr.as_ref().unwrap()[1], &dst_slice)
                                .await
                            }
                        } else {
                            unsafe {
                                atomic_op(op, ep, buf, desc, &self.mapped_addr.as_ref().unwrap()[1], &dst_slice, ctx)
                                .await
                            }
                            .map(|_| {})
                        }
                    }
                    MyEndpoint::Connected(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            unsafe { conn_atomic_inject_op(op, ep, buf, &dst_slice).await }
                        } else {
                            unsafe { conn_atomic_op(op, ep, buf, desc, &dst_slice, ctx).await }
                                .map(|_| {})
                        }
                    }
                }
            })
            .unwrap()
        }

        pub fn atomic_bool(
            &self,
            buf: &[bool],
            dest_addr: usize,
            desc: Option<MemoryRegionDesc>,
            op: AtomicOp,
            ctx: &mut Context,
        ) {
            let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
            let dst_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());

            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            unsafe {
                                atomic_inject_bool_op(op, ep, buf, &self.mapped_addr.as_ref().unwrap()[1], &dst_slice)
                                .await
                            }
                        } else {
                            unsafe {
                                atomic_bool_op(op, ep, buf, desc, &self.mapped_addr.as_ref().unwrap()[1], &dst_slice, ctx)
                                .await
                            }
                            .map(|_| {})
                        }
                    }
                    MyEndpoint::Connected(ep) => {
                        if buf.len() <= self.info_entry.tx_attr().inject_size() {
                            unsafe { conn_atomic_inject_bool_op(op, ep, buf, &dst_slice).await }
                        } else {
                            unsafe { conn_atomic_bool_op(op, ep, buf, desc, &dst_slice, ctx).await }
                                .map(|_| {})
                        }
                    }
                }
            })
            .unwrap()
        }

        pub fn atomicv<T: libfabric::AsFiType>(
            &self,
            ioc: &[libfabric::iovec::Ioc<T>],
            dest_addr: usize,
            desc: Option<&[MemoryRegionDesc]>,
            op: AtomicOp,
            ctx: &mut Context,
        ) {
            let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
            let dest_slice = remote_mem_info
                .slice_mut(dest_addr..dest_addr + ioc.iter().fold(0, |acc, x| acc + x.len()));
            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        atomicv_op(op, ep, ioc, desc, &self.mapped_addr.as_ref().unwrap()[1], &dest_slice, ctx) 
                        .await
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        conn_atomicv_op(op, ep, ioc, desc, &dest_slice, ctx)
                            .await
                    },
                }
            })
            .unwrap();
        }

        pub fn atomicv_bool(
            &self,
            ioc: &[libfabric::iovec::Ioc<bool>],
            dest_addr: usize,
            desc: Option<&[MemoryRegionDesc]>,
            op: AtomicOp,
            ctx: &mut Context,
        ) {
            let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
            let dest_slice = remote_mem_info
                .slice_mut(dest_addr..dest_addr + ioc.iter().fold(0, |acc, x| acc + x.len()));
            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        atomicv_bool_op(op, ep, ioc, desc, &self.mapped_addr.as_ref().unwrap()[1], &dest_slice, ctx) 
                        .await
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        conn_atomicv_bool_op(op, ep, ioc, desc, &dest_slice, ctx)
                            .await
                    },
                }
            })
            .unwrap();
        }

        pub fn atomicmsg<T: libfabric::AsFiType + 'static>(
            &self,
            msg: &mut Either<MsgAtomic<T>, MsgAtomicConnected<T>>,
        ) {
            let opts = AtomicMsgOptions::new();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => unsafe { ep.atomicmsg_to_async(msg, opts).await },
                        Either::Right(_) => todo!(),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => todo!(),
                        Either::Right(msg) => unsafe { ep.atomicmsg_async(msg, opts).await },
                    },
                }
            })
            .unwrap();
        }



        pub fn fetch_atomic<T: libfabric::AsFiType>(
            &self,
            buf: &[T],
            res: &mut [T],
            dest_addr: usize,
            desc: Option<MemoryRegionDesc>,
            res_desc: Option<MemoryRegionDesc>,
            op: FetchAtomicOp,
            ctx: &mut Context,
        ) {
            let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
            let src_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());
            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        get_atomic_fetch_op(
                            op,
                            ep,
                            buf,
                            desc,
                            res,
                            res_desc,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            &src_slice,
                            ctx,
                        )
                        .await
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        get_conn_atomic_fetch_op(op, ep, buf, desc, res, res_desc, &src_slice, ctx)
                            .await
                    },
                }
            })
            .unwrap();
        }

        pub fn fetch_atomicv<T: libfabric::AsFiType>(
            &self,
            ioc: &[libfabric::iovec::Ioc<'_, T>],
            res_ioc: &mut [libfabric::iovec::IocMut<'_, T>],
            dest_addr: usize,
            desc: Option<&[MemoryRegionDesc]>,
            res_desc: Option<&[MemoryRegionDesc]>,
            op: FetchAtomicOp,
            ctx: &mut Context,
        ) {
            let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
            let src_slice: RemoteMemAddrSlice<'_, T> = remote_mem_info
                .slice(dest_addr..dest_addr + ioc.iter().fold(0, |acc, x| acc + x.len()));
            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            let _ = async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        get_atomicv_fetch_op(op,ep,ioc,desc,res_ioc,res_desc,&self.mapped_addr.as_ref().unwrap()[1],&src_slice,ctx)
                        .await
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        get_conn_atomicv_fetch_op(op,ep,ioc, desc, res_ioc, res_desc, &src_slice, ctx)
                        .await
                    },
                }
            })
            .unwrap();
        }


        pub fn fetch_atomic_bool(
            &self,
            buf: &[bool],
            res: &mut [bool],
            dest_addr: usize,
            desc: Option<MemoryRegionDesc>,
            res_desc: Option<MemoryRegionDesc>,
            op: FetchAtomicOp,
            ctx: &mut Context,
        ) {
            let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
            let src_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());
            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        get_atomic_fetch_bool_op(
                            op,
                            ep,
                            buf,
                            desc,
                            res,
                            res_desc,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            &src_slice,
                            ctx,
                        )
                        .await
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        get_conn_atomic_fetch_bool_op(op, ep, buf, desc, res, res_desc, &src_slice, ctx)
                            .await
                    },
                }
            })
            .unwrap();
        }

        pub fn fetch_atomicv_bool(
            &self,
            ioc: &[libfabric::iovec::Ioc<'_, bool>],
            res_ioc: &mut [libfabric::iovec::IocMut<'_, bool>],
            dest_addr: usize,
            desc: Option<&[MemoryRegionDesc]>,
            res_desc: Option<&[MemoryRegionDesc]>,
            op: FetchAtomicOp,
            ctx: &mut Context,
        ) {
            let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
            let src_slice: RemoteMemAddrSlice<'_, bool> = remote_mem_info
                .slice(dest_addr..dest_addr + ioc.iter().fold(0, |acc, x| acc + x.len()));
            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            let _ = async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        get_atomicv_fetch_bool_op(op,ep,ioc,desc,res_ioc,res_desc,&self.mapped_addr.as_ref().unwrap()[1],&src_slice,ctx)
                        .await
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        get_conn_atomicv_fetch_bool_op(op,ep,ioc, desc, res_ioc, res_desc, &src_slice, ctx)
                        .await
                    },
                }
            })
            .unwrap();
        }

        pub fn fetch_atomicmsg<T: libfabric::AsFiType + 'static>(
            &self,
            msg: &mut Either<MsgFetchAtomic<T>, MsgFetchAtomicConnected<T>>,
            res_ioc: &mut [libfabric::iovec::IocMut<T>],
            res_desc: Option<&[MemoryRegionDesc]>,
        ) {
            let opts = AtomicMsgOptions::new();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => unsafe {
                            ep.fetch_atomicmsg_from_async(msg, res_ioc, res_desc, opts)
                                .await
                        },
                        Either::Right(_) => todo!(),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => todo!(),
                        Either::Right(msg) => unsafe {
                            ep.fetch_atomicmsg_async(msg, res_ioc, res_desc, opts).await
                        },
                    },
                }
            })
            .unwrap();
        }

        pub fn compare_atomic<T: libfabric::AsFiType>(
            &self,
            buf: &[T],
            comp: &[T],
            res: &mut [T],
            dest_addr: usize,
            desc: Option<MemoryRegionDesc>,
            comp_desc: Option<MemoryRegionDesc>,
            res_desc: Option<MemoryRegionDesc>,
            op: CompareAtomicOp,
            ctx: &mut Context,
        ) {
            let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
            let dst_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());
            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        get_atomic_compare_op(op, ep, buf, desc, comp, comp_desc, res, res_desc, &self.mapped_addr.as_ref().unwrap()[1], &dst_slice, ctx).await
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        get_conn_atomic_compare_op(op, ep, buf, desc, comp, comp_desc, res, res_desc, &dst_slice, ctx).await
                    },
                }
            })
            .unwrap();
        }

        pub fn compare_atomicv<T: libfabric::AsFiType>(
            &self,
            ioc: &[libfabric::iovec::Ioc<T>],
            comp_ioc: &[libfabric::iovec::Ioc<T>],
            res_ioc: &mut [libfabric::iovec::IocMut<T>],
            dest_addr: usize,
            desc: Option<&[MemoryRegionDesc]>,
            comp_desc: Option<&[MemoryRegionDesc]>,
            res_desc: Option<&[MemoryRegionDesc]>,
            op: CompareAtomicOp,
            ctx: &mut Context,
        ) {
            let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
            let dst_slice = remote_mem_info
                .slice_mut(dest_addr..dest_addr + ioc.iter().fold(0, |acc, x| acc + x.len()));
            // let key = &remote_mem_info.key();
            // let base_addr = remote_mem_info.mem_address();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => unsafe {
                        get_atomicv_compare_op(op,
                            ep,
                            ioc,
                            desc,
                            comp_ioc,
                            comp_desc,
                            res_ioc,
                            res_desc,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            &dst_slice,
                            ctx,
                        )
                        .await
                    },
                    MyEndpoint::Connected(ep) => unsafe {
                        get_conn_atomicv_compare_op(op, ep, ioc, desc, comp_ioc, comp_desc, res_ioc, res_desc, &dst_slice, ctx,)
                        .await
                    },
                }
            })
            .unwrap();
        }

        pub fn compare_atomicmsg<T: libfabric::AsFiType + 'static>(
            &self,
            msg: &mut Either<MsgCompareAtomic<T>, MsgCompareAtomicConnected<T>>,
            comp_ioc: &[libfabric::iovec::Ioc<T>],
            res_ioc: &mut [libfabric::iovec::IocMut<T>],
            comp_desc: Option<&[MemoryRegionDesc]>,
            res_desc: Option<&[MemoryRegionDesc]>,
        ) {
            let opts = AtomicMsgOptions::new();
            async_std::task::block_on(async {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => unsafe {
                            ep.compare_atomicmsg_to_async(
                                msg, comp_ioc, comp_desc, res_ioc, res_desc, opts,
                            )
                            .await
                        },
                        Either::Right(_) => todo!(),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => todo!(),
                        Either::Right(msg) => unsafe {
                            ep.compare_atomicmsg_async(
                                msg, comp_ioc, comp_desc, res_ioc, res_desc, opts,
                            )
                            .await
                        },
                    },
                }
            })
            .unwrap();
        }
    }

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
    fn async_handshake_connected0() {
        handshake(true, "handshake_connected0", Some(InfoCaps::new().msg()));
    }

    #[test]
    fn async_handshake_connected1() {
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
    fn async_handshake_connectionless0() {
        handshake_connectionless(
            true,
            "handshake_connectionless0",
            Some(InfoCaps::new().msg()),
        );
    }

    #[test]
    fn async_handshake_connectionless1() {
        handshake_connectionless(
            false,
            "handshake_connectionless0",
            Some(InfoCaps::new().msg()),
        );
    }

    fn sendrecv(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
        };

        let mut reg_mem: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&ofi.domain)
            .unwrap();

        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = [mr.descriptor(), mr.descriptor()];
        let desc0 = Some(mr.descriptor());
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            // Send a single buffer
            ofi.send(&reg_mem[..512], desc0, None, &mut ctx);
            assert!(
                std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size()
            );

            // Inject a buffer
            ofi.send(&reg_mem[..128], desc0, None, &mut ctx);
            // No cq.sread since inject does not generate completions

            // // Send single Iov
            let iov = [IoVec::from_slice(&reg_mem[..512])];
            ofi.sendv(&iov, Some(&desc[..1]), &mut ctx);

            // Send multi Iov
            let iov = [
                IoVec::from_slice(&reg_mem[..512]),
                IoVec::from_slice(&reg_mem[512..1024]),
            ];
            ofi.sendv(&iov, Some(&desc), &mut ctx);
        } else {

            let expected: Vec<_> = (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect();
            reg_mem.iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            ofi.recv(&mut reg_mem[..512], desc0.clone(), &mut ctx);
            assert_eq!(reg_mem[..512], expected[..512]);

            // Receive inject
            reg_mem.iter_mut().for_each(|v| *v = 0);
            ofi.recv(&mut reg_mem[..128], desc0.clone(), &mut ctx);
            assert_eq!(reg_mem[..128], expected[..128]);

            reg_mem.iter_mut().for_each(|v| *v = 0);
            // // Receive into a single Iov
            let mut iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
            ofi.recvv(&mut iov, Some(&desc[..1]), &mut ctx);
            assert_eq!(reg_mem[..512], expected[..512]);

            reg_mem.iter_mut().for_each(|v| *v = 0);

            // // Receive into multiple Iovs
            let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
            let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            ofi.recvv(&iov, Some(&desc), &mut ctx);

            assert_eq!(mem0, &expected[..512]);
            assert_eq!(mem1, &expected[512..1024]);
        }
    }

    // fn sendrecv_deadlock(server: bool, name: &str, connected: bool) {
    //     let ofi = if connected {
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
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
    //             match disabled_mr {
    //                 libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&ofi.ep, ep_binding_memory_region),
    //                 libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
    //             }
    //         }
    //     };

    //     let desc0 = Some(mr.descriptor());
    //     let mut send_ctx = ofi.info_entry.allocate_context();
    //     let mut recv_ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         // Send a single buffer
    //         // ofi.send(&reg_mem[..512], desc0.as_ref(), None, &mut ctx);
    //         // assert!(
    //         //     std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size()
    //         // );

    //         ofi.sendrecv_deadlock(&mut reg_mem[..512], desc0.as_ref(), None, &mut send_ctx, &mut recv_ctx);
    //         // Inject a buffer
    //         // ofi.send(&reg_mem[..128], desc0.as_ref(), None, &mut ctx);
    //         // // No cq.sread since inject does not generate completions

    //         // // // Send single Iov
    //         // let iov = [IoVec::from_slice(&reg_mem[..512])];
    //         // ofi.sendv(&iov, Some(&desc[..1]), &mut ctx);

    //         // // Send multi Iov
    //         // let iov = [
    //         //     IoVec::from_slice(&reg_mem[..512]),
    //         //     IoVec::from_slice(&reg_mem[512..1024]),
    //         // ];
    //         // ofi.sendv(&iov, Some(&desc), &mut ctx);
    //     } else {
    //         ofi.sendrecv_deadlock(&mut reg_mem[..512], desc0.as_ref(), None, &mut send_ctx, &mut recv_ctx);
    //     }
    // }

    #[test]
    fn async_sendrecv0() {
        sendrecv(true, "sendrecv0", false);
    }

    #[test]
    fn async_sendrecv1() {
        sendrecv(false, "sendrecv0", false);
    }

    // #[test]
    // fn async_sendrecv_deadlock0() {
    //     sendrecv_deadlock(true, "sendrecv0", false);
    // }

    // #[test]
    // fn async_sendrecv_deadlock1() {
    //     sendrecv_deadlock(false, "sendrecv0", false);
    // }

    #[test]
    fn async_conn_sendrecv0() {
        sendrecv(true, "conn_sendrecv0", true);
    }

    #[test]
    fn async_conn_sendrecv1() {
        sendrecv(false, "conn_sendrecv0", true);
    }

    fn sendrecvdata(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
        };

        let mut reg_mem: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&ofi.domain)
            .unwrap();

        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc0 = Some(mr.descriptor());
        let data = Some(128u64);
        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            // Send a single buffer
            ofi.send(&reg_mem[..512], desc0, data, &mut ctx);
        } else {
            let expected: Vec<_> = (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect();
            reg_mem.iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            ofi.recv(&mut reg_mem[..512], desc0, &mut ctx);
            assert_eq!(reg_mem[..512], expected[..512]);
        }
    }

    #[test]
    fn async_sendrecvdata0() {
        sendrecvdata(true, "sendrecvdata0", false);
    }

    #[test]
    fn async_sendrecvdata1() {
        sendrecvdata(false, "sendrecvdata0", false);
    }

    #[test]
    fn async_conn_sendrecvdata0() {
        sendrecvdata(true, "conn_sendrecvdata0", true);
    }

    #[test]
    fn async_conn_sendrecvdata1() {
        sendrecvdata(false, "conn_sendrecvdata0", true);
    }

    fn enable_ep_mr<E: 'static>(ep: &MyEndpoint<E>, mr: EpBindingMemoryRegion) -> MemoryRegion {
        match ep {
            MyEndpoint::Connected(ep) => mr.enable(ep).unwrap(),
            MyEndpoint::Connectionless(ep) => mr.enable(ep).unwrap(),
        }
    }

    fn tsendrecv(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().tagged()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().tagged()))
        };

        let mut reg_mem: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&ofi.domain)
            .unwrap();

        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = [mr.descriptor(), mr.descriptor()];
        let desc0 = Some(mr.descriptor());
        let data = Some(128u64);
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            // Send a single buffer
            ofi.tsend(&reg_mem[..512], desc0, 10, data, &mut ctx);
            // match entry {
            //     Completion::Tagged(entry) => {assert_eq!(entry[0].data(), data.unwrap()); assert_eq!(entry[0].tag(), 10)},
            //     _ => panic!("Unexpected CQ entry format"),
            // }

            assert!(
                std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size()
            );

            // Inject a buffer
            ofi.tsend(&reg_mem[..128], desc0, 1, data, &mut ctx);
            // No cq.sread since inject does not generate completions

            // // Send single Iov
            let iov = [IoVec::from_slice(&reg_mem[..512])];
            ofi.tsendv(&iov, Some(&desc[..1]), 2, &mut ctx);

            // Send multi Iov
            let iov = [
                IoVec::from_slice(&reg_mem[..512]),
                IoVec::from_slice(&reg_mem[512..1024]),
            ];
            ofi.tsendv(&iov, Some(&desc), 3, &mut ctx);
        } else {
            let expected: Vec<_> = (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect();
            reg_mem.iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            ofi.trecv(&mut reg_mem[..512], desc0.clone(), 10, &mut ctx);

            assert_eq!(reg_mem[..512], expected[..512]);

            // Receive inject
            reg_mem.iter_mut().for_each(|v| *v = 0);
            ofi.trecv(&mut reg_mem[..128], desc0.clone(), 1, &mut ctx);
            assert_eq!(reg_mem[..128], expected[..128]);

            reg_mem.iter_mut().for_each(|v| *v = 0);
            // // Receive into a single Iov
            let mut iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
            ofi.trecvv(&mut iov, Some(&desc[..1]), 2, &mut ctx);
            assert_eq!(reg_mem[..512], expected[..512]);

            reg_mem.iter_mut().for_each(|v| *v = 0);

            // // Receive into multiple Iovs
            let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
            let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            ofi.trecvv(&iov, Some(&desc), 3, &mut ctx);

            assert_eq!(mem0, &expected[..512]);
            assert_eq!(mem1, &expected[512..1024]);
        }
    }

    #[test]
    fn async_tsendrecv0() {
        tsendrecv(true, "tsendrecv0", false);
    }

    #[test]
    fn async_tsendrecv1() {
        tsendrecv(false, "tsendrecv0", false);
    }

    #[test]
    fn async_conn_tsendrecv0() {
        tsendrecv(true, "conn_tsendrecv0", true);
    }

    #[test]
    fn async_conn_tsendrecv1() {
        tsendrecv(false, "conn_tsendrecv0", true);
    }

    fn sendrecvmsg(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
        };

        let mut reg_mem: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&ofi.domain)
            .unwrap();

        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = Some(mr.descriptor());
        let descs = [mr.descriptor(), mr.descriptor()];
        let mapped_addr = ofi.mapped_addr.clone();
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            // Single iov message
            let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
            let iov0 = IoVec::from_slice(mem0);
            let iov1 = IoVec::from_slice(mem1);
            let data = Some(128);
            let mut msg = if connected {
                Either::Right(MsgConnected::from_iov(&iov0, desc.as_ref(), data, &mut ctx))
            } else {
                Either::Left(Msg::from_iov(
                    &iov0,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    &mut ctx,
                ))
            };
            ofi.sendmsg(&mut msg);

            // let entry =
            // match entry {
            //     Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
            //     _ => panic!("Unexpected CQ entry format"),
            // }

            // Multi iov message with stride
            let iovs = [iov0, iov1];
            let mut msg = if connected {
                Either::Right(MsgConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    data,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    &mut ctx,
                ))
            };

            ofi.sendmsg(&mut msg);

            // Single iov message
            let mut msg = if connected {
                Either::Right(MsgConnected::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.sendmsg(&mut msg);

            let mut msg = if connected {
                Either::Right(MsgConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };
            ofi.sendmsg(&mut msg);
        } else {
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

            // Receive a single message in a single buffer
            let mut iov = IoVecMut::from_slice(mem0);
            let mut msg = if connected {
                Either::Right(MsgConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&mut msg);

            assert_eq!(mem0.len(), expected[..512].len());
            assert_eq!(mem0, &expected[..512]);

            // Receive a multi iov message in a single buffer
            let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
            let mut msg = if connected {
                Either::Right(MsgConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&mut msg);
            assert_eq!(mem1[..1024], expected);

            // Receive a single iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..256]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
            let mut iovs = [iov, iov1];
            let mut msg = if connected {
                Either::Right(MsgConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&mut msg);
            assert_eq!(mem0[..256], expected[..256]);
            assert_eq!(mem1[..256], expected[256..512]);

            // Receive a two iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..512]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
            let mut iovs = [iov, iov1];
            let mut msg = if connected {
                Either::Right(MsgConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&mut msg);
            assert_eq!(mem0[..512], expected[..512]);
            assert_eq!(mem1[..512], expected[512..1024]);
        }
    }

    #[test]
    fn async_sendrecvmsg0() {
        sendrecvmsg(true, "sendrecvmsg0", false);
    }

    #[test]
    fn async_sendrecvmsg1() {
        sendrecvmsg(false, "sendrecvmsg0", false);
    }

    #[test]
    fn async_conn_sendrecvmsg0() {
        sendrecvmsg(true, "conn_sendrecvmsg0", true);
    }

    #[test]
    fn async_conn_sendrecvmsg1() {
        sendrecvmsg(false, "conn_sendrecvmsg0", true);
    }

    fn tsendrecvmsg(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().tagged()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().tagged()))
        };

        let mut reg_mem: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&ofi.domain)
            .unwrap();
        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = Some(mr.descriptor());
        let descs = [mr.descriptor(), mr.descriptor()];
        let mapped_addr = ofi.mapped_addr.clone();
        let mut ctx = ofi.info_entry.allocate_context();
        let data = Some(128);
        if server {
            // Single iov message
            let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
            let iov0 = IoVec::from_slice(mem0);
            let iov1 = IoVec::from_slice(mem1);
            let mut msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov(
                    &iov0,
                    desc.as_ref(),
                    data,
                    0,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov(
                    &iov0,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    0,
                    None,
                    &mut ctx,
                ))
            };
            ofi.tsendmsg(&mut msg);

            // Multi iov message with stride
            let iovs = [iov0, iov1];
            let mut msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            };

            ofi.tsendmsg(&mut msg);

            // Single iov message
            let mut msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    Some(0),
                    2,
                    None,
                    &mut ctx,
                ))
            };

            ofi.tsendmsg(&mut msg);

            let mut msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            };
            ofi.tsendmsg(&mut msg);
        } else {
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

            // Receive a single message in a single buffer
            let mut iov = IoVecMut::from_slice(mem0);
            let mut msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    0,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    0,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&mut msg);
            assert_eq!(mem0.len(), expected[..512].len());
            assert_eq!(mem0, &expected[..512]);

            // Receive a multi iov message in a single buffer
            let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
            let mut msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&mut msg);
            assert_eq!(mem1[..1024], expected);

            // Receive a single iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..256]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
            let mut iovs = [iov, iov1];
            let mut msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&mut msg);
            assert_eq!(mem0[..256], expected[..256]);
            assert_eq!(mem1[..256], expected[256..512]);

            // Receive a two iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..512]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
            let mut iovs = [iov, iov1];
            let mut msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&mut msg);
            assert_eq!(mem0[..512], expected[..512]);
            assert_eq!(mem1[..512], expected[512..1024]);
        }
    }

    #[test]
    fn async_tsendrecvmsg0() {
        tsendrecvmsg(true, "tsendrecvmsg0", false);
    }

    #[test]
    fn async_tsendrecvmsg1() {
        tsendrecvmsg(false, "tsendrecvmsg0", false);
    }

    #[test]
    fn async_conn_tsendrecvmsg0() {
        tsendrecvmsg(true, "conn_tsendrecvmsg0", true);
    }

    #[test]
    fn async_conn_tsendrecvmsg1() {
        tsendrecvmsg(false, "conn_tsendrecvmsg0", true);
    }

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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = Some(mr.descriptor());
        let descs = [mr.descriptor(), mr.descriptor()];
        // let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            // Write inject a single buffer
            ofi.write(&reg_mem[..128], 0, desc, None, &mut ctx);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Write a single buffer
            ofi.write(&reg_mem[..512], 0, desc, None, &mut ctx);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Write vector of buffers
            let iovs = [
                IoVec::from_slice(&reg_mem[..512]),
                IoVec::from_slice(&reg_mem[512..1024]),
            ];
            ofi.writev(&iovs, 0, Some(&descs), &mut ctx);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc, &mut ctx);
        } else {
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..128], &expected[..128]);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..512], &expected[..512]);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[1024..1536], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..1024], &expected[..1024]);

            reg_mem.iter_mut().for_each(|v| *v = 0);

            // Read buffer from remote memory
            ofi.read(&mut reg_mem[1024..1536], 0, desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);

            // Read vector of buffers from remote memory
            let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
            let iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            ofi.readv(&iovs, 0, Some(&descs), &mut ctx);

            assert_eq!(mem0, &expected[..256]);
            assert_eq!(mem1, &expected[..256]);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    #[test]
    fn async_conn_writeread0() {
        writeread(true, "conn_writeread0", true);
    }

    #[test]
    fn async_conn_writeread1() {
        writeread(false, "conn_writeread0", true);
    }

    #[test]
    fn async_writeread0() {
        writeread(true, "writeread0", false);
    }

    #[test]
    fn async_writeread1() {
        writeread(false, "writeread0", false);
    }

    fn writereadmsg(server: bool, name: &str, connected: bool) {
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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        let desc = Some(mr.descriptor());
        let descs = [mr.descriptor(), mr.descriptor()];
        let mapped_addr = ofi.mapped_addr.clone();

        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let expected: Vec<u8> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
            let rma_addr = remote_mem_info.slice::<u8>(..128);
            let iov = IoVec::from_slice(&reg_mem[..128]);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr);

            let mut msg = if connected {
                Either::Right(MsgRmaConnected::from_iov(
                    &iov,
                    desc.as_ref(),
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRma::from_iov(
                    &iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            };

            // Write inject a single buffer
            ofi.writemsg(&mut msg);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            let iov = IoVec::from_slice(&reg_mem[..512]);
            let rma_addr = remote_mem_info.slice::<u8>(..512);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr);

            let mut msg = if connected {
                Either::Right(MsgRmaConnected::from_iov(
                    &iov,
                    desc.as_ref(),
                    &rma_iov,
                    Some(128),
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRma::from_iov(
                    &iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    Some(128),
                    &mut ctx,
                ))
            };

            // Write a single buffer
            ofi.writemsg(&mut msg);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            let iov0 = IoVec::from_slice(&reg_mem[..512]);
            let iov1 = IoVec::from_slice(&reg_mem[512..1024]);
            let iovs = [iov0, iov1];
            let rma_addr0 = remote_mem_info.slice::<u8>(..512);
            let rma_addr1 = remote_mem_info.slice::<u8>(512..1024);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr0);
            rma_iov.push(rma_addr1);

            let mut msg = if connected {
                Either::Right(MsgRmaConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRma::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            };

            ofi.writemsg(&mut msg);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
        } else {
            let mut remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow_mut();

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..128], &expected[..128]);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..512], &expected[..512]);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[1024..1536], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..1024], &expected[..1024]);

            reg_mem.iter_mut().for_each(|v| *v = 0);

            // let base_addr = remote_mem_info.mem_address();
            {
                let mut iov = IoVecMut::from_slice(&mut reg_mem[1024..1536]);
                let rma_addr = remote_mem_info.slice_mut::<u8>(..512);
                let mut rma_iov = RemoteMemAddrVecMut::new();
                rma_iov.push(rma_addr);

                // RmaIoVec::new()
                //     .address(base_addr)
                //     .len(512)
                //     .mapped_key(&key);
                // Read buffer from remote memory
                let mut msg = if connected {
                    Either::Right(MsgRmaConnectedMut::from_iov(
                        &mut iov,
                        desc.as_ref(),
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                } else {
                    Either::Left(MsgRmaMut::from_iov(
                        &mut iov,
                        desc.as_ref(),
                        &mapped_addr.as_ref().unwrap()[1],
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                };
                ofi.readmsg(&mut msg);
                assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);
            }

            // // Read vector of buffers from remote memory
            let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
            let mut iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            let (rma_addr0, rma_addr1) = remote_mem_info.slice_mut::<u8>(..512).split_at_mut(256);
            let mut rma_iov = RemoteMemAddrVecMut::new();
            rma_iov.push(rma_addr0);
            rma_iov.push(rma_addr1);

            let mut msg = if connected {
                Either::Right(MsgRmaConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRmaMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            };
            ofi.readmsg(&mut msg);

            assert_eq!(mem0, &expected[..256]);
            assert_eq!(mem1, &expected[..256]);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    #[test]
    fn async_writereadmsg0() {
        writereadmsg(true, "writereadmsg0", false);
    }

    #[test]
    fn async_writereadmsg1() {
        writereadmsg(false, "writereadmsg0", false);
    }

    #[test]
    fn async_conn_writereadmsg0() {
        writereadmsg(true, "conn_writereadmsg0", true);
    }

    #[test]
    fn async_conn_writereadmsg1() {
        writereadmsg(false, "conn_writereadmsg0", true);
    }

    fn atomic(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        let desc = Some(mr.descriptor());
        let descs = [mr.descriptor(), mr.descriptor()];
        // let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Min, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Max, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Sum, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Prod, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Bor, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Band, &mut ctx);

            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            // ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Lor, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Bxor, &mut ctx);

            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            // ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Land, &mut ctx);

            // ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Lxor, &mut ctx);

            ofi.atomic(
                &reg_mem[..512],
                0,
                desc,
                AtomicOp::AtomicWrite,
                &mut ctx,
            );
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            let iocs = [
                Ioc::from_slice(&reg_mem[..256]),
                Ioc::from_slice(&reg_mem[256..512]),
            ];

            ofi.atomicv(&iocs, 0, Some(&descs), AtomicOp::Prod, &mut ctx);
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
            // match err {
            //     Err(e) => {
            //         if matches!(e.kind, libfabric::error::ErrorKind::ErrorAvailable) {
            //             let realerr = ofi.cq_type.tx_cq().readerr(0).unwrap();
            //             panic!("{:?}", realerr.error());
            //         }
            //     }
            //     Ok(_) => {}
            // }

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
        } else {
            let mut expected = vec![2u8; 1024 * 2];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..512], &expected[..512]);
            // Send completion ack
            reg_mem.iter_mut().for_each(|v| *v= 1);

            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            expected = vec![3; 1024 * 2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..512], &expected[..512]);
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            expected = vec![2;1024*2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..512], &expected[..512]);

            expected = vec![4; 1024 * 2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..512], &expected[..512]);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_atomic0() {
    //     atomic(true, "conn_atomic0", true);
    // }

    // #[test]
    // fn async_conn_atomic1() {
    //     atomic(false, "conn_atomic0", true);
    // }

    #[test]
    fn async_atomic0() {
        atomic(true, "atomic0", false);
    }

    #[test]
    fn async_atomic1() {
        atomic(false, "atomic0", false);
    }

    fn fetch_atomic(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc0 = Some(mr.descriptor());
        let desc1 = Some(mr.descriptor());
        // let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let mut expected: Vec<_> = vec![1; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(512);
            let (mem0, mem1) = op_mem.split_at_mut(256);
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Min,
                &mut ctx,
            );

            assert_eq!(mem1, &expected[..256]);

            expected = vec![1; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Max,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Sum,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![4; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Prod,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![8; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bor,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![10; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Band,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);

            expected = vec![2; 256];
            // ofi.fetch_atomic(
            //     &mem0,
            //     mem1,
            //     0,
            //     desc0,
            //     desc1.clone(),
            //     FetchAtomicOp::Lor,
            //     &mut ctx,
            // );

            // assert_eq!(mem1, &expected);

            // expected = vec![1; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bxor,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);

            expected = vec![0; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bor,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Band,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::AtomicWrite,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::AtomicRead,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            let (read_mem, write_mem) = op_mem.split_at_mut(256);
            let iocs = [
                Ioc::from_slice(&read_mem[..128]),
                Ioc::from_slice(&read_mem[128..256]),
            ];
            let write_mems = write_mem.split_at_mut(128);
            let mut res_iocs = [
                IocMut::from_slice(write_mems.0),
                IocMut::from_slice(write_mems.1),
            ];

            let desc0 = Some(mr.descriptor());
            let descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            ofi.fetch_atomicv(
                &iocs,
                &mut res_iocs,
                0,
                Some(&descs),
                Some(&res_descs),
                FetchAtomicOp::Prod,
                &mut ctx,
            );

            assert_eq!(write_mem, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);
        } else {
            let mut expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);

            expected = vec![0; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);

            expected = vec![2; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);

            expected = vec![4; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);
        }
    }

    #[test]
    fn async_fetch_atomic0() {
        fetch_atomic(true, "fetch_atomic0", false);
    }

    #[test]
    fn async_fetch_atomic1() {
        fetch_atomic(false, "fetch_atomic0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_fetch_atomic0() {
    //     fetch_atomic(true, "conn_fetch_atomic0", true);
    // }

    // #[test]
    // fn async_conn_fetch_atomic1() {
    //     fetch_atomic(false, "conn_fetch_atomic0", true);
    // }

    fn compare_atomic(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        let desc = Some(mr.descriptor());
        let comp_desc = Some(mr.descriptor());
        let res_desc = Some(mr.descriptor());
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            let mut expected: Vec<_> = vec![1; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
            let (buf, mem1) = op_mem.split_at_mut(256);
            let (comp, res) = mem1.split_at_mut(256);
            comp.iter_mut().for_each(|v| *v = 1);

            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::Cswap,
                &mut ctx,
            );

            assert_eq!(res, &expected[..256]);

            expected = vec![2; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapNe,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 3);
            expected = vec![2; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapLe,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 2);
            expected = vec![3; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapLt,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 3);
            expected = vec![2; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapGe,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            expected = vec![2; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapGt,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, &mut ctx);

            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc.clone(), &mut ctx);

            // expected = vec![2; 256];
            let (buf0, buf1) = buf.split_at_mut(128);
            let (comp0, comp1) = comp.split_at_mut(128);
            let (res0, res1) = res.split_at_mut(128);

            let buf_iocs = [Ioc::from_slice(&buf0), Ioc::from_slice(&buf1)];
            let comp_iocs = [Ioc::from_slice(&comp0), Ioc::from_slice(&comp1)];
            let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
            let buf_descs = [mr.descriptor(), mr.descriptor()];
            let comp_descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];

            ofi.compare_atomicv(
                &buf_iocs,
                &comp_iocs,
                &mut res_iocs,
                0,
                Some(&buf_descs),
                Some(&comp_descs),
                Some(&res_descs),
                CompareAtomicOp::CswapLe,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc.clone(), &mut ctx);
        } else {
            let mut expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            expected = vec![3; 256];
            // // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    #[test]
    fn async_compare_atomic0() {
        compare_atomic(true, "compare_atomic0", false);
    }

    #[test]
    fn async_compare_atomic1() {
        compare_atomic(false, "compare_atomic0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_compare_atomic0() {
    //     compare_atomic(true, "conn_compare_atomic0", true);
    // }

    // #[test]
    // fn async_conn_compare_atomic1() {
    //     compare_atomic(false, "conn_compare_atomic0", true);
    // }

    fn atomicmsg(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        let desc = Some(mr.descriptor());
        let descs = [mr.descriptor(), mr.descriptor()];
        let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let (dst_slice0, dst_slice1) = remote_mem_info.slice::<u8>(..512).split_at(256);

        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            let iocs = [
                Ioc::from_slice(&reg_mem[..256]),
                Ioc::from_slice(&reg_mem[256..512]),
            ];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);

            let mut msg = if connected {
                Either::Right(MsgAtomicConnected::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &rma_iocs,
                    AtomicOp::Bor,
                    Some(128),
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgAtomic::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    AtomicOp::Bor,
                    Some(128),
                    &mut ctx,
                ))
            };

            ofi.atomicmsg(&mut msg);

            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
        } else {
            let expected = vec![3u8; 1024 * 2];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..512], &expected[..512]);
            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_atomic0() {
    //     atomic(true, "conn_atomic0", true);
    // }

    // #[test]
    // fn async_conn_atomic1() {
    //     atomic(false, "conn_atomic0", true);
    // }

    #[test]
    fn async_atomicmsg0() {
        atomicmsg(true, "atomicmsg0", false);
    }

    #[test]
    fn async_atomicmsg1() {
        atomicmsg(false, "atomicmsg0", false);
    }

    fn fetch_atomicmsg(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let (dst_slice0, dst_slice1) = remote_mem_info.slice::<u8>(..256).split_at(128);
        // let base_addr = remote_mem_info.mem_address();
        // let key = &remote_mem_info.key();
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            let expected = vec![1u8; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(512);

            let (read_mem, write_mem) = op_mem.split_at_mut(256);
            let iocs = [
                Ioc::from_slice(&read_mem[..128]),
                Ioc::from_slice(&read_mem[128..256]),
            ];
            let write_mems = write_mem.split_at_mut(128);
            let mut res_iocs = [
                IocMut::from_slice(write_mems.0),
                IocMut::from_slice(write_mems.1),
            ];

            let desc0 = Some(mr.descriptor());
            let descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);

            let mut msg = if connected {
                Either::Right(MsgFetchAtomicConnected::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &rma_iocs,
                    FetchAtomicOp::Prod,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgFetchAtomic::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    FetchAtomicOp::Prod,
                    None,
                    &mut ctx,
                ))
            };

            ofi.fetch_atomicmsg(&mut msg, &mut res_iocs, Some(&res_descs));

            assert_eq!(write_mem, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);
        } else {
            let desc0 = Some(mr.descriptor());
            let expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);
        }
    }

    #[test]
    fn async_fetch_atomicmsg0() {
        fetch_atomicmsg(true, "fetch_atomicmsg0", false);
    }

    #[test]
    fn async_fetch_atomicmsg1() {
        fetch_atomicmsg(false, "fetch_atomicmsg0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_fetch_atomic0() {
    //     fetch_atomic(true, "conn_fetch_atomic0", true);
    // }

    // #[test]
    // fn async_conn_fetch_atomic1() {
    //     fetch_atomic(false, "conn_fetch_atomic0", true);
    // }

    fn compare_atomicmsg(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = Some(mr.descriptor());
        let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let (dst_slice0, dst_slice1) = remote_mem_info.slice::<u8>(..256).split_at(128);
        // let base_addr = remote_mem_info.mem_address();
        // let key = &remote_mem_info.key();
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            let expected = vec![1u8; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
            let (buf, mem1) = op_mem.split_at_mut(256);
            let (comp, res) = mem1.split_at_mut(256);
            comp.iter_mut().for_each(|v| *v = 1);

            // expected = vec![2; 256];
            let (buf0, buf1) = buf.split_at_mut(128);
            let (comp0, comp1) = comp.split_at_mut(128);
            let (res0, res1) = res.split_at_mut(128);

            let buf_iocs = [Ioc::from_slice(&buf0), Ioc::from_slice(&buf1)];
            let comp_iocs = [Ioc::from_slice(&comp0), Ioc::from_slice(&comp1)];
            let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
            let buf_descs = [mr.descriptor(), mr.descriptor()];
            let comp_descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);

            let mut msg = if connected {
                Either::Right(MsgCompareAtomicConnected::from_ioc_slice(
                    &buf_iocs,
                    Some(&buf_descs),
                    &rma_iocs,
                    CompareAtomicOp::CswapGe,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgCompareAtomic::from_ioc_slice(
                    &buf_iocs,
                    Some(&buf_descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    CompareAtomicOp::CswapGe,
                    None,
                    &mut ctx,
                ))
            };

            ofi.compare_atomicmsg(
                &mut msg,
                &comp_iocs,
                &mut res_iocs,
                Some(&comp_descs),
                Some(&res_descs),
            );

            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc.clone(), &mut ctx);
        } else {
            let expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    #[test]
    fn async_compare_atomicmsg0() {
        compare_atomicmsg(true, "compare_atomicmsg0", false);
    }

    #[test]
    fn async_compare_atomicmsg1() {
        compare_atomicmsg(false, "compare_atomicmsg0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_compare_atomic0() {
    //     compare_atomic(true, "conn_compare_atomic0", true);
    // }

    // #[test]
    // fn async_conn_compare_atomic1() {
    //     compare_atomic(false, "conn_compare_atomic0", true);
    // }

    fn collective(
        server: bool,
        name: &str,
        connected: bool,
    ) -> (Ofi<impl CollCap>, MultiCastGroup) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().collective()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().collective()))
        };

        let reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
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

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);

        let mut avset = if server {
            AddressVectorSetBuilder::new_from_range(
                &ofi.av.as_ref().unwrap(),
                &ofi.mapped_addr.as_ref().unwrap()[0],
                &ofi.mapped_addr.as_ref().unwrap()[0],
                1,
            )
            .count(2)
            .build()
            .unwrap()
        } else {
            AddressVectorSetBuilder::new_from_range(
                &ofi.av.as_ref().unwrap(),
                &ofi.mapped_addr.as_ref().unwrap()[1],
                &ofi.mapped_addr.as_ref().unwrap()[1],
                1,
            )
            .count(2)
            .build()
            .unwrap()
        };

        if server {
            for addr in ofi.mapped_addr.as_ref().unwrap().iter().skip(1) {
                avset.insert(addr).unwrap();
            }
        } else {
            avset.insert(&ofi.mapped_addr.as_ref().unwrap()[0]).unwrap();
        }

        let mut ctx = ofi.info_entry.allocate_context();
        let mc = libfabric::mcast::MulticastGroupBuilder::from_av_set(&avset).build();
        let mc = async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(ep) => mc
                    .join_collective_async(&ep, libfabric::enums::JoinOptions::new(), &mut ctx)
                    .await
                    .unwrap(),
                MyEndpoint::Connectionless(ep) => mc
                    .join_collective_async(&ep, libfabric::enums::JoinOptions::new(), &mut ctx)
                    .await
                    .unwrap(),
            }
        });

        (ofi, mc.1)
    }

    #[test]
    fn collective_0() {
        collective(true, "collective_0", false);
    }

    #[test]
    fn collective_1() {
        collective(false, "collective_0", false);
    }

    fn barrier(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => ep.barrier_async(&mc, &mut ctx).await.unwrap(),
            }
        });
    }

    #[test]
    fn barrier0() {
        barrier(true, "barrier0", false);
    }

    #[test]
    fn barrier1() {
        barrier(false, "barrier0", false);
    }

    fn broadcast(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
        };

        let expected = if server {
            reg_mem.clone()
        } else {
            reg_mem.iter().map(|v| v + 1).collect()
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.broadcast_async(
                        &mut reg_mem[..],
                        Some(&mr.descriptor()),
                        &mc,
                        &ofi.mapped_addr.as_ref().unwrap()[0],
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        assert_eq!(reg_mem, expected);
    }

    #[test]
    fn broadcast0() {
        broadcast(true, "broadcast0", false);
    }

    #[test]
    fn broadcast1() {
        broadcast(false, "broadcast0", false);
    }

    fn alltoall(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1024 * 2], vec![1; 1024 * 2])
        } else {
            (vec![1; 1024 * 2], vec![2; 1024 * 2])
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        let half = reg_mem.len() / 2;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(half);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.alltoall_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        recv_buf,
                        Some(&mr.descriptor()),
                        &mc,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        assert_eq!(reg_mem, expected);
    }

    #[test]
    fn alltoall0() {
        alltoall(true, "alltoall0", false);
    }

    #[test]
    fn alltoall1() {
        alltoall(false, "alltoall0", false);
    }

    fn allreduce(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1024 * 2], vec![3; 1024 * 1])
        } else {
            (vec![1; 1024 * 2], vec![3; 1024 * 1])
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        let half = reg_mem.len() / 2;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(half);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.allreduce_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        recv_buf,
                        Some(&mr.descriptor()),
                        &mc,
                        libfabric::enums::CollAtomicOp::Sum,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        assert_eq!(recv_buf, expected);
    }

    #[test]
    fn allreduce0() {
        allreduce(true, "allreduce0", false);
    }

    #[test]
    fn allreduce1() {
        allreduce(false, "allreduce0", false);
    }

    fn allgather(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1536], [vec![2; 512], vec![1; 512]].concat())
        } else {
            (vec![1; 1536], [vec![2; 512], vec![1; 512]].concat())
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(512);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.allgather_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        assert_eq!(recv_buf, expected);
    }

    #[test]
    fn allgather0() {
        allgather(true, "allgather0", false);
    }

    #[test]
    fn allgather1() {
        allgather(false, "allgather0", false);
    }

    fn reduce_scatter(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1024 * 2], vec![3; 1024])
        } else {
            (vec![1; 1024 * 2], vec![3; 1024])
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.reduce_scatter_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        libfabric::enums::CollAtomicOp::Sum,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        assert_eq!(recv_buf[..1024], expected);
    }

    #[test]
    fn reduce_scatter0() {
        reduce_scatter(true, "reduce_scatter0", false);
    }

    #[test]
    fn reduce_scatter1() {
        reduce_scatter(false, "reduce_scatter0", false);
    }

    fn reduce(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1024 * 2], vec![3; 1024])
        } else {
            (vec![1; 1024 * 2], vec![1; 1024])
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.reduce_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        &ofi.mapped_addr.as_ref().unwrap()[0],
                        libfabric::enums::CollAtomicOp::Sum,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        assert_eq!(recv_buf[..1024], expected);
    }

    #[test]
    fn reduce0() {
        reduce(true, "reduce0", false);
    }

    #[test]
    fn reduce1() {
        reduce(false, "reduce0", false);
    }

    fn scatter(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1024 * 2], vec![2; 512])
        } else {
            (vec![1; 1024 * 2], vec![2; 512])
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.scatter_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..512],
                        Some(&mr.descriptor()),
                        &mc,
                        &ofi.mapped_addr.as_ref().unwrap()[0],
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        assert_eq!(recv_buf[..512], expected);
    }

    #[test]
    fn scatter0() {
        scatter(true, "scatter0", false);
    }

    #[test]
    fn scatter1() {
        scatter(false, "scatter0", false);
    }

    fn gather(server: bool, name: &str, connected: bool) {
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1024 * 2], [vec![2; 512], vec![1; 512]].concat())
        } else {
            (vec![1; 1024 * 2], vec![1; 512])
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };
        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(512);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.gather_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        &ofi.mapped_addr.as_ref().unwrap()[0],
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        assert_eq!(recv_buf[..1024], expected);
    }

    #[test]
    fn gather0() {
        gather(true, "gather0", false);
    }

    #[test]
    fn gather1() {
        gather(false, "gather0", false);
    }
}
