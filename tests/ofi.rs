use libfabric::{av::AddressVectorBuilder, comm::message::{RecvEp, SendEp}, cq::{CompletionQueue, CompletionQueueBuilder, ReadCq, WaitCq}, domain::{Domain, DomainBuilder}, enums::{AVOptions, CqFormat, EndpointType, TferOptions, TransferOptions}, ep::{ActiveEndpoint, Address, BaseEndpoint, Endpoint, EndpointBuilder}, eq::{EventQueue, EventQueueBuilder, WaitEq}, error::{Error, ErrorKind}, fabric::FabricBuilder, info::{Info, InfoEntry, InfoHints, Version}, infocapsoptions::{Caps, InfoCaps, MsgCap, MsgDefaultCap}, iovec::{IoVec, IoVecMut}, mr::{default_desc, MemoryRegion, MemoryRegionBuilder, MemoryRegionDesc}, msg::{Msg, MsgConnected, MsgConnectedMut, MsgMut}, Context, CqCaps, EqCaps, MappedAddress};
pub type SpinCq = libfabric::cq_caps_type!(CqCaps::WAIT);
pub type WaitableEq = libfabric::eq_caps_type!(EqCaps::WAIT);
pub mod common;
use common::IP;

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
        match  self {
            CqType::Separate((tx, _)) => tx,
            CqType::Shared(tx) => tx,
        }
    }

    pub fn rx_cq(&self) -> &CompletionQueue<SpinCq> {
        match  self {
            CqType::Separate((_, rx)) => rx,
            CqType::Shared(rx) => rx,
        }
    }
}

// pub enum EpType<I> {
//     Connected(Endpoint<I>, EventQueue<WaitableEq>),
//     Connectionless(Endpoint<I>, MappedAddress),
// }


pub struct Ofi<I> {
    pub info_entry : InfoEntry<I>,
    pub mr: Option<MemoryRegion>,
    pub domain: Domain,
    pub cq_type: CqType,
    pub ep: Endpoint<I>,
    pub mapped_addr: Option<MappedAddress>,
    pub reg_mem: Vec<u8>,
    pub tx_pending_cnt: usize,
    pub tx_complete_cnt: usize,
    pub rx_pending_cnt: usize,
    pub rx_complete_cnt: usize,
}
macro_rules!  post{
    ($post_fn:ident, $prog_fn:ident, $cq:expr, $seq:expr, $cq_cntr:expr, $ep:ident, $( $x:expr),* ) => {
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
            $prog_fn($cq, $cq_cntr);
        }
        $seq+=1;
    };
}

pub fn ft_progress(cq: &impl ReadCq, cq_cntr: &mut usize) {
    let ret = cq.read(0);
    match ret {
        Ok(_) => {panic!("Should not read anything")},
        Err(ref err) => {
            if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
                ret.unwrap();
            }
        }
    }
}


impl<I: MsgDefaultCap + Caps> Ofi<I> {

