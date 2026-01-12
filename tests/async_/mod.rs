use std::cell::RefCell;
use std::ops::Range;
use std::time::Instant;

use libfabric::async_::av::{AddressVector, AddressVectorBuilder};
use libfabric::async_::comm::atomic::{AsyncAtomicCASEp, AsyncAtomicCASRemoteMemAddrSliceEp, AsyncAtomicFetchEp, AsyncAtomicFetchRemoteMemAddrSliceEp, AsyncAtomicWriteEp, AsyncAtomicWriteRemoteMemAddrSliceEp, ConnectedAsyncAtomicCASEp, ConnectedAsyncAtomicCASRemoteMemAddrSliceEp, ConnectedAsyncAtomicFetchEp, ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp, ConnectedAsyncAtomicWriteEp, ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp};
use libfabric::async_::comm::message::{AsyncRecvEp, AsyncSendEp, ConnectedAsyncRecvEp, ConnectedAsyncSendEp};
use libfabric::async_::comm::rma::{AsyncReadEp, AsyncReadRemoteMemAddrSliceEp, AsyncWriteEp, AsyncWriteRemoteMemAddrSliceEp, ConnectedAsyncReadEp, ConnectedAsyncReadRemoteMemAddrSliceEp, ConnectedAsyncWriteEp, ConnectedAsyncWriteRemoteMemAddrSliceEp};
use libfabric::async_::comm::tagged::{AsyncTagRecvEp, AsyncTagSendEp, ConnectedAsyncTagRecvEp, ConnectedAsyncTagSendEp as _};
use libfabric::async_::conn_ep::ConnectedEndpoint;
use libfabric::async_::connless_ep::ConnectionlessEndpoint;
use libfabric::async_::domain::Domain;
use libfabric::async_::ep::{Endpoint, EndpointBuilder};
use libfabric::async_::eq::{EventQueue, EventQueueBuilder};
use libfabric::cq::SingleCompletion;
use libfabric::enums::{AtomicMsgOptions, AtomicOp, CompareAtomicOp, CqFormat, EndpointType, FetchAtomicOp, ReadMsgOptions, TferOptions, WriteMsgOptions};
use libfabric::domain::DomainBuilder;
use libfabric::error::Error;
use libfabric::infocapsoptions::{AtomicDefaultCap, Caps, MsgDefaultCap, RmaDefaultCap, TagDefaultCap};
use libfabric::iovec::{IoVec, IoVecMut};
use libfabric::mr::{EpBindingMemoryRegion, MemoryRegionBuilder, MemoryRegionDesc};
use libfabric::enums::AVOptions;
use libfabric::ep::{Address, BaseEndpoint};
use libfabric::fabric::FabricBuilder;
use libfabric::mr::MemoryRegion;
use libfabric::info::{Info, InfoBuilder, InfoEntry};
use libfabric::async_::cq::{CompletionQueue, CompletionQueueBuilder};
use libfabric::msg::{Msg, MsgAtomic, MsgAtomicConnected, MsgCompareAtomic, MsgCompareAtomicConnected, MsgConnected, MsgConnectedMut, MsgFetchAtomic, MsgFetchAtomicConnected, MsgMut, MsgRma, MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut, MsgTagged, MsgTaggedConnected, MsgTaggedConnectedMut, MsgTaggedMut};
use libfabric::{AsFiType, Context, EqCaps, MappedAddress, MemAddressInfo, MyRc, RemoteMemAddrSlice, RemoteMemAddrSliceMut, RemoteMemAddressInfo};


pub type SpinCq = libfabric::async_cq_caps_type!(CqCaps::WAIT);
pub type WaitableEq = libfabric::eq_caps_type!(EqCaps::FD);
pub type EqOptions = libfabric::async_eq_caps_type!(EqCaps::WAIT);

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
    // pub mr: RefCell<Option<MemoryRegion>>,
    pub mr: RefCell<Option<MemoryRegion>>,
    pub remote_mem_info: Option<RefCell<RemoteMemAddressInfo>>,
    // pub remote_mem_addr: Option<(u64, u64)>,
    pub domain: Domain,
    pub cq_type: CqType,
    pub ep: MyEndpoint<I>,
    pub reg_mem: RefCell<Vec<u8>>,
    pub mapped_addr: Option<Vec<MyRc<MappedAddress>>>,
    pub av: Option<AddressVector>,
    pub eq: EventQueue<EqOptions>,
    pub ctx: RefCell<Context>,
    pub server: bool,
    // pub ctx: RefCell<libfabric::Context>,
    // pub use_shared_cqs: bool,
    // pub server: bool,
    // pub tx_pending_cnt: AtomicUsize,
    // pub tx_complete_cnt: AtomicUsize,
    // pub rx_pending_cnt: AtomicUsize,
    // pub rx_complete_cnt: AtomicUsize,
}

