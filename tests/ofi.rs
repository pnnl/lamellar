use libfabric::av::NoBlockAddressVector;
use libfabric::av_set::AddressVectorSetBuilder;
use libfabric::comm::atomic::AtomicCASRemoteMemAddrSliceEp;
use libfabric::comm::atomic::AtomicFetchRemoteMemAddrSliceEp;
use libfabric::comm::atomic::AtomicWriteRemoteMemAddrSliceEp;
use libfabric::comm::atomic::ConnectedAtomicCASRemoteMemAddrSliceEp;
use libfabric::comm::atomic::ConnectedAtomicFetchRemoteMemAddrSliceEp;
use libfabric::comm::atomic::ConnectedAtomicWriteRemoteMemAddrSliceEp;
use libfabric::comm::collective::CollectiveEp;
use libfabric::comm::message::ConnectedRecvEpMrSlice;
use libfabric::comm::message::ConnectedSendEpMrSlice;
use libfabric::comm::message::RecvEpMrSlice;
use libfabric::comm::message::SendEpMrSlice;
use libfabric::comm::rma::ConnectedReadRemoteMemAddrSliceEp;
use libfabric::comm::rma::ConnectedWriteRemoteMemAddrSliceEp;
use libfabric::comm::rma::ReadRemoteMemAddrSliceEp;
use libfabric::comm::rma::WriteRemoteMemAddrSliceEp;
use libfabric::comm::tagged::ConnectedTagRecvEpMrSlice;
use libfabric::comm::tagged::ConnectedTagSendEpMrSlice;
use libfabric::comm::tagged::TagRecvEpMrSlice;
use libfabric::comm::tagged::TagSendEpMrSlice;
use libfabric::enums::CollectiveOptions;
use libfabric::eq::Event;
use libfabric::eq::EventQueue;
use libfabric::iovec::RemoteMemAddrAtomicVec;
use libfabric::iovec::RemoteMemAddrVec;
use libfabric::iovec::RemoteMemAddrVecMut;
use libfabric::mr::MemoryRegionSlice;
use libfabric::mr::MemoryRegionSliceMut;
use libfabric::mcast::MultiCastGroup;
use std::cell::RefCell;
pub type EqOptions = libfabric::eq_caps_type!(EqCaps::WAIT);
use libfabric::{
    av::AddressVectorBuilder,
    comm::{
        atomic::{
            AtomicCASEp, AtomicFetchEp, AtomicWriteEp, ConnectedAtomicCASEp,
            ConnectedAtomicFetchEp, ConnectedAtomicWriteEp,
        },
        message::{ConnectedRecvEp, ConnectedSendEp, RecvEp, SendEp},
        rma::{ConnectedReadEp, ConnectedWriteEp, ReadEp, WriteEp},
        tagged::{ConnectedTagRecvEp, ConnectedTagSendEp, TagRecvEp, TagSendEp},
    },
    conn_ep::ConnectedEndpoint,
    connless_ep::ConnectionlessEndpoint,
    cq::{Completion, CompletionQueue, CompletionQueueBuilder, ReadCq, WaitCq},
    domain::{Domain, DomainBuilder},
    enums::{
        AVOptions, AtomicMsgOptions, AtomicOp, CompareAtomicOp, CqFormat, EndpointType,
        FetchAtomicOp, ReadMsgOptions, RecvMsgOptions, SendMsgOptions, TaggedRecvMsgOptions,
        TaggedSendMsgOptions, TferOptions, WriteMsgOptions,
    },
    ep::{Address, BaseEndpoint, Endpoint, EndpointBuilder},
    eq::{EventQueueBuilder, WaitEq},
    error::{Error, ErrorKind},
    fabric::FabricBuilder,
    info::{Info, InfoEntry},
    infocapsoptions::{
        AtomicDefaultCap, Caps, CollCap, InfoCaps, MsgDefaultCap, RmaDefaultCap, TagDefaultCap,
    },
    iovec::{IoVec, IoVecMut, Ioc, IocMut},
    mr::{
        EpBindingMemoryRegion, MemoryRegion, MemoryRegionBuilder, MemoryRegionDesc, MemoryRegionKey,
    },
    msg::{
        Msg, MsgAtomic, MsgAtomicConnected, MsgCompareAtomic, MsgCompareAtomicConnected,
        MsgConnected, MsgConnectedMut, MsgFetchAtomic, MsgFetchAtomicConnected, MsgMut, MsgRma,
        MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut, MsgTagged, MsgTaggedConnected,
        MsgTaggedConnectedMut, MsgTaggedMut,
    },
    xcontext::{
        ConnectedRxContext, ConnectedTxContext, ConnlessRxContext, ConnlessTxContext,
        RxContextBuilder, TxContextBuilder,
    },
    Context, CqCaps, EqCaps, MappedAddress, MemAddressInfo, MyRc, RemoteMemAddressInfo,
};
pub type SpinCq = libfabric::cq_caps_type!(CqCaps::WAIT);
pub type WaitableEq = libfabric::eq_caps_type!(EqCaps::WAIT);
pub mod common;

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

pub enum MyTxContext<I> {
    Connected(Result<ConnectedTxContext<I>, libfabric::error::Error>),
    Connectionless(Result<ConnlessTxContext<I>, libfabric::error::Error>),
}

pub enum MyRxContext<I> {
    Connected(Result<ConnectedRxContext<I>, libfabric::error::Error>),
    Connectionless(Result<ConnlessRxContext<I>, libfabric::error::Error>),
}

pub struct Ofi<I> {
    pub info_entry: InfoEntry<I>,
    pub mr: Option<MemoryRegion>,
    // pub remote_key: Option<MappedMemoryRegionKey>,
    // pub remote_mem_addr: Option<(u64, u64)>,
    pub remote_mem_info: Option<RefCell<RemoteMemAddressInfo>>,
    pub domain: Domain,
    pub cq_type: CqType,
    pub ep: MyEndpoint<I>,
    pub tx_context: MyTxContext<I>,
    pub rx_context: MyRxContext<I>,
    pub reg_mem: Vec<u8>,
    pub mapped_addr: Option<Vec<MyRc<MappedAddress>>>,
    pub av: Option<NoBlockAddressVector>,
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

macro_rules!  post{
    ($post_fn:ident, $prog_fn:ident, $cq:expr, $ep:ident, $( $x:expr),* ) => {
        loop {
            let ret = $ep.$post_fn($($x,)*);
            if ret.is_ok() {
                break;
            }
            else if let Err(ref err) = ret {
                if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
                    panic!("Unexpected error!")
                }

            }
            $prog_fn($cq);
        }
    };
}

