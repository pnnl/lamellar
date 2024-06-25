use core::panic;
use std::time::Instant;
use libfabric::{cntr::{Counter, CounterBuilder}, cntroptions::CntrConfig, cq::{CompletionQueue, CompletionQueueBuilder, EntryFormat, UnspecEntry}, cqoptions::{CqConfig,Options}, domain, ep::{EndpointBuilder, Endpoint, Address, PassiveEndpoint}, eq::EventQueueBuilder, eqoptions::EqConfig, fabric, Context, Waitable, infocapsoptions::{RmaCap, TagDefaultCap, MsgDefaultCap, RmaDefaultCap, RmaWriteOnlyCap, self}, MappedAddress, info::{InfoHints, Info, InfoEntry, InfoCapsImpl}, mr::{default_desc, MemoryRegionKey, MappedMemoryRegionKey}, MSG, RMA, TAG, enums::AVOptions};
use libfabric::enums;
pub enum CompMeth {
    Spin,
    Sread,
    WaitSet,
    WaitFd,
    Yield,
}
pub type EventQueueOptions = libfabric::eqoptions::Options<libfabric::eqoptions::Off, libfabric::eqoptions::WaitNoRetrieve, libfabric::eqoptions::Off>;
pub type CompletionQueueOptions = libfabric::cqoptions::Options<libfabric::cqoptions::WaitNoRetrieve, libfabric::cqoptions::Off>;
pub type CounterOptions = libfabric::cntroptions::Options<libfabric::cntroptions::WaitNoRetrieve, libfabric::cntroptions::Off>;

pub const FT_OPT_ACTIVE: u64			= 1 << 0;
pub const FT_OPT_ITER: u64			= 1 << 1;
pub const FT_OPT_SIZE: u64			= 1 << 2;
pub const FT_OPT_RX_CQ: u64			= 1 << 3;
pub const FT_OPT_TX_CQ: u64			= 1 << 4;
pub const FT_OPT_RX_CNTR: u64			= 1 << 5;
pub const FT_OPT_TX_CNTR: u64			= 1 << 6;
pub const FT_OPT_VERIFY_DATA: u64		= 1 << 7;
pub const FT_OPT_ALIGN: u64			= 1 << 8;
pub const FT_OPT_BW: u64			= 1 << 9;
pub const FT_OPT_CQ_SHARED: u64		= 1 << 10;
pub const FT_OPT_OOB_SYNC: u64			= 1 << 11;
pub const FT_OPT_SKIP_MSG_ALLOC: u64		= 1 << 12;
pub const FT_OPT_SKIP_REG_MR: u64		= 1 << 13;
pub const FT_OPT_OOB_ADDR_EXCH: u64		= 1 << 14;
pub const FT_OPT_ALLOC_MULT_MR: u64		= 1 << 15;
pub const FT_OPT_SERVER_PERSIST: u64		= 1 << 16;
pub const FT_OPT_ENABLE_HMEM: u64		= 1 << 17;
pub const FT_OPT_USE_DEVICE: u64		= 1 << 18;
pub const FT_OPT_DOMAIN_EQ: u64		= 1 << 19;
pub const FT_OPT_FORK_CHILD: u64		= 1 << 20;
pub const FT_OPT_SRX: u64			= 1 << 21;
pub const FT_OPT_STX: u64			= 1 << 22;
pub const FT_OPT_SKIP_ADDR_EXCH: u64		= 1 << 23;
pub const FT_OPT_PERF: u64			= 1 << 24;
pub const FT_OPT_DISABLE_TAG_VALIDATION: u64	= 1 << 25;
pub const FT_OPT_ADDR_IS_OOB: u64		= 1 << 26;
pub const FT_OPT_OOB_CTRL: u64			= FT_OPT_OOB_SYNC | FT_OPT_OOB_ADDR_EXCH;

pub struct TestsGlobalCtx {
    pub tx_size: usize,
    pub rx_size: usize,
    pub tx_mr_size: usize,
    pub rx_mr_size: usize,
    pub tx_seq: u64,
    pub rx_seq: u64,
    pub tx_cq_cntr: u64,
    pub rx_cq_cntr: u64,
    pub tx_buf_size: usize,
    pub rx_buf_size: usize,
    pub buf_size: usize,
    pub buf: Vec<u8>,
    pub tx_buf_index: usize,
    pub rx_buf_index: usize,
    pub max_msg_size: usize,
    pub remote_address: Option<libfabric::MappedAddress>,
    pub ft_tag: u64, 
    pub remote_cq_data: u64, 
    pub test_sizes: Vec<usize>,
    pub window_size: usize,
    pub comp_method: CompMeth,
    pub tx_ctx: Context,
    pub rx_ctx: Context,
    pub options: u64,
}


pub type MsgRma = libfabric::caps_type!(MSG, RMA);
pub type MsgTagRma = libfabric::caps_type!(MSG, TAG, RMA);


#[derive(Clone)]
pub enum HintsCaps<M: MsgDefaultCap, T: TagDefaultCap> {
    Msg(InfoHints<M>),
    Tagged(InfoHints<T>),
}


pub struct RmaInfo {
    mem_address: u64,
    len: usize,
    key: MappedMemoryRegionKey,
}

impl RmaInfo {

    pub fn new(mem_address: u64, len: usize, key: MappedMemoryRegionKey) -> Self {
        Self {
            mem_address,
            len,
            key,
        }
    }

    pub fn mem_address(&self) -> u64 {
        self.mem_address
    }
    
    pub fn mem_len(&self) -> usize {
        self.len
    }
    
    pub fn key(&self) -> &MappedMemoryRegionKey {
        &self.key
    }
}

// pub enum Caps<M: MsgDefaultCap, T: TagDefaultCap> {
//     Msg(M),
//     Tagged(T),
// }

// impl EpCap<(),()> {
//     pub fn new<M, T>(caps1: M, caps2: T) -> EpCap<M, T> {
//         EpCap::<M,T> {}
//     }
// }

pub enum CqType {
    Spin(CompletionQueue<Options<libfabric::cqoptions::WaitNone, libfabric::cqoptions::Off>>),
    Sread(CompletionQueue<Options<libfabric::cqoptions::WaitNoRetrieve, libfabric::cqoptions::Off>>),
    WaitSet(CompletionQueue<Options<libfabric::cqoptions::WaitNoRetrieve, libfabric::cqoptions::Off>>),
    WaitFd(CompletionQueue<Options<libfabric::cqoptions::WaitRetrieve, libfabric::cqoptions::On>>),
    WaitYield(CompletionQueue<Options<libfabric::cqoptions::WaitNoRetrieve, libfabric::cqoptions::Off>>),
}

impl TestsGlobalCtx {
    pub fn new( ) -> Self {
        let mem = Vec::new();
        TestsGlobalCtx { tx_size: 0, rx_size: 0, tx_mr_size: 0, rx_mr_size: 0, tx_seq: 0, rx_seq: 0, tx_cq_cntr: 0, rx_cq_cntr: 0, tx_buf_size: 0, rx_buf_size: 0, buf_size: 0, buf: mem, tx_buf_index: 0, rx_buf_index: 0, max_msg_size: 0, remote_address: None, ft_tag: 0, remote_cq_data: 0,
        test_sizes: vec![
            1 << 0,
            1 << 1,
            (1 << 1) + (1 << 0),
            1 << 2,
            (1 <<  2) + (1 <<  1),
            1 <<  3,
            (1 <<  3) + (1 <<  2),
            1 <<  4,
            (1 <<  4) + (1 <<  3),
            1 <<  5,
            (1 <<  5) + (1 <<  4),
            1 <<  6,
            (1 <<  6) + (1 <<  5),
            1 <<  7,
            (1 <<  7) + (1 <<  6),
            1 <<  8,
            (1 <<  8) + (1 <<  7),
            1 <<  9,
            (1 <<  9) + (1 <<  8),
            1 << 10,
            (1 << 10) + (1 <<  9),
            1 << 11,
            (1 << 11) + (1 << 10),
            1 << 12,
            (1 << 12) + (1 << 11),
            1 << 13,
            (1 << 13) + (1 << 12),
            1 << 14,
            (1 << 14) + (1 << 13),
            1 << 15,
            (1 << 15) + (1 << 14),
            1 << 16,
            (1 << 16) + (1 << 15),
            1 << 17,
            (1 << 17) + (1 << 16),
            1 << 18,
            (1 << 18) + (1 << 17),
            1 << 19,
            (1 << 19) + (1 << 18),
            1 << 20,
            (1 << 20) + (1 << 19),
            1 << 21,
            (1 << 21) + (1 << 20),
            1 << 22,
            (1 << 22) + (1 << 21),
            1 << 23,
        ],
        window_size: 64,
        comp_method: CompMeth::Spin, 
        tx_ctx: Context::new(),
        rx_ctx: Context::new(),
        options: FT_OPT_RX_CQ | FT_OPT_TX_CQ}
    }
}

impl Default for TestsGlobalCtx {
    fn default() -> Self {
        Self::new()
    }
}

pub fn ft_open_fabric_res<E>(info: &InfoEntry<E>) -> (libfabric::fabric::Fabric, libfabric::eq::EventQueue<EventQueueOptions>, libfabric::domain::Domain) {
    
    let fab = libfabric::fabric::FabricBuilder::new(info).build().unwrap();
    let eq = EventQueueBuilder::new(&fab).build().unwrap();
    let domain = ft_open_domain_res(info, &fab);

    (fab, eq, domain)
}

pub fn ft_open_domain_res<E>(info: &InfoEntry<E>, fab: &fabric::Fabric) -> libfabric::domain::Domain {

    libfabric::domain::DomainBuilder::new(fab, info).build().unwrap()
}


