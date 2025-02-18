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
        FetchAtomicOp, ReadMsgOptions, TferOptions, WriteMsgOptions,
    },
    ep::{Address, BaseEndpoint, Endpoint, EndpointBuilder},
    eq::{EventQueueBuilder, WaitEq},
    error::{Error, ErrorKind},
    fabric::FabricBuilder,
    info::{Info, InfoEntry, Version},
    infocapsoptions::{
        AtomicDefaultCap, Caps, CollCap, InfoCaps, MsgDefaultCap, RmaDefaultCap, TagDefaultCap,
    },
    iovec::{IoVec, IoVecMut, Ioc, IocMut, RmaIoVec, RmaIoc},
    mr::{
        default_desc, DisabledMemoryRegion, MappedMemoryRegionKey, MemoryRegion,
        MemoryRegionBuilder, MemoryRegionDesc, MemoryRegionKey,
    },
    msg::{
        Msg, MsgAtomic, MsgAtomicConnected, MsgCompareAtomic, MsgCompareAtomicConnected,
        MsgConnected, MsgConnectedMut, MsgFetchAtomic, MsgFetchAtomicConnected, MsgMut, MsgRma,
        MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut, MsgTagged, MsgTaggedConnected,
        MsgTaggedConnectedMut, MsgTaggedMut,
    },
    Context, CqCaps, EqCaps, MappedAddress,
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

pub struct Ofi<I> {
    pub info_entry: InfoEntry<I>,
    pub mr: Option<MemoryRegion>,
    pub key: Option<MemoryRegionKey>,
    pub remote_key: Option<MappedMemoryRegionKey>,
    pub remote_mem_addr: Option<(u64, u64)>,
    pub domain: Domain,
    pub cq_type: CqType,
    pub ep: MyEndpoint<I>,
    pub mapped_addr: Option<MappedAddress>,
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
            EndpointType::Msg | EndpointType::SockStream => match &self.ep {
                MyEndpoint::Connected(ep) => ep.shutdown().unwrap(),
                MyEndpoint::Connectionless(_) => todo!(),
            },
            EndpointType::Unspec
            | EndpointType::Dgram
            | EndpointType::Rdm
            | EndpointType::SockDgram => {}
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
        let key;

        // let mut tx_pending_cnt: usize = 0;
        // let mut tx_complete_cnt: usize = 0;
        // let mut rx_pending_cnt: usize = 0;
        // let mut rx_complete_cnt: usize = 0;
        let mut reg_mem = vec![0u8; 1024 * 1024];