// pub struct Ofi<I> {
//     pub info_entry: InfoEntry<I>,
//     pub mr: RefCell<Option<MemoryRegion>>,
//     // pub remote_key: Option<MappedMemoryRegionKey>,
//     // pub remote_mem_addr: Option<(u64, u64)>,
//     pub remote_mem_info: Option<RefCell<RemoteMemAddressInfo>>,
//     pub domain: Domain,
//     pub cq_type: CqType,
//     pub ep: MyEndpoint<I>,
//     pub tx_context: MyTxContext<I>,
//     pub rx_context: MyRxContext<I>,
//     pub reg_mem: RefCell<Vec<u8>>,
//     pub mapped_addr: Option<Vec<MyRc<MappedAddress>>>,
//     pub av: Option<NoBlockAddressVector>,
//     pub eq: EventQueue<EqOptions>,
//     pub ctx: RefCell<libfabric::Context>,
//     pub use_shared_cqs: bool,
//     pub use_cntrs_for_completion: CntrsCompMeth,
//     pub use_cqs_for_completion: CqsCompMeth,
//     pub server: bool,
//     pub tx_cntr: Option<Counter<DefaultCntr>>,
//     pub rx_cntr: Option<Counter<DefaultCntr>>,
//     pub tx_pending_cnt: RefCell<usize>,
//     pub tx_complete_cnt: RefCell<usize>,
//     pub rx_pending_cnt: RefCell<usize>,
//     pub rx_complete_cnt: RefCell<usize>,
// }

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
        config: TestConfig<I>,
        // info_entry: InfoEntry<I>,
        // shared_cqs: bool,
        // server: bool,
        // name: &str,
    ) -> Result<Self, Error> {
        if config.server && !config.name.is_empty(){
            unsafe { std::env::set_var(&config.name, "1") };
        } else if !config.server && !config.name.is_empty() {
            while std::env::var(&config.name).is_err() {
                std::thread::yield_now();
            }
        }
        let info_entry = config.info_entry;

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
        let mut reg_mem = vec![0u8; config.buf_size];

        let (info_entry, ep, mapped_addr, av, eq) = {
            let eq = EventQueueBuilder::new(&fabric).build().unwrap();
            let info_entry = if matches!(ep_type, EndpointType::Msg) {
                if config.server {
                    let pep = EndpointBuilder::new(&info_entry)
                        .build_passive(&fabric)
                        .unwrap();
                    pep.bind(&eq, 0).unwrap();
                    async_std::task::block_on(async {
                        pep.listen_async().unwrap().next().await
                    })
                    .unwrap()
                    .info()
                    .unwrap()
                } else {
                    info_entry
                }
            } else {
                info_entry
            };

            domain = DomainBuilder::new(&fabric, &info_entry).build().unwrap();

            let ep_builder = EndpointBuilder::new(&info_entry);
            cq_type = if config.use_shared_cqs {
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
                        .access_remote_read()
                        .access_remote_write()
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
                        let mapped_addresses: Vec<MyRc<MappedAddress>> =
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
                            .map(|x| MyRc::new(x))
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
                        let mapped_addresses: Vec<MyRc<MappedAddress>> =
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
                            .map(|x| MyRc::new(x))
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
                                println!("Connecting {}", config.server);
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
                        .access_remote_read()
                        .access_remote_write()
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

        if config.server && !config.name.is_empty() {
            unsafe { std::env::remove_var(&config.name) };
        }
        
        let ctx = info_entry.allocate_context();
        Ok(Self {
            info_entry,
            mapped_addr,
            mr: RefCell::new(mr),
            remote_mem_info: None,
            cq_type,
            domain,
            ep,
            reg_mem: RefCell::new(reg_mem),
            av,
            eq,
            ctx: RefCell::new(ctx),
            server: config.server,
             // tx_pending_cnt,
                // tx_complete_cnt,
                // rx_pending_cnt,
                // rx_complete_cnt,
        })
    }

    pub fn exchange_keys(&mut self) {
        let mr = self.mr.borrow();
        let key = mr.as_ref().unwrap().key().unwrap();
        let mem_info = libfabric::MemAddressInfo::from_slice(&self.reg_mem.borrow(), 0, &key, &self.info_entry);
        let mem_bytes = mem_info.to_bytes();


        println!("Local addr: {:?}, size: {}", self.reg_mem.borrow().as_ptr(), self.reg_mem.borrow().len());


        self.reg_mem.borrow_mut()[..mem_bytes.len()].copy_from_slice(mem_bytes);

        self.send(
            0..mem_bytes.len(),
            None,
        );

        self.recv(
            mem_bytes.len()..2*mem_bytes.len(),
        );

        // self.wait_rx(1);

        let mem_info = unsafe { MemAddressInfo::from_bytes(&self.reg_mem.borrow()[mem_bytes.len()..2*mem_bytes.len()]) };
        let remote_mem_info = mem_info.into_remote_info(&self.domain).unwrap();
        println!("Remote addr: {:?}, size: {}", remote_mem_info.mem_address().as_ptr(), remote_mem_info.mem_len());

        self.remote_mem_info = Some(RefCell::new(remote_mem_info));
    }
}


impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn send(
        &self,
        range: Range<usize>,
        data: Option<u64>,
    ) {
        let borrow = &self.reg_mem.borrow();
        let buf = &borrow[range];
        let mr = self.mr.borrow();
        let desc = mr.as_ref().map_or(None, |mr| Some(mr.descriptor()));
        let ctx = &mut self.ctx.borrow_mut();

        async_std::task::block_on(async {
            let _err = match &self.ep {
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

    pub fn sendv(&self, iov: &[IoVec], desc: Option<&[MemoryRegionDesc]>) {
        let ctx = &mut self.ctx.borrow_mut();

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
    ) {
        let ctx = &mut self.ctx.borrow_mut();

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

    pub fn recv(
        &self,
        range: Range<usize>,
    ) {
        let borrow = &mut self.reg_mem.borrow_mut();
        let buf = &mut borrow[range];
        let mr = self.mr.borrow();
        let desc = mr.as_ref().map_or(None, |mr| Some(mr.descriptor()));
        let ctx = &mut self.ctx.borrow_mut();

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
}


impl<I: TagDefaultCap> Ofi<I> {
    pub fn tsend(
        &self,
        range: Range<usize>,
        tag: u64,
        data: Option<u64>,
    ) {
        let buf = &self.reg_mem.borrow()[range];
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));
        let ctx = &mut self.ctx.borrow_mut();

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
    ) {
        let ctx = &mut self.ctx.borrow_mut();
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
    ) {
        let ctx = &mut self.ctx.borrow_mut();

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

    pub fn trecv(
        &self,
        range: Range<usize>,
        tag: u64,
    ) {
        let buf = &mut self.reg_mem.borrow_mut()[range];
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));
        let ctx = &mut self.ctx.borrow_mut();

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


impl<I: MsgDefaultCap + RmaDefaultCap> Ofi<I> {
    pub fn write(
        &self,
        range: Range<usize>,
        dest_addr: usize,
        data: Option<u64>,
    ) {
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let buf = &self.reg_mem.borrow()[range];
        let dest_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());
        let ctx = &mut self.ctx.borrow_mut();
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));

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

    pub fn read(
        &self,
        range: Range<usize>,
        dest_addr: usize,
    ) {
        let buf = &mut self.reg_mem.borrow_mut()[range];
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
        let src_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());
        let ctx = &mut self.ctx.borrow_mut();
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));

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
    ) {
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let dst_slice = remote_mem_info
            .slice_mut::<u8>(dest_addr..dest_addr + iov.iter().fold(0, |acc, x| acc + x.len()));
        let ctx = &mut self.ctx.borrow_mut();

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
    ) {
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
        let src_slice = remote_mem_info
            .slice::<u8>(dest_addr..dest_addr + iov.iter().fold(0, |acc, x| acc + x.len()));
        let ctx = &mut self.ctx.borrow_mut();

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
    pub fn atomic(
        &self,
        range: Range<usize>,
        dest_addr: usize,
        op: AtomicOp,
    ) {
        let buf = &self.reg_mem.borrow()[range];
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let dst_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));

        let ctx = &mut self.ctx.borrow_mut();
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
    ) {
        let ctx = &mut self.ctx.borrow_mut();
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

    pub fn fetch_atomic(
        &self,
        src_range: Range<usize>,
        res_range: Range<usize>,
        dest_addr: usize,
        desc: Option<MemoryRegionDesc>,
        res_desc: Option<MemoryRegionDesc>,
        op: FetchAtomicOp,
    ) {
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let mut borrow = self.reg_mem.borrow_mut();
        let (split_0, split_1) = borrow.split_at_mut(src_range.end);
        let buf = &split_0[src_range.clone()];
        let res = &mut split_1[res_range.start - src_range.end..res_range.end - src_range.end];
        let src_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());
        let ctx = &mut self.ctx.borrow_mut();

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
    ) {
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
        let src_slice: RemoteMemAddrSlice<'_, T> = remote_mem_info
            .slice(dest_addr..dest_addr + ioc.iter().fold(0, |acc, x| acc + x.len()));
        let ctx = &mut self.ctx.borrow_mut();

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
    ) {
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
        let src_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());
        let ctx = &mut self.ctx.borrow_mut();
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
    ) {
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let dst_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());
        let ctx = &mut self.ctx.borrow_mut();
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
    ) {
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let dst_slice = remote_mem_info
            .slice_mut(dest_addr..dest_addr + ioc.iter().fold(0, |acc, x| acc + x.len()));
        let ctx = &mut self.ctx.borrow_mut();
      
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

pub fn handshake<I: Caps + MsgDefaultCap + 'static>(
    user_ip: Option<&str>,
    server: bool,
    name: &str,
    caps: Option<I>,
) -> Ofi<I> {
    let caps = caps.unwrap();
    let ep_type: EndpointType = EndpointType::Msg;
    let hostname = std::process::Command::new("hostname")
        .output()
        .expect("Failed to execute hostname")
        .stdout;
    let hostname = String::from_utf8(hostname[2..].to_vec()).unwrap();
    let mut ip = "172.17.110.".to_string() + &hostname;
    ip = ip.strip_suffix("\n").unwrap_or(&ip).to_owned();

    if let Some(user_ip) = user_ip {
        ip = user_ip.to_string();
    }
    
    let mut configbuilder = TestConfigBuilder::new(Some(&ip), None, server, caps, ep_type);
    configbuilder.name = name.to_string();

    let config = configbuilder.build(|_| true);


    let info = Ofi::new(config).unwrap();
    info
}


