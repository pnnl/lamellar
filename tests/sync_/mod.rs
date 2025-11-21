#[cfg(test)]
pub mod tests {
use libfabric::av::NoBlockAddressVector;
use libfabric::cntr::Counter;
use libfabric::cntr::CounterBuilder;
use libfabric::cntr::ReadCntr;
use libfabric::cntr::WaitCntr;
use libfabric::comm::atomic::AtomicCASRemoteMemAddrSliceEp;
use libfabric::comm::atomic::AtomicFetchRemoteMemAddrSliceEp;
use libfabric::comm::atomic::AtomicWriteRemoteMemAddrSliceEp;
use libfabric::comm::atomic::ConnectedAtomicCASRemoteMemAddrSliceEp;
use libfabric::comm::atomic::ConnectedAtomicFetchRemoteMemAddrSliceEp;
use libfabric::comm::atomic::ConnectedAtomicWriteRemoteMemAddrSliceEp;
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
use libfabric::eq::Event;
use libfabric::eq::EventQueue;
use libfabric::eq::ReadEq;
use libfabric::info::InfoBuilder;
use libfabric::mr::MemoryRegionSlice;
use libfabric::mr::MemoryRegionSliceMut;
use libfabric::AsFiType;
use libfabric::RemoteMemAddrSlice;
use libfabric::RemoteMemAddrSliceMut;
use std::cell::RefCell;
use std::ops::Range;
use std::time::Instant;
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
    cq::{CompletionQueue, CompletionQueueBuilder, ReadCq, WaitCq},
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
        AtomicDefaultCap, Caps, CollCap, MsgDefaultCap, RmaDefaultCap, TagDefaultCap,
    },
    iovec::{IoVec, IoVecMut},
    mr::{
        EpBindingMemoryRegion, MemoryRegion, MemoryRegionBuilder, MemoryRegionDesc,
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
    CqCaps, EqCaps, CntrCaps, MappedAddress, MemAddressInfo, MyRc, RemoteMemAddressInfo,
};

pub type SpinCq = libfabric::cq_caps_type!(CqCaps::WAIT);
pub type WaitableEq = libfabric::eq_caps_type!(EqCaps::WAIT);
pub type DefaultCntr = libfabric::cntr_caps_type!(CntrCaps::WAIT);

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
    pub mr: RefCell<Option<MemoryRegion>>,
    // pub remote_key: Option<MappedMemoryRegionKey>,
    // pub remote_mem_addr: Option<(u64, u64)>,
    pub remote_mem_info: Option<RefCell<RemoteMemAddressInfo>>,
    pub domain: Domain,
    pub cq_type: CqType,
    pub ep: MyEndpoint<I>,
    pub tx_context: MyTxContext<I>,
    pub rx_context: MyRxContext<I>,
    pub reg_mem: RefCell<Vec<u8>>,
    pub mapped_addr: Option<Vec<MyRc<MappedAddress>>>,
    pub av: Option<NoBlockAddressVector>,
    pub eq: EventQueue<EqOptions>,
    pub ctx: RefCell<libfabric::Context>,
    pub use_shared_cqs: bool,
    pub use_cntrs_for_completion: CntrsCompMeth,
    pub use_cqs_for_completion: CqsCompMeth,
    pub server: bool,
    pub tx_cntr: Option<Counter<DefaultCntr>>,
    pub rx_cntr: Option<Counter<DefaultCntr>>,
    pub tx_pending_cnt: RefCell<usize>,
    pub tx_complete_cnt: RefCell<usize>,
    pub rx_pending_cnt: RefCell<usize>,
    pub rx_complete_cnt: RefCell<usize>,
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
            // panic!("Should not read anything")
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
        config: TestConfig<I>,
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

        let mut tx_pending_cnt: usize = 0;
        let mut tx_complete_cnt: usize = 0;
        let mut rx_pending_cnt: usize = 0;
        let mut rx_complete_cnt: usize = 0;
        let mut reg_mem = vec![0u8; config.buf_size];
        let selective_comp = matches!(config.use_cqs_for_completion, CqsCompMeth::None);
        let (info_entry, ep, tx_context, rx_context, mapped_addr, av, eq, tx_cntr, rx_cntr) = {
            let (info_entry, eq) = if matches!(ep_type, EndpointType::Msg) {
                let eq = EventQueueBuilder::new(&fabric).build().unwrap();
                if config.server {
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

            cq_type = if config.use_shared_cqs {
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
                    .build_with_separate_cqs(&domain, tx_cq, selective_comp, rx_cq, selective_comp)
                    .unwrap(),

                CqType::Shared(ref scq) => ep_builder
                    .build_with_shared_cq(&domain, scq, selective_comp)
                    .unwrap(),
            };
            let (tx_cntr, rx_cntr) = if matches!(config.use_cntrs_for_completion, CntrsCompMeth::None) {
                (None,None)
            }
            else {
                (
                    Some(CounterBuilder::new()
                        .build(&domain)
                        .unwrap())
                    ,
                    Some(CounterBuilder::new()
                        .build(&domain)
                        .unwrap())
                )
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
                    if !matches!(config.use_cntrs_for_completion, CntrsCompMeth::None) {
                        ep.bind_cntr()
                            .send()
                            .write()
                            .cntr(&tx_cntr.as_ref().unwrap())
                            .unwrap();
                        
                        ep.bind_cntr()
                            .recv()
                            .read()
                            .cntr(&rx_cntr.as_ref().unwrap())
                            .unwrap();
                    }
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

                        if !matches!(config.use_cqs_for_completion, CqsCompMeth::None) {
                            cq_type.tx_cq().sread(1, -1).unwrap();
                        }
                        else if !matches!(config.use_cntrs_for_completion, CntrsCompMeth::None){
                            tx_cntr.as_ref().unwrap().wait(1, -1).unwrap();
                        }
                        else {
                            panic!("One of Completion Queues or Counters needs to be enabled");
                        }
                        // ep.recv(std::slice::from_mut(&mut ack), &mut default_desc()).unwrap();
                        post!(
                            recv_from_any,
                            ft_progress,
                            cq_type.rx_cq(),
                            ep,
                            std::slice::from_mut(&mut reg_mem[0]),
                            None
                        );

                        if !matches!(config.use_cqs_for_completion, CqsCompMeth::None) {
                            cq_type.rx_cq().sread(1, -1).unwrap();
                        }
                        else if !matches!(config.use_cntrs_for_completion, CntrsCompMeth::None){
                            rx_cntr.as_ref().unwrap().wait(1, -1).unwrap();
                        }
                        else {
                            panic!("One of Completion Queues or Counters needs to be enabled");
                        }
                        tx_pending_cnt += 1;
                        rx_pending_cnt += 1;
                        tx_complete_cnt += 1;
                        rx_complete_cnt += 1;
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

                        if !matches!(config.use_cqs_for_completion, CqsCompMeth::None) {
                            cq_type.rx_cq().sread(1, -1).unwrap();
                        }
                        else if !matches!(config.use_cntrs_for_completion, CntrsCompMeth::None){
                            rx_cntr.as_ref().unwrap().wait(1, -1).unwrap();
                        }
                        else {
                            panic!("One of Completion Queues or Counters needs to be enabled");
                        }
                        // cq_type.rx_cq().sread(1, -1).unwrap();
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
                        if !matches!(config.use_cqs_for_completion, CqsCompMeth::None) {
                            cq_type.tx_cq().sread(1, -1).unwrap();
                        }
                        else if !matches!(config.use_cntrs_for_completion, CntrsCompMeth::None){
                            tx_cntr.as_ref().unwrap().wait(1, -1).unwrap();
                        }
                        else {
                            panic!("One of Completion Queues or Counters needs to be enabled");
                        }
                        // cq_type.tx_cq().sread(1, -1).unwrap();
                        tx_pending_cnt += 1;
                        rx_pending_cnt += 1;
                        tx_complete_cnt += 1;
                        rx_complete_cnt += 1;
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
                        tx_cntr,
                        rx_cntr
                    )
                }
                Endpoint::ConnectionOriented(ep) => {
                    let eq = eq.unwrap();
                    if !matches!(config.use_cntrs_for_completion, CntrsCompMeth::None) {
                        ep.bind_cntr()
                            .send()
                            .write()
                            .cntr(&tx_cntr.as_ref().unwrap())
                            .unwrap();
                        
                        ep.bind_cntr()
                            .recv()
                            .read()
                            .cntr(&rx_cntr.as_ref().unwrap())
                            .unwrap();
                    }
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

                    (
                        info_entry,
                        MyEndpoint::Connected(ep),
                        MyTxContext::Connected(tx_context),
                        MyRxContext::Connected(rx_context),
                        None,
                        None,
                        eq,
                        tx_cntr,
                        rx_cntr
                    )
                }
            }
        };
        if config.server && !config.name.is_empty() {

            unsafe { std::env::remove_var(&config.name) };
        }
        let ctx=   RefCell::new(info_entry.allocate_context());
        Ok(Self {
            info_entry,
            mapped_addr,
            mr: RefCell::new(mr),
            remote_mem_info: None,
            cq_type,
            domain,
            ep,
            tx_context,
            rx_context,
            reg_mem: RefCell::new(reg_mem),
            av,
            eq,
            ctx,
            use_cntrs_for_completion: config.use_cntrs_for_completion,
            use_cqs_for_completion: config.use_cqs_for_completion,
            use_shared_cqs: config.use_shared_cqs,
            server: config.server,
            tx_cntr,
            rx_cntr,
            tx_pending_cnt: RefCell::new(tx_pending_cnt),
            tx_complete_cnt: RefCell::new(tx_complete_cnt),
            rx_pending_cnt: RefCell::new(rx_pending_cnt),
            rx_complete_cnt: RefCell::new(rx_complete_cnt),
        })
    }
}