pub fn ft_alloc_ep_res<E>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx, domain: &libfabric::domain::Domain) -> (CqType, Option<libfabric::cntr::Counter<CounterOptions>>, CqType, Option<libfabric::cntr::Counter<CounterOptions>>, Option<libfabric::cntr::Counter<CounterOptions>>, Option<libfabric::av::AddressVector>){

    let format = if info.get_caps().is_tagged() {
        enums::CqFormat::TAGGED
    }
    else {
        enums::CqFormat::CONTEXT
    };

    let tx_cq_builder = CompletionQueueBuilder::new(domain)
        .size(info.get_tx_attr().get_size())
        .format(format);
    // .format_ctx();
    // .build()
    // .unwrap();
    
    let rx_cq_builder = CompletionQueueBuilder::new(domain)
        .size(info.get_rx_attr().get_size())
        .format(format);
        // .build()
        // .unwrap();
    
    let (tx_cq, rx_cq) = match gl_ctx.comp_method {
        CompMeth::Spin => {
            (CqType::Spin(tx_cq_builder.wait_none().build().unwrap()), CqType::Spin(rx_cq_builder.wait_none().build().unwrap()))
        },
        CompMeth::Sread => {
            (CqType::Sread(tx_cq_builder.build().unwrap()), CqType::Sread(rx_cq_builder.build().unwrap()))
        },
        CompMeth::WaitSet => todo!(),
        CompMeth::WaitFd => {
            (CqType::WaitFd(tx_cq_builder.wait_fd().build().unwrap()), CqType::WaitFd(rx_cq_builder.wait_fd().build().unwrap()))
        },
        CompMeth::Yield => {
            (CqType::WaitYield(tx_cq_builder.wait_yield().build().unwrap()), CqType::WaitYield(rx_cq_builder.wait_yield().build().unwrap()))
        },
    };

    
    let tx_cntr = if gl_ctx.options & FT_OPT_TX_CNTR != 0{
        Some(CounterBuilder::new(domain).build().unwrap())
    }
    else{
        None
    };

    let rx_cntr = if gl_ctx.options & FT_OPT_RX_CNTR != 0{
        Some(CounterBuilder::new(domain).build().unwrap())
    }
    else {
        None
    };

    let rma_cntr = if gl_ctx.options & FT_OPT_RX_CNTR != 0 && info.get_caps().is_rma() {
        Some(CounterBuilder::new(domain).build().unwrap())
    }
    else {
        None
    };

    let av = match info.get_ep_attr().get_type() {
        libfabric::enums::EndpointType::Rdm | libfabric::enums::EndpointType::Dgram  => {
                
                
                let av = match info.get_domain_attr().get_av_type() {
                    libfabric::enums::AddressVectorType::Unspec => libfabric::av::AddressVectorBuilder::new(domain),
                    _ => libfabric::av::AddressVectorBuilder::new(domain).type_(info.get_domain_attr().get_av_type()),
                }.count(1)
                .build()
                .unwrap();
                Some(av)
            }
        _ => None,
    };

    (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_cntr, av)
    
}

#[allow(clippy::type_complexity)]
pub fn ft_alloc_active_res<E>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx, domain: &libfabric::domain::Domain) -> (CqType, Option<libfabric::cntr::Counter<CounterOptions>>, CqType, Option<libfabric::cntr::Counter<CounterOptions>>, Option<libfabric::cntr::Counter<CounterOptions>>, libfabric::ep::Endpoint<E>, Option<libfabric::av::AddressVector>) {
    
    let (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_cntr, av) = ft_alloc_ep_res(info, gl_ctx, domain);


    let ep = EndpointBuilder::new(info).build(domain).unwrap();
    (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_cntr, ep, av)
}

#[allow(clippy::too_many_arguments)]
pub fn ft_enable_ep<T: EqConfig + 'static, CNTR: libfabric::cntroptions::CntrConfig + 'static, I, E>(info: &InfoEntry<I>, gl_ctx: &mut TestsGlobalCtx, ep: &mut libfabric::ep::Endpoint<E>, tx_cq: &CqType, rx_cq: &CqType, eq: &libfabric::eq::EventQueue<T>, av: &Option<libfabric::av::AddressVector>, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, rma_cntr: &Option<Counter<CNTR>>) {
    
    match info.get_ep_attr().get_type() {
        libfabric::enums::EndpointType::Msg => ep.bind_eq(eq).unwrap(),
        _ => if info.get_caps().is_collective() || info.get_caps().is_multicast() {
            ep.bind_eq(eq).unwrap();
        }
    }

    if let Some(av_val) = av { ep.bind_av(av_val).unwrap() }

    bind_tx_cq(tx_cq, ep, gl_ctx);
    bind_rx_cq(rx_cq, ep, gl_ctx);

    let mut bind_cntr = ep.bind_cntr();
    
    if gl_ctx.options & FT_OPT_TX_CQ == 0 {
        bind_cntr.send();
    }

    if info.get_caps().is_rma() || info.get_caps().is_atomic() {
        bind_cntr.write().read();
    }


    if let Some(cntr) = tx_cntr {
        bind_cntr.cntr(cntr).unwrap();
    }

    let mut bind_cntr = ep.bind_cntr();
    
    if gl_ctx.options & FT_OPT_RX_CQ == 0 {
        bind_cntr.recv();
    }


    if let Some(cntr) = rx_cntr {
        bind_cntr.cntr(cntr).unwrap();
    }

    if info.get_caps().is_rma() || info.get_caps().is_atomic() && info.get_caps().is_rma_event() {
        let mut bind_cntr = ep.bind_cntr();
        if info.get_caps().is_remote_write() {
            bind_cntr.remote_write();
        }
        if info.get_caps().is_remote_read() {
            bind_cntr.remote_read();
        }
        if let Some(cntr) = rma_cntr {
            bind_cntr.cntr(cntr).unwrap();
        }
    }

    ep.enable().unwrap();
}