        let (info_entry, ep, mapped_addr) = match ep_type {
            EndpointType::Msg | EndpointType::SockStream => {
                let eq = EventQueueBuilder::new(&fabric).build().unwrap();

                let info_entry = if server {
                    let pep = EndpointBuilder::new(&info_entry)
                        .build_passive(&fabric)
                        .unwrap();
                    pep.bind(&eq, 0).unwrap();
                    pep.listen().unwrap();
                    let event = eq.sread(-1).unwrap();
                    match event {
                        libfabric::eq::Event::ConnReq(entry) => entry.get_info().unwrap(),
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
                let ep = match EndpointBuilder::new(&info_entry).build(&domain).unwrap() {
                    Endpoint::Connectionless(_) => panic!("Expected connected EP"),
                    Endpoint::ConnectionOriented(unconn_ep) => unconn_ep,
                };
                ep.bind_eq(&eq).unwrap();
                match cq_type {
                    CqType::Separate((ref tx_cq, ref rx_cq)) => {
                        ep.bind_separate_cqs(tx_cq, false, rx_cq, false).unwrap()
                    }
                    CqType::Shared(ref scq) => ep.bind_shared_cq(&scq, false).unwrap(),
                }

                let ep = ep.enable().unwrap();

                if !server {
                    ep.connect(info_entry.dest_addr().unwrap()).unwrap();
                } else {
                    ep.accept().unwrap();
                }

                let ep = match eq.sread(-1) {
                    Ok(event) => match event {
                        libfabric::eq::Event::Connected(event) => ep.connect_complete(event),
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

                (mr, key) = if info_entry.domain_attr().mr_mode().is_local()
                    || info_entry.caps().is_rma()
                {
                    let mr =
                        MemoryRegionBuilder::new(&mut reg_mem, libfabric::enums::HmemIface::System)
                            .access_read()
                            .access_write()
                            .access_send()
                            .access_recv()
                            .build(&domain)?;
                    let mr = match mr {
                        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
                            mr.bind_ep(&ep).unwrap();
                            mr.enable().unwrap()
                        }
                    };
                    let key = mr.key().unwrap();
                    (Some(mr), Some(key))
                } else {
                    (None, None)
                };

                (info_entry, MyEndpoint::Connected(ep), None)
            }
            _ => {
                domain = DomainBuilder::new(&fabric, &info_entry).build().unwrap();

                cq_type = if shared_cqs {
                    CqType::Shared(shared_cq_builder.build(&domain).unwrap())
                } else {
                    CqType::Separate((
                        tx_cq_builder.build(&domain).unwrap(),
                        rx_cq_builder.build(&domain).unwrap(),
                    ))
                };

                let ep = match EndpointBuilder::new(&info_entry).build(&domain).unwrap() {
                    Endpoint::Connectionless(ep) => ep,
                    Endpoint::ConnectionOriented(_) => panic!("Expected connectionless ep"),
                };
                match cq_type {
                    CqType::Separate((ref tx_cq, ref rx_cq)) => {
                        ep.bind_separate_cqs(tx_cq, false, rx_cq, false).unwrap()
                    }
                    CqType::Shared(ref scq) => ep.bind_shared_cq(&scq, false).unwrap(),
                }

                let av = match info_entry.domain_attr().av_type() {
                    libfabric::enums::AddressVectorType::Unspec => AddressVectorBuilder::new(),
                    _ => AddressVectorBuilder::new().type_(*info_entry.domain_attr().av_type()),
                }
                .build(&domain)
                .unwrap();
                ep.bind_av(&av).unwrap();
                let ep = ep.enable().unwrap();

                (mr, key) = if info_entry.domain_attr().mr_mode().is_local()
                    || info_entry.caps().is_rma()
                {
                    let mr =
                        MemoryRegionBuilder::new(&mut reg_mem, libfabric::enums::HmemIface::System)
                            .access_read()
                            .access_write()
                            .access_send()
                            .access_recv()
                            .build(&domain)?;
                    let mr = match mr {
                        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
                            mr.bind_ep(&ep).unwrap();
                            mr.enable().unwrap()
                        }
                    };
                    let key = mr.key().unwrap();
                    (Some(mr), Some(key))
                } else {
                    (None, None)
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

                    post!(
                        send_to,
                        ft_progress,
                        cq_type.tx_cq(),
                        ep,
                        &reg_mem[..addrlen],
                        &mut default_desc(),
                        &mapped_address
                    );
                    cq_type.tx_cq().sread(1, -1).unwrap();

                    // ep.recv(std::slice::from_mut(&mut ack), &mut default_desc()).unwrap();
                    post!(
                        recv_from_any,
                        ft_progress,
                        cq_type.rx_cq(),
                        ep,
                        std::slice::from_mut(&mut reg_mem[0]),
                        &mut default_desc()
                    );
                    cq_type.rx_cq().sread(1, -1).unwrap();

                    mapped_address
                } else {
                    let epname = ep.getname().unwrap();
                    let addrlen = epname.as_bytes().len();

                    let mut mr_desc = if let Some(ref mr) = mr {
                        mr.descriptor()
                    } else {
                        default_desc()
                    };

                    post!(
                        recv_from_any,
                        ft_progress,
                        cq_type.rx_cq(),
                        ep,
                        &mut reg_mem[..addrlen],
                        &mut mr_desc
                    );
                    cq_type.rx_cq().sread(1, -1).unwrap();
                    // ep.recv(&mut reg_mem, &mut mr_desc).unwrap();
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
                    post!(
                        send_to,
                        ft_progress,
                        cq_type.tx_cq(),
                        ep,
                        &std::slice::from_ref(&reg_mem[0]),
                        &mut mr_desc,
                        &mapped_address
                    );
                    cq_type.tx_cq().sread(1, -1).unwrap();

                    mapped_address
                };
                (
                    info_entry,
                    MyEndpoint::Connectionless(ep),
                    Some(mapped_address),
                )
            }
        };
        if server {
            unsafe { std::env::remove_var(name) };
        }

        Ok(Self {
            info_entry,
            mapped_addr,
            mr,
            key,
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

impl<I: TagDefaultCap> Ofi<I> {
    pub fn tsend<T>(&self, buf: &[T], desc: &mut MemoryRegionDesc, tag: u64, data: Option<u64>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if data.is_some() {
                            ep.tinjectdata_to(
                                &buf,
                                data.unwrap(),
                                self.mapped_addr.as_ref().unwrap(),
                                tag,
                            )
                        } else {
                            ep.tinject_to(&buf, self.mapped_addr.as_ref().unwrap(), tag)
                        }
                    } else {
                        if data.is_some() {
                            ep.tsenddata_to(
                                &buf,
                                desc,
                                data.unwrap(),
                                self.mapped_addr.as_ref().unwrap(),
                                tag,
                            )
                        } else {
                            ep.tsend_to(&buf, desc, self.mapped_addr.as_ref().unwrap(), tag)
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if data.is_some() {
                            ep.tinjectdata(&buf, data.unwrap(), tag)
                        } else {
                            ep.tinject(&buf, tag)
                        }
                    } else {
                        if data.is_some() {
                            ep.tsenddata(&buf, desc, data.unwrap(), tag)
                        } else {
                            ep.tsend(&buf, desc, tag)
                        }
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

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn tsendv(&mut self, iov: &[IoVec], desc: &mut [MemoryRegionDesc], tag: u64) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    ep.tsendv_to(iov, desc, self.mapped_addr.as_ref().unwrap(), tag)
                }
                MyEndpoint::Connected(ep) => ep.tsendv(iov, desc, tag),
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn trecvv(&mut self, iov: &[IoVecMut], desc: &mut [MemoryRegionDesc], tag: u64) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    ep.trecvv_from(iov, desc, self.mapped_addr.as_ref().unwrap(), tag, None)
                }
                MyEndpoint::Connected(ep) => ep.trecvv(iov, desc, tag, None),
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn trecv<T>(&mut self, buf: &mut [T], desc: &mut MemoryRegionDesc, tag: u64) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    ep.trecv_from(buf, desc, self.mapped_addr.as_ref().unwrap(), tag, None)
                }
                MyEndpoint::Connected(ep) => ep.trecv(buf, desc, tag, None),
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn tsendmsg(&mut self, msg: &Either<MsgTagged, MsgTaggedConnected>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => ep.tsendmsg_to(msg, TferOptions::new().remote_cq_data()),
                    Either::Right(_) => panic!("Wrong message type used"),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => panic!("Wrong message type used"),
                    Either::Right(msg) => ep.tsendmsg(msg, TferOptions::new().remote_cq_data()),
                },
            };

            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn trecvmsg(&mut self, msg: &Either<MsgTaggedMut, MsgTaggedConnectedMut>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => ep.trecvmsg_from(msg, TferOptions::new()),
                    Either::Right(_) => panic!("Wrong message type"),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => panic!("Wrong message type"),
                    Either::Right(msg) => ep.trecvmsg(msg, TferOptions::new()),
                },
            };

            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }
}

impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn send<T>(&self, buf: &[T], desc: &mut MemoryRegionDesc, data: Option<u64>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if data.is_some() {
                            ep.injectdata_to(buf, data.unwrap(), self.mapped_addr.as_ref().unwrap())
                        } else {
                            ep.inject_to(&buf, self.mapped_addr.as_ref().unwrap())
                        }
                    } else {
                        if data.is_some() {
                            ep.senddata_to(
                                &buf,
                                desc,
                                data.unwrap(),
                                self.mapped_addr.as_ref().unwrap(),
                            )
                        } else {
                            ep.send_to(&buf, desc, self.mapped_addr.as_ref().unwrap())
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if data.is_some() {
                            ep.injectdata(&buf, data.unwrap())
                        } else {
                            ep.inject(&buf)
                        }
                    } else {
                        if data.is_some() {
                            ep.senddata(&buf, desc, data.unwrap())
                        } else {
                            ep.send(&buf, desc)
                        }
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

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn send_with_context<T>(
        &self,
        buf: &[T],
        desc: &mut MemoryRegionDesc,
        data: Option<u64>,
        context: &mut Context,
    ) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if data.is_some() {
                            ep.injectdata_to(buf, data.unwrap(), self.mapped_addr.as_ref().unwrap())
                        } else {
                            ep.inject_to(&buf, self.mapped_addr.as_ref().unwrap())
                        }
                    } else {
                        if data.is_some() {
                            ep.senddata_to_with_context(
                                &buf,
                                desc,
                                data.unwrap(),
                                self.mapped_addr.as_ref().unwrap(),
                                context,
                            )
                        } else {
                            ep.send_to_with_context(
                                &buf,
                                desc,
                                self.mapped_addr.as_ref().unwrap(),
                                context,
                            )
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if data.is_some() {
                            ep.injectdata(&buf, data.unwrap())
                        } else {
                            ep.inject(&buf)
                        }
                    } else {
                        if data.is_some() {
                            ep.senddata_with_context(&buf, desc, data.unwrap(), context)
                        } else {
                            ep.send_with_context(&buf, desc, context)
                        }
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

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn sendv(&mut self, iov: &[IoVec], desc: &mut [MemoryRegionDesc]) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    ep.sendv_to(iov, desc, self.mapped_addr.as_ref().unwrap())
                }
                MyEndpoint::Connected(ep) => ep.sendv(iov, desc),
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn recvv(&mut self, iov: &[IoVecMut], desc: &mut [MemoryRegionDesc]) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    ep.recvv_from(iov, desc, self.mapped_addr.as_ref().unwrap())
                }
                MyEndpoint::Connected(ep) => ep.recvv(iov, desc),
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn recv<T>(&mut self, buf: &mut [T], desc: &mut MemoryRegionDesc) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    ep.recv_from(buf, desc, self.mapped_addr.as_ref().unwrap())
                }
                MyEndpoint::Connected(ep) => ep.recv(buf, desc),
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn sendmsg(&mut self, msg: &Either<Msg, MsgConnected>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => ep.sendmsg_to(msg, TferOptions::new().remote_cq_data()),
                    Either::Right(_) => panic!("Wrong msg type"),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => panic!("Wrong msg type"),
                    Either::Right(msg) => ep.sendmsg(msg, TferOptions::new().remote_cq_data()),
                },
            };

            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn recvmsg(&mut self, msg: &Either<MsgMut, MsgConnectedMut>) {
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => match msg {
                    Either::Left(msg) => ep.recvmsg_from(msg, TferOptions::new()),
                    Either::Right(_) => panic!("Wrong message type"),
                },
                MyEndpoint::Connected(ep) => match msg {
                    Either::Left(_) => panic!("Wrong message type"),
                    Either::Right(msg) => ep.recvmsg(msg, TferOptions::new()),
                },
            };

            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn exchange_keys(&mut self, key: MemoryRegionKey, addr: usize, len: usize) {
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
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
                bind_mr(&self.ep, &mr);
                mr.enable().unwrap()
            }
        };

        let mut desc = mr.descriptor();
        self.send(
            &reg_mem[..key_bytes.len() + 2 * std::mem::size_of::<usize>()],
            &mut desc,
            None,
        );
        self.recv(
            &mut reg_mem[key_bytes.len() + 2 * std::mem::size_of::<usize>()
                ..2 * key_bytes.len() + 4 * std::mem::size_of::<usize>()],
            &mut desc,
        );

        self.cq_type.rx_cq().sread(1, -1).unwrap();
        let remote_key = unsafe {
            MemoryRegionKey::from_bytes(
                &reg_mem[key_bytes.len() + 2 * std::mem::size_of::<usize>()
                    ..2 * key_bytes.len() + 2 * std::mem::size_of::<usize>()],
                &self.domain,
            )
        }
        .into_mapped(&self.domain)
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