impl<I> Ofi<I> {
    pub fn check_and_progress(&self, err: Result<(), libfabric::error::Error>) -> bool {
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



impl<I> Ofi<I> {
    pub fn wait_tx(&self, to_wait: usize) {
        match self.use_cqs_for_completion {
            CqsCompMeth::None => {},
            CqsCompMeth::Spin => self.spin_wait_cq_tx(to_wait),
            CqsCompMeth::Sread | CqsCompMeth::Yield | CqsCompMeth::WaitFd | CqsCompMeth::WaitSet => {self.cq_type.tx_cq().sread(to_wait, -1).unwrap();},
        }

        match self.use_cntrs_for_completion {
            CntrsCompMeth::None => {},
            CntrsCompMeth::Spin => self.spin_wait_cntr_tx(to_wait),
            CntrsCompMeth::Sread => todo!(),
            CntrsCompMeth::Yield => todo!(),
        }
        *self.tx_complete_cnt.borrow_mut() += to_wait;
    }

    pub fn wait_rx(&self, to_wait: usize) {
        match self.use_cqs_for_completion {
            CqsCompMeth::None => {},
            CqsCompMeth::Spin => self.spin_wait_cq_rx(to_wait),
            CqsCompMeth::Sread | CqsCompMeth::Yield | CqsCompMeth::WaitFd | CqsCompMeth::WaitSet => {self.cq_type.rx_cq().sread(to_wait, -1).unwrap();},
        }

        match self.use_cntrs_for_completion {
            CntrsCompMeth::None => {},
            CntrsCompMeth::Spin => self.spin_wait_cntr_rx(to_wait),
            CntrsCompMeth::Sread | CntrsCompMeth::Yield => {self.rx_cntr.as_ref().unwrap().wait((*self.tx_complete_cnt.borrow() + to_wait) as u64, -1).unwrap();},
        }

        *self.rx_complete_cnt.borrow_mut() += to_wait;
    }

    pub fn spin_wait_cq_tx(&self, to_wait: usize)  {
        for _ in 0..to_wait {
            loop {
                if let Ok(_) = self.cq_type.tx_cq().read(1) {
                    break
                }
            }
        }
    } 

    pub fn spin_wait_cntr_tx(&self, to_wait: usize)  {
        let mut cnt = self.tx_cntr.as_ref().unwrap().read();
        while *self.tx_complete_cnt.borrow() + to_wait > cnt as usize {
            cnt = self.tx_cntr.as_ref().unwrap().read();
        }
    } 
    
    pub fn spin_wait_cq_rx(&self, to_wait: usize)  {
        for _ in 0..to_wait {
            loop {
                if let Ok(_) = self.cq_type.rx_cq().read(1) {
                    break
                }
            }
        }
    } 

    pub fn spin_wait_cntr_rx(&self, to_wait: usize)  {
        let mut cnt = self.rx_cntr.as_ref().unwrap().read();
        while *self.rx_complete_cnt.borrow() + to_wait > cnt as usize {
            cnt = self.rx_cntr.as_ref().unwrap().read();
        }
    } 
    
}

pub enum CqsCompMeth {
    None,
    Spin,
    Sread,
    WaitSet,
    WaitFd,
    Yield,
}

pub enum CntrsCompMeth {
    None,
    Spin,
    Sread,
    Yield,
}


pub struct TestConfigBuilder<I> {
    info_builder: InfoBuilder<I>,
    pub use_shared_cqs: bool,
    pub use_cntrs_for_completion: CntrsCompMeth,
    pub use_cqs_for_completion: CqsCompMeth,
    pub buf_size: usize,
    pub name: String,
    server: bool,
}

pub struct TestConfig<I> {
    info_entry: InfoEntry<I>,
    use_shared_cqs: bool,
    use_cntrs_for_completion: CntrsCompMeth,
    use_cqs_for_completion: CqsCompMeth,
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
            use_cntrs_for_completion: CntrsCompMeth::None,
            use_cqs_for_completion: CqsCompMeth::Spin,
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
        if matches!(self.use_cntrs_for_completion, CntrsCompMeth::None) && matches!(self.use_cqs_for_completion, CqsCompMeth::None) {
            panic!("Either Counters or Completions Queues need to be chosen");
        }
        TestConfig {
            info_entry,
            use_shared_cqs:  self.use_shared_cqs,
            use_cntrs_for_completion:  self.use_cntrs_for_completion,
            use_cqs_for_completion:  self.use_cqs_for_completion,
            buf_size: self.buf_size,
            server: self.server,
            name: self.name,
        }
    }
}
pub fn get_ip(user_ip: Option<&str>) -> String {
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
    ip
}

pub fn handshake<I: Caps + MsgDefaultCap + 'static>(
    user_ip: Option<&str>,
    server: bool,
    name: &str,
    caps: Option<I>,
) -> Ofi<I> {
    let caps = caps.unwrap();
    let ep_type: EndpointType = EndpointType::Msg;

    let ip = get_ip(user_ip);
    
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
    let ip = get_ip(user_ip);

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




fn conn_send<T>(
    sender: &impl ConnectedSendEp,
    buf: &[T],
    desc: Option<MemoryRegionDesc>,
    data: Option<u64>,
    max_inject_size: usize,
     tx_complete_cnt: &RefCell<usize>,
) -> Result<(), libfabric::error::Error> {
    *tx_complete_cnt.borrow_mut() +=1;
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
    tx_complete_cnt: &RefCell<usize>,
) -> Result<(), libfabric::error::Error> {
    if mr_slice.as_slice().len() <= max_inject_size {
        *tx_complete_cnt.borrow_mut() +=1;
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
    tx_complete_cnt: &RefCell<usize>,
) -> Result<(), libfabric::error::Error> {
    if buf.len() <= max_inject_size {
        *tx_complete_cnt.borrow_mut() +=1;
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
    tx_complete_cnt: &RefCell<usize>,
) -> Result<(), libfabric::error::Error> {
    if mr_slice.as_slice().len() <= max_inject_size {
        *tx_complete_cnt.borrow_mut() +=1;
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
     tx_complete_cnt: &RefCell<usize>,
) -> Result<(), libfabric::error::Error> {
    if buf.len() <= max_inject_size {
        *tx_complete_cnt.borrow_mut() += 1;
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
     tx_complete_cnt: &RefCell<usize>,
) -> Result<(), libfabric::error::Error> {
    if mr_slice.as_slice().len() <= max_inject_size {
        *tx_complete_cnt.borrow_mut() += 1;

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
     tx_complete_cnt: &RefCell<usize>,
) -> Result<(), libfabric::error::Error> {
    if buf.len() <= max_inject_size {
        *tx_complete_cnt.borrow_mut() += 1;

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
     tx_complete_cnt: &RefCell<usize>,
) -> Result<(), libfabric::error::Error> {
    if mr_slice.as_slice().len() <= max_inject_size {
        *tx_complete_cnt.borrow_mut() += 1;

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


impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn send(
        &self,
        range: Range<usize>,
        data: Option<u64>,
        use_context: bool,
    ) {
        let borrow = &self.reg_mem.borrow();
        let buf = &borrow[range];
        let mr = self.mr.borrow();
        let desc = mr.as_ref().map_or(None, |mr| Some(mr.descriptor()));
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
                        &self.tx_complete_cnt
                    ),
                    MyEndpoint::Connected(ep) => {
                        conn_send(ep, buf, desc, data, self.info_entry.tx_attr().inject_size(), &self.tx_complete_cnt)
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
                        &self.tx_complete_cnt,
                    ),
                    MyTxContext::Connected(tx_context) => conn_send(
                        tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        buf,
                        desc,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                        &self.tx_complete_cnt,
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
        *self.tx_pending_cnt.borrow_mut() += 1;
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
                        &self.tx_complete_cnt,
                    ),
                    MyEndpoint::Connected(ep) => {
                        conn_send_mr(ep, mr_slice, data, self.info_entry.tx_attr().inject_size(),&self.tx_complete_cnt)
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
                        &self.tx_complete_cnt,
                    ),
                    MyTxContext::Connected(tx_context) => conn_send_mr(
                        tx_context.as_ref().expect("Tx/Rx Contexts not supported"),
                        mr_slice,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                        &self.tx_complete_cnt,
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }

        }
        *self.tx_pending_cnt.borrow_mut() += 1;
    }

    pub fn send_with_context(
        &self,
        range: Range<usize>,
        data: Option<u64>,
    ) {
        let borrow = &self.reg_mem.borrow();
        let buf = &borrow[range];
        let mr = self.mr.borrow();

        let desc = mr.as_ref().map_or(None, |mr| Some(mr.descriptor()));
        let mut context = self.ctx.borrow_mut();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        *self.tx_complete_cnt.borrow_mut() +=1;
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
                            &mut context,
                        )
                    } else {
                        ep.send_to_with_context(
                            buf,
                            desc,
                            &self.mapped_addr.as_ref().unwrap()[1],
                            &mut context,
                        )
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        *self.tx_complete_cnt.borrow_mut() +=1;
                        if let Some(data) = data {
                            ep.injectdata(buf, data)
                        } else {
                            ep.inject(buf)
                        }
                    } else if let Some(data) = data {
                        ep.senddata_with_context(buf, desc, data, &mut context)
                    } else {
                        ep.send_with_context(buf, desc, &mut context)
                    }
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
        *self.tx_pending_cnt.borrow_mut() += 1;
    }

    pub fn sendv(&self, iov: &[IoVec], desc: Option<&[MemoryRegionDesc]>, use_context: bool) {
        loop {
            let err = if !use_context {
                match &self.ep {
                    MyEndpoint::Connectionless(ep) => {
                        connless_sendv(ep, iov, desc, &self.mapped_addr.as_ref().unwrap()[1])
                    }
                    MyEndpoint::Connected(ep) => conn_sendv(ep, iov, desc,),
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
        *self.tx_pending_cnt.borrow_mut() += 1;
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
        *self.rx_pending_cnt.borrow_mut() += 1;
    }

    pub fn recv(&self, range: Range<usize>, use_context: bool) {
        let borrow = &mut self.reg_mem.borrow_mut();
        let buf = &mut borrow[range];
        let mr = self.mr.borrow();

        let desc = mr.as_ref().map_or(None, |mr| Some(mr.descriptor()));
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
        *self.rx_pending_cnt.borrow_mut() += 1;
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
        *self.rx_pending_cnt.borrow_mut() += 1;
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
        *self.tx_pending_cnt.borrow_mut() += 1;
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
        *self.rx_pending_cnt.borrow_mut() += 1;
    }

    pub fn exchange_addresses(&mut self) -> Address {
        let epname = match self.ep {
            MyEndpoint::Connected(ref ep) => ep.getname(),
            MyEndpoint::Connectionless(ref ep) => ep.getname(),
        };

        let address_bytes = epname.unwrap().as_bytes().to_vec();
        self.reg_mem.borrow_mut().copy_from_slice(&address_bytes);

        self.send(0..address_bytes.len(), None, false);
        self.recv(0..address_bytes.len(), false);
        self.wait_rx(1);
        
        unsafe { Address::from_bytes(&self.reg_mem.borrow()[0..address_bytes.len()]) }
    }

    pub fn exchange_keys(&mut self) {
        let mr = self.mr.borrow();
        let key = mr.as_ref().unwrap().key().unwrap();
        let mem_info = libfabric::MemAddressInfo::from_slice(&self.reg_mem.borrow(), 0, &key, &self.info_entry);
        let mem_bytes = mem_info.to_bytes();
        println!("Local addr: {:?}, size: {}", self.reg_mem.borrow().as_ptr(), self.reg_mem.borrow().len());
        let len = mem_bytes.len();
        println!("Mem info bytes len: {}", len);
        self.reg_mem.borrow_mut()[..len].copy_from_slice(mem_bytes);
        println!("Sending : {:?}", &self.reg_mem.borrow()[..len]);
        self.send(
            0..len,
            None,
            false,
        );
        self.recv(
            len..2*len,
            false,
        );
        
        // self.cq_type.rx_cq().sread(1, -1).unwrap();
        self.wait_rx(1);
        let mem_info = unsafe { MemAddressInfo::from_bytes(&self.reg_mem.borrow()[len..2*len]) };
        let remote_mem_info = mem_info.into_remote_info(&self.domain).unwrap();
        println!("Received : {:?}", &self.reg_mem.borrow()[len..2*len]);
        println!("Remote addr: {:?}, size: {}", remote_mem_info.mem_address().as_ptr(), remote_mem_info.mem_len());
        self.send(
            0..len,
            None,
            false,
        );
        self.recv(
            len..2*len,
            false,
        );
        self.wait_rx(1);

        self.remote_mem_info = Some(RefCell::new(remote_mem_info));
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
        let write_slice = remote_mem_info.slice_mut(dest_addr..dest_addr + buf.len());
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));
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
        *self.tx_pending_cnt.borrow_mut() += 1;
    }

    pub fn read(
        &self, 
        range: Range<usize>, 
        dest_addr: usize, 
    ) {
        let buf = &mut self.reg_mem.borrow_mut()[range];
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));
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
        *self.rx_pending_cnt.borrow_mut() += 1;
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
        *self.tx_pending_cnt.borrow_mut() += 1;
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
        *self.rx_pending_cnt.borrow_mut() += 1;
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
        *self.tx_pending_cnt.borrow_mut() += 1;
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
        *self.rx_pending_cnt.borrow_mut() += 1;
    }
}



impl<I: TagDefaultCap> Ofi<I> {
    pub fn tsend(
        &self,
        range: Range<usize>,
        tag: u64,
        data: Option<u64>,
        use_context: bool,
    ) {
        let buf = &self.reg_mem.borrow()[range];
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));
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
                        &self.tx_complete_cnt,
                    ),
                    MyEndpoint::Connected(ep) => conn_tsend(
                        ep,
                        buf,
                        desc,
                        tag,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                        &self.tx_complete_cnt,
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
                        &self.tx_complete_cnt,
                    ),
                    MyTxContext::Connected(tx_context) => conn_tsend(
                        tx_context.as_ref().unwrap(),
                        buf,
                        desc,
                        tag,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                        &self.tx_complete_cnt,
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
        *self.tx_pending_cnt.borrow_mut() += 1;
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
                        &self.tx_complete_cnt,
                    ),
                    MyEndpoint::Connected(ep) => conn_tsend_mr(
                        ep,
                        mr_slice,
                        tag,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                        &self.tx_complete_cnt,
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
                        &self.tx_complete_cnt,
                    ),
                    MyTxContext::Connected(tx_context) => conn_tsend_mr(
                        tx_context.as_ref().unwrap(),
                        mr_slice,
                        tag,
                        data,
                        self.info_entry.tx_attr().inject_size(),
                        &self.tx_complete_cnt,
                    ),
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
        *self.tx_pending_cnt.borrow_mut() += 1;
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
        *self.tx_pending_cnt.borrow_mut() += 1;
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
        *self.rx_pending_cnt.borrow_mut() += 1;
    }

    pub fn trecv(
        &self,
        range: Range<usize>,
        tag: u64,
        use_context: bool,
    ) {
        let buf = &mut self.reg_mem.borrow_mut()[range];
        let borrow = self.mr.borrow();
        let desc = borrow.as_ref().map_or(None, |mr| Some(mr.descriptor()));
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
        *self.rx_pending_cnt.borrow_mut() += 1;
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
        *self.rx_pending_cnt.borrow_mut() += 1;
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
        *self.tx_pending_cnt.borrow_mut() += 1;
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
        *self.rx_pending_cnt.borrow_mut() += 1;
    }
}