fn bind_rx_cq<E>(rx_cq: &CqType, ep: &mut libfabric::ep::Endpoint<E>, gl_ctx: &mut TestsGlobalCtx) {

    match rx_cq {
        CqType::Spin(rx_cq) => {    
            ep.bind_cq()
                .recv(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(rx_cq).unwrap();
        }
        CqType::Sread(rx_cq) => {    
            ep.bind_cq()
                .recv(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(rx_cq).unwrap();
        }
        CqType::WaitSet(rx_cq) => {    
            ep.bind_cq()
                .recv(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(rx_cq).unwrap();
        }
        CqType::WaitFd(rx_cq) => {    
            ep.bind_cq()
                .recv(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(rx_cq).unwrap();
        }
        CqType::WaitYield(rx_cq) => {    
            ep.bind_cq()
                .recv(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(rx_cq).unwrap();
        }
    }
}

fn bind_tx_cq<E>(tx_cq: &CqType, ep: &mut libfabric::ep::Endpoint<E>, gl_ctx: &mut TestsGlobalCtx) {

    match tx_cq {
        CqType::Spin(tx_cq) => {    
            ep.bind_cq()
                .transmit(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(tx_cq).unwrap();
        }
        CqType::Sread(tx_cq) => {    
            ep.bind_cq()
                .transmit(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(tx_cq).unwrap();
        }
        CqType::WaitSet(tx_cq) => {    
            ep.bind_cq()
                .transmit(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(tx_cq).unwrap();
        }
        CqType::WaitFd(tx_cq) => {    
            ep.bind_cq()
                .transmit(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(tx_cq).unwrap();
        }
        CqType::WaitYield(tx_cq) => {    
            ep.bind_cq()
                .transmit(gl_ctx.options & FT_OPT_TX_CQ == 0)
                .cq(tx_cq).unwrap();
        }
    }
}

pub fn ft_complete_connect<T: EqConfig + libfabric::Waitable>(eq: &libfabric::eq::EventQueue<T>) { // [TODO] Do not panic, return errors
    
    // if let libfabric::eq::EventQueue::Waitable(eq) = eq {

    
    if let Ok(event) = eq.sread(-1) {
        if let libfabric::eq::Event::Connected(_) = event {
    
        }
        else {
            panic!("Unexpected Event Type");
        }
    }
    else {
        let err_entry = eq.readerr().unwrap();
        panic!("{}\n", eq.strerror(&err_entry));

    }

    
    // }
    // else {
    //     panic!("Not implemented!");
    // }
}

pub fn ft_accept_connection<EQ: EqConfig+ libfabric::Waitable, M:MsgDefaultCap, T:TagDefaultCap>(ep: &EndpointCaps<M, T>, eq: &libfabric::eq::EventQueue<EQ>) {
    match ep {
        EndpointCaps::Msg(ep) => ep.accept().unwrap(),
        EndpointCaps::Tagged(ep) => ep.accept().unwrap(),
    }
    
    
    ft_complete_connect(eq);
}

pub fn ft_retrieve_conn_req<T: EqConfig + libfabric::Waitable, E: infocapsoptions::Caps>(eq: &libfabric::eq::EventQueue<T>, pep: &PassiveEndpoint<E>) -> InfoEntry<E> { // [TODO] Do not panic, return errors
    
    let event = eq.sread(-1).unwrap();
    
    if let libfabric::eq::Event::ConnReq(entry) = event {
        entry.get_info().unwrap()
    } 
    else {
        panic!("Unexpected EventQueueEntry type");
    }
}

pub enum EndpointCaps<M: MsgDefaultCap, T: TagDefaultCap> {
    Msg(Endpoint<M>),
    Tagged(Endpoint<T>),
}
pub enum PassiveEndpointCaps<M: MsgDefaultCap, T: TagDefaultCap> {
    Msg(PassiveEndpoint<M>),
    Tagged(PassiveEndpoint<T>),
}



#[allow(clippy::type_complexity)]
pub fn ft_server_connect<T: EqConfig + libfabric::Waitable + 'static, M: infocapsoptions::Caps + MsgDefaultCap, TT: infocapsoptions::Caps +TagDefaultCap>(pep: &PassiveEndpointCaps<M, TT>, gl_ctx: &mut TestsGlobalCtx, eq: &libfabric::eq::EventQueue<T>, fab: &fabric::Fabric) -> (libfabric::domain::Domain, CqType, CqType, Option<libfabric::cntr::Counter<CounterOptions>>, Option<libfabric::cntr::Counter<CounterOptions>>, EndpointCaps<M,TT>,  Option<libfabric::mr::MemoryRegion>, Option<libfabric::mr::MemoryRegionDesc>) {

    match pep {

        PassiveEndpointCaps::Msg(pep) => {
            let new_info = ft_retrieve_conn_req(eq, pep);
            let domain = ft_open_domain_res(&new_info, fab);
            let (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_cntr, ep, _) = ft_alloc_active_res(&new_info, gl_ctx, &domain);
            let mut ep = EndpointCaps::Msg(ep);
            let (mr, mr_desc) =  ft_enable_ep_recv(&new_info, gl_ctx, &mut ep, &domain, &tx_cq, &rx_cq, eq, &None, &tx_cntr, &rx_cntr, &rma_cntr);
            ft_accept_connection(&ep, eq);
            (domain, tx_cq, rx_cq, tx_cntr, rx_cntr, ep,  mr, mr_desc)
        }
        PassiveEndpointCaps::Tagged(pep) => {
            let new_info = ft_retrieve_conn_req(eq, pep);
            let domain = ft_open_domain_res(&new_info, fab);
            let (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_cntr, ep, _) = ft_alloc_active_res(&new_info, gl_ctx, &domain);
            let mut ep = EndpointCaps::Tagged(ep);
            let (mr, mr_desc) =  ft_enable_ep_recv(&new_info, gl_ctx, &mut ep, &domain, &tx_cq, &rx_cq, eq, &None, &tx_cntr, &rx_cntr, &rma_cntr);
            ft_accept_connection(&ep, eq);
            (domain, tx_cq, rx_cq, tx_cntr, rx_cntr, ep,  mr, mr_desc)
        }
    }
}

pub fn ft_getinfo<T>(hints: InfoHints<T>, node: String, service: String, source: bool) -> Info<T> {
    let mut ep_attr = hints.get_ep_attr();


    let hints = match ep_attr.get_type() {
        libfabric::enums::EndpointType::Unspec => {ep_attr.ep_type(libfabric::enums::EndpointType::Rdm); hints.ep_attr(ep_attr)},
        _ => hints ,
    };

    let info = 
        if source {
            Info::new_source(libfabric::info::InfoSourceOpt::Service(service))
        }
        else {
            Info::new().service(&service).node(&node)
        };

     info.hints(&hints).request().unwrap()

}

pub fn ft_connect_ep<T: EqConfig + libfabric::Waitable, E>(ep: &libfabric::ep::Endpoint<E>, eq: &libfabric::eq::EventQueue<T>, addr: &libfabric::ep::Address) {
    
    ep.connect(addr).unwrap();
    ft_complete_connect(eq);
}

// fn ft_av_insert<T0>(addr: T0, count: size, fi_addr: libfabric::Address, flags: u64) {
//pub      a
// }


pub fn ft_rx_prefix_size<E>(info: &InfoEntry<E>) -> usize {

    if info.get_rx_attr().get_mode().is_msg_prefix(){ 
        info.get_ep_attr().get_max_msg_size()
    }
    else {
        0
    }
}

pub fn ft_tx_prefix_size<E>(info: &InfoEntry<E>) -> usize {

    if info.get_tx_attr().get_mode().is_msg_prefix() { 
        info.get_ep_attr().get_max_msg_size()
    }
    else {
        0
    }
}
pub const WINDOW_SIZE : usize = 64;
pub const FT_MAX_CTRL_MSG : usize = 1024;
pub const FT_RMA_SYNC_MSG_BYTES : usize = 4;

pub fn ft_set_tx_rx_sizes<E>(info: &InfoEntry<E>, max_test_size: usize, tx_size: &mut usize, rx_size: &mut usize) {
    *tx_size = max_test_size;
    if *tx_size > info.get_ep_attr().get_max_msg_size() {
        *tx_size = info.get_ep_attr().get_max_msg_size();
    }
    println!("FT PREFIX = {}", ft_rx_prefix_size(info));
    *rx_size = *tx_size + ft_rx_prefix_size(info);
    *tx_size +=  ft_tx_prefix_size(info);
}

pub fn ft_alloc_msgs<I, E>(info: &InfoEntry<I>, gl_ctx: &mut TestsGlobalCtx, domain: &libfabric::domain::Domain, ep: &libfabric::ep::Endpoint<E>) -> (Option<libfabric::mr::MemoryRegion>, Option<libfabric::mr::MemoryRegionDesc>) {

    let alignment: usize = 64;
    ft_set_tx_rx_sizes(info, *gl_ctx.test_sizes.last().unwrap(), &mut gl_ctx.tx_size, &mut gl_ctx.rx_size);
    gl_ctx.rx_buf_size = std::cmp::max(gl_ctx.rx_size, FT_MAX_CTRL_MSG) * WINDOW_SIZE;
    gl_ctx.tx_buf_size = std::cmp::max(gl_ctx.tx_size, FT_MAX_CTRL_MSG) * WINDOW_SIZE;


    let rma_resv_bytes = FT_RMA_SYNC_MSG_BYTES + std::cmp::max(ft_tx_prefix_size(info), ft_rx_prefix_size(info));
    gl_ctx.tx_buf_size += rma_resv_bytes;
    gl_ctx.rx_buf_size += rma_resv_bytes;

    gl_ctx.buf_size = gl_ctx.rx_buf_size + gl_ctx.tx_buf_size;

    gl_ctx.buf_size += alignment;
    gl_ctx.buf.resize(gl_ctx.buf_size, 0);
    println!("Buf size: {}", gl_ctx.buf_size);
    gl_ctx.max_msg_size = gl_ctx.tx_size;
    
    gl_ctx.rx_buf_index = 0;
    println!("rx_buf_index: {}", gl_ctx.rx_buf_index);
    gl_ctx.tx_buf_index = gl_ctx.rx_buf_size;
    println!("tx_buf_index: {}", gl_ctx.tx_buf_index);

    gl_ctx.remote_cq_data = ft_init_cq_data(info);

    ft_reg_mr(info, domain, ep, &mut gl_ctx.buf, 0xC0DE)
}

// pub fn ft_alloc_ctx_array(gl_ctx: &mut TestsGlobalCtx) {

// }

#[allow(clippy::too_many_arguments)]
pub fn ft_enable_ep_recv<EQ: EqConfig + 'static, CNTR: CntrConfig + 'static, E, M: MsgDefaultCap, T: TagDefaultCap>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx, ep: &mut EndpointCaps<M, T>, domain: &libfabric::domain::Domain, tx_cq: &CqType, rx_cq: &CqType, eq: &libfabric::eq::EventQueue<EQ>, av: &Option<libfabric::av::AddressVector>, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, rma_cntr: &Option<Counter<CNTR>>) -> (Option<libfabric::mr::MemoryRegion>, Option<libfabric::mr::MemoryRegionDesc>) {
    
    let (mr, mut data_desc)  = match ep {
        EndpointCaps::Msg(ep) => {
            ft_enable_ep(info, gl_ctx, ep, tx_cq, rx_cq, eq, av, tx_cntr, rx_cntr, rma_cntr);
            ft_alloc_msgs(info, gl_ctx, domain, ep)
        }
        EndpointCaps::Tagged(ep) => {
            ft_enable_ep(info, gl_ctx, ep, tx_cq, rx_cq, eq, av, tx_cntr, rx_cntr, rma_cntr);
            ft_alloc_msgs(info, gl_ctx, domain, ep)
        }
    };

    match rx_cq {
        CqType::Spin(rx_cq) => {ft_post_rx(gl_ctx, ep, std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size), NO_CQ_DATA, &mut data_desc, rx_cq)},
        CqType::Sread(rx_cq) => {ft_post_rx(gl_ctx, ep, std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size), NO_CQ_DATA, &mut data_desc, rx_cq)},
        CqType::WaitSet(rx_cq) => {ft_post_rx(gl_ctx, ep, std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size), NO_CQ_DATA, &mut data_desc, rx_cq)},
        CqType::WaitFd(rx_cq) => {ft_post_rx(gl_ctx, ep, std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size), NO_CQ_DATA, &mut data_desc, rx_cq)},
        CqType::WaitYield(rx_cq) => {ft_post_rx(gl_ctx, ep, std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size), NO_CQ_DATA, &mut data_desc, rx_cq)},
    }
    

    (mr, data_desc)
}

pub enum InfoWithCaps<M,T> {
    Msg(Info<M>),
    Tagged(Info<T>),
}

#[allow(clippy::type_complexity)]
pub fn ft_init_fabric<M: MsgDefaultCap, T: TagDefaultCap>(hints: HintsCaps<M, T>, gl_ctx: &mut TestsGlobalCtx, node: String, service: String, source: bool) -> (InfoWithCaps<M, T>, libfabric::fabric::Fabric, EndpointCaps<M, T>, libfabric::domain::Domain, CqType, CqType, Option<Counter<CounterOptions>>, Option<Counter<CounterOptions>>, libfabric::eq::EventQueue<EventQueueOptions>, Option<libfabric::mr::MemoryRegion>, libfabric::av::AddressVector, Option<libfabric::mr::MemoryRegionDesc>) {
    
    match hints {
        HintsCaps::Msg(hints) => {
            let info = ft_getinfo(hints, node.clone(), service.clone(), source);
            let entries = info.get();
            let (fabric, eq, domain) = ft_open_fabric_res(&entries[0]);
            let (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_ctr, ep, av) =  ft_alloc_active_res(&entries[0], gl_ctx, &domain);
            let mut ep =  EndpointCaps::Msg(ep);
            let (mr, mut mr_desc)  = ft_enable_ep_recv(&entries[0], gl_ctx, &mut ep, &domain, &tx_cq, &rx_cq, &eq, &av, &tx_cntr, &rx_cntr, &rma_ctr);
            let av = av.unwrap();
            ft_init_av(&entries[0], gl_ctx, &av , &ep, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&mut mr_desc, node.is_empty());
            (InfoWithCaps::Msg(info), fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, eq, mr, av, mr_desc)
        }
        HintsCaps::Tagged(hints) => {
            let info =ft_getinfo(hints, node.clone(), service.clone(), source);
            let entries = info.get();
            let (fabric, eq, domain) = ft_open_fabric_res(&entries[0]);
            let (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_ctr, ep, av) =  ft_alloc_active_res(&entries[0], gl_ctx, &domain);
            let mut ep = EndpointCaps::Tagged(ep);
            
            let (mr, mut mr_desc)  = ft_enable_ep_recv(&entries[0], gl_ctx, &mut ep, &domain, &tx_cq, &rx_cq, &eq, &av, &tx_cntr, &rx_cntr, &rma_ctr);
            let av = av.unwrap();
            ft_init_av(&entries[0], gl_ctx, &av , &ep, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&mut mr_desc, node.is_empty());
            (InfoWithCaps::Tagged(info), fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, eq, mr, av, mr_desc)
        }
    }
    // (info, fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, eq, mr, av, mr_desc)
}

pub fn ft_av_insert(av: &libfabric::av::AddressVector, addr: &Address, options: AVOptions) -> MappedAddress {

    let mut added = av.insert(std::slice::from_ref(addr), options).unwrap();
    added.pop().unwrap().expect("Could not add address to address vector")
}

pub const NO_CQ_DATA: u64 = 0;


macro_rules!  ft_post{
    ($post_fn:ident, $prog_fn:ident, $cq:ident, $seq:expr, $cq_cntr:expr, $op_str:literal, $ep:ident, $( $x:ident),* ) => {
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
            $prog_fn($cq, $seq, $cq_cntr);
        }
        $seq+=1;
    };
}

#[allow(non_camel_case_types)]
pub enum RmaOp {
    RMA_WRITE,
    RMA_WRITEDATA,
    RMA_READ,
}

pub enum SendOp {
    Send,
    MsgSend,
    Inject,
}
pub enum RecvOp {
    Recv,
    MsgRecv,
}

pub enum TagSendOp {
    TagSend,
    TagMsgSend,
    TagInject,
}

pub enum TagRecvOp {
    TagRecv,
    TagMsgRecv,
}

pub fn ft_init_cq_data<E>(info: &InfoEntry<E>) -> u64 {
    if info.get_domain_attr().get_cq_data_size() >= std::mem::size_of::<u64>() as u64 {
        0x0123456789abcdef_u64
    }
    else {
        0x0123456789abcdef & ((1 << (info.get_domain_attr().get_cq_data_size() * 8)) - 1)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ft_post_rma_inject<CQ: CqConfig, E: RmaWriteOnlyCap>(gl_ctx: &mut TestsGlobalCtx, rma_op: &RmaOp, offset: usize, size: usize, remote: &RmaInfo, ep: &libfabric::ep::Endpoint<E>, tx_cq: &CompletionQueue<CQ>) {
    
    let fi_addr = gl_ctx.remote_address.as_ref().unwrap();
    match rma_op {
        
        RmaOp::RMA_WRITE => {
            let addr = remote.mem_address() + offset as u64;
            let key = remote.key();
            let buf = &gl_ctx.buf[gl_ctx.tx_buf_index+offset..gl_ctx.tx_buf_index+offset+size];
            unsafe{ ft_post!(inject_write, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_write", ep, buf, fi_addr, addr, key); }
        }

        RmaOp::RMA_WRITEDATA => {
            let addr = remote.mem_address() + offset as u64;
            let key = remote.key();
            let buf = &gl_ctx.buf[gl_ctx.tx_buf_index+offset..gl_ctx.tx_buf_index+offset+size];
            let remote_cq_data = gl_ctx.remote_cq_data;
            unsafe{ ft_post!(inject_writedata, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_write", ep, buf, remote_cq_data, fi_addr, addr, key); }
        }
        RmaOp::RMA_READ => {
            panic!("ft_post_rma_inject does not support read");
        }
    }

    gl_ctx.tx_cq_cntr+=1;
}

#[allow(clippy::too_many_arguments)]
pub fn ft_post_rma<CQ: CqConfig, E: RmaDefaultCap>(gl_ctx: &mut TestsGlobalCtx, rma_op: &RmaOp, offset: usize, size: usize, remote: &RmaInfo, ep: &libfabric::ep::Endpoint<E>, data_desc: &mut impl libfabric::mr::DataDescriptor, tx_cq: &CompletionQueue<CQ>) {
    
    let fi_addr = gl_ctx.remote_address.as_ref().unwrap();
    match rma_op {
        
        RmaOp::RMA_WRITE => {
            let addr = remote.mem_address() + offset as u64;
            let key = remote.key();
            let buf = &gl_ctx.buf[gl_ctx.tx_buf_index+offset..gl_ctx.tx_buf_index+offset+size];
            unsafe{ ft_post!(write, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_write", ep, buf, data_desc, fi_addr, addr, key); }
        }

        RmaOp::RMA_WRITEDATA => {
            let addr = remote.mem_address() + offset as u64;
            let key = remote.key();
            let buf = &gl_ctx.buf[gl_ctx.tx_buf_index+offset..gl_ctx.tx_buf_index+offset+size];
            let remote_cq_data = gl_ctx.remote_cq_data;
            unsafe{ ft_post!(writedata, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_write", ep, buf, data_desc, remote_cq_data, fi_addr, addr, key); }
        }
        
        RmaOp::RMA_READ => {
            let addr = remote.mem_address() + offset as u64;
            let key = remote.key();
            let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index+offset..gl_ctx.tx_buf_index+offset+size];
            let _remote_cq_data = gl_ctx.remote_cq_data;
            unsafe{ ft_post!(read, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_write", ep, buf, data_desc, fi_addr, addr, key); }
        }
    }
}

pub fn msg_post<CQ: CqConfig, E: MsgDefaultCap>(op: SendOp, tx_seq: &mut u64, tx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &Option<MappedAddress>, tx_cq: &CompletionQueue<CQ>, ep: &libfabric::ep::Endpoint<E>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], data: u64) {
    match op {
        SendOp::MsgSend => {
            let iov = libfabric::iovec::IoVec::from_slice(base);
            let msg = 
                if let Some(fi_addr) = remote_address {

                    if let Some(mr_desc) = data_desc.as_mut() {
                        libfabric::msg::Msg::new(std::slice::from_ref(&iov), std::slice::from_mut(mr_desc), fi_addr)
                    }
                    else {
                        libfabric::msg::Msg::new(std::slice::from_ref(&iov), &mut [default_desc()], fi_addr)
                    }
                }
                else {
                    if let Some(mr_desc) = data_desc.as_mut() {
                        libfabric::msg::Msg::new_connected(std::slice::from_ref(&iov), std::slice::from_mut(mr_desc))
                    }
                    else {
                        libfabric::msg::Msg::new_connected(std::slice::from_ref(&iov), &mut [default_desc()])
                    }
                };
            let msg_ref = &msg;
            let flag = libfabric::enums::SendMsgOptions::new().transmit_complete();
            
            ft_post!(sendmsg, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "sendmsg", ep, msg_ref, flag);
        }
        SendOp::Send => {
            // let ctx = &mut tx_ctx;
            if let Some(fi_address) = remote_address {

                if data != NO_CQ_DATA {
                    if let Some(mr_desc) = data_desc.as_mut() {
                        ft_post!(senddata_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, data, fi_address, ctx);
                    }
                    else {
                        let mr_desc = &mut default_desc();
                        ft_post!(senddata_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, data, fi_address, ctx);
                    }
                }
                else {
                    if let Some(mr_desc) = data_desc.as_mut() {
                        ft_post!(send_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, fi_address, ctx);
                    }
                    else {
                        let mr_desc = &mut default_desc();
                        ft_post!(send_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, fi_address, ctx);
                    }
                }
            }
            else {
                if data != NO_CQ_DATA {
                    if let Some(mr_desc) = data_desc.as_mut() {
                        ft_post!(senddata_connected_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, data, ctx);
                    }
                    else {
                        let mr_desc = &mut default_desc();
                        ft_post!(senddata_connected_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, data, ctx);
                    }
                }
                else
                 {
                    if let Some(mr_desc) = data_desc.as_mut() {
                        ft_post!(send_connected_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, ctx);
                    }
                    else {
                        let mr_desc = &mut default_desc();
                        ft_post!(send_connected_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, ctx);
                    }
                }
                
            }
        }
        SendOp::Inject => {
            if let Some(fi_address) = remote_address {
                ft_post!(inject, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "inject", ep, base, fi_address);
            }
            else {
                ft_post!(inject_connected, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "inject", ep, base);
            }
        },
    }
}

pub fn msg_post_recv<CQ: CqConfig, E: MsgDefaultCap>(op: RecvOp, rx_seq: &mut u64, rx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &Option<MappedAddress>,  rx_cq: &CompletionQueue<CQ>, ep: &libfabric::ep::Endpoint<E>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], _data: u64) {
    match op {
        RecvOp::MsgRecv => {
            todo!()
        }
        RecvOp::Recv => {
            // let ctx = &mut gl_ctx.tx_ctx;
            if let Some(fi_address) = remote_address {

                if let Some(mr_desc) = data_desc.as_mut() {
                    
                    ft_post!(recv_with_context, ft_progress, rx_cq, *rx_seq, rx_cq_cntr, "receive", ep, base, mr_desc, fi_address, ctx);
                }
                else {
                    let mr_desc = &mut default_desc();
                    ft_post!(recv_with_context, ft_progress, rx_cq, *rx_seq, rx_cq_cntr, "receive", ep, base, mr_desc, fi_address, ctx);
                }
            }
            else {
                if let Some(mr_desc) = data_desc.as_mut() {
                    
                    ft_post!(recv_connected_with_context, ft_progress, rx_cq, *rx_seq, rx_cq_cntr, "receive", ep, base, mr_desc, ctx);
                }
                else {
                    let mr_desc = &mut default_desc();
                    ft_post!(recv_connected_with_context, ft_progress, rx_cq, *rx_seq, rx_cq_cntr, "receive", ep, base, mr_desc, ctx);
                }
            }
        }
    }
}


pub fn tagged_post<CQ: CqConfig,E: TagDefaultCap>(op: TagSendOp, tx_seq: &mut u64, tx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &Option<MappedAddress>,  ft_tag: u64, tx_cq: &CompletionQueue<CQ>, ep: &libfabric::ep::Endpoint<E>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], data: u64) {
    match op {
        TagSendOp::TagMsgSend => {
            let iov = libfabric::iovec::IoVec::from_slice(base);
            let msg = 
                if let Some(fi_address) = remote_address {

                    if let Some(mr_desc) = data_desc.as_mut() {
                        libfabric::msg::MsgTagged::new(std::slice::from_ref(&iov), std::slice::from_mut(mr_desc), fi_address, 0, *tx_seq, 0)
                    }
                    else {
                        libfabric::msg::MsgTagged::new(std::slice::from_ref(&iov), &mut [default_desc()], fi_address, 0, *tx_seq, 0)
                    }
                }
                else {
                    
                    if let Some(mr_desc) = data_desc.as_mut() {
                        libfabric::msg::MsgTagged::new_connected(std::slice::from_ref(&iov), std::slice::from_mut(mr_desc), 0, *tx_seq, 0)
                    }
                    else {
                        libfabric::msg::MsgTagged::new_connected(std::slice::from_ref(&iov), &mut [default_desc()], 0, *tx_seq, 0)
                    }
                };
            let msg_ref = &msg;
            let flag = libfabric::enums::TaggedSendMsgOptions::new().transmit_complete();
            
            ft_post!(tsendmsg, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "sendmsg", ep, msg_ref, flag);
        }
        TagSendOp::TagSend => {
            let op_tag = if ft_tag != 0 {ft_tag} else {*tx_seq};
            if let Some(fi_address) = remote_address {

                // let ctx = &mut tx_ctx;
                
                if data != NO_CQ_DATA {
                    if let Some(mr_desc) = data_desc.as_mut() {
                        ft_post!(tsenddata_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, data, fi_address, op_tag, ctx);
                    }
                    else {
                        let mr_desc = &mut default_desc();
                        ft_post!(tsenddata_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, data, fi_address, op_tag, ctx);
                    }
                }
                else { 
                    if let Some(mr_desc) = data_desc.as_mut() {
                        ft_post!(tsend_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, fi_address, op_tag, ctx);
                    }
                    else {
                        let mr_desc = &mut default_desc();
                        ft_post!(tsend_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, fi_address, op_tag, ctx);
                    }
                }
            }
            else {
                if data != NO_CQ_DATA {
                    if let Some(mr_desc) = data_desc.as_mut() {
                        ft_post!(tsenddata_connected_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, data, op_tag, ctx);
                    }
                    else {
                        let mr_desc = &mut default_desc();
                        ft_post!(tsenddata_connected_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, data, op_tag, ctx);
                    }
                }
                else { 
                    if let Some(mr_desc) = data_desc.as_mut() {
                        ft_post!(tsend_connected_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, op_tag, ctx);
                    }
                    else {
                        let mr_desc = &mut default_desc();
                        ft_post!(tsend_connected_with_context, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "transmit", ep, base, mr_desc, op_tag, ctx);
                    }
                }
            }
        },
        TagSendOp::TagInject => {
            let tag = *tx_seq;
            if let Some(fi_address) = remote_address {

                ft_post!(tinject, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "inject", ep, base, fi_address, tag);  
            }
            else {
                ft_post!(tinject_connected, ft_progress, tx_cq, *tx_seq, tx_cq_cntr, "inject", ep, base, tag);  
            }
        },
    }
}

pub fn tagged_post_recv<CQ: CqConfig,E: TagDefaultCap>(op: TagRecvOp, rx_seq: &mut u64, rx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &Option<MappedAddress>, ft_tag: u64,  rx_cq: &CompletionQueue<CQ>, ep: &libfabric::ep::Endpoint<E>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], _data: u64) {
    match op {
        TagRecvOp::TagMsgRecv => {
            todo!()
        }
        TagRecvOp::TagRecv => {
            let op_tag = if ft_tag != 0 {ft_tag} else {*rx_seq};
            let zero = 0;
            if let Some(fi_address) = remote_address {

                // let ctx = &mut tx_ctx;
                if let Some(mr_desc) = data_desc.as_mut() {
                    ft_post!(trecv_with_context, ft_progress, rx_cq, *rx_seq, rx_cq_cntr, "receive", ep, base, mr_desc, fi_address, op_tag, zero, ctx );
                }
                else {
                    let mr_desc = &mut default_desc();
                    ft_post!(trecv_with_context, ft_progress, rx_cq, *rx_seq, rx_cq_cntr, "receive", ep, base, mr_desc, fi_address, op_tag, zero, ctx);
                }
            }
            else {
                if let Some(mr_desc) = data_desc.as_mut() {
                    ft_post!(trecv_connected_with_context, ft_progress, rx_cq, *rx_seq, rx_cq_cntr, "receive", ep, base, mr_desc, op_tag, zero, ctx );
                }
                else {
                    let mr_desc = &mut default_desc();
                    ft_post!(trecv_connected_with_context, ft_progress, rx_cq, *rx_seq, rx_cq_cntr, "receive", ep, base, mr_desc, op_tag, zero, ctx);
                }
            }
        }
    }
}



#[allow(clippy::too_many_arguments)]
pub fn ft_post_tx<CQ: CqConfig, M: MsgDefaultCap, T: TagDefaultCap>(gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, size: usize, data: u64, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, tx_cq: &CompletionQueue<CQ>) {
    
    // size += ft_tx_prefix_size(info);
    let fi_addr = &gl_ctx.remote_address;
    let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index+size];
    match ep {
        EndpointCaps::Msg(ep) => {
            msg_post(SendOp::Send, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, fi_addr, tx_cq, ep, data_desc, buf, data);
        }
        EndpointCaps::Tagged(ep) => {
            tagged_post(TagSendOp::TagSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, fi_addr, gl_ctx.ft_tag, tx_cq, ep, data_desc, buf, data);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ft_tx<CNTR: CntrConfig + libfabric::Waitable, M: MsgDefaultCap, T: TagDefaultCap>(gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, size: usize, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, tx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>) {

    match tx_cq {
        CqType::Spin(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq);},
        CqType::Sread(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq);},
        CqType::WaitSet(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq);},
        CqType::WaitFd(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq);},
        CqType::WaitYield(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq);},
    }
    
    ft_get_tx_comp(gl_ctx, tx_cntr, tx_cq, gl_ctx.tx_seq);
}

#[allow(clippy::too_many_arguments)]
pub fn ft_post_rx<CQ: CqConfig, M: MsgDefaultCap, T: TagDefaultCap>(gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, mut size: usize, _data: u64, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, rx_cq: &CompletionQueue<CQ>) {
    size = std::cmp::max(size, FT_MAX_CTRL_MSG) ; //+  ft_tx_prefix_size(info);
    let buf = &mut gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index+size];

    match ep {
        EndpointCaps::Msg(ep) => {
            msg_post_recv(RecvOp::Recv, &mut gl_ctx.rx_seq, &mut gl_ctx.rx_cq_cntr, &mut gl_ctx.rx_ctx, &gl_ctx.remote_address, rx_cq, ep, data_desc, buf, NO_CQ_DATA);
        }
        EndpointCaps::Tagged(ep) => {
            tagged_post_recv(TagRecvOp::TagRecv, &mut gl_ctx.rx_seq, &mut gl_ctx.rx_cq_cntr, &mut gl_ctx.rx_ctx, &gl_ctx.remote_address, gl_ctx.ft_tag, rx_cq, ep, data_desc, buf, NO_CQ_DATA);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ft_rx<CNTR: CntrConfig + libfabric::Waitable, M: MsgDefaultCap, T: TagDefaultCap>(gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, _size: usize, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, rx_cq: &CqType, rx_cntr: &Option<libfabric::cntr::Counter<CNTR>>) {

    ft_get_rx_comp(gl_ctx, rx_cntr, rx_cq, gl_ctx.rx_seq);
    match rx_cq {
        CqType::Spin(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq),
        CqType::Sread(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq),
        CqType::WaitSet(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq),
        CqType::WaitFd(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq),
        CqType::WaitYield(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq),
    }
}

pub fn ft_post_inject<CQ: CqConfig, M: MsgDefaultCap, T: TagDefaultCap>(gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, size: usize, tx_cq: &libfabric::cq::CompletionQueue<CQ>) {
    // size += ft_tx_prefix_size(info);
    let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index+size];
    let fi_addr = &gl_ctx.remote_address;
    
    match ep {
        EndpointCaps::Msg(ep) => {
            msg_post(SendOp::Inject, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, fi_addr, tx_cq, ep, &mut None, buf, NO_CQ_DATA);
        }
        EndpointCaps::Tagged(ep) => {
            tagged_post(TagSendOp::TagInject, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, fi_addr, gl_ctx.ft_tag, tx_cq, ep, &mut None, buf, NO_CQ_DATA)
        }
    }
    gl_ctx.tx_cq_cntr += 1;
}

pub fn ft_inject<M: MsgDefaultCap, T:TagDefaultCap>(gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, size: usize, tx_cq: &CqType) {
    
    match tx_cq{
        CqType::Spin(tx_cq) => {ft_post_inject(gl_ctx, ep, size, tx_cq);},
        CqType::Sread(tx_cq) => {ft_post_inject(gl_ctx, ep, size, tx_cq);},
        CqType::WaitSet(tx_cq) => {ft_post_inject(gl_ctx, ep, size, tx_cq);},
        CqType::WaitFd(tx_cq) => {ft_post_inject(gl_ctx, ep, size, tx_cq);},
        CqType::WaitYield(tx_cq) => {ft_post_inject(gl_ctx, ep, size, tx_cq);},
    }
}

pub fn ft_progress<CQ: CqConfig>(cq: &libfabric::cq::CompletionQueue<CQ>, _total: u64, cq_cntr: &mut u64) {
    let ret = cq.read(1);
    match ret {
        Ok(_) => {*cq_cntr += 1;},
        Err(ref err) => {
            if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
                ret.unwrap();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ft_init_av_dst_addr<CNTR: CntrConfig + libfabric::Waitable, E, M: MsgDefaultCap, T:TagDefaultCap>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx,  av: &libfabric::av::AddressVector, ep: &EndpointCaps<M,T>, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, server: bool) {
    if !server {
        gl_ctx.remote_address = Some(ft_av_insert(av, &info.get_dest_addr(),  AVOptions::new()));
        let epname = match ep {
            EndpointCaps::Msg(ep) => {
                ep.getname().unwrap()
            }
            EndpointCaps::Tagged(ep) => {
                ep.getname().unwrap()
            }
        };
        let epname_bytes = epname.as_bytes();
        let len = epname_bytes.len();
        gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index+len].copy_from_slice(epname_bytes );

        ft_tx(gl_ctx, ep, len, data_desc, tx_cq, tx_cntr);
        ft_rx(gl_ctx, ep, 1, data_desc, rx_cq, rx_cntr);

    }
    else {
        let mut v = [0_u8; FT_MAX_CTRL_MSG];

        ft_get_rx_comp(gl_ctx, rx_cntr, rx_cq, gl_ctx.rx_seq);
        v.copy_from_slice(&gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index+FT_MAX_CTRL_MSG]);
        let address = unsafe {Address::from_bytes(&v)};

        gl_ctx.remote_address = Some(ft_av_insert(av, &address, AVOptions::new()));
        // if matches!(info.get_domain_attr().get_av_type(), libfabric::enums::AddressVectorType::Table ) {
        //     let mut zero = 0;
        //     ft_av_insert(av, &v, &mut zero, 0);
        // }
        // else {
        //     ft_av_insert(av, &v, &mut gl_ctx.remote_address, 0);
        // }
        
        match rx_cq {
            CqType::Spin(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
            CqType::Sread(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
            CqType::WaitSet(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
            CqType::WaitFd(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
            CqType::WaitYield(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
        }
        
        
        // if matches!(info.get_domain_attr().get_av_type(), libfabric::enums::AddressVectorType::Table) {
        //     gl_ctx.remote_address = 0;
        // }
        
        ft_tx(gl_ctx, ep, 1, data_desc, tx_cq, tx_cntr);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ft_init_av<CNTR: CntrConfig + libfabric::Waitable, E, M: MsgDefaultCap, T:TagDefaultCap>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx, av: &libfabric::av::AddressVector, ep: &EndpointCaps<M, T>, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, mr_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, server: bool) {

    ft_init_av_dst_addr(info, gl_ctx, av, ep,  tx_cq, rx_cq, tx_cntr, rx_cntr, mr_desc, server);
}

pub fn ft_spin_for_comp<CQ: CqConfig>(cq: &libfabric::cq::CompletionQueue<CQ>, curr: &mut u64, total: u64, _timeout: i32, _tag: u64) {
    
    while total - *curr > 0 {
        loop {

            let err = cq.read(1);
            match err {
                Ok(_) => {
                    break
                },
                Err(err) => {
                    if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
                        let err_entry = cq.readerr( 0).unwrap();
            
                        cq.print_error(&err_entry);
                        panic!("ERROR IN CQ_READ {}", err);
                    }
                     
                }
            }
        }
        *curr += 1;
    }
}

pub fn ft_wait_for_comp<CQ: CqConfig + Waitable>(cq: &libfabric::cq::CompletionQueue<CQ>, curr: &mut u64, total: u64, _timeout: i32, _tag: u64) {
    
    while total - *curr > 0 {
        let ret = cq.sread( 1, -1);
        if ret.is_ok() {
            *curr += 1;
        }
    }
}

pub fn ft_read_cq(cq: &CqType, curr: &mut u64, total: u64, timeout: i32, tag: u64) {

    match cq {
        CqType::Spin(cq) => ft_spin_for_comp(cq, curr, total, timeout, tag),
        CqType::Sread(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
        CqType::WaitSet(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
        CqType::WaitFd(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
        CqType::WaitYield(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
    }
}

pub fn ft_spin_for_cntr<CNTR: CntrConfig>(cntr: &Counter<CNTR>, total: u64) {

    loop {
        let cur = cntr.read();
        if cur >= total {
            break;
        }
    }
}

pub fn ft_wait_for_cntr<CNTR: CntrConfig + Waitable>(cntr: &Counter<CNTR>, total: u64) {

    while total > cntr.read() {
        let ret = cntr.wait(total, -1);
        if matches!(ret, Ok(())) {
            break;
        }
    }
}

pub fn ft_get_cq_comp(rx_curr: &mut u64, rx_cq: &CqType, total: u64) {
    ft_read_cq(rx_cq, rx_curr, total, -1, 0);
}

pub fn ft_get_cntr_comp<CNTR: CntrConfig + libfabric::Waitable>(cntr: &Option<Counter<CNTR>>, total: u64) {
    
    if let Some(cntr_v) = cntr{
        ft_wait_for_cntr(cntr_v, total);
    //     match cntr_v  {
    //         Counter::Waitable(cntr) => { ft_wait_for_cntr(cntr, total);}
    //         Counter::NonWaitable(cntr) => { ft_spin_for_cntr(cntr, total);}
    //     }
    }
    // else {
    //     panic!("Counter not set");
    // }
}

pub fn ft_get_rx_comp<CNTR: CntrConfig + libfabric::Waitable>(gl_ctx: &mut TestsGlobalCtx, rx_cntr: &Option<Counter<CNTR>>, rx_cq: &CqType, total: u64) {

    if gl_ctx.options & FT_OPT_RX_CQ != 0{
        ft_get_cq_comp(&mut gl_ctx.rx_cq_cntr, rx_cq, total);
    }
    else {
        ft_get_cntr_comp(rx_cntr, total);
    }
}

pub fn ft_get_tx_comp<CNTR: CntrConfig + libfabric::Waitable>(gl_ctx: &mut TestsGlobalCtx, tx_cntr: &Option<Counter<CNTR>>, tx_cq: &CqType, total: u64) {

    if gl_ctx.options & FT_OPT_RX_CQ != 0{
        ft_get_cq_comp(&mut gl_ctx.tx_cq_cntr, tx_cq, total);
    }
    else {
        ft_get_cntr_comp(tx_cntr, total);
    }
}

pub fn ft_need_mr_reg<E>(info: &InfoEntry<E>) -> bool {

    if info.get_caps().is_rma() || info.get_caps().is_atomic() {
        true
    }
    else {
        info.get_domain_attr().get_mr_mode().is_local()
    }
}

pub fn ft_chek_mr_local_flag<E>(info: &InfoEntry<E>) -> bool {
    info.get_mode().is_local_mr() || info.get_domain_attr().get_mr_mode().is_local()
}

pub fn ft_rma_read_target_allowed(caps: &InfoCapsImpl) -> bool {
    if caps.is_rma() || caps.is_atomic() {
        if caps.is_remote_read() {
            return true
        }
        else {
            return ! (caps.is_read() || caps.is_write() || caps.is_remote_write());
        }
    }

    false
}

pub fn ft_rma_write_target_allowed(caps: &InfoCapsImpl) -> bool {
    if caps.is_rma() || caps.is_atomic() {
        if caps.is_remote_write() {
            return true
        }
        else {
            return ! (caps.is_read() || caps.is_write() || caps.is_remote_write());
        }
    }

    false
}

pub fn ft_info_to_mr_builder<'a, 'b, E>(domain: &'a libfabric::domain::Domain, buff: &'b [u8], info: &InfoEntry<E>) -> libfabric::mr::MemoryRegionBuilder<'a, 'b, u8> {

    let mut mr_builder = libfabric::mr::MemoryRegionBuilder::new(domain, buff);

    if ft_chek_mr_local_flag(info) {
        if info.get_caps().is_msg() || info.get_caps().is_tagged() {
            let mut temp = info.get_caps().is_send();
            if temp {
                mr_builder = mr_builder.access_send();
            }
            temp |= info.get_caps().is_recv();
            if temp {
                mr_builder = mr_builder.access_recv();
            }
            if !temp {
                mr_builder = mr_builder.access_send().access_recv();
            }
        }
    }
    else if info.get_caps().is_rma() || info.get_caps().is_atomic() {
        if ft_rma_read_target_allowed(info.get_caps()) {
            mr_builder = mr_builder.access_remote_read();
        }
        if ft_rma_write_target_allowed(info.get_caps()) {
            mr_builder = mr_builder.access_remote_write();
        }
    }

    mr_builder
}

pub fn ft_reg_mr<I,E>(info: &InfoEntry<I>, domain: &libfabric::domain::Domain, ep: &libfabric::ep::Endpoint<E>, buf: &mut [u8], key: u64) -> (Option<libfabric::mr::MemoryRegion>, Option<libfabric::mr::MemoryRegionDesc>) {

    if ! ft_need_mr_reg(info) {
        println!("MR not needed");
        return (None, None)
    }
    // let iov = libfabric::iovec::IoVec::from_slice(buf);
    // let mut mr_attr = libfabric::mr::MemoryRegionAttr::new().iov(std::slice::from_ref(&iov)).requested_key(key).iface(libfabric::enums::HmemIface::SYSTEM);
    
    let mr = ft_info_to_mr_builder(domain, buf, info).requested_key(key).iface(libfabric::enums::HmemIface::SYSTEM).build().unwrap();

    let desc = mr.description();

    if info.get_domain_attr().get_mr_mode().is_endpoint() {
        todo!();
        // println!("MR ENDPOINT");
        // mr.bind_ep(ep).unwrap();
    }

    (mr.into(),desc.into())

}

#[allow(clippy::too_many_arguments)]
pub fn ft_sync<CNTR: CntrConfig + libfabric::Waitable, M: MsgDefaultCap, T:TagDefaultCap>( ep: &EndpointCaps<M, T>, gl_ctx: &mut TestsGlobalCtx, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, mr_desc: &mut Option<libfabric::mr::MemoryRegionDesc>) {
    
    // println!("TX SEQ: {},  TX_CTR: {}", gl_ctx.tx_seq, gl_ctx.tx_cq_cntr);
    // println!("RX SEQ: {},  RX_CTR: {}", gl_ctx.rx_seq, gl_ctx.rx_cq_cntr);
    ft_tx(gl_ctx, ep, 1, mr_desc, tx_cq, tx_cntr);
    ft_rx(gl_ctx, ep, 1, mr_desc, rx_cq, rx_cntr);
}

#[allow(clippy::too_many_arguments)]
pub fn ft_exchange_keys<CNTR: CntrConfig + libfabric::Waitable, E, M:MsgDefaultCap, T:TagDefaultCap>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx, mr: &mut libfabric::mr::MemoryRegion, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, domain: &libfabric::domain::Domain, ep: &EndpointCaps<M, T>, mr_desc: &mut Option<libfabric::mr::MemoryRegionDesc>) -> RmaInfo{
    // let mut addr ; 
    // let mut key_size = 0;
    let mut rma_iov = libfabric::iovec::RmaIoVec::new();
    
    // if info.get_domain_attr().get_mr_mode().is_raw() { 
    //     addr = mr.address( 0).unwrap(); // [TODO] Change this to return base_addr, key_size
    // }

    let len = std::mem::size_of::<libfabric::iovec::RmaIoVec>();
    // if key_size >= len - std::mem::size_of_val(&rma_iov.get_key()) {
    //     panic!("Key size does not fit");
    // }

    if info.get_domain_attr().get_mr_mode().is_basic() || info.get_domain_attr().get_mr_mode().is_virt_addr() {
        let addr = gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + ft_rx_prefix_size(info)].as_mut_ptr() as u64;
        rma_iov = rma_iov.address(addr);
    }
    

    let key = mr.key().unwrap();
    // if info.get_domain_attr().get_mr_mode().is_raw() {
    //     // panic!("Not handled currently");
    //     let mr_key = mr.raw_key(0).unwrap();
    //     let raw_key_bytes = mr_key.as_bytes();

    //     if std::mem::size_of_val(raw_key_bytes) > std::mem::size_of_val(&rma_iov.get_key()) {
    //         panic!("Key size does not fit");
    //     }
    //     else {
    //         let mut raw_key = 0u64;
    //         unsafe {std::slice::from_raw_parts_mut(&mut raw_key as *mut u64 as * mut u8, 8).copy_from_slice(raw_key_bytes)};
    //         rma_iov = rma_iov.key(raw_key);
    //     }
    // }
    // else {
    //     rma_iov = rma_iov.key(mr.key().unwrap());
    // }
    rma_iov = match key {
        MemoryRegionKey::Key(simple_key) => {
            rma_iov.key(simple_key)
        }
        MemoryRegionKey::RawKey(raw_key) => {
            if raw_key.0.len() > std::mem::size_of::<u64>() {
                todo!();
            }

            let mut key = 0u64;
            unsafe {std::slice::from_raw_parts_mut(&mut key as *mut u64 as * mut u8, 8).copy_from_slice(&raw_key.0)};
            rma_iov.key(key)
                .address(0)
        }
    };

    gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index+len].copy_from_slice(unsafe{ std::slice::from_raw_parts(&rma_iov as *const libfabric::iovec::RmaIoVec as *const u8, std::mem::size_of::<libfabric::iovec::RmaIoVec>())});
    
    ft_tx(gl_ctx, ep, len + ft_tx_prefix_size(info), mr_desc, tx_cq, tx_cntr);
    ft_get_rx_comp(gl_ctx, rx_cntr, rx_cq, gl_ctx.rx_seq);
    
    unsafe{ std::slice::from_raw_parts_mut(&mut rma_iov as *mut libfabric::iovec::RmaIoVec as *mut u8,std::mem::size_of::<libfabric::iovec::RmaIoVec>())}.copy_from_slice(&gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index+len]);
    let mr_key = unsafe{MemoryRegionKey::from_bytes(&gl_ctx.buf[(gl_ctx.rx_buf_index + len - std::mem::size_of::<u64>())..gl_ctx.rx_buf_index+len], domain)};
    let mapped_key = mr_key.into_mapped(domain).unwrap();
    let peer_info = RmaInfo::new(rma_iov.get_address(), rma_iov.get_len(), mapped_key);

    match rx_cq {
        CqType::Spin(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr_desc, rx_cq),
        CqType::Sread(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr_desc, rx_cq),
        CqType::WaitSet(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr_desc, rx_cq),
        CqType::WaitFd(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr_desc, rx_cq),
        CqType::WaitYield(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr_desc, rx_cq),
    }
    
    ft_sync(ep, gl_ctx,  tx_cq, rx_cq, tx_cntr, rx_cntr, mr_desc);

    peer_info
}

pub fn start_server<M: MsgDefaultCap, T:TagDefaultCap>(hints: HintsCaps<M, T>, node: String, service: String) -> (InfoWithCaps<M, T>, fabric::Fabric, libfabric::eq::EventQueue<EventQueueOptions>, PassiveEndpointCaps<M, T>) {
   
    match hints {
        HintsCaps::Msg(hints) => {
            let info = ft_getinfo(hints, node, service, true);
            let entries = info.get();
            
            if entries.is_empty() {
                panic!("No entires in fi_info");
            }
            let fab = libfabric::fabric::FabricBuilder::new(&entries[0]).build().unwrap();

            let eq = EventQueueBuilder::new(&fab).build().unwrap();


            let pep = EndpointBuilder::new(&entries[0])
                .build_passive(&fab)
                .unwrap();
        
            pep.bind(&eq, 0).unwrap();
            pep.listen().unwrap();


            (InfoWithCaps::Msg(info), fab, eq, PassiveEndpointCaps::Msg(pep))
        }
        HintsCaps::Tagged(hints) => {
            let info = ft_getinfo(hints, node, service, true);
            let entries = info.get();
            
            if entries.is_empty() {
                panic!("No entires in fi_info");
            }
            let fab = libfabric::fabric::FabricBuilder::new(&entries[0]).build().unwrap();

            let eq = EventQueueBuilder::new(&fab).build().unwrap();


            let pep = EndpointBuilder::new(&entries[0])
                .build_passive(&fab)
                .unwrap();
        
            pep.bind(&eq, 0).unwrap();
            pep.listen().unwrap();


            (InfoWithCaps::Tagged(info), fab, eq, PassiveEndpointCaps::Tagged(pep))
        }
    }

}

#[allow(clippy::type_complexity)]
pub fn ft_client_connect<M: MsgDefaultCap, T: TagDefaultCap>(hints: HintsCaps<M, T>, gl_ctx: &mut TestsGlobalCtx, node: String, service: String) -> (InfoWithCaps<M, T>, fabric::Fabric,  domain::Domain, libfabric::eq::EventQueue<EventQueueOptions>, CqType, CqType, Option<libfabric::cntr::Counter<CounterOptions>>, Option<libfabric::cntr::Counter<CounterOptions>>, EndpointCaps<M, T>, Option<libfabric::mr::MemoryRegion>, Option<libfabric::mr::MemoryRegionDesc>) {
    match hints {
        HintsCaps::Msg(hints) => {
            let info = ft_getinfo(hints, node, service, false);

            let entries = info.get();

            if entries.is_empty() {
                panic!("No entires in fi_info");
            }

            let (fab, eq, domain) = ft_open_fabric_res(&entries[0]);
            let (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_cntr, ep, _) = ft_alloc_active_res(&entries[0], gl_ctx,&domain);
            
            let mut ep = EndpointCaps::Msg(ep);
            let (mr, mr_desc)  = ft_enable_ep_recv(&entries[0], gl_ctx, &mut ep, &domain, &tx_cq, &rx_cq, &eq, &None, &tx_cntr, &rx_cntr, &rma_cntr);
            match ep {
                EndpointCaps::Msg(ref ep) => ft_connect_ep(ep, &eq, &entries[0].get_dest_addr()),
                EndpointCaps::Tagged(ref ep) => ft_connect_ep(ep, &eq, &entries[0].get_dest_addr()),
            }
            
            (InfoWithCaps::Msg(info), fab, domain, eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, mr, mr_desc)

        }
        HintsCaps::Tagged(hints) => {
            let info = ft_getinfo(hints, node, service, false);

            let entries = info.get();

            if entries.is_empty() {
                panic!("No entires in fi_info");
            }

            let (fab, eq, domain) = ft_open_fabric_res(&entries[0]);
            let (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_cntr, ep, _) = ft_alloc_active_res(&entries[0], gl_ctx,&domain);
            
            let mut ep = EndpointCaps::Tagged(ep);
            let (mr, mr_desc)  = ft_enable_ep_recv(&entries[0], gl_ctx, &mut ep, &domain, &tx_cq, &rx_cq, &eq, &None, &tx_cntr, &rx_cntr, &rma_cntr);
            match ep {
                EndpointCaps::Msg(ref ep) => ft_connect_ep(ep, &eq, &entries[0].get_dest_addr()),
                EndpointCaps::Tagged(ref ep) => ft_connect_ep(ep, &eq, &entries[0].get_dest_addr()),
            }
            (InfoWithCaps::Tagged(info), fab, domain, eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, mr, mr_desc)
        }
    }
    
    // (info, fab, domain, eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, mr, mr_desc)
}


#[allow(clippy::too_many_arguments)]
pub fn ft_finalize_ep<CNTR: CntrConfig + libfabric::Waitable, E, M: MsgDefaultCap, T:TagDefaultCap>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>) {

    
    let base = &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + 4 + ft_tx_prefix_size(info)];

    match ep {
        EndpointCaps::Msg(ep) => {
            match tx_cq {
                CqType::Spin(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
                CqType::Sread(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
                CqType::WaitSet(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
                CqType::WaitFd(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
                CqType::WaitYield(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
            }
        }
        EndpointCaps::Tagged(ep) => {
            match tx_cq {
                CqType::Spin(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
                CqType::Sread(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
                CqType::WaitSet(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
                CqType::WaitFd(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
                CqType::WaitYield(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, data_desc, base, NO_CQ_DATA);},
            }
        }
    }

    ft_get_tx_comp(gl_ctx, tx_cntr, tx_cq, gl_ctx.tx_seq);
    ft_get_rx_comp(gl_ctx, rx_cntr, rx_cq, gl_ctx.rx_seq);

}

#[allow(clippy::too_many_arguments)]
pub fn ft_finalize<CNTR: CntrConfig + libfabric::Waitable, E, M: MsgDefaultCap, T: TagDefaultCap>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, domain: &libfabric::domain::Domain, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>) {

    // if info.get_domain_attr().get_mr_mode().is_raw() { 
    //     domain.unmap_key(0xC0DE).unwrap();
    // }

    ft_finalize_ep(info, gl_ctx, ep, data_desc, tx_cq, rx_cq, tx_cntr, rx_cntr);
}

// pub fn close_all_pep(fab: libfabric::fabric::Fabric, domain: libfabric::domain::Domain, eq :libfabric::eq::EventQueue, rx_cq: libfabric::cq::CompletionQueue, tx_cq: libfabric::cq::CompletionQueue, ep: libfabric::ep::Endpoint<E>, pep: libfabric::ep::PassiveEndpoint, mr: Option<libfabric::mr::MemoryRegion>) {
//     ep.close().unwrap();
//     pep.close().unwrap();
//     eq.close().unwrap();
//     tx_cq.close().unwrap();
//     rx_cq.close().unwrap();
//     if let Some(mr_val) = mr { mr_val.close().unwrap(); }
//     domain.close().unwrap();
//     fab.close().unwrap();        
// }

// pub fn close_all(fab: &mut libfabric::fabric::Fabric, domain: &mut libfabric::domain::Domain, eq :&mut libfabric::eq::EventQueue, rx_cq: &mut libfabric::cq::CompletionQueue, tx_cq: &mut libfabric::cq::CompletionQueue, tx_cntr: Option<Counter>, rx_cntr: Option<Counter>, ep: &mut libfabric::ep::Endpoint<E>, mr: Option<&mut libfabric::mr::MemoryRegion>, av: Option<&mut libfabric::av::AddressVector>) {
    
//     ep.close().unwrap();
//     eq.close().unwrap();
//     tx_cq.close().unwrap();
//     rx_cq.close().unwrap();
//     if let Some(mr_val) = mr { mr_val.close().unwrap() }
//     if let Some(av_val) = av { av_val.close().unwrap() }
//     if let Some(rxcntr_val) = rx_cntr { rxcntr_val.close().unwrap() }
//     if let Some(txcnr_val) = tx_cntr { txcnr_val.close().unwrap() }
//     domain.close().unwrap();
//     fab.close().unwrap();    
// }

#[allow(clippy::too_many_arguments)]
pub fn pingpong<CNTR: CntrConfig + libfabric::Waitable, M: MsgDefaultCap, T: TagDefaultCap>(inject_size: usize, gl_ctx: &mut TestsGlobalCtx, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, ep: &EndpointCaps<M, T>, mr_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, iters: usize, warmup: usize, size: usize, server: bool) {
    // let inject_size = info.get_tx_attr().get_inject_size();

    ft_sync(ep, gl_ctx, tx_cq, rx_cq, tx_cntr, rx_cntr, mr_desc);
    let mut now = Instant::now();
    if ! server {
        for i in 0..warmup+iters {
            if i == warmup {
                now = Instant::now();    // Start timer
            }

            if size < inject_size {
                ft_inject(gl_ctx, ep, size, tx_cq);
            }
            else {
                ft_tx(gl_ctx, ep, size, mr_desc, tx_cq, tx_cntr);
            }

            ft_rx(gl_ctx, ep, size, mr_desc, rx_cq, rx_cntr);
        }
    }
    else {
        for i in 0..warmup+iters {
            if i == warmup {
                // Start timer
            }

            ft_rx(gl_ctx, ep, size, mr_desc, rx_cq, rx_cntr);


            if size < inject_size {
                ft_inject(gl_ctx, ep, size, tx_cq);
            }
            else {
                ft_tx(gl_ctx, ep, size, mr_desc, tx_cq, tx_cntr);
            }
        }
    }
    if size == 1 {
        println!("bytes iters total time MB/sec usec/xfer Mxfers/sec", );
    }
    // println!("Done");
    // Stop timer
    let elapsed = now.elapsed();
    let bytes = iters * size * 2;
    let usec_per_xfer = elapsed.as_micros() as f64 /iters as f64 / 2_f64;
    println!("{} {} {} {} s {} {} {}", size, iters, bytes, elapsed.as_secs(), bytes as f64 /elapsed.as_micros() as f64, usec_per_xfer, 1.0/usec_per_xfer);
    // print perf data
}

#[allow(clippy::too_many_arguments)]
pub fn bw_tx_comp<CNTR: CntrConfig + libfabric::Waitable, M: MsgDefaultCap, T: TagDefaultCap>(gl_ctx: &mut TestsGlobalCtx, ep: &EndpointCaps<M, T>, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, mr_desc: &mut Option<libfabric::mr::MemoryRegionDesc>) {

    ft_get_tx_comp(gl_ctx, tx_cntr, tx_cq, gl_ctx.tx_seq);
    ft_rx(gl_ctx, ep, FT_RMA_SYNC_MSG_BYTES, mr_desc, rx_cq, rx_cntr);
}

#[allow(clippy::too_many_arguments)]
pub fn bw_rma_comp<CNTR: CntrConfig + libfabric::Waitable, M:MsgDefaultCap, T: TagDefaultCap>(gl_ctx: &mut TestsGlobalCtx, op: &RmaOp, ep: &EndpointCaps<M, T>, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, mr_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, server: bool) {
    if matches!(op, RmaOp::RMA_WRITEDATA) {
        if ! server {
            bw_tx_comp(gl_ctx, ep, tx_cq, rx_cq, tx_cntr, rx_cntr, mr_desc);
        }
    }
    else {
        ft_get_tx_comp(gl_ctx, tx_cntr, tx_cq, gl_ctx.tx_seq);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn pingpong_rma<CNTR: CntrConfig + libfabric::Waitable, E: RmaCap, M: MsgDefaultCap + RmaDefaultCap, T: TagDefaultCap + RmaDefaultCap>(info: &InfoEntry<E>, gl_ctx: &mut TestsGlobalCtx, tx_cq: &CqType, rx_cq: &CqType, tx_cntr: &Option<Counter<CNTR>>, rx_cntr: &Option<Counter<CNTR>>, ep: &EndpointCaps<M, T>, mr_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, op: RmaOp, remote: &RmaInfo, iters: usize, warmup: usize, size: usize, server: bool) {
    let inject_size = info.get_tx_attr().get_inject_size();

    ft_sync(ep, gl_ctx, tx_cq, rx_cq, tx_cntr, rx_cntr, mr_desc);
    let offest_rma_start = FT_RMA_SYNC_MSG_BYTES + std::cmp::max(ft_tx_prefix_size(info), ft_rx_prefix_size(info));


    let mut now = Instant::now();
    let mut j = 0;
    let mut offset = 0;
    for i in 0..warmup+iters {
        if i == warmup {
            now = Instant::now();    // Start timer
        }
        if j == 0 {
            offset = offest_rma_start;
        }

        if matches!(&op, RmaOp::RMA_WRITE) {

            match ep {
                EndpointCaps::Msg(ep) => {
                    if size < inject_size {
                        match tx_cq {
                            CqType::Spin(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                            CqType::Sread(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                            CqType::WaitSet(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                            CqType::WaitFd(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                            CqType::WaitYield(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                        }
                    }
                    else {
                        match tx_cq {
                            CqType::Spin(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                            CqType::Sread(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                            CqType::WaitSet(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                            CqType::WaitFd(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                            CqType::WaitYield(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                        }
                    }
                }
                EndpointCaps::Tagged(ep) => {
                    if size < inject_size {
                        match tx_cq {
                            CqType::Spin(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                            CqType::Sread(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                            CqType::WaitSet(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                            CqType::WaitFd(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                            CqType::WaitYield(tx_cq) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq),
                        }
                    }
                    else {
                        match tx_cq {
                            CqType::Spin(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                            CqType::Sread(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                            CqType::WaitSet(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                            CqType::WaitFd(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                            CqType::WaitYield(tx_cq) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), tx_cq),
                        }
                    }
                }
            }

        }


        j+=1;

        if j == gl_ctx.window_size {
            bw_rma_comp(gl_ctx, &op, ep, tx_cq, rx_cq, tx_cntr, rx_cntr, mr_desc, server);
            j = 0;
        }

        offset += size;
    }

    bw_rma_comp(gl_ctx, &op, ep, tx_cq, rx_cq, tx_cntr, rx_cntr, mr_desc, server);
    

    if size == 1 {
        println!("bytes iters total time MB/sec usec/xfer Mxfers/sec");
    }
    // println!("Done");
    // Stop timer
    let elapsed = now.elapsed();
    let bytes = iters * size * 2;
    let usec_per_xfer = elapsed.as_micros() as f64 /iters as f64 / 2_f64;
    println!("{} {} {} {} s {} {} {}", size, iters, bytes, elapsed.as_secs(), bytes as f64 /elapsed.as_micros() as f64, usec_per_xfer, 1.0/usec_per_xfer);
    // print perf data
}