pub fn handshake_connectionless<I: MsgDefaultCap + Caps + 'static>(
    user_ip: Option<&str>,
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
    let mut ip = "172.17.110.".to_string() + &hostname;
    ip = ip.strip_suffix("\n").unwrap_or(&ip).to_owned();

    if let Some(user_ip) = user_ip {
        ip = user_ip.to_string();
    }

    let mut configbuilder = TestConfigBuilder::new(Some(&ip), None, server, caps, ep_type);
    configbuilder.name = name.to_string();

    let config = configbuilder.build(|_| true);


    let info = Ofi::new(config).unwrap();
    info
}

pub fn enable_ep_mr<E: 'static>(ep: &MyEndpoint<E>, mr: EpBindingMemoryRegion) -> MemoryRegion {
    match ep {
        MyEndpoint::Connected(ep) => mr.enable(ep).unwrap(),
        MyEndpoint::Connectionless(ep) => mr.enable(ep).unwrap(),
    }
}



impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn pingpong(&self, warmup: usize, iters: usize, size: usize) {
        self.sync().unwrap();
        let mut now = Instant::now();
        if !self.server {
            for i in 0..warmup + iters {
                if i == warmup {
                    now = Instant::now(); // Start timer
                }

                self.send(0..size, None);
                self.recv(0..size);
            }
        } else {
            for i in 0..warmup + iters {
                if i == warmup {
                    now = Instant::now(); // Start timer
                }

                self.recv(0..size);
                self.send(0..size, None);
            }
        }

        let elapsed = now.elapsed();

        if size == 1 {
            println!("bytes iters total time MB/sec usec/xfer Mxfers/sec",);
        }

        let bytes = iters * size * 2;
        let usec_per_xfer = elapsed.as_micros() as f64 / iters as f64 / 2_f64;
        println!(
            "{} {} {} {} s {} {} {}",
            size,
            iters,
            bytes,
            elapsed.as_secs(),
            bytes as f64 / elapsed.as_micros() as f64,
            usec_per_xfer,
            1.0 / usec_per_xfer
        );
    }
}

impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn sync(&self) -> Result<(), Error> {
        if self.server {

            self.send(0..1, None);
            self.recv(0..1);
        }
        else {
            self.recv(0..1);
            self.send(0..1, None);
        }
        Ok(())
    }
}

impl<I: MsgDefaultCap + TagDefaultCap + 'static> Ofi<I> {
    pub fn pingpong_tagged(&self, warmup: usize, iters: usize, size: usize) {
        self.sync().unwrap();
        let mut now = Instant::now();
        if !self.server {
            for i in 0..warmup + iters {
                if i == warmup {
                    now = Instant::now(); // Start timer
                }

                self.tsend(0..size, 0, None);
                if size > self.info_entry.tx_attr().inject_size() {

                }
                self.trecv(0..size, 0);
            }
        } else {
            for i in 0..warmup + iters {
                if i == warmup {
                    now = Instant::now(); // Start timer
                }

                self.trecv(0..size, 0);
                self.tsend(0..size, 0, None);

            }
        }

        let elapsed = now.elapsed();

        if size == 1 {
            println!("bytes iters total time MB/sec usec/xfer Mxfers/sec",);
        }

        let bytes = iters * size * 2;
        let usec_per_xfer = elapsed.as_micros() as f64 / iters as f64 / 2_f64;
        println!(
            "{} {} {} {} s {} {} {}",
            size,
            iters,
            bytes,
            elapsed.as_secs(),
            bytes as f64 / elapsed.as_micros() as f64,
            usec_per_xfer,
            1.0 / usec_per_xfer
        );
    }
}