#[cfg(feature = "threading-fid")]
pub trait IsSyncSend: Send + Sync {}

#[cfg(feature = "threading-fid")]
impl<I> IsSyncSend for Ofi<I> {}
pub const DEFAULT_BUF_SIZE: usize = 1024*1024;

fn get_atomic_op<T, A>(op: libfabric::enums::AtomicOp) -> unsafe fn(
    &A,
    &[T],
    Option<MemoryRegionDesc>,
    &MappedAddress,
    &RemoteMemAddrSliceMut<T>,
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: AtomicWriteEp, 
{

    match op {
        AtomicOp::Min => AtomicWriteRemoteMemAddrSliceEp::atomic_min_mr_slice_to,
        AtomicOp::Max => AtomicWriteRemoteMemAddrSliceEp::atomic_max_mr_slice_to,
        AtomicOp::Sum => AtomicWriteRemoteMemAddrSliceEp::atomic_sum_mr_slice_to,
        AtomicOp::Prod => AtomicWriteRemoteMemAddrSliceEp::atomic_prod_mr_slice_to,
        // AtomicOp::Lor => AtomicWriteRemoteMemAddrSliceEp::atomic_lor_mr_slice_to,
        // AtomicOp::Land => AtomicWriteRemoteMemAddrSliceEp::atomic_land_mr_slice_to,
        AtomicOp::Bor => AtomicWriteRemoteMemAddrSliceEp::atomic_bor_mr_slice_to,
        AtomicOp::Band => AtomicWriteRemoteMemAddrSliceEp::atomic_band_mr_slice_to,
        // AtomicOp::Lxor => AtomicWriteRemoteMemAddrSliceEp::atomic_lxor_mr_slice_to,
        AtomicOp::Bxor => AtomicWriteRemoteMemAddrSliceEp::atomic_bxor_mr_slice_to,
        AtomicOp::AtomicWrite => AtomicWriteRemoteMemAddrSliceEp::atomic_write_mr_slice_to,
        _ => todo!(),
    }
}

fn get_atomic_bool_op<A>(op: libfabric::enums::AtomicOp) -> unsafe fn(
    &A,
    &[bool],
    Option<MemoryRegionDesc>,
    &MappedAddress,
    &RemoteMemAddrSliceMut<bool>,
) 
-> Result<(), Error>
where
    A: AtomicWriteEp, 
{

    match op {
        AtomicOp::Lor => AtomicWriteRemoteMemAddrSliceEp::atomic_lor_mr_slice_to,
        AtomicOp::Land => AtomicWriteRemoteMemAddrSliceEp::atomic_land_mr_slice_to,
        AtomicOp::Lxor => AtomicWriteRemoteMemAddrSliceEp::atomic_lxor_mr_slice_to,
        _ => todo!(),
    }
}

fn get_atomicv_op<T, A>(op: libfabric::enums::AtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<T>], 
    desc: Option<&[MemoryRegionDesc<'_>]>,
    &MappedAddress,
    &RemoteMemAddrSliceMut<T>,
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: AtomicWriteEp, 
{

    match op {
        AtomicOp::Min => AtomicWriteRemoteMemAddrSliceEp::atomicv_min_mr_slice_to,
        AtomicOp::Max => AtomicWriteRemoteMemAddrSliceEp::atomicv_max_mr_slice_to,
        AtomicOp::Sum => AtomicWriteRemoteMemAddrSliceEp::atomicv_sum_mr_slice_to,
        AtomicOp::Prod => AtomicWriteRemoteMemAddrSliceEp::atomicv_prod_mr_slice_to,
        // AtomicOp::Lor => AtomicWriteRemoteMemAddrSliceEp::atomicv_lor_mr_slice_to,
        // AtomicOp::Land => AtomicWriteRemoteMemAddrSliceEp::atomicv_land_mr_slice_to,
        AtomicOp::Bor => AtomicWriteRemoteMemAddrSliceEp::atomicv_bor_mr_slice_to,
        AtomicOp::Band => AtomicWriteRemoteMemAddrSliceEp::atomicv_band_mr_slice_to,
        // AtomicOp::Lxor => AtomicWriteRemoteMemAddrSliceEp::atomicv_lxor_mr_slice_to,
        AtomicOp::Bxor => AtomicWriteRemoteMemAddrSliceEp::atomicv_bxor_mr_slice_to,
        AtomicOp::AtomicWrite => AtomicWriteRemoteMemAddrSliceEp::atomicv_write_mr_slice_to,
        _ => todo!(),
    }
}
fn get_atomicv_bool_op<A>(op: libfabric::enums::AtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<bool>], 
    desc: Option<&[MemoryRegionDesc<'_>]>,
    &MappedAddress,
    &RemoteMemAddrSliceMut<bool>,
) 
-> Result<(), Error>
where
    A: AtomicWriteEp, 
{

    match op {
        AtomicOp::Lor => AtomicWriteRemoteMemAddrSliceEp::atomicv_lor_mr_slice_to,
        AtomicOp::Land => AtomicWriteRemoteMemAddrSliceEp::atomicv_land_mr_slice_to,
        AtomicOp::Lxor => AtomicWriteRemoteMemAddrSliceEp::atomicv_lxor_mr_slice_to,
        _ => todo!(),
    }
}

fn get_conn_atomicv_bool_op<A>(op: libfabric::enums::AtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<bool>], 
    desc: Option<&[MemoryRegionDesc<'_>]>,
    &RemoteMemAddrSliceMut<bool>,
) 
-> Result<(), Error>
where
    A: ConnectedAtomicWriteEp, 
{

    match op {
        AtomicOp::Lor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_lor_mr_slice,
        AtomicOp::Land => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_land_mr_slice,
        AtomicOp::Lxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_lxor_mr_slice,
        _ => todo!(),
    }
}

fn get_conn_atomicv_op<T, A>(op: libfabric::enums::AtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<T>], 
    desc: Option<&[MemoryRegionDesc<'_>]>,
    &RemoteMemAddrSliceMut<T>,
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: ConnectedAtomicWriteEp, 
{

    match op {
        AtomicOp::Min => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_min_mr_slice,
        AtomicOp::Max => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_max_mr_slice,
        AtomicOp::Sum => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_sum_mr_slice,
        AtomicOp::Prod => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_prod_mr_slice,
        // AtomicOp::Lor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_lor_mr_slice,
        // AtomicOp::Land => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_land_mr_slice,
        AtomicOp::Bor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_bor_mr_slice,
        AtomicOp::Band => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_band_mr_slice,
        // AtomicOp::Lxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_lxor_mr_slice,
        AtomicOp::Bxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_bxor_mr_slice,
        AtomicOp::AtomicWrite => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomicv_write_mr_slice,
        _ => todo!(),
    }
}

fn get_conn_atomic_op<T, A>(op: libfabric::enums::AtomicOp) -> unsafe fn(
    &A,
    &[T],
    Option<MemoryRegionDesc>,
    &RemoteMemAddrSliceMut<T>,
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: ConnectedAtomicWriteEp, 
{

    match op {
        AtomicOp::Min => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_min_mr_slice,
        AtomicOp::Max => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_max_mr_slice,
        AtomicOp::Sum => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_sum_mr_slice,
        AtomicOp::Prod => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_prod_mr_slice,
        // AtomicOp::Lor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_lor_mr_slice,
        // AtomicOp::Land => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_land_mr_slice,
        AtomicOp::Bor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_bor_mr_slice,
        AtomicOp::Band => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_band_mr_slice,
        // AtomicOp::Lxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_lxor_mr_slice,
        AtomicOp::Bxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_bxor_mr_slice,
        AtomicOp::AtomicWrite => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_write_mr_slice,
        _ => todo!(),
    }
}

fn get_conn_atomic_bool_op<A>(op: libfabric::enums::AtomicOp) -> unsafe fn(
    &A,
    &[bool],
    Option<MemoryRegionDesc>,
    &RemoteMemAddrSliceMut<bool>,
) 
-> Result<(), Error>
where
    A: ConnectedAtomicWriteEp, 
{

    match op {
        AtomicOp::Lor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_lor_mr_slice,
        AtomicOp::Land => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_land_mr_slice,
        AtomicOp::Lxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_lxor_mr_slice,
        _ => todo!(),
    }
}

fn get_atomic_inject_op<T, A>(
    op: libfabric::enums::AtomicOp,
) -> unsafe fn(
    &A,
    &[T],
    &MappedAddress,
    &RemoteMemAddrSliceMut<T>,
) -> Result<(), Error>
where
    T: AsFiType,
    A: AtomicWriteEp,
{
    match op {
        AtomicOp::Min => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_min_mr_slice_to,
        AtomicOp::Max => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_max_mr_slice_to,
        AtomicOp::Sum => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_sum_mr_slice_to,
        AtomicOp::Prod => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_prod_mr_slice_to,
        // AtomicOp::Lor => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_lor_mr_slice_to,
        // AtomicOp::Land => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_land_mr_slice_to,
        AtomicOp::Bor => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_bor_mr_slice_to,
        AtomicOp::Band => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_band_mr_slice_to,
        // AtomicOp::Lxor => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_lxor_mr_slice_to,
        AtomicOp::Bxor => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_bxor_mr_slice_to,
        AtomicOp::AtomicWrite => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_write_mr_slice_to,
        _ => todo!(),
    }
}

fn get_atomic_inject_bool_op<A>(
    op: libfabric::enums::AtomicOp,
) -> unsafe fn(
    &A,
    &[bool],
    &MappedAddress,
    &RemoteMemAddrSliceMut<bool>,
) -> Result<(), Error>
where
    A: AtomicWriteEp,
{
    match op {
        AtomicOp::Lor => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_lor_mr_slice_to,
        AtomicOp::Land => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_land_mr_slice_to,
        AtomicOp::Lxor => AtomicWriteRemoteMemAddrSliceEp::atomic_inject_lxor_mr_slice_to,
        _ => todo!(),
    }
}

fn get_conn_atomic_inject_op<T, A>(
    op: libfabric::enums::AtomicOp,
) -> unsafe fn(
    &A,
    &[T],
    &RemoteMemAddrSliceMut<T>,
) -> Result<(), Error>
where
    T: AsFiType,
    A: ConnectedAtomicWriteEp,
{
    match op {
        AtomicOp::Min => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_min_mr_slice,
        AtomicOp::Max => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_max_mr_slice,
        AtomicOp::Sum => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_sum_mr_slice,
        AtomicOp::Prod => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_prod_mr_slice,
        // AtomicOp::Lor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lor_mr_slice,
        // AtomicOp::Land => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_land_mr_slice,
        AtomicOp::Bor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_bor_mr_slice,
        AtomicOp::Band => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_band_mr_slice,
        // AtomicOp::Lxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lxor_mr_slice,
        AtomicOp::Bxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_bxor_mr_slice,
        AtomicOp::AtomicWrite => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_write_mr_slice,
        _ => todo!(),
    }
}

fn get_conn_atomic_inject_bool_op<A>(
    op: libfabric::enums::AtomicOp,
) -> unsafe fn(
    &A,
    &[bool],
    &RemoteMemAddrSliceMut<bool>,
) -> Result<(), Error>
where
    A: ConnectedAtomicWriteEp,
{
    match op {
        AtomicOp::Lor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lor_mr_slice,
        AtomicOp::Land => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_land_mr_slice,
        AtomicOp::Lxor => ConnectedAtomicWriteRemoteMemAddrSliceEp::atomic_inject_lxor_mr_slice,
        _ => todo!(),
    }
}

fn get_atomic_fetch_op<T, A>(op: libfabric::enums::FetchAtomicOp) -> unsafe fn(
    &A,
    &[T],
    Option<MemoryRegionDesc>,
    &mut [T],
    Option<MemoryRegionDesc>,
    &MappedAddress,
    &RemoteMemAddrSlice<T>,
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: AtomicFetchEp, 
{

    match op {
        FetchAtomicOp::Min => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_min_mr_slice_from,
        FetchAtomicOp::Max => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_max_mr_slice_from,
        FetchAtomicOp::Sum => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_sum_mr_slice_from,
        FetchAtomicOp::Prod => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_prod_mr_slice_from,
        FetchAtomicOp::Bor => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_bor_mr_slice_from,
        FetchAtomicOp::Band => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_band_mr_slice_from,
        FetchAtomicOp::Bxor => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_bxor_mr_slice_from,
        FetchAtomicOp::AtomicWrite => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_write_mr_slice_from,
        FetchAtomicOp::AtomicRead => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_read_mr_slice_from,
        // FetchAtomicOp::Lxor => AtomicFetchRemoteMemAddrSliceEp::atomic_mr_slice_lxor_to,
        _ => todo!(),
    }
}

fn get_atomic_fetch_bool_op<A>(op: libfabric::enums::FetchAtomicOp) -> unsafe fn(
    &A,
    &[bool],
    Option<MemoryRegionDesc>,
    &mut [bool],
    Option<MemoryRegionDesc>,
    &MappedAddress,
    &RemoteMemAddrSlice<bool>,
) 
-> Result<(), Error>
where
    A: AtomicFetchEp, 
{

    match op {
        FetchAtomicOp::Lor => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_lor_mr_slice_from,
        FetchAtomicOp::Lxor => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_lxor_mr_slice_from,
        FetchAtomicOp::Land => AtomicFetchRemoteMemAddrSliceEp::fetch_atomic_land_mr_slice_from,
        _ => todo!(),
    }
}

fn get_atomicv_fetch_op<T, A>(op: libfabric::enums::FetchAtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<T>], 
    desc: Option<&[MemoryRegionDesc<'_>]>,
    resioc: &mut [libfabric::iovec::IocMut<T>], 
    res_desc: Option<&[MemoryRegionDesc<'_>]>,
    &MappedAddress,
    &RemoteMemAddrSlice<T>,
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: AtomicFetchEp, 
{

    match op {
        FetchAtomicOp::Min => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_min_mr_slice_from,
        FetchAtomicOp::Max => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_max_mr_slice_from,
        FetchAtomicOp::Sum => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_sum_mr_slice_from,
        FetchAtomicOp::Prod => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_prod_mr_slice_from,
        FetchAtomicOp::Bor => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_bor_mr_slice_from,
        FetchAtomicOp::Band => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_band_mr_slice_from,
        FetchAtomicOp::Bxor => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_bxor_mr_slice_from,
        FetchAtomicOp::AtomicWrite => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_write_mr_slice_from,
        FetchAtomicOp::AtomicRead => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_read_mr_slice_from,
        _ => todo!(),
    }
}

fn get_atomicv_fetch_bool_op<A>(op: libfabric::enums::FetchAtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<bool>], 
    desc: Option<&[MemoryRegionDesc<'_>]>,
    resioc: &mut [libfabric::iovec::IocMut<bool>], 
    res_desc: Option<&[MemoryRegionDesc<'_>]>,
    &MappedAddress,
    &RemoteMemAddrSlice<bool>,
) 
-> Result<(), Error>
where
    A: AtomicFetchEp, 
{

    match op {
        FetchAtomicOp::Lor => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_lor_mr_slice_from,
        FetchAtomicOp::Land => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_land_mr_slice_from,
        FetchAtomicOp::Lxor => AtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_lxor_mr_slice_from,
        _ => todo!(),
    }
}

fn get_conn_atomic_fetch_op<T, A>(op: libfabric::enums::FetchAtomicOp) -> unsafe fn(
    &A,
    &[T],
    Option<MemoryRegionDesc>,
    &mut [T],
    Option<MemoryRegionDesc>,
    &RemoteMemAddrSlice<T>,
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: ConnectedAtomicFetchEp, 
{

    match op {
        FetchAtomicOp::Min => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_min_mr_slice,
        FetchAtomicOp::Max => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_max_mr_slice,
        FetchAtomicOp::Sum => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_sum_mr_slice,
        FetchAtomicOp::Prod => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_prod_mr_slice,
        FetchAtomicOp::Bor => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_bor_mr_slice,
        FetchAtomicOp::Band => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_band_mr_slice,
        FetchAtomicOp::Bxor => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_bxor_mr_slice,
        FetchAtomicOp::AtomicWrite => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_write_mr_slice,
        FetchAtomicOp::AtomicRead => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_read_mr_slice,
        _ => todo!(),
    }
}

fn get_conn_atomicv_fetch_op<T, A>(op: libfabric::enums::FetchAtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<T>], 
    desc: Option<&[MemoryRegionDesc<'_>]>,
    resioc: &mut [libfabric::iovec::IocMut<T>], 
    res_desc: Option<&[MemoryRegionDesc<'_>]>,
    slice: &RemoteMemAddrSlice<T>,
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: ConnectedAtomicFetchEp, 
{

    match op {
        FetchAtomicOp::Min => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_min_mr_slice,
        FetchAtomicOp::Max => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_max_mr_slice,
        FetchAtomicOp::Sum => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_sum_mr_slice,
        FetchAtomicOp::Prod => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_prod_mr_slice,
        FetchAtomicOp::Bor => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_bor_mr_slice,
        FetchAtomicOp::Band => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_band_mr_slice,
        FetchAtomicOp::Bxor => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_bxor_mr_slice,
        FetchAtomicOp::AtomicWrite => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_write_mr_slice,
        FetchAtomicOp::AtomicRead => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_read_mr_slice,
        _ => todo!(),
    }
}

fn get_conn_atomic_fetch_bool_op<A>(op: libfabric::enums::FetchAtomicOp) -> unsafe fn(
    &A,
    &[bool],
    Option<MemoryRegionDesc>,
    &mut [bool],
    Option<MemoryRegionDesc>,
    &RemoteMemAddrSlice<bool>,
) 
-> Result<(), Error>
where
    A: ConnectedAtomicFetchEp, 
{

    match op {
        FetchAtomicOp::Lor => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_lor_mr_slice,
        FetchAtomicOp::Land => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_land_mr_slice,
        FetchAtomicOp::Lxor => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomic_lxor_mr_slice,
        _ => todo!(),
    }
}

fn get_conn_atomicv_fetch_bool_op<A>(op: libfabric::enums::FetchAtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<bool>], 
    desc: Option<&[MemoryRegionDesc<'_>]>,
    resioc: &mut [libfabric::iovec::IocMut<bool>], 
    res_desc: Option<&[MemoryRegionDesc<'_>]>,
    slice: &RemoteMemAddrSlice<bool>,
) 
-> Result<(), Error>
where
    A: ConnectedAtomicFetchEp, 
{

    match op {
        FetchAtomicOp::Lor => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_lor_mr_slice,
        FetchAtomicOp::Land => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_land_mr_slice,
        FetchAtomicOp::Lxor => ConnectedAtomicFetchRemoteMemAddrSliceEp::fetch_atomicv_lxor_mr_slice,
        _ => todo!(),
    }
}

fn get_atomicv_compare_op<T, A>(op: libfabric::enums::CompareAtomicOp) -> unsafe fn(

    &A,
    ioc: &[libfabric::iovec::Ioc<T>], 
    desc: Option<&[MemoryRegionDesc<'_>]>, 
    comparetv: &[libfabric::iovec::Ioc<T>], 
    compare_desc: Option<&[MemoryRegionDesc<'_>]>, 
    resultv: &mut [libfabric::iovec::IocMut<T>], 
    res_desc: Option<&[MemoryRegionDesc<'_>]>, 
    dest_addr: &libfabric::MappedAddress, 
    dest_slice: &RemoteMemAddrSliceMut<T>
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: AtomicCASEp, 
{

    match op {
        CompareAtomicOp::Cswap => AtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_mr_slice_to,
        CompareAtomicOp::CswapGe => AtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_ge_mr_slice_to,
        CompareAtomicOp::CswapGt => AtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_gt_mr_slice_to,
        CompareAtomicOp::CswapLe => AtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_le_mr_slice_to,
        CompareAtomicOp::CswapLt => AtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_lt_mr_slice_to,
        CompareAtomicOp::CswapNe => AtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_ne_mr_slice_to,
        CompareAtomicOp::Mswap => AtomicCASRemoteMemAddrSliceEp::compare_atomicv_mswap_mr_slice_to,
    }
}

fn get_atomic_compare_op<T, A>(op: libfabric::enums::CompareAtomicOp) -> unsafe fn(
    &A,
    buf: &[T], 
    desc: Option<MemoryRegionDesc<'_>>, 
    compare: &[T], 
    compare_desc: Option<MemoryRegionDesc<'_>>, 
    result: &mut [T], 
    result_desc: Option<MemoryRegionDesc<'_>>, 
    dest_addr: &MappedAddress, 
    dest_slice: &RemoteMemAddrSliceMut<T>
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: AtomicCASEp, 
{

    match op {
        CompareAtomicOp::Cswap => AtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_mr_slice_to,
        CompareAtomicOp::CswapGe => AtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_ge_mr_slice_to,
        CompareAtomicOp::CswapGt => AtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_gt_mr_slice_to,
        CompareAtomicOp::CswapLe => AtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_le_mr_slice_to,
        CompareAtomicOp::CswapLt => AtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_lt_mr_slice_to,
        CompareAtomicOp::CswapNe => AtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_ne_mr_slice_to,
        CompareAtomicOp::Mswap => AtomicCASRemoteMemAddrSliceEp::compare_atomic_mswap_mr_slice_to,
    }
}

fn get_conn_atomic_compare_op<T, A>(op: libfabric::enums::CompareAtomicOp) -> unsafe fn(
    &A,
    buf: &[T], 
    desc: Option<MemoryRegionDesc<'_>>, 
    compare: &[T], 
    compare_desc: Option<MemoryRegionDesc<'_>>, 
    result: &mut [T], 
    result_desc: Option<MemoryRegionDesc<'_>>, 
    dest_slice: &RemoteMemAddrSliceMut<T>
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: ConnectedAtomicCASEp, 
{

    match op {
        CompareAtomicOp::Cswap => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_mr_slice,
        CompareAtomicOp::CswapGe => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_ge_mr_slice,
        CompareAtomicOp::CswapGt => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_gt_mr_slice,
        CompareAtomicOp::CswapLe => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_le_mr_slice,
        CompareAtomicOp::CswapLt => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_lt_mr_slice,
        CompareAtomicOp::CswapNe => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomic_swap_ne_mr_slice,
        CompareAtomicOp::Mswap => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomic_mswap_mr_slice,
    }
}

fn get_conn_atomicv_compare_op<T, A>(op: libfabric::enums::CompareAtomicOp) -> unsafe fn(
    &A,
    ioc: &[libfabric::iovec::Ioc<T>], 
    desc: Option<&[MemoryRegionDesc<'_>]>, 
    comparetv: &[libfabric::iovec::Ioc<T>], 
    compare_desc: Option<&[MemoryRegionDesc<'_>]>, 
    resultv: &mut [libfabric::iovec::IocMut<T>], 
    res_desc: Option<&[MemoryRegionDesc<'_>]>, 
    dest_slice: &RemoteMemAddrSliceMut<T>
) 
-> Result<(), Error>
where
    T: AsFiType,
    A: ConnectedAtomicCASEp, 
{

    match op {
        CompareAtomicOp::Cswap => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_mr_slice,
        CompareAtomicOp::CswapGe => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_ge_mr_slice,
        CompareAtomicOp::CswapGt => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_gt_mr_slice,
        CompareAtomicOp::CswapLe => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_le_mr_slice,
        CompareAtomicOp::CswapLt => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_lt_mr_slice,
        CompareAtomicOp::CswapNe => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomicv_swap_ne_mr_slice,
        CompareAtomicOp::Mswap => ConnectedAtomicCASRemoteMemAddrSliceEp::compare_atomicv_mswap_mr_slice,
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

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        unsafe {
                            get_atomic_inject_op(op)(
                                ep, 
                                buf,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &dst_slice,
                            )
                        }
                    } else {
                        unsafe {
                            get_atomic_op(op)(
                                ep,
                                buf,
                                desc,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &dst_slice,
                            )
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        unsafe { get_conn_atomic_inject_op(op)(ep,buf, &dst_slice) }
                    } else {
                        unsafe { get_conn_atomic_op(op)(ep, buf, desc, &dst_slice) }
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
                    get_atomicv_op(op)(
                        ep,
                        ioc,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &dst_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe { get_conn_atomicv_op(op)(ep, ioc, desc, &dst_slice) },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn atomic_bool(
        &self,
        buf: &[bool],
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
                            get_atomic_inject_bool_op(op)(
                                ep, 
                                buf,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &dst_slice,
                            )
                        }
                    } else {
                        unsafe {
                            get_atomic_bool_op(op)(
                                ep,
                                buf,
                                desc,
                                &self.mapped_addr.as_ref().unwrap()[1],
                                &dst_slice,
                            )
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        unsafe { get_conn_atomic_inject_bool_op(op)(ep,buf, &dst_slice) }
                    } else {
                        unsafe { get_conn_atomic_bool_op(op)(ep, buf, desc, &dst_slice) }
                    }
                }
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn atomicv_bool(
        &self,
        ioc: &[libfabric::iovec::Ioc<bool>],
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
                    get_atomicv_bool_op(op)(
                        ep,
                        ioc,
                        desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &dst_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe { get_conn_atomicv_bool_op(op)(ep, ioc, desc, &dst_slice) },
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

        // let base_mem_addr = remote_mem_info.borrow().mem_address();
        // let key = remote_mem_info.borrow().key();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    get_atomic_fetch_op(op)(
                        ep,
                        buf,
                        desc,
                        res,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &src_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    get_conn_atomic_fetch_op(op)(ep,buf, desc, res, res_desc, &src_slice)
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
                    get_atomicv_fetch_op(op)
                    (
                        ep,
                        ioc,
                        desc,
                        res_ioc,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &src_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    get_conn_atomicv_fetch_op(op)
                    (
                       ep, 
                       ioc, 
                       desc, 
                       res_ioc, 
                       res_desc, 
                       &src_slice
                    )
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
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
        let remote_mem_info = self.remote_mem_info.as_ref().unwrap().borrow_mut();
        let src_slice = remote_mem_info.slice(dest_addr..dest_addr + buf.len());

        // let base_mem_addr = remote_mem_info.borrow().mem_address();
        // let key = remote_mem_info.borrow().key();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    get_atomic_fetch_bool_op(op)(
                        ep,
                        buf,
                        desc,
                        res,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &src_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    get_conn_atomic_fetch_bool_op(op)(ep,buf, desc, res, res_desc, &src_slice)
                },
            };

            if self.check_and_progress(err) {
                break;
            }
        }
    }

    pub fn fetch_atomicv_bool(
        &self,
        ioc: &[libfabric::iovec::Ioc<bool>],
        res_ioc: &mut [libfabric::iovec::IocMut<bool>],
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
                    get_atomicv_fetch_bool_op(op)
                    (
                        ep,
                        ioc,
                        desc,
                        res_ioc,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &src_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    get_conn_atomicv_fetch_bool_op(op)
                    (
                       ep, 
                       ioc, 
                       desc, 
                       res_ioc, 
                       res_desc, 
                       &src_slice
                    )
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
                    get_atomic_compare_op(op)
                    ( 
                        ep,
                        buf,
                        desc,
                        comp,
                        comp_desc,
                        res,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &dst_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    get_conn_atomic_compare_op(op)
                    (
                        ep, buf, desc, comp, comp_desc, res, res_desc, &dst_slice,
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
                    get_atomicv_compare_op(op)
                    (
                        ep,
                        ioc,
                        desc,
                        comp_ioc,
                        comp_desc,
                        res_ioc,
                        res_desc,
                        &self.mapped_addr.as_ref().unwrap()[1],
                        &dst_slice,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    get_conn_atomicv_compare_op(op)(
                        ep,ioc, desc, comp_ioc, comp_desc, res_ioc, res_desc, &dst_slice,
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

impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn sync(&self) -> Result<(), Error> {
        self.send(0..1, None, false);
        self.recv(0..1, false);
        self.wait_rx(1);
        Ok(())
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

                self.send(0..size, None, false);
                if size > self.info_entry.tx_attr().inject_size() {
                    self.wait_tx(1);
                    // self.cq_type.tx_cq().sread(1, -1).unwrap();
                }
                self.recv(0..size, false);
                self.wait_rx(1);
                // self.cq_type.rx_cq().sread(1, -1).unwrap();
            }
        } else {
            for i in 0..warmup + iters {
                if i == warmup {
                    now = Instant::now(); // Start timer
                }

                self.recv(0..size, false);
                self.wait_rx(1);
                
                // self.cq_type.rx_cq().sread(1, -1).unwrap();
                self.send(0..size, None, false);
                if size > self.info_entry.tx_attr().inject_size() {
                    self.wait_tx(1);
                    // self.cq_type.tx_cq().sread(1, -1).unwrap();
                }
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

impl<I: MsgDefaultCap + TagDefaultCap + 'static> Ofi<I> {
    pub fn pingpong_tagged(&self, warmup: usize, iters: usize, size: usize) {
        self.sync().unwrap();
        let mut now = Instant::now();
        if !self.server {
            for i in 0..warmup + iters {
                if i == warmup {
                    now = Instant::now(); // Start timer
                }

                self.tsend(0..size, 0, None, false);
                if size > self.info_entry.tx_attr().inject_size() {
                    self.wait_tx(1);
                }
                self.trecv(0..size, 0, false);
                self.wait_rx(1);
            }
        } else {
            for i in 0..warmup + iters {
                if i == warmup {
                    now = Instant::now(); // Start timer
                }

                self.trecv(0..size, 0, false);
                self.wait_rx(1);
                self.tsend(0..size, 0, None, false);
                if size > self.info_entry.tx_attr().inject_size() {
                    self.wait_tx(1);
                }
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
                if size > self.info_entry.tx_attr().inject_size() {
                    self.wait_tx(1);
                }
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
}