    pub fn new(info_entry: InfoEntry<I>, shared_cqs: bool, server: bool, name: &str) -> Result<Self, Error> {
        // if server {
        //     unsafe{std::env::set_var(name, "1")};
        // } else {
        //     while std::env::var(name).is_err() {
        //         std::thread::yield_now();
        //     }
        // }

        let format = if info_entry.caps().is_tagged() {
            CqFormat::Tagged
        }
        else {
            CqFormat::Context
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
        let mut reg_mem = vec![0u8;1024*1024];

        let (info_entry, ep, mapped_addr) = match ep_type {
            EndpointType::Msg | EndpointType::SockStream => {
                let eq = EventQueueBuilder::new(&fabric).build().unwrap();

                let info_entry = if server {
                    let pep = EndpointBuilder::new(&info_entry).build_passive(&fabric).unwrap();
                    pep.bind(&eq, 0).unwrap();
                    pep.listen().unwrap();
                    let event = eq.sread(-1).unwrap();
                    match event {
                        libfabric::eq::Event::ConnReq(entry) => {
                            entry.get_info().unwrap()
                        }
                        _ => panic!("Unexpected event"),
                    }
                } else {
                    info_entry
                };
                
                domain = DomainBuilder::new(&fabric, &info_entry)
                    .build().unwrap();

                cq_type = if shared_cqs {
                    CqType::Shared(shared_cq_builder.build(&domain).unwrap())
                } else {
                    CqType::Separate((tx_cq_builder.build(&domain).unwrap(), rx_cq_builder.build(&domain).unwrap()))
                };

                let ep = EndpointBuilder::new(&info_entry).build(&domain).unwrap();
                ep.bind_eq(&eq).unwrap();
                match cq_type {
                    CqType::Separate((ref tx_cq,ref rx_cq)) => ep.bind_separate_cqs(tx_cq, false, rx_cq, false).unwrap(),
                    CqType::Shared(ref scq) => ep.bind_shared_cq(&scq, false).unwrap(),
                }

                ep.enable().unwrap();
                
                if !server  {
                    ep.connect(info_entry.dest_addr().unwrap()).unwrap();
                }
                else {
                    ep.accept().unwrap();
                }
                match  eq.sread(-1) {
                    Ok(event) => {
                        match event {
                            libfabric::eq::Event::Connected(_) => {},
                            _ => panic!("Unexpected request"),
                        }
                    }
                    Err(err) => {
                        if matches!(err.kind, ErrorKind::ErrorAvailable)  {
                                let err = eq.readerr().unwrap();
                                panic!("Error in EQ: {}", eq.strerror(&err));
                        }
                        else {
                            panic!("Error in EQ: {:?}", err);
                        }
                    }
                }
                mr = if info_entry.domain_attr().mr_mode().is_local() {
                    Some(
                        MemoryRegionBuilder::new(&mut reg_mem, libfabric::enums::HmemIface::System)
                        .access_read()
                        .access_write()
                        .access_send()
                        .access_recv()
                        .build(&domain)?
                    )
                } else {
                    None
                };

                (info_entry, ep, None)
            },
            _ =>  {
                domain = DomainBuilder::new(&fabric, &info_entry)
                    .build().unwrap();

                cq_type = if shared_cqs {
                    CqType::Shared(shared_cq_builder.build(&domain).unwrap())
                } else {
                    CqType::Separate((tx_cq_builder.build(&domain).unwrap(), rx_cq_builder.build(&domain).unwrap()))
                };

                let ep = EndpointBuilder::new(&info_entry).build(&domain).unwrap();
                match cq_type {
                    CqType::Separate((ref tx_cq,ref rx_cq)) => ep.bind_separate_cqs(tx_cq, false, rx_cq, false).unwrap(),
                    CqType::Shared(ref scq) => ep.bind_shared_cq(&scq, false).unwrap(),
                }
                
                let av = match info_entry.domain_attr().av_type() {
                    libfabric::enums::AddressVectorType::Unspec => AddressVectorBuilder::new(),
                    _ => AddressVectorBuilder::new().type_(*info_entry.domain_attr().av_type()),
                }.build(&domain).unwrap();
                ep.bind_av(&av).unwrap();
                ep.enable().unwrap();

                mr = if info_entry.domain_attr().mr_mode().is_local() {
                    Some(
                        MemoryRegionBuilder::new(&mut reg_mem, libfabric::enums::HmemIface::System)
                        .access_read()
                        .access_write()
                        .access_send()
                        .access_recv()
                        .build(&domain)?
                    )
                } else {
                    None
                };
                
                let mapped_address = if let Some(dest_addr) = info_entry.dest_addr() {

                    let mapped_address = av.insert(std::slice::from_ref(dest_addr).into(), AVOptions::new()).unwrap().pop().unwrap().unwrap();    
                    let epname = ep.getname().unwrap();
                    let epname_bytes = epname.as_bytes();
                    let addrlen = epname_bytes.len();
                    reg_mem[..addrlen].copy_from_slice(epname_bytes);

                    post!(send_to, ft_progress, cq_type.tx_cq(), tx_pending_cnt, &mut tx_complete_cnt, ep, &reg_mem[..addrlen], &mut default_desc(), &mapped_address);
                    cq_type.tx_cq().sread(1, -1).unwrap();
                    
                    // ep.recv(std::slice::from_mut(&mut ack), &mut default_desc()).unwrap();
                    post!(recv, ft_progress, cq_type.rx_cq(), rx_pending_cnt, &mut rx_complete_cnt, ep, std::slice::from_mut(&mut reg_mem[0]), &mut default_desc());
                    cq_type.rx_cq().sread(1, -1).unwrap();
                    
                    mapped_address
                }
                else {
                    let epname = ep.getname().unwrap();
                    let addrlen = epname.as_bytes().len();

                    let mut mr_desc = if let Some(ref mr) = mr {
                        mr.description()
                    } else {
                        default_desc()
                    };

                    post!(recv, ft_progress, cq_type.rx_cq(), rx_pending_cnt, &mut rx_complete_cnt, ep, &mut reg_mem[..addrlen], &mut mr_desc);
                    cq_type.rx_cq().sread(1, -1).unwrap();
                    // ep.recv(&mut reg_mem, &mut mr_desc).unwrap();
                    let remote_address = unsafe {Address::from_bytes(&reg_mem)};
                    let mapped_address = av.insert(std::slice::from_ref(&remote_address).into(), AVOptions::new()).unwrap().pop().unwrap().unwrap();
                    post!(send_to, ft_progress, cq_type.tx_cq(), tx_pending_cnt, &mut tx_complete_cnt, ep, &std::slice::from_ref(&reg_mem[0]), &mut mr_desc, &mapped_address);
                    cq_type.tx_cq().sread(1, -1).unwrap();
                    
                    mapped_address
                };
                (info_entry, ep, Some(mapped_address))
            }   
        };
        if server {
            unsafe{std::env::remove_var(name)};
        }

        Ok( 
            Self {
                info_entry,
                mapped_addr,
                mr,
                cq_type,
                domain,
                ep,
                reg_mem,
                tx_pending_cnt,
                tx_complete_cnt,
                rx_pending_cnt,
                rx_complete_cnt,
            }
        )
    }
}

impl<I: MsgDefaultCap> Ofi<I> {
    pub fn send<T>(&mut self, buf: &[T], desc: &mut MemoryRegionDesc) {
        loop {
            let err = match self.mapped_addr {
                Some(ref addr) => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        self.ep.inject_to(&buf, addr)
                    } else {
                        self.ep.send_to(&buf, desc, addr)
                    }
                    
                },
                None => {
                    if buf.len() <= self.info_entry.tx_attr().inject_size() {
                        self.ep.inject(&buf)
                    } else {
                        self.ep.send(&buf, desc)
                    }
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if ! matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq(), &mut self.tx_pending_cnt);
            ft_progress(self.cq_type.rx_cq(), &mut self.rx_pending_cnt);
        } 
    }