pub fn ft_progress(cq: &impl ReadCq) {
    let ret = cq.read(0);
    match ret {
        Ok(_) => {
            panic!("Should not read anything")
        }
        Err(ref err) => {
            if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
                ret.unwrap();
            }
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

        let (info_entry, ep, tx_context, rx_context, mapped_addr, av, eq) = {
            let (info_entry, eq) = if matches!(ep_type, EndpointType::Msg) {
                let eq = EventQueueBuilder::new(&fabric).build().unwrap();
                if server {
                    let pep = EndpointBuilder::new(&info_entry)
                        .build_passive(&fabric)
                        .unwrap();
                    pep.bind(&eq, 0).unwrap();
                    pep.listen().unwrap();
                    let event = eq.sread(-1).unwrap();
                    match event {
                        libfabric::eq::Event::ConnReq(entry) => (entry.info().unwrap(), Some(eq)),
                        _ => panic!("Unexpected event"),
                    }
                } else {
                    (info_entry, Some(eq))
                }
            } else {
                (info_entry, None)
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

            let ep = match cq_type {
                CqType::Separate((ref tx_cq, ref rx_cq)) => ep_builder
                    .build_with_separate_cqs(&domain, tx_cq, false, rx_cq, false)
                    .unwrap(),

                CqType::Shared(ref scq) => ep_builder
                    .build_with_shared_cq(&domain, scq, false)
                    .unwrap(),
            };

            match ep {
                Endpoint::Connectionless(ep) => {
                    let eq = EventQueueBuilder::new(&fabric).build()?;
                    let av = match info_entry.domain_attr().av_type() {
                        libfabric::enums::AddressVectorType::Unspec => AddressVectorBuilder::new()
                            .no_block(&eq)
                            .build(&domain)
                            .unwrap(),
                        _ => AddressVectorBuilder::new()
                            .type_(*info_entry.domain_attr().av_type())
                            .no_block(&eq)
                            .build(&domain)
                            .unwrap(),
                    };
                    ep.bind_eq(&eq)?;
                    let ep = ep.enable(&av).unwrap();
                    let tx_context = TxContextBuilder::new(&ep, 0).build();

                    let rx_context = RxContextBuilder::new(&ep, 0).build();

                    mr = if info_entry.domain_attr().mr_mode().is_local()
                        || info_entry.caps().is_rma()
                    {
                        let mr = MemoryRegionBuilder::new(
                            &reg_mem,
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
                        let mapped_addresses: Vec<std::rc::Rc<MappedAddress>> = {
                            let pending = av
                                .insert_no_block(all_addresses.as_ref().into(), AVOptions::new())
                                .unwrap();

                            let event = eq.sread(-1).unwrap();
                            if let Event::AVComplete(av_complete) = event {
                                pending
                                    .av_complete(av_complete)
                                    .into_iter()
                                    .map(std::rc::Rc::new)
                                    .collect()
                            } else {
                                panic!("Unexpected event retrieved");
                            }
                        };
                        // .pop()
                        // .unwrap()
                        // .unwrap();
                        let epname = ep.getname().unwrap();
                        let epname_bytes = epname.as_bytes();
                        let addrlen = epname_bytes.len();
                        reg_mem[..addrlen].copy_from_slice(epname_bytes);

                        post!(
                            send_to,
                            ft_progress,
                            cq_type.tx_cq(),
                            ep,
                            &reg_mem[..addrlen],
                            None,
                            &mapped_addresses[1]
                        );
                        cq_type.tx_cq().sread(1, -1).unwrap();

                        // ep.recv(std::slice::from_mut(&mut ack), &mut default_desc()).unwrap();
                        post!(
                            recv_from_any,
                            ft_progress,
                            cq_type.rx_cq(),
                            ep,
                            std::slice::from_mut(&mut reg_mem[0]),
                            None
                        );
                        cq_type.rx_cq().sread(1, -1).unwrap();
                        mapped_addresses
                    } else {
                        let epname = ep.getname().unwrap();
                        let addrlen = epname.as_bytes().len();

                        let mr_desc = mr.as_ref().map(|mr| mr.descriptor());

                        post!(
                            recv_from_any,
                            ft_progress,
                            cq_type.rx_cq(),
                            ep,
                            &mut reg_mem[..addrlen],
                            mr_desc
                        );
                        cq_type.rx_cq().sread(1, -1).unwrap();
                        // ep.recv(&mut reg_mem, &mut mr_desc).unwrap();
                        let remote_address = unsafe { Address::from_bytes(&reg_mem) };
                        let all_addresses = [epname, remote_address];
                        let mapped_addresses: Vec<std::rc::Rc<MappedAddress>> = {
                            let pending = av
                                .insert_no_block(all_addresses.as_ref().into(), AVOptions::new())
                                .unwrap();

                            let event = eq.sread(-1).unwrap();
                            if let Event::AVComplete(av_complete) = event {
                                pending
                                    .av_complete(av_complete)
                                    .into_iter()
                                    .map(std::rc::Rc::new)
                                    .collect()
                            } else {
                                panic!("Unexpected event retrieved");
                            }
                        };

                        post!(
                            send_to,
                            ft_progress,
                            cq_type.tx_cq(),
                            ep,
                            &std::slice::from_ref(&reg_mem[0]),
                            mr_desc,
                            &mapped_addresses[1]
                        );
                        cq_type.tx_cq().sread(1, -1).unwrap();

                        mapped_addresses
                    };
                    (
                        info_entry,
                        MyEndpoint::Connectionless(ep),
                        MyTxContext::Connectionless(tx_context),
                        MyRxContext::Connectionless(rx_context),
                        Some(mapped_addresses),
                        Some(av),
                        eq,
                    )
                }
                Endpoint::ConnectionOriented(ep) => {
                    let eq = eq.unwrap();
                    let ep = ep.enable(&eq).unwrap();

                    let connection_pending = match ep {
                        libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(ep) => {
                            ep.connect(info_entry.dest_addr().unwrap()).unwrap()
                        }
                        libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(
                            ep,
                        ) => ep.accept().unwrap(),
                    };

                    let ep = match eq.sread(-1) {
                        Ok(event) => match event {
                            libfabric::eq::Event::Connected(event) => {
                                connection_pending.connect_complete(event)
                            }
                            _ => panic!("Unexpected Event type"),
                        },
                        Err(err) => {
                            if matches!(err.kind, ErrorKind::ErrorAvailable) {
                                let err = eq.readerr().unwrap();
                                panic!("Error in EQ: {}", eq.strerror(&err))
                            } else {
                                panic!("Error in EQ: {:?}", err)
                            }
                        }
                    };

                    let tx_context = TxContextBuilder::new(&ep, 0).build();

                    let rx_context = RxContextBuilder::new(&ep, 0).build();

                    mr = if info_entry.domain_attr().mr_mode().is_local()
                        || info_entry.caps().is_rma()
                    {
                        let mr = MemoryRegionBuilder::new(
                            &reg_mem,
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

                    (
                        info_entry,
                        MyEndpoint::Connected(ep),
                        MyTxContext::Connected(tx_context),
                        MyRxContext::Connected(rx_context),
                        None,
                        None,
                        eq,
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
            remote_mem_info: None,
            cq_type,
            domain,
            ep,
            tx_context,
            rx_context,
            reg_mem,
            av,
            eq,
            // tx_pending_cnt,
            // tx_complete_cnt,
            // rx_pending_cnt,
            // rx_complete_cnt,
        })
    }
}

fn conn_send<T>(
    sender: &impl ConnectedSendEp,
    buf: &[T],
    desc: Option<MemoryRegionDesc>,
    data: Option<u64>,
    max_inject_size: usize,
) -> Result<(), libfabric::error::Error> {
    if buf.len() <= max_inject_size {
        if let Some(data) = data {
            sender.injectdata(buf, data)
        } else {
            sender.inject(buf)
        }
    } else if let Some(data) = data {
        sender.senddata(buf, desc, data)
    } else {
        sender.send(buf, desc)
    }
}


fn conn_send_mr(
    sender: &impl ConnectedSendEpMrSlice,
    mr_slice: &MemoryRegionSlice,
    data: Option<u64>,
    max_inject_size: usize,
) -> Result<(), libfabric::error::Error> {
    if mr_slice.as_slice().len() <= max_inject_size {
        if data.is_some() {
            sender.injectdata_mr_slice(mr_slice, data.unwrap())
        } else {
            sender.inject_mr_slice(mr_slice)
        }
    } else {
        if data.is_some() {
            sender.senddata_mr_slice(mr_slice, data.unwrap())
        } else {
            sender.send_mr_slice(mr_slice)
        }
    }
}

fn connless_send<T>(
    sender: &impl SendEp,
    buf: &[T],
    desc: Option<MemoryRegionDesc>,
    data: Option<u64>,
    addr: &MyRc<MappedAddress>,
    max_inject_size: usize,
) -> Result<(), libfabric::error::Error> {
    if buf.len() <= max_inject_size {
        if let Some(data) = data {
            sender.injectdata_to(buf, data, addr.as_ref())
        } else {
            sender.inject_to(buf, addr.as_ref())
        }
    } else if let Some(data) = data {
        sender.senddata_to(buf, desc, data, addr.as_ref())
    } else {
        sender.send_to(buf, desc, addr.as_ref())
    }
}

fn connless_send_mr(
    sender: &impl SendEpMrSlice,
    mr_slice: &MemoryRegionSlice,
    data: Option<u64>,
    addr: &MyRc<MappedAddress>,
    max_inject_size: usize,
) -> Result<(), libfabric::error::Error> {
    if mr_slice.as_slice().len() <= max_inject_size {
        if data.is_some() {
            sender.injectdata_mr_slice_to(mr_slice, data.unwrap(), addr.as_ref())
        } else {
            sender.inject_mr_slice_to(mr_slice, addr.as_ref())
        }
    } else {
        if data.is_some() {
            sender.senddata_mr_slice_to(mr_slice, data.unwrap(), addr.as_ref())
        } else {
            sender.send_mr_slice_to(mr_slice, addr.as_ref())
        }
    }
}

fn conn_sendv(
    sender: &impl ConnectedSendEp,
    iov: &[IoVec],
    desc: Option<&[MemoryRegionDesc]>,
) -> Result<(), libfabric::error::Error> {
    sender.sendv(iov, desc)
}

fn connless_sendv(
    sender: &impl SendEp,
    iov: &[IoVec],
    desc: Option<&[MemoryRegionDesc]>,
    mapped_addr: &MyRc<MappedAddress>,
) -> Result<(), libfabric::error::Error> {
    sender.sendv_to(iov, desc, mapped_addr)
}

fn connless_recv<T>(
    sender: &impl RecvEp,
    buf: &mut [T],
    desc: Option<MemoryRegionDesc>,
    mapped_addr: &MyRc<MappedAddress>,
) -> Result<(), libfabric::error::Error> {
    sender.recv_from(buf, desc, mapped_addr)
}

fn connless_recv_mr(
    sender: &impl RecvEpMrSlice,
    mr_slice: &mut MemoryRegionSliceMut,
    mapped_addr: &MyRc<MappedAddress>,
) -> Result<(), libfabric::error::Error> {
    sender.recv_mr_slice_from(mr_slice, mapped_addr)
}

fn conn_recv<T>(
    sender: &impl ConnectedRecvEp,
    buf: &mut [T],
    desc: Option<MemoryRegionDesc>,
) -> Result<(), libfabric::error::Error> {
    sender.recv(buf, desc)
}


fn conn_recv_mr(
    sender: &impl ConnectedRecvEpMrSlice,
    mr_slice: &mut MemoryRegionSliceMut,
) -> Result<(), libfabric::error::Error> {
    sender.recv_mr_slice(mr_slice)
}

fn conn_sendmsg(
    sender: &impl ConnectedSendEp,
    msg: &MsgConnected,
    options: SendMsgOptions,
) -> Result<(), libfabric::error::Error> {
    sender.sendmsg(msg, options)
}

fn connless_sendmsg(
    sender: &impl SendEp,
    msg: &Msg,
    options: SendMsgOptions,
) -> Result<(), libfabric::error::Error> {
    sender.sendmsg_to(msg, options)
}

fn connless_recvmsg(
    recver: &impl RecvEp,
    msg: &MsgMut,
    options: RecvMsgOptions,
) -> Result<(), libfabric::error::Error> {
    recver.recvmsg_from(msg, options)
}

fn conn_recvmsg(
    recver: &impl ConnectedRecvEp,
    msg: &MsgConnectedMut,
    options: RecvMsgOptions,
) -> Result<(), libfabric::error::Error> {
    recver.recvmsg(msg, options)
}

fn conn_tsend<T>(
    sender: &impl ConnectedTagSendEp,
    buf: &[T],
    desc: Option<MemoryRegionDesc>,
    tag: u64,
    data: Option<u64>,
    max_inject_size: usize,
) -> Result<(), libfabric::error::Error> {
    if buf.len() <= max_inject_size {
        if let Some(data) = data {
            sender.tinjectdata(buf, data, tag)
        } else {
            sender.tinject(buf, tag)
        }
    } else if let Some(data) = data {
        sender.tsenddata(buf, desc, data, tag)
    } else {
        sender.tsend(buf, desc, tag)
    }
}

fn conn_tsend_mr(
    sender: &impl ConnectedTagSendEpMrSlice,
    mr_slice: &MemoryRegionSlice,
    tag: u64,
    data: Option<u64>,
    max_inject_size: usize,
) -> Result<(), libfabric::error::Error> {
    if mr_slice.as_slice().len() <= max_inject_size {
        if data.is_some() {
            sender.tinjectdata_mr_slice(mr_slice, data.unwrap(), tag)
        } else {
            sender.tinject_mr_slice(mr_slice, tag)
        }
    } else {
        if data.is_some() {
            sender.tsenddata_mr_slice(mr_slice, data.unwrap(), tag)
        } else {
            sender.tsend_mr_slice(mr_slice, tag)
        }
    }
}

fn connless_tsend<T>(
    sender: &impl TagSendEp,
    buf: &[T],
    desc: Option<MemoryRegionDesc>,
    tag: u64,
    data: Option<u64>,
    addr: &MyRc<MappedAddress>,
    max_inject_size: usize,
) -> Result<(), libfabric::error::Error> {
    if buf.len() <= max_inject_size {
        if let Some(data) = data {
            sender.tinjectdata_to(buf, data, addr.as_ref(), tag)
        } else {
            sender.tinject_to(buf, addr.as_ref(), tag)
        }
    } else if let Some(data) = data {
        sender.tsenddata_to(buf, desc, data, addr.as_ref(), tag)
    } else {
        sender.tsend_to(buf, desc, addr.as_ref(), tag)
    }
}


fn connless_tsend_mr(
    sender: &impl TagSendEpMrSlice,
    mr_slice: &MemoryRegionSlice,
    tag: u64,
    data: Option<u64>,
    addr: &MyRc<MappedAddress>,
    max_inject_size: usize,
) -> Result<(), libfabric::error::Error> {
    if mr_slice.as_slice().len() <= max_inject_size {
        if data.is_some() {
            sender.tinjectdata_mr_slice_to(mr_slice, data.unwrap(), addr.as_ref(), tag)
        } else {
            sender.tinject_mr_slice_to(mr_slice, addr.as_ref(), tag)
        }
    } else {
        if data.is_some() {
            sender.tsenddata_mr_slice_to(mr_slice, data.unwrap(), addr.as_ref(), tag)
        } else {
            sender.tsend_mr_slice_to(mr_slice, addr.as_ref(), tag)
        }
    }
}

fn conn_tsendv(
    sender: &impl ConnectedTagSendEp,
    iov: &[IoVec],
    desc: Option<&[MemoryRegionDesc]>,
    tag: u64,
) -> Result<(), libfabric::error::Error> {
    sender.tsendv(iov, desc, tag)
}

fn connless_tsendv(
    sender: &impl TagSendEp,
    iov: &[IoVec],
    desc: Option<&[MemoryRegionDesc]>,
    mapped_addr: &MyRc<MappedAddress>,
    tag: u64,
) -> Result<(), libfabric::error::Error> {
    sender.tsendv_to(iov, desc, mapped_addr, tag)
}

fn connless_trecv<T>(
    sender: &impl TagRecvEp,
    buf: &mut [T],
    desc: Option<MemoryRegionDesc>,
    mapped_addr: &MyRc<MappedAddress>,
    tag: u64,
    ignore: Option<u64>,
) -> Result<(), libfabric::error::Error> {
    sender.trecv_from(buf, desc, mapped_addr, tag, ignore)
}

fn connless_trecv_mr(
    sender: &impl TagRecvEpMrSlice,
    mr_slice: &mut MemoryRegionSliceMut,
    mapped_addr: &MyRc<MappedAddress>,
    tag: u64,
    ignore: Option<u64>,
) -> Result<(), libfabric::error::Error> {
    sender.trecv_mr_slice_from(mr_slice, mapped_addr, tag, ignore)
}

fn conn_trecv<T>(
    sender: &impl ConnectedTagRecvEp,
    buf: &mut [T],
    desc: Option<MemoryRegionDesc>,
    tag: u64,
    ignore: Option<u64>,
) -> Result<(), libfabric::error::Error> {
    sender.trecv(buf, desc, tag, ignore)
}

fn conn_trecv_mr(
    sender: &impl ConnectedTagRecvEpMrSlice,
    mr_slice: &mut MemoryRegionSliceMut,
    tag: u64,
    ignore: Option<u64>,
) -> Result<(), libfabric::error::Error> {
    sender.trecv_mr_slice(mr_slice, tag, ignore)
}


fn connless_trecvv(
    recver: &impl TagRecvEp,
    iov: &[IoVecMut],
    desc: Option<&[MemoryRegionDesc]>,
    mapped_addr: &MyRc<MappedAddress>,
    tag: u64,
    ignore: Option<u64>,
) -> Result<(), libfabric::error::Error> {
    recver.trecvv_from(iov, desc, mapped_addr, tag, ignore)
}

fn conn_trecvv(
    recver: &impl ConnectedTagRecvEp,
    iov: &[IoVecMut],
    desc: Option<&[MemoryRegionDesc]>,
    tag: u64,
    ignore: Option<u64>,
) -> Result<(), libfabric::error::Error> {
    recver.trecvv(iov, desc, tag, ignore)
}

fn conn_tsendmsg(
    sender: &impl ConnectedTagSendEp,
    msg: &MsgTaggedConnected,
    options: TaggedSendMsgOptions,
) -> Result<(), libfabric::error::Error> {
    sender.tsendmsg(msg, options)
}

fn connless_tsendmsg(
    sender: &impl TagSendEp,
    msg: &MsgTagged,
    options: TaggedSendMsgOptions,
) -> Result<(), libfabric::error::Error> {
    sender.tsendmsg_to(msg, options)
}

fn connless_trecvmsg(
    recver: &impl TagRecvEp,
    msg: &MsgTaggedMut,
    options: TaggedRecvMsgOptions,
) -> Result<(), libfabric::error::Error> {
    recver.trecvmsg_from(msg, options)
}

fn conn_trecvmsg(
    recver: &impl ConnectedTagRecvEp,
    msg: &MsgTaggedConnectedMut,
    options: TaggedRecvMsgOptions,
) -> Result<(), libfabric::error::Error> {
    recver.trecvmsg(msg, options)
}

impl<I> Ofi<I> {
    fn check_and_progress(&self, err: Result<(), libfabric::error::Error>) -> bool {
        let res = match err {
            Ok(_) => true,
            Err(err) => {
                if !matches!(err.kind, ErrorKind::TryAgain) {
                    panic!("{:?}", err);
                } else {
                    false
                }
            }
        };
        if !res {
            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }

        res
    }
}

impl<I: TagDefaultCap> Ofi<I> {
    pub fn tsend<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc>,
        tag: u64,
        data: Option<u64>,
        use_context: bool,
    ) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => connless_tsend(
                        ep,
                        buf,
                        desc,
                        tag,
                        data,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        self.info_entry.tx_attr().inject_size(),
                    ),
                    MyEndpoint::Connected(ep) => conn_tsend(
                        ep,
                        buf,
                        desc,
                        tag,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                    ),
                }
            } else {
                match &self.tx_context {
                    MyTxContext::Connectionless(tx_context) => connless_tsend(
                        tx_context.as_ref().unwrap(),
                        buf,
                        desc,
                        tag,
                        data,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        self.info_entry.tx_attr().inject_size(),
                    ),
                    MyTxContext::Connected(tx_context) => conn_tsend(
                        tx_context.as_ref().unwrap(),
                        buf,
                        desc,
                        tag,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn tsend_mr(
        &self,
        mr_slice: &MemoryRegionSlice,
        tag: u64,
        data: Option<u64>,
        use_context: bool,
    ) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => connless_tsend_mr(
                        ep,
                        mr_slice,
                        tag,
                        data,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        self.info_entry.tx_attr().inject_size(),
                    ),
                    MyEndpoint::Connected(ep) => conn_tsend_mr(
                        ep,
                        mr_slice,
                        tag,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                    ),
                }
            } else {
                match &self.tx_context {
                    MyTxContext::Connectionless(tx_context) => connless_tsend_mr(
                        tx_context.as_ref().unwrap(),
                        mr_slice,
                        tag,
                        data,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        self.info_entry.tx_attr().inject_size(),
                    ),
                    MyTxContext::Connected(tx_context) => conn_tsend_mr(
                        tx_context.as_ref().unwrap(),
                        mr_slice,
                        tag,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn tsendv(
        &self,
        iov: &[IoVec],
        desc: Option<&[MemoryRegionDesc]>,
        tag: u64,
        use_context: bool,
    ) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        connless_tsendv(ep, iov, desc, &self.mapped_addr.as_ref().unwrap()[1], tag)
                    }
                    MyEndpoint::Connected(ep) => conn_tsendv(ep, iov, desc, tag),
                }
            } else {
                match &self.tx_context {
                    MyTxContext::Connected(tx_context) => {
                        conn_tsendv(tx_context.as_ref().unwrap(), iov, desc, tag)
                    }
                    MyTxContext::Connectionless(tx_context) => connless_tsendv(
                        tx_context.as_ref().unwrap(),
                        iov,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        tag,
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn trecvv(
        &self,
        iov: &[IoVecMut],
        desc: Option<&[MemoryRegionDesc]>,
        tag: u64,
        use_context: bool,
    ) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => connless_trecvv(
                        ep,
                        iov,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        tag,
                        None,
                    ),
                    MyEndpoint::Connected(ep) => conn_trecvv(ep, iov, desc, tag, None),
                }
            } else {
                match &self.rx_context {
                    MyRxContext::Connectionless(rx_context) => connless_trecvv(
                        rx_context.as_ref().unwrap(),
                        iov,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        tag,
                        None,
                    ),
                    MyRxContext::Connected(rx_context) => {
                        conn_trecvv(rx_context.as_ref().unwrap(), iov, desc, tag, None)
                    }
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn trecv<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc>,
        tag: u64,
        use_context: bool,
    ) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => connless_trecv(
                        ep,
                        buf,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        tag,
                        None,
                    ),
                    MyEndpoint::Connected(ep) => conn_trecv(ep, buf, desc, tag, None),
                }
            } else {
                match &self.rx_context {
                    MyRxContext::Connectionless(rx_context) => connless_trecv(
                        rx_context.as_ref().unwrap(),
                        buf,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        tag,
                        None,
                    ),
                    MyRxContext::Connected(rx_context) => {
                        conn_trecv(rx_context.as_ref().unwrap(), buf, desc, tag, None)
                    }
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn trecv_mr(
        &self,
        mr_slice: &mut MemoryRegionSliceMut,
        tag: u64,
        use_context: bool,
    ) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        connless_trecv_mr(ep, mr_slice, &self.mapped_addr.as_ref().unwrap()[1], tag, None)
                    }
                    MyEndpoint::Connected(ep) => conn_trecv_mr(ep, mr_slice, tag, None),
                }
            } else {
                match &self.rx_context {
                    MyRxContext::Connectionless(rx_context) => connless_trecv_mr(
                        rx_context.as_ref().unwrap(),
                        mr_slice,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        tag,
                        None,
                    ),
                    MyRxContext::Connected(rx_context) => {
                        conn_trecv_mr(rx_context.as_ref().unwrap(), mr_slice, tag, None)
                    }
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn tsendmsg(&self, msg: &Either<MsgTagged, MsgTaggedConnected>, use_context: bool) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => {
                            connless_tsendmsg(ep, msg, TferOptions::new().remote_cq_data())
                        }
                        Either::Right(_) => panic!("Wrong message type used"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong message type used"),
                        Either::Right(msg) => {
                            conn_tsendmsg(ep, msg, TferOptions::new().remote_cq_data())
                        }
                    },
                }
            } else {
                match &self.tx_context {
                    MyTxContext::Connectionless(tx_context) => match msg {
                        Either::Left(msg) => connless_tsendmsg(
                            tx_context.as_ref().unwrap(),
                            msg,
                            TferOptions::new().remote_cq_data(),
                        ),
                        Either::Right(_) => panic!("Wrong message type used"),
                    },
                    MyTxContext::Connected(tx_context) => match msg {
                        Either::Left(_) => panic!("Wrong message type used"),
                        Either::Right(msg) => conn_tsendmsg(
                            tx_context.as_ref().unwrap(),
                            msg,
                            TferOptions::new().remote_cq_data(),
                        ),
                    },
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn trecvmsg(&self, msg: &Either<MsgTaggedMut, MsgTaggedConnectedMut>, use_context: bool) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => connless_trecvmsg(ep, msg, TferOptions::new()),
                        Either::Right(_) => panic!("Wrong message type"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong message type"),
                        Either::Right(msg) => conn_trecvmsg(ep, msg, TferOptions::new()),
                    },
                }
            } else {
                match &self.rx_context {
                    MyRxContext::Connectionless(rx_context) => match msg {
                        Either::Left(msg) => connless_trecvmsg(
                            rx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                            msg,
                            TferOptions::new(),
                        ),
                        Either::Right(_) => panic!("Wrong message type"),
                    },
                    MyRxContext::Connected(rx_context) => match msg {
                        Either::Left(_) => panic!("Wrong message type"),
                        Either::Right(msg) => conn_trecvmsg(
                            rx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                            msg,
                            TferOptions::new(),
                        ),
                    },
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }
}

impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn send<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: Option<u64>,
        use_context: bool,
    ) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => connless_send(
                        ep,
                        buf,
                        desc,
                        data,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        self.info_entry.tx_attr().inject_size(),
                    ),
                    MyEndpoint::Connected(ep) => {
                        conn_send(ep, buf, desc, data, self.info_entry.tx_attr().inject_size())
                    }
                }
            } else {
                match &self.tx_context {
                    MyTxContext::Connectionless(tx_context) => connless_send(
                        tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        buf,
                        desc,
                        data,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        self.info_entry.tx_attr().inject_size(),
                    ),
                    MyTxContext::Connected(tx_context) => conn_send(
                        tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        buf,
                        desc,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn send_mr(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: Option<u64>,
        use_context: bool,
    ) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => connless_send_mr(
                        ep,
                        mr_slice,
                        data,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        self.info_entry.tx_attr().inject_size(),
                    ),
                    MyEndpoint::Connected(ep) => {
                        conn_send_mr(ep, mr_slice, data, self.info_entry.tx_attr().inject_size())
                    }
                }
            } else {
                match &self.tx_context {
                    MyTxContext::Connectionless(tx_context) => connless_send_mr(
                        tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        mr_slice,
                        data,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        self.info_entry.tx_attr().inject_size(),
                    ),
                    MyTxContext::Connected(tx_context) => conn_send_mr(
                        tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        mr_slice,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn send_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: Option<u64>,
        context: &mut Context,
    ) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if let Some(data) = data {
                            ep.injectdata_to(
                                buf,
                                data,
                                &self.mapped_addr.as_ref().unwrap()[1],
                            )
                        } else {
                            ep.inject_to(buf, &self.mapped_addr.as_ref().unwrap()[1])
                        }
                    } else if let Some(data) = data {
                        ep.senddata_to_with_context(
                            buf,
                            desc,
                            data,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            context,
                        )
                    } else {
                        ep.send_to_with_context(
                            buf,
                            desc,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            context,
                        )
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if let Some(data) = data {
                            ep.injectdata(buf, data)
                        } else {
                            ep.inject(buf)
                        }
                    } else if let Some(data) = data {
                        ep.senddata_with_context(buf, desc, data, context)
                    } else {
                        ep.send_with_context(buf, desc, context)
                    }
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn sendv(&self, iov: &[IoVec], desc: Option<&[MemoryRegionDesc]>, use_context: bool) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        connless_sendv(ep, iov, desc, &self.mapped_addr.as_ref().unwrap()[1])
                    }
                    MyEndpoint::Connected(ep) => conn_sendv(ep, iov, desc),
                }
            } else {
                match &self.tx_context {
                    MyTxContext::Connectionless(tx_context) => connless_sendv(
                        tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        iov,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                    ),
                    MyTxContext::Connected(tx_context) => conn_sendv(
                        tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        iov,
                        desc,
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn recvv(&self, iov: &[IoVecMut], desc: Option<&[MemoryRegionDesc]>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    ep.recvv_from(iov, desc, &self.mapped_addr.as_ref().unwrap()[1])
                }
                MyEndpoint::Connected(ep) => ep.recvv(iov, desc),
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn recv<T>(&self, buf: &mut [T], desc: Option<MemoryRegionDesc>, use_context: bool) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        connless_recv(ep, buf, desc, &self.mapped_addr.as_ref().unwrap()[1])
                    }
                    MyEndpoint::Connected(ep) => conn_recv(ep, buf, desc),
                }
            } else {
                match &self.rx_context {
                    MyRxContext::Connectionless(rx_context) => connless_recv(
                        rx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        buf,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                    ),
                    MyRxContext::Connected(rx_context) => conn_recv(
                        rx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        buf,
                        desc,
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn recv_mr(&self, mr_slice: &mut MemoryRegionSliceMut, use_context: bool) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        connless_recv_mr(ep, mr_slice, &self.mapped_addr.as_ref().unwrap()[1])
                    }
                    MyEndpoint::Connected(ep) => conn_recv_mr(ep, mr_slice),
                }
            } else {
                match &self.rx_context {
                    MyRxContext::Connectionless(rx_context) => connless_recv_mr(
                        rx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        mr_slice,
                        &self.mapped_addr.as_ref().unwrap()[1],
                    ),
                    MyRxContext::Connected(rx_context) => conn_recv_mr(
                        rx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        mr_slice,
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn sendmsg(&self, msg: &Either<Msg, MsgConnected>, use_context: bool) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => {
                            connless_sendmsg(ep, msg, TferOptions::new().remote_cq_data())
                        }
                        Either::Right(_) => panic!("Wrong msg type"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong msg type"),
                        Either::Right(msg) => {
                            conn_sendmsg(ep, msg, TferOptions::new().remote_cq_data())
                        }
                    },
                }
            } else {
                match &self.tx_context {
                    MyTxContext::Connectionless(tx_context) => match msg {
                        Either::Left(msg) => connless_sendmsg(
                            tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                            msg,
                            TferOptions::new().remote_cq_data(),
                        ),
                        Either::Right(_) => panic!("Wrong msg type"),
                    },
                    MyTxContext::Connected(tx_context) => match msg {
                        Either::Left(_) => panic!("Wrong msg type"),
                        Either::Right(msg) => conn_sendmsg(
                            tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                            msg,
                            TferOptions::new().remote_cq_data(),
                        ),
                    },
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn recvmsg(&self, msg: &Either<MsgMut, MsgConnectedMut>, use_context: bool) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => match msg {
                        Either::Left(msg) => connless_recvmsg(ep, msg, TferOptions::new()),
                        Either::Right(_) => panic!("Wrong message type"),
                    },
                    MyEndpoint::Connected(ep) => match msg {
                        Either::Left(_) => panic!("Wrong message type"),
                        Either::Right(msg) => conn_recvmsg(ep, msg, TferOptions::new()),
                    },
                }
            } else {
                match &self.rx_context {
                    MyRxContext::Connectionless(rx_context) => match msg {
                        Either::Left(msg) => connless_recvmsg(
                            rx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                            msg,
                            TferOptions::new(),
                        ),
                        Either::Right(_) => panic!("Wrong message type"),
                    },
                    MyRxContext::Connected(rx_context) => match msg {
                        Either::Left(_) => panic!("Wrong message type"),
                        Either::Right(msg) => conn_recvmsg(
                            rx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                            msg,
                            TferOptions::new(),
                        ),
                    },
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn exchange_addresses(&mut self) -> Address {
        let epname = match self.ep {
            MyEndpoint::Connected(ref ep) => ep.getname(),
            MyEndpoint::Connectionless(ref ep) => ep.getname(),
        };

        let mut address_bytes = epname.unwrap().as_bytes().to_vec();

        let mr = MemoryRegionBuilder::new(&address_bytes, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&self.domain)
            .unwrap();

        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&self.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = Some(mr.descriptor());

        self.send(&address_bytes, desc, None, false);
        self.recv(&mut address_bytes, desc, false);
        self.cq_type.rx_cq().sread(1, -1).unwrap();

        unsafe { Address::from_bytes(&address_bytes) }
    }

    pub fn exchange_keys<T: Copy>(&mut self, key: &MemoryRegionKey, mem_slice: &[T]) {
        let mem_info = libfabric::MemAddressInfo::from_slice(mem_slice, 0, key, &self.info_entry);
        let mut mem_bytes = mem_info.to_bytes().to_vec();

        let mr = MemoryRegionBuilder::new(&mem_bytes, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&self.domain)
            .unwrap();

        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&self.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = Some(mr.descriptor());
        self.send(
            &mem_bytes,
            desc,
            None,
            false,
        );
        self.recv(
        &mut mem_bytes,
            desc,
            false,
        );

        self.cq_type.rx_cq().sread(1, -1).unwrap();
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
    ) {
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let write_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if let Some(data) = data {
                            unsafe {
                                ep.inject_writedata_slice_to(
                                    buf,
                                    data,
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                    &write_slice,
                                )
                            }
                        } else {
                            unsafe {
                                ep.inject_write_slice_to(
                                    buf,
                                    &self.mapped_addr.as_ref().unwrap()[1],
                                    &write_slice,
                                )
                            }
                        }
                    } else if let Some(data) = data {
                        unsafe {
                            ep.writedata_slice_to(
                                buf,
                                desc,
                                data,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &write_slice,
                            )
                        }
                    } else {
                        unsafe {
                            ep.write_slice_to(
                                buf,
                                desc,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &write_slice,
                            )
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if let Some(data) = data {
                            unsafe { ep.inject_writedata_slice(buf, data, &write_slice) }
                        } else {
                            unsafe { ep.inject_write_slice(buf, &write_slice) }
                        }
                    } else if let Some(data) = data {
                        unsafe { ep.writedata_slice(buf, desc, data, &write_slice) }
                    } else {
                        unsafe { ep.write_slice(buf, desc, &write_slice) }
                    }
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn read<T: Copy>(&self, buf: &mut [T], dest_addr: usize, desc: Option<MemoryRegionDesc>) {
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
        let read_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.read_slice_from(
                        buf,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &read_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe { ep.read_slice(buf, desc, &read_slice) },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn writev(&self, iov: &[IoVec], dest_addr: usize, desc: Option<&[MemoryRegionDesc]>) {
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let write_slice = remote_mem_info
            .slice_mut(dest_addr..dest_addr + iov.iter().fold(0, |acc, x| acc + x.len()));

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.writev_slice_to(
                        iov,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &write_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe { ep.writev_slice(iov, desc, &write_slice) },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn readv(&self, iov: &[IoVecMut], dest_addr: usize, desc: Option<&[MemoryRegionDesc]>) {
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
        let src_slice = remote_mem_info
            .slice(dest_addr..dest_addr + iov.iter().fold(0, |acc, x| acc + x.len()));

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.readv_slice_from(
                        iov,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &src_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe { ep.readv_slice(iov, desc, &src_slice) },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    // [TODO] Enabling .remote_cq_data causes the buffer not being written correctly
    // on the remote side.
    pub fn writemsg(&self, msg: &Either<MsgRma, MsgRmaConnected>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => unsafe { ep.writemsg_to(msg, WriteMsgOptions::new()) },
                    Either::Right(_) => panic!("Wrong message type"),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => panic!("Wrong message type"),
                    Either::Right(msg) => unsafe { ep.writemsg(msg, WriteMsgOptions::new()) },
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn readmsg(&self, msg: &Either<MsgRmaMut, MsgRmaConnectedMut>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => unsafe { ep.readmsg_from(msg, ReadMsgOptions::new()) },
                    Either::Right(_) => todo!(),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => panic!("Wrong message type"),
                    Either::Right(msg) => unsafe { ep.readmsg(msg, ReadMsgOptions::new()) },
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }
}

impl<I: AtomicDefaultCap> Ofi<I> {
    pub fn atomic<T: libfabric::AsFiType>(
        &self,
        buf: &[T],
        dest_addr: usize,
        desc: Option<MemoryRegionDesc>,
        op: AtomicOp,
    ) {
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let dst_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        unsafe {
                            ep.inject_atomic_slice_to(
                                buf,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &dst_slice,
                                op,
                            )
                        }
                    } else {
                        unsafe {
                            ep.atomic_slice_to(
                                buf,
                                desc,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &dst_slice,
                                op,
                            )
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        unsafe { ep.inject_atomic_slice(buf, &dst_slice, op) }
                    } else {
                        unsafe { ep.atomic_slice(buf, desc, &dst_slice, op) }
                    }
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn atomicv<T: libfabric::AsFiType>(
        &self,
        ioc: &[libfabric::iovec::Ioc<T>],
        dest_addr: usize,
        desc: Option<&[MemoryRegionDesc]>,
        op: AtomicOp,
    ) {
        let mut remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let ioc_len = ioc.iter().fold(0, |acc, x| acc + x.len());
        let dst_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + ioc_len);
        // let base_mem_addr = remote_mem_info.borrow().mem_address();
        // let key = remote_mem_info.borrow().key();

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.atomicv_slice_to(
                        ioc,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &dst_slice,
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe { ep.atomicv_slice(ioc, desc, &dst_slice, op) },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn atomicmsg<T: libfabric::AsFiType>(
        &self,
        msg: &Either<MsgAtomic<T>, MsgAtomicConnected<T>>,
    ) {
        let opts = AtomicMsgOptions::new();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => unsafe { ep.atomicmsg_to(msg, opts) },
                    Either::Right(_) => todo!(),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => todo!(),
                    Either::Right(msg) => unsafe { ep.atomicmsg(msg, opts) },
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn fetch_atomic<T: libfabric::AsFiType>(
        &self,
        buf: &[T],
        res: &mut [T],
        dest_addr: usize,
        desc: Option<MemoryRegionDesc>,
        res_desc: Option<MemoryRegionDesc>,
        op: FetchAtomicOp,
    ) {
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let src_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());

        // let base_mem_addr = remote_mem_info.borrow().mem_address();
        // let key = remote_mem_info.borrow().key();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.fetch_atomic_slice_from(
                        buf,
                        desc,
                        res,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &src_slice,
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.fetch_atomic_slice(buf, desc, res, res_desc, &src_slice, op)
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn fetch_atomicv<T: libfabric::AsFiType>(
        &self,
        ioc: &[libfabric::iovec::Ioc<T>],
        res_ioc: &mut [libfabric::iovec::IocMut<T>],
        dest_addr: usize,
        desc: Option<&[MemoryRegionDesc]>,
        res_desc: Option<&[MemoryRegionDesc]>,
        op: FetchAtomicOp,
    ) {
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow();
        let src_slice = remote_mem_info
            .slice(dest_addr..dest_addr + ioc.iter().fold(0, |acc, x| acc + x.len()));
        // let base_mem_addr = remote_mem_info.borrow().mem_address();
        // let key = remote_mem_info.borrow().key();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.fetch_atomicv_slice_from(
                        ioc,
                        desc,
                        res_ioc,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &src_slice,
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.fetch_atomicv_slice(ioc, desc, res_ioc, res_desc, &src_slice, op)
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn fetch_atomicmsg<T: libfabric::AsFiType>(
        &self,
        msg: &Either<MsgFetchAtomic<T>, MsgFetchAtomicConnected<T>>,
        res_ioc: &mut [libfabric::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc]>,
    ) {
        let opts = AtomicMsgOptions::new();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => unsafe {
                        ep.fetch_atomicmsg_from(msg, res_ioc, res_desc, opts)
                    },
                    Either::Right(_) => todo!(),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => todo!(),
                    Either::Right(msg) => unsafe {
                        ep.fetch_atomicmsg(msg, res_ioc, res_desc, opts)
                    },
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
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

        // let base_mem_addr = remote_mem_info.borrow().mem_address();
        // let key = remote_mem_info.borrow().key();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.compare_atomic_slice_to(
                        buf,
                        desc,
                        comp,
                        comp_desc,
                        res,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &dst_slice,
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.compare_atomic_slice(
                        buf, desc, comp, comp_desc, res, res_desc, &dst_slice, op,
                    )
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
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

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.compare_atomicv_slice_to(
                        ioc,
                        desc,
                        comp_ioc,
                        comp_desc,
                        res_ioc,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &dst_slice,
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.compare_atomicv_slice(
                        ioc, desc, comp_ioc, comp_desc, res_ioc, res_desc, &dst_slice, op,
                    )
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn compare_atomicmsg<T: libfabric::AsFiType>(
        &self,
        msg: &Either<MsgCompareAtomic<T>, MsgCompareAtomicConnected<T>>,
        comp_ioc: &[libfabric::iovec::Ioc<T>],
        res_ioc: &mut [libfabric::iovec::IocMut<T>],
        comp_desc: Option<&[MemoryRegionDesc]>,
        res_desc: Option<&[MemoryRegionDesc]>,
    ) {
        let opts = AtomicMsgOptions::new();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => unsafe {
                        ep.compare_atomicmsg_to(msg, comp_ioc, comp_desc, res_ioc, res_desc, opts)
                    },
                    Either::Right(_) => todo!(),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => todo!(),
                    Either::Right(msg) => unsafe {
                        ep.compare_atomicmsg(msg, comp_ioc, comp_desc, res_ioc, res_desc, opts)
                    },
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
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
                    // .tx_ctx_cnt(1)
                    // .rx_ctx_cnt(1)
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
                    // .op_flags(libfabric::enums::TransferOptions::new().delivery_complete())
                    .leave_tx_attr()
                    .enter_rx_attr()
                    // .caps(RxCaps::new().recv().collective())
                    .leave_rx_attr()
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

    let info = gen_info!(
        ep_type,
        caps,
        false,
        ip.strip_suffix("\n").unwrap_or(&ip),
        server,
        name
    );
    info
}

#[test]
fn handshake_connected0() {
    handshake(true, "handshake_connected0", Some(InfoCaps::new().msg()));
}

#[test]
fn handshake_connected1() {
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

    let info = gen_info!(
        ep_type,
        caps,
        false,
        ip.strip_suffix("\n").unwrap_or(&ip),
        server,
        name
    );

    info
}

#[test]
fn handshake_connectionless0() {
    handshake_connectionless(
        true,
        "handshake_connectionless0",
        Some(InfoCaps::new().msg()),
    );
}

#[test]
fn handshake_connectionless1() {
    handshake_connectionless(
        false,
        "handshake_connectionless0",
        Some(InfoCaps::new().msg()),
    );
}

fn sendrecv(server: bool, name: &str, connected: bool, use_context: bool) {
    let ofi = if connected {
        handshake(server, name, Some(InfoCaps::new().msg()))
    } else {
        handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
    };

    let mut reg_mem: Vec<_> = (0..1024 * 2)
        .map(|v: usize| (v % 256) as u8)
        .collect();
    let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
        .access_recv()
        .access_send()
        .build(&ofi.domain)
        .unwrap();

    let mut mr = match mr {
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
    let desc = [mr.descriptor(), mr.descriptor()];
    let mut ctx = ofi.info_entry.allocate_context();

    if server {
        // Send a single buffer
        ofi.send_with_context(&reg_mem[..512], desc0, None, &mut ctx);

        let completion = ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        match completion {
            Completion::Data(entry) => {
                assert!(entry[0].is_op_context_equal(&ctx))
            }
            _ => panic!("unexpected completion type"),
        }

        assert!(std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size());

        // Inject a buffer
        ofi.send(&reg_mem[..128], desc0, None, use_context);
        // No cq.sread since inject does not generate completions

        // // Send single Iov
        let iov = [IoVec::from_slice(&reg_mem[..512])];
        ofi.sendv(&iov, Some(&desc[..1]), use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send multi Iov
        let iov = [
            IoVec::from_slice(&reg_mem[..512]),
            IoVec::from_slice(&reg_mem[512..1024]),
        ];
        ofi.sendv(&iov, Some(&desc), use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send a single buffer
        // ofi.send_mr(&unsafe{mr.slice(0, ..512)}, None, false);
        let m1 = unsafe{ mr.slice(..512)};
        ofi.send_mr(&m1, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    } else {
        let expected: Vec<_> = (0..1024 * 2)
            .map(|v: usize| (v % 256) as u8)
            .collect();
        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Receive a single buffer
        ofi.recv(&mut reg_mem[..512], desc0, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..512], expected[..512]);

        // Receive inject
        reg_mem.iter_mut().for_each(|v| *v = 0);
        ofi.recv(&mut reg_mem[..128], desc0, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..128], expected[..128]);

        reg_mem.iter_mut().for_each(|v| *v = 0);
        // // Receive into a single Iov
        let iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
        ofi.recvv(&iov, Some(&desc[..1]));
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..512], expected[..512]);

        reg_mem.iter_mut().for_each(|v| *v = 0);

        // // Receive into multiple Iovs
        let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
        let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
        ofi.recvv(&iov, Some(&desc));
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        assert_eq!(mem0, &expected[..512]);
        assert_eq!(mem1, &expected[512..1024]);

        reg_mem.iter_mut().for_each(|v| *v = 0);
        let mut mr0  = unsafe{mr.slice_mut( ..512)};
        // let (mr00, mr01) = mr0.split_at_mut(256);
        let mmr0 = mr0.as_mut_slice();
        mmr0[0] = 0;
        ofi.recv_mr(&mut mr0, false);

        // Send a single buffer
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(unsafe{mr.slice(..512)}.as_slice(), &expected[..512]);

    }
}

#[test]
fn sendrecv0() {
    sendrecv(true, "sendrecv0", false, false);
}

#[test]
fn sendrecv1() {
    sendrecv(false, "sendrecv0", false, false);
}

#[test]
fn conn_sendrecv0() {
    sendrecv(true, "conn_sendrecv0", true, false);
}

#[test]
fn conn_sendrecv1() {
    sendrecv(false, "conn_sendrecv0", true, false);
}

// #[test]
// fn context_sendrecv0() {
//     sendrecv(true, "sendrecv0", false, true);
// }

// #[test]
// fn context_sendrecv1() {
//     sendrecv(false, "sendrecv0", false, true);
// }

// #[test]
// fn context_conn_sendrecv0() {
//     sendrecv(true, "conn_sendrecv0", true, true);
// }

// #[test]
// fn context_conn_sendrecv1() {
//     sendrecv(false, "conn_sendrecv0", true, true);
// }

fn sendrecvdata(server: bool, name: &str, connected: bool, use_context: bool) {
    let ofi = if connected {
        handshake(server, name, Some(InfoCaps::new().msg()))
    } else {
        handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
    };

    let mut reg_mem: Vec<_> = (0..1024 * 2)
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
    if server {
        // Send a single buffer
        ofi.send(&reg_mem[..512], desc0, data, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    } else {
        let expected: Vec<_> = (0..1024 * 2)
            .map(|v: usize| (v % 256) as u8)
            .collect();
        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Receive a single buffer
        ofi.recv(&mut reg_mem[..512], desc0, use_context);

        let entry = ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        match entry {
            Completion::Data(entry) => assert_eq!(entry[0].data(), data.unwrap()),
            _ => panic!("Unexpected CQ entry format"),
        }
        assert_eq!(reg_mem[..512], expected[..512]);
    }
}

#[test]
fn sendrecvdata0() {
    sendrecvdata(true, "sendrecvdata0", false, false);
}

#[test]
fn sendrecvdata1() {
    sendrecvdata(false, "sendrecvdata0", false, false);
}

#[test]
fn conn_sendrecvdata0() {
    sendrecvdata(true, "conn_sendrecvdata0", true, false);
}

#[test]
fn conn_sendrecvdata1() {
    sendrecvdata(false, "conn_sendrecvdata0", true, false);
}

// #[test]
// fn context_sendrecvdata0() {
//     sendrecvdata(true, "sendrecvdata0", false, true);
// }

// #[test]
// fn context_sendrecvdata1() {
//     sendrecvdata(false, "sendrecvdata0", false, true);
// }

// #[test]
// fn context_conn_sendrecvdata0() {
//     sendrecvdata(true, "conn_sendrecvdata0", true, true);
// }

// #[test]
// fn context_conn_sendrecvdata1() {
//     sendrecvdata(false, "conn_sendrecvdata0", true, true);
// }

fn enable_ep_mr<E: 'static>(ep: &MyEndpoint<E>, mr: EpBindingMemoryRegion) -> MemoryRegion {
    match ep {
        MyEndpoint::Connected(ep) => mr.enable(ep).unwrap(),
        MyEndpoint::Connectionless(ep) => mr.enable(ep).unwrap(),
    }
}

fn tsendrecv(server: bool, name: &str, connected: bool, use_context: bool) {
    let ofi = if connected {
        handshake(server, name, Some(InfoCaps::new().msg().tagged()))
    } else {
        handshake_connectionless(server, name, Some(InfoCaps::new().msg().tagged()))
    };

    let mut reg_mem: Vec<_> = (0..1024 * 2)
        .map(|v: usize| (v % 256) as u8)
        .collect();

    let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
        .access_recv()
        .access_send()
        .build(&ofi.domain)
        .unwrap();

    let mut mr = match mr {
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
    if server {
        // Send a single buffer
        ofi.tsend(&reg_mem[..512], desc0, 10, data, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // match entry {
        //     Completion::Tagged(entry) => {assert_eq!(entry[0].data(), data.unwrap()); assert_eq!(entry[0].tag(), 10)},
        //     _ => panic!("Unexpected CQ entry format"),
        // }

        assert!(std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size());

        // Inject a buffer
        ofi.tsend(&reg_mem[..128], desc0, 1, data, use_context);
        // No cq.sread since inject does not generate completions

        // // Send single Iov
        let iov = [IoVec::from_slice(&reg_mem[..512])];
        ofi.tsendv(&iov, Some(&desc[..1]), 2, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send multi Iov
        let iov = [
            IoVec::from_slice(&reg_mem[..512]),
            IoVec::from_slice(&reg_mem[512..1024]),
        ];
        ofi.tsendv(&iov, Some(&desc), 3, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send a single buffer
        ofi.tsend_mr(unsafe{&mr.slice(..512)}, 0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        
    } else {
        let expected: Vec<_> = (0..1024 * 2)
            .map(|v: usize| (v % 256) as u8)
            .collect();
        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Receive a single buffer
        ofi.trecv(&mut reg_mem[..512], desc0, 10, use_context);
        let entry = ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        match entry {
            Completion::Tagged(entry) => {
                assert_eq!(entry[0].data(), data.unwrap());
                assert_eq!(entry[0].tag(), 10)
            }
            _ => panic!("Unexpected CQ entry format"),
        }
        assert_eq!(reg_mem[..512], expected[..512]);

        // Receive inject
        reg_mem.iter_mut().for_each(|v| *v = 0);
        ofi.trecv(&mut reg_mem[..128], desc0, 1, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..128], expected[..128]);

        reg_mem.iter_mut().for_each(|v| *v = 0);
        // // Receive into a single Iov
        let iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
        ofi.trecvv(&iov, Some(&desc[..1]), 2, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..512], expected[..512]);

        reg_mem.iter_mut().for_each(|v| *v = 0);

        // // Receive into multiple Iovs
        let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
        let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
        ofi.trecvv(&iov, Some(&desc), 3, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        assert_eq!(mem0, &expected[..512]);
        assert_eq!(mem1, &expected[512..1024]);

        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Send a single buffer
        ofi.trecv_mr(&mut unsafe{mr.slice_mut(..512)}, 0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        assert_eq!(unsafe{mr.slice(..512)}.as_slice(), &expected[..512]);
    }
}

#[test]
fn tsendrecv0() {
    tsendrecv(true, "tsendrecv0", false, false);
}

#[test]
fn tsendrecv1() {
    tsendrecv(false, "tsendrecv0", false, false);
}

#[test]
fn conn_tsendrecv0() {
    tsendrecv(true, "conn_tsendrecv0", true, false);
}

#[test]
fn conn_tsendrecv1() {
    tsendrecv(false, "conn_tsendrecv0", true, false);
}

// #[test]
// fn context_tsendrecv0() {
//     tsendrecv(true, "tsendrecv0", false, true);
// }

// #[test]
// fn context_tsendrecv1() {
//     tsendrecv(false, "tsendrecv0", false, true);
// }

// #[test]
// fn context_conn_tsendrecv0() {
//     tsendrecv(true, "conn_tsendrecv0", true, true);
// }

// #[test]
// fn context_conn_tsendrecv1() {
//     tsendrecv(false, "conn_tsendrecv0", true, true);
// }

fn sendrecvmsg(server: bool, name: &str, connected: bool, use_context: bool) {
    let ofi = if connected {
        handshake(server, name, Some(InfoCaps::new().msg()))
    } else {
        handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
    };

    let mut reg_mem: Vec<_> = (0..1024 * 2)
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
        let msg = if connected {
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
        ofi.sendmsg(&msg, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // let entry =
        // match entry {
        //     Completion::Data(entry) => assert_eq!(entry[0].data(), data),
        //     _ => panic!("Unexpected CQ entry format"),
        // }

        // Multi iov message with stride
        let iovs = [iov0, iov1];
        let msg = if connected {
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

        ofi.sendmsg(&msg, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // let entry =
        // match entry {
        //     Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
        //     _ => panic!("Unexpected CQ entry format"),
        // }

        // Single iov message
        let msg = if connected {
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

        ofi.sendmsg(&msg, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let msg = if connected {
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
        ofi.sendmsg(&msg, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    } else {
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

        // Receive a single message in a single buffer
        let mut iov = IoVecMut::from_slice(mem0);
        let msg = if connected {
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

        ofi.recvmsg(&msg, use_context);
        // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        let entry = ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        match entry {
            Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
            _ => panic!("Unexpected CQ entry format"),
        }
        assert_eq!(mem0.len(), expected[..512].len());
        assert_eq!(mem0, &expected[..512]);

        // Receive a multi iov message in a single buffer
        let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
        let msg = if connected {
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

        ofi.recvmsg(&msg, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        // let entry =
        // match entry {
        //     Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
        //     _ => panic!("Unexpected CQ entry format"),
        // }
        assert_eq!(mem1[..1024], expected);

        // Receive a single iov message into two buffers
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let iov = IoVecMut::from_slice(&mut mem0[..256]);
        let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
        let mut iovs = [iov, iov1];
        let msg = if connected {
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

        ofi.recvmsg(&msg, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem0[..256], expected[..256]);
        assert_eq!(mem1[..256], expected[256..512]);

        // Receive a two iov message into two buffers
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let iov = IoVecMut::from_slice(&mut mem0[..512]);
        let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
        let mut iovs = [iov, iov1];
        let msg = if connected {
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

        ofi.recvmsg(&msg, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem0[..512], expected[..512]);
        assert_eq!(mem1[..512], expected[512..1024]);
    }
}

#[test]
fn sendrecvmsg0() {
    sendrecvmsg(true, "sendrecvmsg0", false, false);
}

#[test]
fn sendrecvmsg1() {
    sendrecvmsg(false, "sendrecvmsg0", false, false);
}

#[test]
fn conn_sendrecvmsg0() {
    sendrecvmsg(true, "conn_sendrecvmsg0", true, false);
}

#[test]
fn conn_sendrecvmsg1() {
    sendrecvmsg(false, "conn_sendrecvmsg0", true, false);
}

// #[test]
// fn context_sendrecvmsg0() {
//     sendrecvmsg(true, "sendrecvmsg0", false, true);
// }

// #[test]
// fn context_sendrecvmsg1() {
//     sendrecvmsg(false, "sendrecvmsg0", false, true);
// }

// #[test]
// fn context_conn_sendrecvmsg0() {
//     sendrecvmsg(true, "conn_sendrecvmsg0", true, true);
// }

// #[test]
// fn context_conn_sendrecvmsg1() {
//     sendrecvmsg(false, "conn_sendrecvmsg0", true, true);
// }

fn tsendrecvmsg(server: bool, name: &str, connected: bool, use_context: bool) {
    let ofi = if connected {
        handshake(server, name, Some(InfoCaps::new().msg().tagged()))
    } else {
        handshake_connectionless(server, name, Some(InfoCaps::new().msg().tagged()))
    };

    let mut reg_mem: Vec<_> = (0..1024 * 2)
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

    let descs = [mr.descriptor(), mr.descriptor()];
    let desc0 = Some(mr.descriptor());
    let mapped_addr = ofi.mapped_addr.clone();
    let data = Some(128);
    let mut ctx = ofi.info_entry.allocate_context();
    if server {
        // Single iov message
        let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
        let iov0 = IoVec::from_slice(mem0);
        let iov1 = IoVec::from_slice(mem1);
        let msg = if connected {
            Either::Right(MsgTaggedConnected::from_iov(
                &iov0,
                Some(&descs[0]),
                data,
                0,
                None,
                &mut ctx,
            ))
        } else {
            Either::Left(MsgTagged::from_iov(
                &iov0,
                Some(&descs[0]),
                &mapped_addr.as_ref().unwrap()[1],
                data,
                0,
                None,
                &mut ctx,
            ))
        };
        ofi.tsendmsg(&msg, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // let entry =
        // match entry {
        //     Completion::Tagged(entry) => assert_eq!(entry[0].data(), 128),
        //     _ => panic!("Unexpected CQ entry format"),
        // }

        // Multi iov message with stride
        let iovs = [iov0, iov1];
        let msg = if connected {
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

        ofi.tsendmsg(&msg, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Single iov message
        let msg = if connected {
            Either::Right(MsgTaggedConnected::from_iov(
                &iovs[0],
                desc0.as_ref(),
                None,
                2,
                None,
                &mut ctx,
            ))
        } else {
            Either::Left(MsgTagged::from_iov(
                &iovs[0],
                desc0.as_ref(),
                &mapped_addr.as_ref().unwrap()[1],
                None,
                2,
                None,
                &mut ctx,
            ))
        };

        ofi.tsendmsg(&msg, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let msg = if connected {
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
        ofi.tsendmsg(&msg, use_context);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    } else {
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

        // Receive a single message in a single buffer
        let mut iov = IoVecMut::from_slice(mem0);
        let msg = if connected {
            Either::Right(MsgTaggedConnectedMut::from_iov(
                &mut iov,
                desc0.as_ref(),
                None,
                0,
                None,
                &mut ctx,
            ))
        } else {
            Either::Left(MsgTaggedMut::from_iov(
                &mut iov,
                desc0.as_ref(),
                &mapped_addr.as_ref().unwrap()[1],
                None,
                0,
                None,
                &mut ctx,
            ))
        };

        ofi.trecvmsg(&msg, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        // let entry =
        // match entry {
        //     Completion::Tagged(entry) => assert_eq!(entry[0].data(), 128),
        //     _ => panic!("Unexpected CQ entry format"),
        // }
        assert_eq!(mem0.len(), expected[..512].len());
        assert_eq!(mem0, &expected[..512]);

        // Receive a multi iov message in a single buffer
        let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
        let msg = if connected {
            Either::Right(MsgTaggedConnectedMut::from_iov(
                &mut iov,
                desc0.as_ref(),
                None,
                1,
                None,
                &mut ctx,
            ))
        } else {
            Either::Left(MsgTaggedMut::from_iov(
                &mut iov,
                desc0.as_ref(),
                &mapped_addr.as_ref().unwrap()[1],
                None,
                1,
                None,
                &mut ctx,
            ))
        };

        ofi.trecvmsg(&msg, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1[..1024], expected);

        // Receive a single iov message into two buffers
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let iov = IoVecMut::from_slice(&mut mem0[..256]);
        let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
        let mut iovs = [iov, iov1];
        let msg = if connected {
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

        ofi.trecvmsg(&msg, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem0[..256], expected[..256]);
        assert_eq!(mem1[..256], expected[256..512]);

        // Receive a two iov message into two buffers
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let iov = IoVecMut::from_slice(&mut mem0[..512]);
        let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
        let mut iovs = [iov, iov1];
        let msg = if connected {
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

        ofi.trecvmsg(&msg, use_context);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem0[..512], expected[..512]);
        assert_eq!(mem1[..512], expected[512..1024]);
    }
}

#[test]
fn tsendrecvmsg0() {
    tsendrecvmsg(true, "tsendrecvmsg0", false, false);
}

#[test]
fn tsendrecvmsg1() {
    tsendrecvmsg(false, "tsendrecvmsg0", false, false);
}

// #[test]
// fn conn_tsendrecvmsg0() {
//     tsendrecvmsg(true, "conn_tsendrecvmsg0", true, false);
// }

// #[test]
// fn conn_tsendrecvmsg1() {
//     tsendrecvmsg(false, "conn_tsendrecvmsg0", true, false);
// }

// #[test]
// fn context_tsendrecvmsg0() {
//     tsendrecvmsg(true, "tsendrecvmsg0", false, true);
// }

// #[test]
// fn context_tsendrecvmsg1() {
//     tsendrecvmsg(false, "tsendrecvmsg0", false, true);
// }

// #[test]
// fn context_conn_tsendrecvmsg0() {
//     tsendrecvmsg(true, "conn_tsendrecvmsg0", true, true);
// }

// #[test]
// fn context_conn_tsendrecvmsg1() {
//     tsendrecvmsg(false, "conn_tsendrecvmsg0", true, true);
// }

fn writeread(server: bool, name: &str, connected: bool) {
    let mut ofi = if connected {
        handshake(server, name, Some(InfoCaps::new().msg().rma()))
    } else {
        handshake_connectionless(server, name, Some(InfoCaps::new().msg().rma()))
    };

    let mut reg_mem: Vec<_> = if server {
        (0..1024 * 2)
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

    let descs = [mr.descriptor(), mr.descriptor()];
    let desc0 = Some(mr.descriptor());
    // let mapped_addr = ofi.mapped_addr.clone();
    let key = mr.key().unwrap();
    ofi.exchange_keys(&key, &reg_mem[..]);
    let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();
    if server {
        // Write inject a single buffer
        ofi.write(&reg_mem[..128], 0, desc0, None);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Write a single buffer
        ofi.write(&reg_mem[..512], 0, desc0, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Write vector of buffers
        let iovs = [
            IoVec::from_slice(&reg_mem[..512]),
            IoVec::from_slice(&reg_mem[512..1024]),
        ];
        ofi.writev(&iovs, 0, Some(&descs));
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..128], &expected[..128]);

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);

        // Recv a completion ack
        ofi.recv(&mut reg_mem[1024..1536], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..1024], &expected[..1024]);

        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Read buffer from remote memory
        ofi.read(&mut reg_mem[1024..1536], 0, desc0);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);

        // Read vector of buffers from remote memory
        let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
        let iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
        ofi.readv(&iovs, 0, Some(&descs));
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        assert_eq!(mem0, &expected[..256]);
        assert_eq!(mem1, &expected[..256]);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    }
}

#[test]
fn conn_writeread0() {
    writeread(true, "conn_writeread0", true);
}

#[test]
fn conn_writeread1() {
    writeread(false, "conn_writeread0", true);
}

#[test]
fn writeread0() {
    writeread(true, "writeread0", false);
}

#[test]
fn writeread1() {
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
    let descs = [mr.descriptor(), mr.descriptor()];
    let desc0 = Some(mr.descriptor());
    let mapped_addr = ofi.mapped_addr.clone();

    let key = mr.key().unwrap();
    ofi.exchange_keys(&key, &reg_mem[..]);
    let expected: Vec<u8> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

    let mut ctx = ofi.info_entry.allocate_context();
    if server {
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let rma_addr = remote_mem_info.slice::<u8>(..128);
        let mut rma_iov = RemoteMemAddrVec::new();
        rma_iov.push(rma_addr);
        let iov = IoVec::from_slice(&reg_mem[..128]);
        let msg = if connected {
            Either::Right(MsgRmaConnected::from_iov(
                &iov,
                desc0.as_ref(),
                &rma_iov,
                None,
                &mut ctx,
            ))
        } else {
            Either::Left(MsgRma::from_iov(
                &iov,
                desc0.as_ref(),
                &mapped_addr.as_ref().unwrap()[1],
                &rma_iov,
                None,
                &mut ctx,
            ))
        };

        // Write inject a single buffer
        ofi.writemsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let iov = IoVec::from_slice(&reg_mem[..512]);

        let rma_addr = remote_mem_info.slice::<u8>(..512);
        let mut rma_iov = RemoteMemAddrVec::new();
        rma_iov.push(rma_addr);

        let msg = if connected {
            Either::Right(MsgRmaConnected::from_iov(
                &iov,
                desc0.as_ref(),
                &rma_iov,
                Some(128),
                &mut ctx,
            ))
        } else {
            Either::Left(MsgRma::from_iov(
                &iov,
                desc0.as_ref(),
                &mapped_addr.as_ref().unwrap()[1],
                &rma_iov,
                Some(128),
                &mut ctx,
            ))
        };

        // Write a single buffer
        ofi.writemsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let iov0 = IoVec::from_slice(&reg_mem[..512]);
        let iov1 = IoVec::from_slice(&reg_mem[512..1024]);
        let iovs = [iov0, iov1];
        let rma_addr0 = remote_mem_info.slice::<u8>(..512);

        let rma_addr1 = remote_mem_info.slice::<u8>(512..1024);

        let mut rma_iov = RemoteMemAddrVec::new();
        rma_iov.push(rma_addr0);
        rma_iov.push(rma_addr1);

        let msg = if connected {
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

        ofi.writemsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let mut remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow_mut();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..128], &expected[..128]);

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);

        // Recv a completion ack
        ofi.recv(&mut reg_mem[1024..1536], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..1024], &expected[..1024]);

        reg_mem.iter_mut().for_each(|v| *v = 0);
        // let base_addr = remote_mem_info.borrow().mem_address();
        // let mapped_key  = &remote_mem_info.borrow().key();
        {
            let mut iov = IoVecMut::from_slice(&mut reg_mem[1024..1536]);
            let rma_addr = remote_mem_info.slice_mut::<u8>(..);
            let mut rma_iov = RemoteMemAddrVecMut::new();
            rma_iov.push(rma_addr);

            // Read buffer from remote memory
            let msg = if connected {
                Either::Right(MsgRmaConnectedMut::from_iov(
                    &mut iov,
                    desc0.as_ref(),
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRmaMut::from_iov(
                    &mut iov,
                    desc0.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            };
            ofi.readmsg(&msg);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);
        }

        // Read vector of buffers from remote memory
        let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
        let mut iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
        let (rma_addr0, rma_addr1) = remote_mem_info.slice_mut::<u8>(..512).split_at_mut(256);

        let mut rma_iov = RemoteMemAddrVecMut::new();
        rma_iov.push(rma_addr0);
        rma_iov.push(rma_addr1);

        let msg = if connected {
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
        ofi.readmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        assert_eq!(mem0, &expected[..256]);
        assert_eq!(mem1, &expected[..256]);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    }
}

#[test]
fn writereadmsg0() {
    writereadmsg(true, "writereadmsg0", false);
}

#[test]
fn writereadmsg1() {
    writereadmsg(false, "writereadmsg0", false);
}

#[test]
fn conn_writereadmsg0() {
    writereadmsg(true, "conn_writereadmsg0", true);
}

#[test]
fn conn_writereadmsg1() {
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
    let descs = [mr.descriptor(), mr.descriptor()];
    let desc0 = Some(mr.descriptor());
    // let mapped_addr = ofi.mapped_addr.clone();
    let key = mr.key().unwrap();
    ofi.exchange_keys(&key, &reg_mem[..]);
    if server {
        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Min);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Max);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Sum);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Prod);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Bor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Band);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Lor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Bxor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Land);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Lxor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::AtomicWrite);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let iocs = [
            Ioc::from_slice(&reg_mem[..256]),
            Ioc::from_slice(&reg_mem[256..512]),
        ];

        ofi.atomicv(&iocs, 0, Some(&descs), AtomicOp::Prod);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        let err = ofi.cq_type.tx_cq().sread(1, -1);
        if let Err(e) = err {
            if matches!(e.kind, libfabric::error::ErrorKind::ErrorAvailable) {
                let realerr = ofi.cq_type.tx_cq().readerr(0).unwrap();
                panic!("{:?}", realerr.error());
            }
        }

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let mut expected = vec![2u8; 1024 * 2];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);
        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![3; 1024 * 2];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // expected = vec![2;1024*2];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        // assert_eq!(&reg_mem[..512], &expected[..512]);

        expected = vec![4; 1024 * 2];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    }
}

// [TODO Not sure why, but connected endpoints fail with atomic ops
// #[test]
// fn conn_atomic0() {
//     atomic(true, "conn_atomic0", true);
// }

// #[test]
// fn conn_atomic1() {
//     atomic(false, "conn_atomic0", true);
// }

#[test]
fn atomic0() {
    atomic(true, "atomic0", false);
}

#[test]
fn atomic1() {
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
    if server {
        let mut expected: Vec<u64> = vec![1; 256];
        let (op_mem, ack_mem) = reg_mem.split_at_mut(512);
        let (mem0, mem1) = op_mem.split_at_mut(256);
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Min,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected[..256]);

        expected = vec![1; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Max,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![2; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Sum,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![4; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Prod,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![8; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Bor,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![10; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Band,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // Send a done ack

        ofi.recv(&mut ack_mem[..512], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        expected = vec![2; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Lor,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![1; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Bxor,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // Send a done ack

        ofi.recv(&mut ack_mem[..512], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        expected = vec![3; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Land,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![1; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::Lxor,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![0; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::AtomicWrite,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // Send a done ack

        ofi.recv(&mut ack_mem[..512], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        expected = vec![2; 256];
        ofi.fetch_atomic(
            mem0,
            mem1,
            0,
            desc0,
            desc1,
            FetchAtomicOp::AtomicRead,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
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
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(write_mem, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut ack_mem[..512], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let mut expected = vec![2u64; 256];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![3; 256];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![2; 256];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![4; 256];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    }
}

#[test]
fn fetch_atomic0() {
    fetch_atomic(true, "fetch_atomic0", false);
}

#[test]
fn fetch_atomic1() {
    fetch_atomic(false, "fetch_atomic0", false);
}

// [TODO Not sure why, but connected endpoints fail with atomic ops
// #[test]
// fn conn_fetch_atomic0() {
//     fetch_atomic(true, "conn_fetch_atomic0", true);
// }

// #[test]
// fn conn_fetch_atomic1() {
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
    if server {
        let mut expected: Vec<_> = vec![1; 256];
        let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
        let (buf, mem1) = op_mem.split_at_mut(256);
        let (comp, res) = mem1.split_at_mut(256);
        comp.iter_mut().for_each(|v| *v = 1);

        ofi.compare_atomic(
            buf,
            comp,
            res,
            0,
            desc,
            comp_desc,
            res_desc,
            CompareAtomicOp::Cswap,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected[..256]);

        expected = vec![2; 256];
        ofi.compare_atomic(
            buf,
            comp,
            res,
            0,
            desc,
            comp_desc,
            res_desc,
            CompareAtomicOp::CswapNe,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        buf.iter_mut().for_each(|v| *v = 3);
        expected = vec![2; 256];
        ofi.compare_atomic(
            buf,
            comp,
            res,
            0,
            desc,
            comp_desc,
            res_desc,
            CompareAtomicOp::CswapLe,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        buf.iter_mut().for_each(|v| *v = 2);
        expected = vec![3; 256];
        ofi.compare_atomic(
            buf,
            comp,
            res,
            0,
            desc,
            comp_desc,
            res_desc,
            CompareAtomicOp::CswapLt,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        buf.iter_mut().for_each(|v| *v = 3);
        expected = vec![2; 256];
        ofi.compare_atomic(
            buf,
            comp,
            res,
            0,
            desc,
            comp_desc,
            res_desc,
            CompareAtomicOp::CswapGe,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        expected = vec![2; 256];
        ofi.compare_atomic(
            buf,
            comp,
            res,
            0,
            desc,
            comp_desc,
            res_desc,
            CompareAtomicOp::CswapGt,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], desc, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // Send a done ack

        ofi.recv(&mut ack_mem[..512], desc, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        // expected = vec![2; 256];
        let (buf0, buf1) = buf.split_at_mut(128);
        let (comp0, comp1) = comp.split_at_mut(128);
        let (res0, res1) = res.split_at_mut(128);

        let buf_iocs = [Ioc::from_slice(buf0), Ioc::from_slice(buf1)];
        let comp_iocs = [Ioc::from_slice(comp0), Ioc::from_slice(comp1)];
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
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], desc, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut ack_mem[..512], desc, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let mut expected = vec![2u8; 256];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![3; 256];
        // // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);
        ofi.send(&reg_mem[512..1024], desc, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    }
}

#[test]
fn compare_atomic0() {
    compare_atomic(true, "compare_atomic0", false);
}

#[test]
fn compare_atomic1() {
    compare_atomic(false, "compare_atomic0", false);
}

// [TODO Not sure why, but connected endpoints fail with atomic ops
// #[test]
// fn conn_compare_atomic0() {
//     compare_atomic(true, "conn_compare_atomic0", true);
// }

// #[test]
// fn conn_compare_atomic1() {
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
    let dst_slice = remote_mem_info.slice(..512);
    let (dst_slice0, dst_slice1) = dst_slice.split_at(256 * std::mem::size_of::<u8>());

    let mut ctx = ofi.info_entry.allocate_context();
    if server {
        let iocs = [
            Ioc::from_slice(&reg_mem[..256]),
            Ioc::from_slice(&reg_mem[256..512]),
        ];
        let mut rma_iocs = RemoteMemAddrAtomicVec::new();
        rma_iocs.push(dst_slice0);
        rma_iocs.push(dst_slice1);

        let msg = if connected {
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

        ofi.atomicmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.send(&reg_mem[512..1024], desc, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let expected = vec![3u8; 1024 * 2];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);
        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    }
}

// [TODO Not sure why, but connected endpoints fail with atomic ops
// #[test]
// fn conn_atomic0() {
//     atomic(true, "conn_atomic0", true);
// }

// #[test]
// fn conn_atomic1() {
//     atomic(false, "conn_atomic0", true);
// }

#[test]
fn atomicmsg0() {
    atomicmsg(true, "atomicmsg0", false);
}

#[test]
fn atomicmsg1() {
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
    let dst_slice = remote_mem_info.slice(..256);
    let (dst_slice0, dst_slice1) = dst_slice.split_at(128);

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

        let msg = if connected {
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

        ofi.fetch_atomicmsg(&msg, &mut res_iocs, Some(&res_descs));
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(write_mem, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut ack_mem[..512], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let desc0 = Some(mr.descriptor());
        let expected = vec![2u8; 256];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc0, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc0, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    }
}

#[test]
fn fetch_atomicmsg0() {
    fetch_atomicmsg(true, "fetch_atomicmsg0", false);
}

#[test]
fn fetch_atomicmsg1() {
    fetch_atomicmsg(false, "fetch_atomicmsg0", false);
}

// [TODO Not sure why, but connected endpoints fail with atomic ops
// #[test]
// fn conn_fetch_atomic0() {
//     fetch_atomic(true, "conn_fetch_atomic0", true);
// }

// #[test]
// fn conn_fetch_atomic1() {
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
    let dst_slice = remote_mem_info.slice(..256);
    let (dst_slice0, dst_slice1) = dst_slice.split_at(128);

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

        let buf_iocs = [Ioc::from_slice(buf0), Ioc::from_slice(buf1)];
        let comp_iocs = [Ioc::from_slice(comp0), Ioc::from_slice(comp1)];
        let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
        let buf_descs = [mr.descriptor(), mr.descriptor()];
        let comp_descs = [mr.descriptor(), mr.descriptor()];
        let res_descs = [mr.descriptor(), mr.descriptor()];
        let mut rma_iocs = RemoteMemAddrAtomicVec::new();
        rma_iocs.push(dst_slice0);
        rma_iocs.push(dst_slice1);
        // let rma_ioc0 = RmaIoc::from_slice(&dst_slice0);
        // let rma_ioc1 = RmaIoc::from_slice(&dst_slice1);
        // let rma_iocs = [rma_ioc0, rma_ioc1];

        let msg = if connected {
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
            &msg,
            &comp_iocs,
            &mut res_iocs,
            Some(&comp_descs),
            Some(&res_descs),
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], desc, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut ack_mem[..512], desc, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let expected = vec![2u8; 256];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], desc, false);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], desc, None, false);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    }
}

#[test]
fn compare_atomicmsg0() {
    compare_atomicmsg(true, "compare_atomicmsg0", false);
}

#[test]
fn compare_atomicmsg1() {
    compare_atomicmsg(false, "compare_atomicmsg0", false);
}

// [TODO Not sure why, but connected endpoints fail with atomic ops
// #[test]
// fn conn_compare_atomic0() {
//     compare_atomic(true, "conn_compare_atomic0", true);
// }

// #[test]
// fn conn_compare_atomic1() {
//     compare_atomic(false, "conn_compare_atomic0", true);
// }

fn collective(server: bool, name: &str, connected: bool) -> (Ofi<impl CollCap>, MultiCastGroup) {
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
            ofi.av.as_ref().unwrap(),
            &ofi.mapped_addr.as_ref().unwrap()[0],
            &ofi.mapped_addr.as_ref().unwrap()[0],
            1,
        )
        .count(2)
        .build()
        .unwrap()
    } else {
        AddressVectorSetBuilder::new_from_range(
            ofi.av.as_ref().unwrap(),
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

    let mc = match &ofi.ep {
        MyEndpoint::Connected(ep) => mc
            .join_collective_with_context(ep, libfabric::enums::JoinOptions::new(), &mut ctx)
            .unwrap(),
        MyEndpoint::Connectionless(ep) => mc
            .join_collective_with_context(ep, libfabric::enums::JoinOptions::new(), &mut ctx)
            .unwrap(),
    };

    let join_event;
    loop {
        let event = ofi.eq.read();
        if let Ok(Event::JoinComplete(join)) = event {
            join_event = join;
            break;
        }
        let _ = ofi.cq_type.tx_cq().read(0);
        let _ = ofi.cq_type.rx_cq().read(0);
    }

    (ofi, mc.join_complete(join_event))
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
    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => ep.barrier(&mc).unwrap(),
    }
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

    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => {
            ep.broadcast(
                &mut reg_mem[..],
                Some(&mr.descriptor()),
                &mc,
                &ofi.mapped_addr.as_ref().unwrap()[0],
                CollectiveOptions::new(),
            )
            .unwrap();
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            assert_eq!(reg_mem, expected);
        }
    }
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
    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => {
            ep.alltoall(
                send_buf,
                Some(&mr.descriptor()),
                recv_buf,
                Some(&mr.descriptor()),
                &mc,
                CollectiveOptions::new(),
            )
            .unwrap();
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            assert_eq!(reg_mem, expected);
        }
    }
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
    let half = reg_mem.len() / 2;
    let (send_buf, recv_buf) = reg_mem.split_at_mut(half);
    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => {
            ep.allreduce(
                send_buf,
                Some(&mr.descriptor()),
                recv_buf,
                Some(&mr.descriptor()),
                &mc,
                libfabric::enums::CollAtomicOp::Sum,
                CollectiveOptions::new(),
            )
            .unwrap();
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            assert_eq!(recv_buf, expected);
        }
    }
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
    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => {
            ep.allgather(
                send_buf,
                Some(&mr.descriptor()),
                &mut recv_buf[..1024],
                Some(&mr.descriptor()),
                &mc,
                CollectiveOptions::new(),
            )
            .unwrap();
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(recv_buf, expected);
        }
    }
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
    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => {
            ep.reduce_scatter(
                send_buf,
                Some(&mr.descriptor()),
                &mut recv_buf[..1024],
                Some(&mr.descriptor()),
                &mc,
                libfabric::enums::CollAtomicOp::Sum,
                CollectiveOptions::new(),
            )
            .unwrap();
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(recv_buf[..1024], expected);
        }
    }
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
    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => {
            ep.reduce(
                send_buf,
                Some(&mr.descriptor()),
                &mut recv_buf[..1024],
                Some(&mr.descriptor()),
                &mc,
                &ofi.mapped_addr.as_ref().unwrap()[0],
                libfabric::enums::CollAtomicOp::Sum,
                CollectiveOptions::new(),
            )
            .unwrap();
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(recv_buf[..1024], expected);
        }
    }
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
    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => {
            ep.scatter(
                send_buf,
                Some(&mr.descriptor()),
                &mut recv_buf[..512],
                Some(&mr.descriptor()),
                &mc,
                &ofi.mapped_addr.as_ref().unwrap()[0],
                CollectiveOptions::new(),
            )
            .unwrap();
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(recv_buf[..512], expected);
        }
    }
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
    match &ofi.ep {
        MyEndpoint::Connected(_) => todo!(),
        MyEndpoint::Connectionless(ep) => {
            ep.gather(
                send_buf,
                Some(&mr.descriptor()),
                &mut recv_buf[..1024],
                Some(&mr.descriptor()),
                &mc,
                &ofi.mapped_addr.as_ref().unwrap()[0],
                CollectiveOptions::new(),
            )
            .unwrap();
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(recv_buf[..1024], expected);
        }
    }
}

#[test]
fn gather0() {
    gather(true, "gather0", false);
}

#[test]
fn gather1() {
    gather(false, "gather0", false);
}