impl<I: MsgDefaultCap + RmaDefaultCap> Ofi<I> {
    pub fn write<T>(
        &mut self,
        buf: &[T],
        dest_addr: u64,
        desc: &mut MemoryRegionDesc,
        data: Option<u64>,
    ) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if data.is_some() {
                            unsafe {
                                ep.inject_writedata_to(
                                    buf,
                                    data.unwrap(),
                                    self.mapped_addr.as_ref().unwrap(),
                                    start + dest_addr,
                                    self.remote_key.as_ref().unwrap(),
                                )
                            }
                        } else {
                            unsafe {
                                ep.inject_write_to(
                                    buf,
                                    self.mapped_addr.as_ref().unwrap(),
                                    start + dest_addr,
                                    self.remote_key.as_ref().unwrap(),
                                )
                            }
                        }
                    } else {
                        if data.is_some() {
                            unsafe {
                                ep.writedata_to(
                                    buf,
                                    desc,
                                    data.unwrap(),
                                    self.mapped_addr.as_ref().unwrap(),
                                    start + dest_addr,
                                    self.remote_key.as_ref().unwrap(),
                                )
                            }
                        } else {
                            unsafe {
                                ep.write_to(
                                    buf,
                                    desc,
                                    self.mapped_addr.as_ref().unwrap(),
                                    start + dest_addr,
                                    self.remote_key.as_ref().unwrap(),
                                )
                            }
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        if data.is_some() {
                            unsafe {
                                ep.inject_writedata(
                                    buf,
                                    data.unwrap(),
                                    start + dest_addr,
                                    self.remote_key.as_ref().unwrap(),
                                )
                            }
                        } else {
                            unsafe {
                                ep.inject_write(
                                    buf,
                                    start + dest_addr,
                                    self.remote_key.as_ref().unwrap(),
                                )
                            }
                        }
                    } else {
                        if data.is_some() {
                            unsafe {
                                ep.writedata(
                                    buf,
                                    desc,
                                    data.unwrap(),
                                    start + dest_addr,
                                    self.remote_key.as_ref().unwrap(),
                                )
                            }
                        } else {
                            unsafe {
                                ep.write(
                                    buf,
                                    desc,
                                    start + dest_addr,
                                    self.remote_key.as_ref().unwrap(),
                                )
                            }
                        }
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

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn read<T>(&mut self, buf: &mut [T], dest_addr: u64, desc: &mut MemoryRegionDesc) {
        let (start, _end) = self.remote_mem_addr.unwrap();

        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.read_from(
                        buf,
                        desc,
                        self.mapped_addr.as_ref().unwrap(),
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.read(
                        buf,
                        desc,
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                    )
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn writev(&mut self, iov: &[IoVec], dest_addr: u64, desc: &mut [MemoryRegionDesc]) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.writev_to(
                        iov,
                        desc,
                        self.mapped_addr.as_ref().unwrap(),
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.writev(
                        iov,
                        desc,
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                    )
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn readv(&mut self, iov: &[IoVecMut], dest_addr: u64, desc: &mut [MemoryRegionDesc]) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.readv_from(
                        iov,
                        desc,
                        self.mapped_addr.as_ref().unwrap(),
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.readv(
                        iov,
                        desc,
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                    )
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    // [TODO] Enabling .remote_cq_data causes the buffer not being written correctly
    // on the remote side.
    pub fn writemsg(&mut self, msg: &Either<MsgRma, MsgRmaConnected>) {
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
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn readmsg(&mut self, msg: &Either<MsgRmaMut, MsgRmaConnectedMut>) {
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
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }
}

impl<I: AtomicDefaultCap> Ofi<I> {
    pub fn atomic<T: libfabric::AsFiType>(
        &mut self,
        buf: &[T],
        dest_addr: u64,
        desc: &mut MemoryRegionDesc,
        op: AtomicOp,
    ) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        unsafe {
                            ep.inject_atomic_to(
                                buf,
                                self.mapped_addr.as_ref().unwrap(),
                                start + dest_addr,
                                self.remote_key.as_ref().unwrap(),
                                op,
                            )
                        }
                    } else {
                        unsafe {
                            ep.atomic_to(
                                buf,
                                desc,
                                self.mapped_addr.as_ref().unwrap(),
                                start + dest_addr,
                                self.remote_key.as_ref().unwrap(),
                                op,
                            )
                        }
                    }
                }
                MyEndpoint::Connected(ep) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        unsafe {
                            ep.inject_atomic(
                                buf,
                                start + dest_addr,
                                self.remote_key.as_ref().unwrap(),
                                op,
                            )
                        }
                    } else {
                        unsafe {
                            ep.atomic(
                                buf,
                                desc,
                                start + dest_addr,
                                self.remote_key.as_ref().unwrap(),
                                op,
                            )
                        }
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

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn atomicv<T: libfabric::AsFiType>(
        &mut self,
        ioc: &[libfabric::iovec::Ioc<T>],
        dest_addr: u64,
        desc: &mut [MemoryRegionDesc],
        op: AtomicOp,
    ) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.atomicv_to(
                        ioc,
                        desc,
                        self.mapped_addr.as_ref().unwrap(),
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.atomicv(
                        ioc,
                        desc,
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn atomicmsg<T: libfabric::AsFiType>(
        &mut self,
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
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn fetch_atomic<T: libfabric::AsFiType>(
        &mut self,
        buf: &[T],
        res: &mut [T],
        dest_addr: u64,
        desc: &mut MemoryRegionDesc,
        res_desc: &mut MemoryRegionDesc,
        op: FetchAtomicOp,
    ) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.fetch_atomic_from(
                        buf,
                        desc,
                        res,
                        res_desc,
                        self.mapped_addr.as_ref().unwrap(),
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.fetch_atomic(
                        buf,
                        desc,
                        res,
                        res_desc,
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn fetch_atomicv<T: libfabric::AsFiType>(
        &mut self,
        ioc: &[libfabric::iovec::Ioc<T>],
        res_ioc: &mut [libfabric::iovec::IocMut<T>],
        dest_addr: u64,
        desc: &mut [MemoryRegionDesc],
        res_desc: &mut [MemoryRegionDesc],
        op: FetchAtomicOp,
    ) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.fetch_atomicv_from(
                        ioc,
                        desc,
                        res_ioc,
                        res_desc,
                        self.mapped_addr.as_ref().unwrap(),
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.fetch_atomicv(
                        ioc,
                        desc,
                        res_ioc,
                        res_desc,
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn fetch_atomicmsg<T: libfabric::AsFiType>(
        &mut self,
        msg: &Either<MsgFetchAtomic<T>, MsgFetchAtomicConnected<T>>,
        res_ioc: &mut [libfabric::iovec::IocMut<T>],
        res_desc: &mut [MemoryRegionDesc],
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
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn compare_atomic<T: libfabric::AsFiType>(
        &mut self,
        buf: &[T],
        comp: &[T],
        res: &mut [T],
        dest_addr: u64,
        desc: &mut MemoryRegionDesc,
        comp_desc: &mut MemoryRegionDesc,
        res_desc: &mut MemoryRegionDesc,
        op: CompareAtomicOp,
    ) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.compare_atomic_to(
                        buf,
                        desc,
                        comp,
                        comp_desc,
                        res,
                        res_desc,
                        self.mapped_addr.as_ref().unwrap(),
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.compare_atomic(
                        buf,
                        desc,
                        comp,
                        comp_desc,
                        res,
                        res_desc,
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn compare_atomicv<T: libfabric::AsFiType>(
        &mut self,
        ioc: &[libfabric::iovec::Ioc<T>],
        comp_ioc: &[libfabric::iovec::Ioc<T>],
        res_ioc: &mut [libfabric::iovec::IocMut<T>],
        dest_addr: u64,
        desc: &mut [MemoryRegionDesc],
        comp_desc: &mut [MemoryRegionDesc],
        res_desc: &mut [MemoryRegionDesc],
        op: CompareAtomicOp,
    ) {
        let (start, _end) = self.remote_mem_addr.unwrap();
        loop {
            let err = match &self.ep {
                MyEndpoint::Connectionless(ep) => unsafe {
                    ep.compare_atomicv_to(
                        ioc,
                        desc,
                        comp_ioc,
                        comp_desc,
                        res_ioc,
                        res_desc,
                        self.mapped_addr.as_ref().unwrap(),
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
                MyEndpoint::Connected(ep) => unsafe {
                    ep.compare_atomicv(
                        ioc,
                        desc,
                        comp_ioc,
                        comp_desc,
                        res_ioc,
                        res_desc,
                        start + dest_addr,
                        self.remote_key.as_ref().unwrap(),
                        op,
                    )
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }

    pub fn compare_atomicmsg<T: libfabric::AsFiType>(
        &mut self,
        msg: &Either<MsgCompareAtomic<T>, MsgCompareAtomicConnected<T>>,
        comp_ioc: &[libfabric::iovec::Ioc<T>],
        res_ioc: &mut [libfabric::iovec::IocMut<T>],
        comp_desc: &mut [MemoryRegionDesc],
        res_desc: &mut [MemoryRegionDesc],
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
            match err {
                Ok(_) => break,
                Err(err) => {
                    if !matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq());
            ft_progress(self.cq_type.rx_cq());
        }
    }
}

impl<I: CollCap> Ofi<I> {}

macro_rules! gen_info {
    ($ep_type: ident, $caps: ident, $shared_cq: literal, $ip: expr, $server: ident, $name: ident) => {
        Ofi::new(
            {
                let info = Info::new(&Version {
                    major: 1,
                    minor: 19,
                })
                .enter_hints()
                .enter_ep_attr()
                .type_($ep_type)
                .leave_ep_attr()
                .enter_domain_attr()
                .threading(libfabric::enums::Threading::Domain)
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

fn sendrecv(server: bool, name: &str, connected: bool) {
    let mut ofi = if connected {
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };

    let mut desc = [mr.descriptor(), mr.descriptor()];
    let mut ctx = ofi.info_entry.allocate_context();

    if server {
        // Send a single buffer
        ofi.send_with_context(&reg_mem[..512], &mut desc[0], None, &mut ctx);

        let completion = ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        match completion {
            Completion::Data(entry) => {
                assert!(entry[0].is_op_context_equal(&ctx))
            }
            _ => panic!("unexpected completion type"),
        }

        assert!(std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size());

        // Inject a buffer
        ofi.send(&reg_mem[..128], &mut desc[0], None);
        // No cq.sread since inject does not generate completions

        // // Send single Iov
        let iov = [IoVec::from_slice(&reg_mem[..512])];
        ofi.sendv(&iov, &mut desc[..1]);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send multi Iov
        let iov = [
            IoVec::from_slice(&reg_mem[..512]),
            IoVec::from_slice(&reg_mem[512..1024]),
        ];
        ofi.sendv(&iov, &mut desc);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    } else {
        let expected: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Receive a single buffer
        ofi.recv(&mut reg_mem[..512], &mut desc[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..512], expected[..512]);

        // Receive inject
        reg_mem.iter_mut().for_each(|v| *v = 0);
        ofi.recv(&mut reg_mem[..128], &mut desc[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..128], expected[..128]);

        reg_mem.iter_mut().for_each(|v| *v = 0);
        // // Receive into a single Iov
        let mut iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
        ofi.recvv(&mut iov, &mut desc[..1]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..512], expected[..512]);

        reg_mem.iter_mut().for_each(|v| *v = 0);

        // // Receive into multiple Iovs
        let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
        let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
        ofi.recvv(&iov, &mut desc);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        assert_eq!(mem0, &expected[..512]);
        assert_eq!(mem1, &expected[512..1024]);
    }
}

#[test]
fn sendrecv0() {
    sendrecv(true, "sendrecv0", false);
}

#[test]
fn sendrecv1() {
    sendrecv(false, "sendrecv0", false);
}

#[test]
fn conn_sendrecv0() {
    sendrecv(true, "conn_sendrecv0", true);
}

#[test]
fn conn_sendrecv1() {
    sendrecv(false, "conn_sendrecv0", true);
}

fn sendrecvdata(server: bool, name: &str, connected: bool) {
    let mut ofi = if connected {
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };

    let mut desc = [mr.descriptor(), mr.descriptor()];
    let data = Some(128u64);
    if server {
        // Send a single buffer
        ofi.send(&reg_mem[..512], &mut desc[0], data);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    } else {
        let expected: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Receive a single buffer
        ofi.recv(&mut reg_mem[..512], &mut desc[0]);

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
    sendrecvdata(true, "sendrecvdata0", false);
}

#[test]
fn sendrecvdata1() {
    sendrecvdata(false, "sendrecvdata0", false);
}

#[test]
fn conn_sendrecvdata0() {
    sendrecvdata(true, "conn_sendrecvdata0", true);
}

#[test]
fn conn_sendrecvdata1() {
    sendrecvdata(false, "conn_sendrecvdata0", true);
}

fn bind_mr<E: 'static>(ep: &MyEndpoint<E>, mr: &DisabledMemoryRegion) {
    match ep {
        MyEndpoint::Connected(ep) => mr.bind_ep(ep).unwrap(),
        MyEndpoint::Connectionless(ep) => mr.bind_ep(ep).unwrap(),
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
    let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
        .access_recv()
        .access_send()
        .build(&ofi.domain)
        .unwrap();

    let mr = match mr {
        libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };

    let mut desc = [mr.descriptor(), mr.descriptor()];
    let data = Some(128u64);

    if server {
        // Send a single buffer
        ofi.tsend(&reg_mem[..512], &mut desc[0], 10, data);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // match entry {
        //     Completion::Tagged(entry) => {assert_eq!(entry[0].data(), data.unwrap()); assert_eq!(entry[0].tag(), 10)},
        //     _ => panic!("Unexpected CQ entry format"),
        // }

        assert!(std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size());

        // Inject a buffer
        ofi.tsend(&reg_mem[..128], &mut desc[0], 1, data);
        // No cq.sread since inject does not generate completions

        // // Send single Iov
        let iov = [IoVec::from_slice(&reg_mem[..512])];
        ofi.tsendv(&iov, &mut desc[..1], 2);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send multi Iov
        let iov = [
            IoVec::from_slice(&reg_mem[..512]),
            IoVec::from_slice(&reg_mem[512..1024]),
        ];
        ofi.tsendv(&iov, &mut desc, 3);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    } else {
        let expected: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Receive a single buffer
        ofi.trecv(&mut reg_mem[..512], &mut desc[0], 10);
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
        ofi.trecv(&mut reg_mem[..128], &mut desc[0], 1);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..128], expected[..128]);

        reg_mem.iter_mut().for_each(|v| *v = 0);
        // // Receive into a single Iov
        let mut iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
        ofi.trecvv(&mut iov, &mut desc[..1], 2);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..512], expected[..512]);

        reg_mem.iter_mut().for_each(|v| *v = 0);

        // // Receive into multiple Iovs
        let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
        let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
        ofi.trecvv(&iov, &mut desc, 3);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        assert_eq!(mem0, &expected[..512]);
        assert_eq!(mem1, &expected[512..1024]);
    }
}

#[test]
fn tsendrecv0() {
    tsendrecv(true, "tsendrecv0", false);
}

#[test]
fn tsendrecv1() {
    tsendrecv(false, "tsendrecv0", false);
}

#[test]
fn conn_tsendrecv0() {
    tsendrecv(true, "conn_tsendrecv0", true);
}

#[test]
fn conn_tsendrecv1() {
    tsendrecv(false, "conn_tsendrecv0", true);
}

fn sendrecvmsg(server: bool, name: &str, connected: bool) {
    let mut ofi = if connected {
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };

    let desc = mr.descriptor();
    let mut descs = [desc.clone(), desc];
    let mapped_addr = ofi.mapped_addr.clone();
    let mut ctx = ofi.info_entry.allocate_context();
    let data = Some(128);
    if server {
        // Single iov message
        let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
        let iov0 = IoVec::from_slice(mem0);
        let iov1 = IoVec::from_slice(mem1);
        let msg = if connected {
            Either::Right(MsgConnected::from_iov(&iov0, &mut descs[0], data))
        } else {
            Either::Left(Msg::from_iov_with_context(
                &iov0,
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                data,
                &mut ctx,
            ))
        };
        ofi.sendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // let entry =
        // match entry {
        //     Completion::Data(entry) => assert_eq!(entry[0].data(), data),
        //     _ => panic!("Unexpected CQ entry format"),
        // }

        // Multi iov message with stride
        let iovs = [iov0, iov1];
        let msg = if connected {
            Either::Right(MsgConnected::from_iov_slice(&iovs, &mut descs, data))
        } else {
            Either::Left(Msg::from_iov_slice(
                &iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                data,
            ))
        };

        ofi.sendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // let entry =
        // match entry {
        //     Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
        //     _ => panic!("Unexpected CQ entry format"),
        // }

        // Single iov message
        let msg = if connected {
            Either::Right(MsgConnected::from_iov(&iovs[0], &mut descs[0], None))
        } else {
            Either::Left(Msg::from_iov(
                &iovs[0],
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                None,
            ))
        };

        ofi.sendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let msg = if connected {
            Either::Right(MsgConnected::from_iov_slice(&iovs, &mut descs, None))
        } else {
            Either::Left(Msg::from_iov_slice(
                &iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                None,
            ))
        };
        ofi.sendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
    } else {
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

        // Receive a single message in a single buffer
        let mut iov = IoVecMut::from_slice(mem0);
        let msg = if connected {
            Either::Right(MsgConnectedMut::from_iov(&mut iov, &mut descs[0], None))
        } else {
            Either::Left(MsgMut::from_iov(
                &mut iov,
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                None,
            ))
        };

        ofi.recvmsg(&msg);
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
            Either::Right(MsgConnectedMut::from_iov(&mut iov, &mut descs[0], None))
        } else {
            Either::Left(MsgMut::from_iov(
                &mut iov,
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                None,
            ))
        };

        ofi.recvmsg(&msg);
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
            Either::Right(MsgConnectedMut::from_iov_slice(&mut iovs, &mut descs, None))
        } else {
            Either::Left(MsgMut::from_iov_slice(
                &mut iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                None,
            ))
        };

        ofi.recvmsg(&msg);
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
            Either::Right(MsgConnectedMut::from_iov_slice(&mut iovs, &mut descs, None))
        } else {
            Either::Left(MsgMut::from_iov_slice(
                &mut iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                None,
            ))
        };

        ofi.recvmsg(&msg);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem0[..512], expected[..512]);
        assert_eq!(mem1[..512], expected[512..1024]);
    }
}

#[test]
fn sendrecvmsg0() {
    sendrecvmsg(true, "sendrecvmsg0", false);
}

#[test]
fn sendrecvmsg1() {
    sendrecvmsg(false, "sendrecvmsg0", false);
}

#[test]
fn conn_sendrecvmsg0() {
    sendrecvmsg(true, "conn_sendrecvmsg0", true);
}

#[test]
fn conn_sendrecvmsg1() {
    sendrecvmsg(false, "conn_sendrecvmsg0", true);
}

fn tsendrecvmsg(server: bool, name: &str, connected: bool) {
    let mut ofi = if connected {
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };

    let desc = mr.descriptor();
    let mut descs = [desc.clone(), desc];
    let mapped_addr = ofi.mapped_addr.clone();
    let data = Some(128);
    if server {
        // Single iov message
        let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
        let iov0 = IoVec::from_slice(mem0);
        let iov1 = IoVec::from_slice(mem1);
        let msg = if connected {
            Either::Right(MsgTaggedConnected::from_iov(
                &iov0,
                &mut descs[0],
                data,
                0,
                None,
            ))
        } else {
            Either::Left(MsgTagged::from_iov(
                &iov0,
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                data,
                0,
                None,
            ))
        };
        ofi.tsendmsg(&msg);
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
                &iovs, &mut descs, None, 1, None,
            ))
        } else {
            Either::Left(MsgTagged::from_iov_slice(
                &iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                None,
                1,
                None,
            ))
        };

        ofi.tsendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Single iov message
        let msg = if connected {
            Either::Right(MsgTaggedConnected::from_iov(
                &iovs[0],
                &mut descs[0],
                None,
                2,
                None,
            ))
        } else {
            Either::Left(MsgTagged::from_iov(
                &iovs[0],
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                None,
                2,
                None,
            ))
        };

        ofi.tsendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let msg = if connected {
            Either::Right(MsgTaggedConnected::from_iov_slice(
                &iovs, &mut descs, None, 3, None,
            ))
        } else {
            Either::Left(MsgTagged::from_iov_slice(
                &iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                None,
                3,
                None,
            ))
        };
        ofi.tsendmsg(&msg);
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
                &mut descs[0],
                None,
                0,
                None,
            ))
        } else {
            Either::Left(MsgTaggedMut::from_iov(
                &mut iov,
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                None,
                0,
                None,
            ))
        };

        ofi.trecvmsg(&msg);
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
                &mut descs[0],
                None,
                1,
                None,
            ))
        } else {
            Either::Left(MsgTaggedMut::from_iov(
                &mut iov,
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                None,
                1,
                None,
            ))
        };

        ofi.trecvmsg(&msg);
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
                &mut descs,
                None,
                2,
                None,
            ))
        } else {
            Either::Left(MsgTaggedMut::from_iov_slice(
                &mut iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                None,
                2,
                None,
            ))
        };

        ofi.trecvmsg(&msg);
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
                &mut descs,
                None,
                3,
                None,
            ))
        } else {
            Either::Left(MsgTaggedMut::from_iov_slice(
                &mut iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                None,
                3,
                None,
            ))
        };

        ofi.trecvmsg(&msg);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem0[..512], expected[..512]);
        assert_eq!(mem1[..512], expected[512..1024]);
    }
}