    pub fn sendv(&mut self, iov: &[IoVec], desc: &mut [MemoryRegionDesc]) {
        loop {
            let err = match self.mapped_addr {
                Some(ref addr) => {
                    self.ep.sendv_to(iov, desc, addr)
                },
                None => {
                    self.ep.sendv(iov, desc)
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if ! matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq(), &mut self.tx_pending_cnt);
            ft_progress(self.cq_type.rx_cq(), &mut self.rx_pending_cnt);
        } 
    }

    pub fn recvv(&mut self, iov: &[IoVecMut], desc: &mut [MemoryRegionDesc]) {
        loop {
            let err = match self.mapped_addr {
                Some(ref addr) => {
                    self.ep.recvv_from(iov, desc, addr)
                },
                None => {
                    self.ep.recvv(iov, desc)
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if ! matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq(), &mut self.tx_pending_cnt);
            ft_progress(self.cq_type.rx_cq(), &mut self.rx_pending_cnt);
        } 
    }

    pub fn recv<T>(&mut self, buf: &mut [T], desc: &mut MemoryRegionDesc) {
        loop {
            let err = match self.mapped_addr {
                Some(ref addr) => {
                    self.ep.recv_from(buf, desc, addr)
                    
                },
                None => {
                    self.ep.recv(buf, desc)
                },
            };
            match err {
                Ok(_) => break,
                Err(err) => {
                    if ! matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq(), &mut self.tx_pending_cnt);
            ft_progress(self.cq_type.rx_cq(), &mut self.rx_pending_cnt);
        } 
    }

    pub fn sendmsg(&mut self, msg: &Either<Msg, MsgConnected>) {
        loop {
            let err = match msg {
                Either::Left(msg) => {
                    self.ep.sendmsg_to(msg, TferOptions::new())
                }
                Either::Right(con_msg) => {
                    self.ep.sendmsg(con_msg, TferOptions::new())
                }
            };

            match err {
                Ok(_) => break,
                Err(err) => {
                    if ! matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }


            ft_progress(self.cq_type.tx_cq(), &mut self.tx_pending_cnt);
            ft_progress(self.cq_type.rx_cq(), &mut self.rx_pending_cnt);
        }
    }

    pub fn recvmsg(&mut self, msg: &Either<MsgMut, MsgConnectedMut>) {
        loop {
            let err = match msg {
                Either::Left(msg) => {
                    self.ep.recvmsg_from(msg, TferOptions::new())
                }
                Either::Right(con_msg) => {
                    self.ep.recvmsg(con_msg, TferOptions::new())
                }
            };

            match err {
                Ok(_) => break,
                Err(err) => {
                    if ! matches!(err.kind, ErrorKind::TryAgain) {
                        panic!("{:?}", err);
                    }
                }
            }

            ft_progress(self.cq_type.tx_cq(), &mut self.tx_pending_cnt);
            ft_progress(self.cq_type.rx_cq(), &mut self.rx_pending_cnt);
        }
    }

} 


macro_rules! gen_info {
    ($ep_type: ident, $caps: ident, $shared_cq: literal, $server: ident, $name: ident) => {
        Ofi::new(if !$server {
            Info::new(&Version{major: 1, minor: 19})
                .enter_hints()
                    .enter_ep_attr()
                        .type_($ep_type)
                    .leave_ep_attr()
                    .enter_domain_attr()
                        .threading(libfabric::enums::Threading::Domain)
                        .mr_mode(libfabric::enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw())
                    .leave_domain_attr()
                    .enter_tx_attr()
                        .traffic_class(libfabric::enums::TrafficClass::LowLatency)
                    .leave_tx_attr()
                    .addr_format(libfabric::enums::AddressFormat::Unspec)
                    .caps($caps)
                .leave_hints()
                .node(IP)
                .service("9222")
                .get()
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
        } else {
            Info::new(&Version{major: 1, minor: 19})
                .enter_hints()
                    .enter_ep_attr()
                        .type_($ep_type)
                    .leave_ep_attr()
                    .enter_domain_attr()
                        .threading(libfabric::enums::Threading::Domain)
                        .mr_mode(libfabric::enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw())
                    .leave_domain_attr()
                    .enter_tx_attr()
                        .traffic_class(libfabric::enums::TrafficClass::LowLatency)
                    .leave_tx_attr()
                    .addr_format(libfabric::enums::AddressFormat::Unspec)
                    .caps($caps)
                .leave_hints()
                .source(libfabric::info::ServiceAddress::Service("9222".to_owned()))
                .get()
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
        }, $shared_cq, $server, $name).unwrap()
    };
}

fn handshake<I: Caps + MsgDefaultCap>(server: bool, name: &str, caps: Option<I>) -> Ofi<I> {
    let caps = caps.unwrap();
    let ep_type = EndpointType::Msg;
    gen_info!(ep_type, caps, false, server, name)
}

#[test]
fn handshake_connected0() {
    handshake(true, "handshake_connected0", Some(InfoCaps::new().msg()));
}

#[test]
fn handshake_connected1() {
    handshake(false, "handshake_connected0", Some(InfoCaps::new().msg()));
}


fn handshake_connectionless<I: MsgDefaultCap + Caps>(server: bool, name: &str, caps: Option<I>) -> Ofi<I> {
    let caps = caps.unwrap();
    let ep_type = EndpointType::Rdm;
    gen_info!(ep_type, caps, false, server, name)
}

#[test]
fn handshake_connectionless0() {
    handshake_connectionless(true, "handshake_connectionless0", Some(InfoCaps::new().msg()));
}

#[test]
fn handshake_connectionless1() {
    handshake_connectionless(false, "handshake_connectionless0", Some(InfoCaps::new().msg()));
}

fn sendrecv(server: bool, name: &str, connected: bool) {
    let mut ofi = if connected {
        println!("Running connected");
        handshake(server, name, Some(InfoCaps::new().msg()))
    } else {
        println!("Running connectionless");
        handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
    };

    let mut reg_mem: Vec<_> = (0..1024*2).into_iter().map(|v: usize| (v % 256) as u8).collect();
    let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
        .access_recv()
        .access_send()
        .build(&ofi.domain)
        .unwrap();

    let mut desc = [mr.description(), mr.description()];

    if server{
        // Send a single buffer
        ofi.send(&reg_mem[..512], &mut desc[0]);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        assert!(std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size());
        println!("Inject size {}", ofi.info_entry.tx_attr().inject_size());
        
        // Inject a buffer
        ofi.send(&reg_mem[..128], &mut desc[0]);
        println!("Injected");
        // No cq.sread since inject does not generate completions


        // // Send single Iov
        let iov = [IoVec::from_slice(&reg_mem[..512])];
        ofi.sendv(&iov, &mut desc[..1]);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Send multi Iov
        let iov = [IoVec::from_slice(&reg_mem[..512]), IoVec::from_slice(&reg_mem[512..1024])];
        ofi.sendv(&iov, &mut desc);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

    } else {
        let expected: Vec<_> = (0..1024*2).into_iter().map(|v: usize| (v % 256) as u8).collect();
        reg_mem.iter_mut().for_each(|v| *v = 0);
        
        // Receive a single buffer
        ofi.recv(&mut reg_mem[..512], &mut desc[0]);
        println!("Done Posting receive");
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(reg_mem[..512], expected[..512]);


        // Receive inject 
        reg_mem.iter_mut().for_each(|v| *v = 0);
        ofi.recv(&mut reg_mem[..128], &mut desc[0]);
        println!("Done Posting receive");
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

    if connected {
        ofi.ep.shutdown().unwrap();
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

fn sendrecvmsg(server: bool, name: &str, connected: bool) {
    let mut ofi = if connected {
        handshake(server, name, Some(InfoCaps::new().msg()))
    }
    else {
        handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
    };
    
    let mut reg_mem: Vec<_> = (0..1024*2).into_iter().map(|v: usize| (v % 256) as u8).collect();
    let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
        .access_recv()
        .access_send()
        .build(&ofi.domain)
        .unwrap();

    let desc = mr.description();
    let mut descs = [desc.clone(), desc];
    let mapped_addr = ofi.mapped_addr.clone();

    if server{
        // Single iov message
        let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
        let iov0 = IoVec::from_slice(mem0);
        let iov1 = IoVec::from_slice(mem1);
        let msg = if connected {
            Either::Right(MsgConnected::from_iov(&iov0, &mut descs[0]))
        }
        else {
            Either::Left(Msg::from_iov(&iov0, &mut descs[0], mapped_addr.as_ref().unwrap()))
        };
        ofi.sendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();


        // Multi iov message with stride
        let iovs = [iov0, iov1];
        let msg = if connected {
            Either::Right(MsgConnected::from_iov_slice(&iovs, &mut descs))
        } else {
            Either::Left(Msg::from_iov_slice(&iovs, &mut descs, mapped_addr.as_ref().unwrap()))
        };
        
        ofi.sendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        // Single iov message
        let msg = if connected {
            Either::Right(MsgConnected::from_iov(&iovs[0], &mut descs[0]))
        } else {
            Either::Left(Msg::from_iov(&iovs[0], &mut descs[0], mapped_addr.as_ref().unwrap()))
        };

        ofi.sendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

        let msg = if connected {
            Either::Right(MsgConnected::from_iov_slice(&iovs, &mut descs))
        } else {
            Either::Left(Msg::from_iov_slice(&iovs, &mut descs, mapped_addr.as_ref().unwrap()))
        };
        ofi.sendmsg(&msg);
        ofi.cq_type.tx_cq().sread(1, -1).unwrap();

    } else {
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let expected: Vec<_> = (0..1024).map(|v: usize| (v %256) as u8).collect();
        
        
        // Receive a single message in a single buffer
        let mut iov = IoVecMut::from_slice(mem0);
        let msg = if connected {
            Either::Right(MsgConnectedMut::from_iov(&mut iov, &mut descs[0]))
        } else {
            Either::Left(MsgMut::from_iov(&mut iov, &mut descs[0], mapped_addr.as_ref().unwrap()))
        };

        ofi.recvmsg(&msg);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem0.len(), expected[..512].len());
        assert_eq!(mem0, &expected[..512]);

        // Receive a multi iov message in a single buffer
        let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
        let msg = if connected {
            Either::Right(MsgConnectedMut::from_iov(&mut iov, &mut descs[0]))
        } else {
            
            Either::Left(MsgMut::from_iov(&mut iov, &mut descs[0], mapped_addr.as_ref().unwrap()))
        };

        ofi.recvmsg(&msg);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem1[..1024], expected);
        
        // Receive a single iov message into two buffers
        reg_mem.iter_mut().for_each(|v| *v = 0);
        let (mem0, mem1) = reg_mem.split_at_mut(512);
        let iov = IoVecMut::from_slice(&mut mem0[..256]);
        let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
        let mut iovs = [iov, iov1];
        let msg = if connected {
            Either::Right(MsgConnectedMut::from_iov_slice(&mut iovs, &mut descs))
        } else {
            
            Either::Left(MsgMut::from_iov_slice(&mut iovs, &mut descs, mapped_addr.as_ref().unwrap()))
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
            Either::Right(MsgConnectedMut::from_iov_slice(&mut iovs, &mut descs))
        } else {
            Either::Left(MsgMut::from_iov_slice(&mut iovs, &mut descs, mapped_addr.as_ref().unwrap()))
        };

        ofi.recvmsg(&msg);
        ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        assert_eq!(mem0[..512], expected[..512]);
        assert_eq!(mem1[..512], expected[512..1024]);
    }

    if connected {
        ofi.ep.shutdown().unwrap();
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