impl<I: MsgDefaultCap + RmaDefaultCap + 'static> Ofi<I> {
    pub fn pingpong_rma(&self, warmup: usize, iters: usize, size: usize, _window_size: usize) {
        self.sync().unwrap();
        let mut now = Instant::now();

        for i in 0..warmup + iters {
            if i == warmup {
                now = Instant::now(); // Start timer
            }

            self.write(0..size, 0, None);
                        
            // j += 1;
            // if j == window_size  {

                // j = 0;
            // }
        }
        // if size > self.info_entry.tx_attr().inject_size() {
        //     self.cq_type.tx_cq().sread(1, -1).unwrap();
        // }
        
        let elapsed = now.elapsed();

        if size == 1 {
            println!("bytes iters total time MB/sec usec/xfer Mxfers/sec",);
        }

        let bytes = iters * size * 2;
        let usec_per_xfer = elapsed.as_micros() as f64 / iters as f64 / 2_f64;
        println!(
            "{} {} {} {} s {} {} {}",
            size,
            iters,
            bytes,
            elapsed.as_secs(),
            bytes as f64 / elapsed.as_micros() as f64,
            usec_per_xfer,
            1.0 / usec_per_xfer
        );
    }
}



pub struct TestConfigBuilder<I> {
    info_builder: InfoBuilder<I>,
    pub use_shared_cqs: bool,
    pub buf_size: usize,
    pub name: String,
    server: bool,
}

pub struct TestConfig<I> {
    info_entry: InfoEntry<I>,
    use_shared_cqs: bool,
    buf_size: usize,
    server: bool,
    name: String,
}

impl<I: Caps> TestConfigBuilder<I> {
    pub fn new(node: Option<&str>, service: Option<&str>, server: bool, caps: I, eptype: EndpointType) -> Self {
        let info_builder = Info::new(&libfabric::info::libfabric_version())
                    .enter_hints()
                    .enter_ep_attr()
                        .type_(eptype)
                    // .tx_ctx_cnt(1)
                    // .rx_ctx_cnt(1)
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
                    // .op_flags(libfabric::enums::TransferOptions::new().delivery_complete())
                    .leave_tx_attr()
                    .enter_rx_attr()
                    // .caps(RxCaps::new().recv().collective())
                    .leave_rx_attr()
                    .addr_format(libfabric::enums::AddressFormat::Unspec)
                    .caps(caps)
                    .leave_hints();
        
        let info_builder = if server {
            info_builder.source(libfabric::info::ServiceAddress::Service(service.unwrap_or("9222").to_owned()))
        }
        else {
            info_builder
                .node(node.expect("Error: No server IP specified"))
                .service(service.unwrap_or("9222"))
        };

        Self {
            info_builder,
            use_shared_cqs: false,
            buf_size: 1024* 1024 * 2,
            server,
            name: "".to_string(),
        }
    }

    pub fn modify_info<F>(self, mut closure: F)  -> Self where F: FnMut(InfoBuilder<I>) -> InfoBuilder<I> {
        let new_info = closure(self.info_builder);
        
        Self {
            info_builder: new_info,
            ..self
        }
    }


    pub fn build<F>(self, filter: F) -> TestConfig<I> where F: Fn(&InfoEntry<I>) -> bool {
        let info = self.info_builder.get().unwrap();
        let info_entry = info.into_iter().find(filter).unwrap();
        
        TestConfig {
            info_entry,
            use_shared_cqs:  self.use_shared_cqs,
            buf_size: self.buf_size,
            server: self.server,
            name: self.name,
        }
    }
}