#[test]
fn tsendrecvmsg0() {
    tsendrecvmsg(true, "tsendrecvmsg0", false);
}

#[test]
fn tsendrecvmsg1() {
    tsendrecvmsg(false, "tsendrecvmsg0", false);
}

#[test]
fn conn_tsendrecvmsg0() {
    tsendrecvmsg(true, "conn_tsendrecvmsg0", true);
}

#[test]
fn conn_tsendrecvmsg1() {
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };

    let desc = mr.descriptor();
    let mut descs = [desc.clone(), desc];
    // let mapped_addr = ofi.mapped_addr.clone();
    let key = mr.key().unwrap();
    ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();
    if server {
        // Write inject a single buffer
        ofi.write(&reg_mem[..128], 0, &mut descs[0], None);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Write a single buffer
        ofi.write(&reg_mem[..512], 0, &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Write vector of buffers
        let iovs = [
            IoVec::from_slice(&reg_mem[..512]),
            IoVec::from_slice(&reg_mem[512..1024]),
        ];
        ofi.writev(&iovs, 0, &mut descs);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..128], &expected[..128]);

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);

        // Recv a completion ack
        ofi.recv(&mut reg_mem[1024..1536], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..1024], &expected[..1024]);

        reg_mem.iter_mut().for_each(|v| *v = 0);

        // Read buffer from remote memory
        ofi.read(&mut reg_mem[1024..1536], 0, &mut descs[0]);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);

        // Read vector of buffers from remote memory
        let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
        let iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
        ofi.readv(&iovs, 0, &mut descs);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        assert_eq!(mem0, &expected[..256]);
        assert_eq!(mem1, &expected[..256]);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };
    let desc = mr.descriptor();
    let mut descs = [desc.clone(), desc];
    let mapped_addr = ofi.mapped_addr.clone();

    let key = mr.key().unwrap();
    ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    let expected: Vec<u8> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

    let (start, _end) = ofi.remote_mem_addr.unwrap();
    if server {
        let rma_iov = RmaIoVec::new()
            .address(start)
            .len(128)
            .mapped_key(ofi.remote_key.as_ref().unwrap());

        let iov = IoVec::from_slice(&reg_mem[..128]);
        let msg = if connected {
            Either::Right(MsgRmaConnected::from_iov(
                &iov,
                &mut descs[0],
                &rma_iov,
                None,
            ))
        } else {
            Either::Left(MsgRma::from_iov(
                &iov,
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                &rma_iov,
                None,
            ))
        };

        // Write inject a single buffer
        ofi.writemsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let iov = IoVec::from_slice(&reg_mem[..512]);
        let rma_iov = RmaIoVec::new()
            .address(start)
            .len(512)
            .mapped_key(ofi.remote_key.as_ref().unwrap());

        let msg = if connected {
            Either::Right(MsgRmaConnected::from_iov(
                &iov,
                &mut descs[0],
                &rma_iov,
                Some(128),
            ))
        } else {
            Either::Left(MsgRma::from_iov(
                &iov,
                &mut descs[0],
                mapped_addr.as_ref().unwrap(),
                &rma_iov,
                Some(128),
            ))
        };

        // Write a single buffer
        ofi.writemsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let iov0 = IoVec::from_slice(&reg_mem[..512]);
        let iov1 = IoVec::from_slice(&reg_mem[512..1024]);
        let iovs = [iov0, iov1];
        let rma_iov0 = RmaIoVec::new()
            .address(start)
            .len(512)
            .mapped_key(ofi.remote_key.as_ref().unwrap());

        let rma_iov1 = RmaIoVec::new()
            .address(start + 512)
            .len(512)
            .mapped_key(ofi.remote_key.as_ref().unwrap());
        let rma_iovs = [rma_iov0, rma_iov1];

        let msg = if connected {
            Either::Right(MsgRmaConnected::from_iov_slice(
                &iovs, &mut descs, &rma_iovs, None,
            ))
        } else {
            Either::Left(MsgRma::from_iov_slice(
                &iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                &rma_iovs,
                None,
            ))
        };

        ofi.writemsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..128], &expected[..128]);

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);

        // Recv a completion ack
        ofi.recv(&mut reg_mem[1024..1536], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..1024], &expected[..1024]);

        reg_mem.iter_mut().for_each(|v| *v = 0);

        {
            let mut iov = IoVecMut::from_slice(&mut reg_mem[1024..1536]);
            let rma_iov = RmaIoVec::new()
                .address(start)
                .len(512)
                .mapped_key(ofi.remote_key.as_ref().unwrap());
            // Read buffer from remote memory
            let msg = if connected {
                Either::Right(MsgRmaConnectedMut::from_iov(
                    &mut iov,
                    &mut descs[0],
                    &rma_iov,
                    None
                ))
            } else {
                Either::Left(MsgRmaMut::from_iov(
                    &mut iov,
                    &mut descs[0],
                    mapped_addr.as_ref().unwrap(),
                    &rma_iov,
                    None
                ))
            };
            ofi.readmsg(&msg);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);
        }

        // // Read vector of buffers from remote memory
        let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
        let mut iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
        let rma_iov0 = RmaIoVec::new()
            .address(start)
            .len(256)
            .mapped_key(ofi.remote_key.as_ref().unwrap());
        let rma_iov1 = RmaIoVec::new()
            .address(start + 256)
            .len(256)
            .mapped_key(ofi.remote_key.as_ref().unwrap());
        let rma_iovs = [rma_iov0, rma_iov1];

        let msg = if connected {
            Either::Right(MsgRmaConnectedMut::from_iov_slice(
                &mut iovs, &mut descs, &rma_iovs, None
            ))
        } else {
            Either::Left(MsgRmaMut::from_iov_slice(
                &mut iovs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                &rma_iovs,
                None
            ))
        };
        ofi.readmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        assert_eq!(mem0, &expected[..256]);
        assert_eq!(mem1, &expected[..256]);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };
    let desc = mr.descriptor();
    let mut descs = [desc.clone(), desc];
    // let mapped_addr = ofi.mapped_addr.clone();
    let key = mr.key().unwrap();
    ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    if server {
        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Min);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Max);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Sum);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Prod);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Bor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Band);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Lor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Bxor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Land);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::Lxor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.atomic(&reg_mem[..512], 0, &mut descs[0], AtomicOp::AtomicWrite);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let iocs = [
            Ioc::from_slice(&reg_mem[..256]),
            Ioc::from_slice(&reg_mem[256..512]),
        ];

        ofi.atomicv(&iocs, 0, &mut descs, AtomicOp::Prod);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        let err = ofi.cq_type.tx_cq().sread(1, -1);
        match err {
            Err(e) => {
                if matches!(e.kind, libfabric::error::ErrorKind::ErrorAvailable) {
                    let realerr = ofi.cq_type.tx_cq().readerr(0).unwrap();
                    panic!("{:?}", realerr.error());
                }
            }
            Ok(_) => {}
        }

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let mut expected = vec![2u8; 1024 * 2];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);
        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![3; 1024 * 2];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // expected = vec![2;1024*2];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        // assert_eq!(&reg_mem[..512], &expected[..512]);

        expected = vec![4; 1024 * 2];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };

    let mut desc0 = mr.descriptor();
    let mut desc1 = mr.descriptor();
    // let mapped_addr = ofi.mapped_addr.clone();
    let key = mr.key().unwrap();
    ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    if server {
        let mut expected: Vec<_> = vec![1; 256];
        let (op_mem, ack_mem) = reg_mem.split_at_mut(512);
        let (mem0, mem1) = op_mem.split_at_mut(256);
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Min);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected[..256]);

        expected = vec![1; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Max);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![2; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Sum);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![4; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Prod);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![8; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Bor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![10; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Band);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], &mut desc0, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // Send a done ack

        ofi.recv(&mut ack_mem[..512], &mut desc0);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        expected = vec![2; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Lor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![1; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Bxor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], &mut desc0, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // Send a done ack

        ofi.recv(&mut ack_mem[..512], &mut desc0);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        expected = vec![3; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Land);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![1; 256];
        ofi.fetch_atomic(&mem0, mem1, 0, &mut desc0, &mut desc1, FetchAtomicOp::Lxor);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        expected = vec![0; 256];
        ofi.fetch_atomic(
            &mem0,
            mem1,
            0,
            &mut desc0,
            &mut desc1,
            FetchAtomicOp::AtomicWrite,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], &mut desc0, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // Send a done ack

        ofi.recv(&mut ack_mem[..512], &mut desc0);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        expected = vec![2; 256];
        ofi.fetch_atomic(
            &mem0,
            mem1,
            0,
            &mut desc0,
            &mut desc1,
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

        let desc0 = mr.descriptor();
        let desc1 = mr.descriptor();
        let desc2 = mr.descriptor();
        let desc3 = mr.descriptor();
        let mut descs = [desc0, desc1];
        let mut res_descs = [desc2, desc3];
        ofi.fetch_atomicv(
            &iocs,
            &mut res_iocs,
            0,
            &mut descs,
            &mut res_descs,
            FetchAtomicOp::Prod,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(write_mem, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut ack_mem[..512], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let mut expected = vec![2u8; 256];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut desc0);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut desc0, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![3; 256];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut desc0);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);
        ofi.send(&reg_mem[512..1024], &mut desc0, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![2; 256];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut desc0);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);
        ofi.send(&reg_mem[512..1024], &mut desc0, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![4; 256];
        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut desc0);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);
        ofi.send(&reg_mem[512..1024], &mut desc0, None);
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };
    let mut desc = mr.descriptor();
    let mut comp_desc = mr.descriptor();
    let mut res_desc = mr.descriptor();
    let key = mr.key().unwrap();
    ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
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
            &mut desc,
            &mut comp_desc,
            &mut res_desc,
            CompareAtomicOp::Cswap,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected[..256]);

        expected = vec![2; 256];
        ofi.compare_atomic(
            &buf,
            comp,
            res,
            0,
            &mut desc,
            &mut comp_desc,
            &mut res_desc,
            CompareAtomicOp::CswapNe,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        buf.iter_mut().for_each(|v| *v = 3);
        expected = vec![2; 256];
        ofi.compare_atomic(
            &buf,
            comp,
            res,
            0,
            &mut desc,
            &mut comp_desc,
            &mut res_desc,
            CompareAtomicOp::CswapLe,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        buf.iter_mut().for_each(|v| *v = 2);
        expected = vec![3; 256];
        ofi.compare_atomic(
            &buf,
            comp,
            res,
            0,
            &mut desc,
            &mut comp_desc,
            &mut res_desc,
            CompareAtomicOp::CswapLt,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        buf.iter_mut().for_each(|v| *v = 3);
        expected = vec![2; 256];
        ofi.compare_atomic(
            &buf,
            comp,
            res,
            0,
            &mut desc,
            &mut comp_desc,
            &mut res_desc,
            CompareAtomicOp::CswapGe,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        expected = vec![2; 256];
        ofi.compare_atomic(
            &buf,
            comp,
            res,
            0,
            &mut desc,
            &mut comp_desc,
            &mut res_desc,
            CompareAtomicOp::CswapGt,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], &mut desc, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        // Send a done ack

        ofi.recv(&mut ack_mem[..512], &mut desc);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();

        // expected = vec![2; 256];
        let (buf0, buf1) = buf.split_at_mut(128);
        let (comp0, comp1) = comp.split_at_mut(128);
        let (res0, res1) = res.split_at_mut(128);

        let buf_iocs = [Ioc::from_slice(&buf0), Ioc::from_slice(&buf1)];
        let comp_iocs = [Ioc::from_slice(&comp0), Ioc::from_slice(&comp1)];
        let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
        let mut buf_descs = [mr.descriptor(), mr.descriptor()];
        let mut comp_descs = [mr.descriptor(), mr.descriptor()];
        let mut res_descs = [mr.descriptor(), mr.descriptor()];

        ofi.compare_atomicv(
            &buf_iocs,
            &comp_iocs,
            &mut res_iocs,
            0,
            &mut buf_descs,
            &mut comp_descs,
            &mut res_descs,
            CompareAtomicOp::CswapLe,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], &mut desc, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut ack_mem[..512], &mut desc);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let mut expected = vec![2u8; 256];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut desc);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut desc, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        expected = vec![3; 256];
        // // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut desc);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);
        ofi.send(&reg_mem[512..1024], &mut desc, None);
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };
    let desc = mr.descriptor();
    let mut descs = [desc.clone(), desc];
    let mapped_addr = ofi.mapped_addr.clone();
    let key = mr.key().unwrap();
    ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    let (start, _end) = ofi.remote_mem_addr.unwrap();
    if server {
        let iocs = [
            Ioc::from_slice(&reg_mem[..256]),
            Ioc::from_slice(&reg_mem[256..512]),
        ];
        let rma_ioc0 = RmaIoc::new(start, 256, ofi.remote_key.as_ref().unwrap());
        let rma_ioc1 = RmaIoc::new(start + 256, 256, ofi.remote_key.as_ref().unwrap());
        let rma_iocs = [rma_ioc0, rma_ioc1];

        let msg = if connected {
            Either::Right(MsgAtomicConnected::from_ioc_slice(
                &iocs,
                &mut descs,
                &rma_iocs,
                AtomicOp::Bor,
                Some(128),
            ))
        } else {
            Either::Left(MsgAtomic::from_ioc_slice(
                &iocs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                &rma_iocs,
                AtomicOp::Bor,
                Some(128),
            ))
        };

        ofi.atomicmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let expected = vec![3u8; 1024 * 2];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..512], &expected[..512]);
        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut descs[0], None);
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };
    let mapped_addr = ofi.mapped_addr.clone();
    let key = mr.key().unwrap();
    ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    let (start, _end) = ofi.remote_mem_addr.unwrap();

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

        let desc0 = mr.descriptor();
        let desc1 = mr.descriptor();
        let desc2 = mr.descriptor();
        let desc3 = mr.descriptor();
        let mut descs = [desc0, desc1];
        let mut res_descs = [desc2, desc3];
        let rma_ioc0 = RmaIoc::new(start, 128, ofi.remote_key.as_ref().unwrap());
        let rma_ioc1 = RmaIoc::new(start + 128, 128, ofi.remote_key.as_ref().unwrap());
        let rma_iocs = [rma_ioc0, rma_ioc1];

        let msg = if connected {
            Either::Right(MsgFetchAtomicConnected::from_ioc_slice(
                &iocs,
                &mut descs,
                &rma_iocs,
                FetchAtomicOp::Prod,
                None,
            ))
        } else {
            Either::Left(MsgFetchAtomic::from_ioc_slice(
                &iocs,
                &mut descs,
                mapped_addr.as_ref().unwrap(),
                &rma_iocs,
                FetchAtomicOp::Prod,
                None,
            ))
        };

        ofi.fetch_atomicmsg(&msg, &mut res_iocs, &mut res_descs);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(write_mem, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], &mut descs[0], None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut ack_mem[..512], &mut descs[0]);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let mut desc0 = mr.descriptor();
        let expected = vec![2u8; 256];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut desc0);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut desc0, None);
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
        libfabric::mr::MaybeDisabledMemoryRegion::Disabled(mr) => {
            bind_mr(&ofi.ep, &mr);
            mr.enable().unwrap()
        }
    };

    let mut desc = mr.descriptor();
    let mapped_addr = ofi.mapped_addr.clone();
    let key = mr.key().unwrap();
    ofi.exchange_keys(key, reg_mem.as_ptr() as usize, 1024 * 2);
    let (start, _end) = ofi.remote_mem_addr.unwrap();

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
        let mut buf_descs = [mr.descriptor(), mr.descriptor()];
        let mut comp_descs = [mr.descriptor(), mr.descriptor()];
        let mut res_descs = [mr.descriptor(), mr.descriptor()];
        let rma_ioc0 = RmaIoc::new(start, 128, ofi.remote_key.as_ref().unwrap());
        let rma_ioc1 = RmaIoc::new(start + 128, 128, ofi.remote_key.as_ref().unwrap());
        let rma_iocs = [rma_ioc0, rma_ioc1];

        let msg = if connected {
            Either::Right(MsgCompareAtomicConnected::from_ioc_slice(
                &buf_iocs,
                &mut buf_descs,
                &rma_iocs,
                CompareAtomicOp::CswapGe,
                None,
            ))
        } else {
            Either::Left(MsgCompareAtomic::from_ioc_slice(
                &buf_iocs,
                &mut buf_descs,
                mapped_addr.as_ref().unwrap(),
                &rma_iocs,
                CompareAtomicOp::CswapGe,
                None,
            ))
        };

        ofi.compare_atomicmsg(
            &msg,
            &comp_iocs,
            &mut res_iocs,
            &mut comp_descs,
            &mut res_descs,
        );
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        assert_eq!(res, &expected);

        // Send a done ack
        ofi.send(&ack_mem[..512], &mut desc, None);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Recv a completion ack
        ofi.recv(&mut ack_mem[..512], &mut desc);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
    } else {
        let expected = vec![2u8; 256];

        // Recv a completion ack
        ofi.recv(&mut reg_mem[512..1024], &mut desc);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(&reg_mem[..256], &expected);

        // Send completion ack
        ofi.send(&reg_mem[512..1024], &mut desc, None);
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
