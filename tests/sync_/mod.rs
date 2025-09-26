// use core::panic;
// use libfabric::conn_ep::AcceptPendingEndpoint;
// use libfabric::conn_ep::ConnectionPendingEndpoint;
// use libfabric::connless_ep::ConnectionlessEndpoint;
// use libfabric::mr::MemoryRegion;
// use libfabric::{
//     cntr::{Counter, CounterBuilder, ReadCntr, WaitCntr},
//     comm::{
//         message::{ConnectedRecvEp, ConnectedSendEp, RecvEp, SendEp},
//         rma::{ReadWriteEp, WriteEp},
//         tagged::{ConnectedTagRecvEp, ConnectedTagSendEp, TagRecvEp, TagSendEp},
//     },
//     conn_ep::{ConnectedEndpoint, UnconnectedEndpoint},
//     cq::{CompletionQueue, CompletionQueueBuilder, ReadCq, WaitCq},
//     domain::{BoundDomain, Domain},
//     enums::AVOptions,
//     ep::{Address, BaseEndpoint, Endpoint, EndpointBuilder, PassiveEndpoint},
//     eq::{EventQueueBuilder, ReadEq, WaitEq},
//     fabric,
//     info::{Info, InfoCapsImpl, InfoEntry, InfoHints},
//     infocapsoptions::{self, MsgDefaultCap, RmaCap, RmaDefaultCap, TagDefaultCap},
//     Context, MappedAddress,
// };
// use libfabric::{enums, MemAddressInfo, RemoteMemAddressInfo};
// use std::time::Instant;


// pub type EventQueueOptions = libfabric::eq_caps_type!(EqCaps::WAIT);
// pub type DefaultCntr = libfabric::cntr_caps_type!(CntrCaps::WAIT);

// pub const FT_OPT_ACTIVE: u64 = 1 << 0;
// pub const FT_OPT_ITER: u64 = 1 << 1;
// pub const FT_OPT_SIZE: u64 = 1 << 2;
// pub const FT_OPT_RX_CQ: u64 = 1 << 3;
// pub const FT_OPT_TX_CQ: u64 = 1 << 4;
// pub const FT_OPT_RX_CNTR: u64 = 1 << 5;
// pub const FT_OPT_TX_CNTR: u64 = 1 << 6;
// pub const FT_OPT_VERIFY_DATA: u64 = 1 << 7;
// pub const FT_OPT_ALIGN: u64 = 1 << 8;
// pub const FT_OPT_BW: u64 = 1 << 9;
// pub const FT_OPT_CQ_SHARED: u64 = 1 << 10;
// pub const FT_OPT_OOB_SYNC: u64 = 1 << 11;
// pub const FT_OPT_SKIP_MSG_ALLOC: u64 = 1 << 12;
// pub const FT_OPT_SKIP_REG_MR: u64 = 1 << 13;
// pub const FT_OPT_OOB_ADDR_EXCH: u64 = 1 << 14;
// pub const FT_OPT_ALLOC_MULT_MR: u64 = 1 << 15;
// pub const FT_OPT_SERVER_PERSIST: u64 = 1 << 16;
// pub const FT_OPT_ENABLE_HMEM: u64 = 1 << 17;
// pub const FT_OPT_USE_DEVICE: u64 = 1 << 18;
// pub const FT_OPT_DOMAIN_EQ: u64 = 1 << 19;
// pub const FT_OPT_FORK_CHILD: u64 = 1 << 20;
// pub const FT_OPT_SRX: u64 = 1 << 21;
// pub const FT_OPT_STX: u64 = 1 << 22;
// pub const FT_OPT_SKIP_ADDR_EXCH: u64 = 1 << 23;
// pub const FT_OPT_PERF: u64 = 1 << 24;
// pub const FT_OPT_DISABLE_TAG_VALIDATION: u64 = 1 << 25;
// pub const FT_OPT_ADDR_IS_OOB: u64 = 1 << 26;
// pub const FT_OPT_OOB_CTRL: u64 = FT_OPT_OOB_SYNC | FT_OPT_OOB_ADDR_EXCH;

// pub struct TestsGlobalCtx {
//     pub tx_size: usize,
//     pub rx_size: usize,
//     pub tx_mr_size: usize,
//     pub rx_mr_size: usize,
//     pub tx_seq: u64,
//     pub rx_seq: u64,
//     pub tx_cq_cntr: u64,
//     pub rx_cq_cntr: u64,
//     pub tx_buf_size: usize,
//     pub rx_buf_size: usize,
//     pub buf_size: usize,
//     pub buf: Vec<u8>,
//     pub tx_buf_index: usize,
//     pub rx_buf_index: usize,
//     pub max_msg_size: usize,
//     pub remote_address: Option<MappedAddress>,
//     pub ft_tag: u64,
//     pub remote_cq_data: u64,
//     pub test_sizes: Vec<usize>,
//     pub window_size: usize,
//     pub comp_method: CompMeth,
//     pub tx_ctx: Option<Context>,
//     pub rx_ctx: Option<Context>,
//     pub options: u64,
// }
// use libfabric::CntrCaps;
// use libfabric::CqCaps;
// use libfabric::EqCaps;
// use libfabric::FabInfoCaps;

// pub type MsgRma = libfabric::info_caps_type!(FabInfoCaps::MSG, FabInfoCaps::RMA);
// pub type MsgTagRma =
//     libfabric::info_caps_type!(FabInfoCaps::MSG, FabInfoCaps::TAG, FabInfoCaps::RMA);
// pub type SpinCq = libfabric::cq_caps_type!();
// pub type SreadCq = libfabric::cq_caps_type!(CqCaps::WAIT);
// pub type FdCq = libfabric::cq_caps_type!(CqCaps::WAIT, CqCaps::RETRIEVE, CqCaps::FD);

// // #[derive(Clone)]
// pub enum HintsCaps<M: MsgDefaultCap, T: TagDefaultCap> {
//     Msg(InfoHints<M>),
//     Tagged(InfoHints<T>),
// }

// // pub enum Caps<M: MsgDefaultCap, T: TagDefaultCap> {
// //     Msg(M),
// //     Tagged(T),
// // }

// // impl EpCap<(),()> {
// //     pub fn new<M, T>(caps1: M, caps2: T) -> EpCap<M, T> {
// //         EpCap::<M,T> {}
// //     }
// // }

// pub enum EqCqOpt<T> {
//     Shared(CompletionQueue<T>),
//     Separate(CompletionQueue<T>, CompletionQueue<T>),
// }

// pub enum CqType {
//     Spin(EqCqOpt<SpinCq>),
//     Sread(EqCqOpt<SreadCq>),
//     WaitSet(EqCqOpt<SreadCq>),
//     WaitFd(EqCqOpt<FdCq>),
//     WaitYield(EqCqOpt<SreadCq>),
// }

// impl TestsGlobalCtx {
//     pub fn new() -> Self {
//         let mem = Vec::new();
//         TestsGlobalCtx {
//             tx_size: 0,
//             rx_size: 0,
//             tx_mr_size: 0,
//             rx_mr_size: 0,
//             tx_seq: 0,
//             rx_seq: 0,
//             tx_cq_cntr: 0,
//             rx_cq_cntr: 0,
//             tx_buf_size: 0,
//             rx_buf_size: 0,
//             buf_size: 0,
//             buf: mem,
//             tx_buf_index: 0,
//             rx_buf_index: 0,
//             max_msg_size: 0,
//             remote_address: None,
//             ft_tag: 0,
//             remote_cq_data: 0,
//             test_sizes: vec![
//                 1 << 0,
//                 1 << 1,
//                 (1 << 1) + (1 << 0),
//                 1 << 2,
//                 (1 << 2) + (1 << 1),
//                 1 << 3,
//                 (1 << 3) + (1 << 2),
//                 1 << 4,
//                 (1 << 4) + (1 << 3),
//                 1 << 5,
//                 (1 << 5) + (1 << 4),
//                 1 << 6,
//                 (1 << 6) + (1 << 5),
//                 1 << 7,
//                 (1 << 7) + (1 << 6),
//                 1 << 8,
//                 (1 << 8) + (1 << 7),
//                 1 << 9,
//                 (1 << 9) + (1 << 8),
//                 1 << 10,
//                 (1 << 10) + (1 << 9),
//                 1 << 11,
//                 (1 << 11) + (1 << 10),
//                 1 << 12,
//                 (1 << 12) + (1 << 11),
//                 1 << 13,
//                 (1 << 13) + (1 << 12),
//                 1 << 14,
//                 (1 << 14) + (1 << 13),
//                 1 << 15,
//                 (1 << 15) + (1 << 14),
//                 1 << 16,
//                 (1 << 16) + (1 << 15),
//                 1 << 17,
//                 (1 << 17) + (1 << 16),
//                 1 << 18,
//                 (1 << 18) + (1 << 17),
//                 1 << 19,
//                 (1 << 19) + (1 << 18),
//                 1 << 20,
//                 (1 << 20) + (1 << 19),
//                 1 << 21,
//                 (1 << 21) + (1 << 20),
//                 1 << 22,
//                 (1 << 22) + (1 << 21),
//                 1 << 23,
//             ],
//             window_size: 64,
//             comp_method: CompMeth::Spin,
//             tx_ctx: None,
//             rx_ctx: None,
//             options: FT_OPT_RX_CQ | FT_OPT_TX_CQ | FT_OPT_CQ_SHARED,
//         }
//     }
// }

// impl Default for TestsGlobalCtx {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// pub enum ConfDomain {
//     Unbound(Domain),
//     Bound(BoundDomain),
// }

// pub fn ft_open_fabric_res<E>(
//     info: &InfoEntry<E>,
// ) -> (
//     libfabric::fabric::Fabric,
//     libfabric::eq::EventQueue<EventQueueOptions>,
//     ConfDomain,
// ) {
//     let fab = libfabric::fabric::FabricBuilder::new().build(info).unwrap();
//     let eq = EventQueueBuilder::new(&fab).build().unwrap();
//     let domain = ft_open_domain_res(info, &fab, &eq);

//     (fab, eq, domain)
// }

// pub fn ft_open_domain_res<E, EQ: ReadEq + 'static>(
//     info: &InfoEntry<E>,
//     fab: &fabric::Fabric,
//     eq: &libfabric::eq::EventQueue<EQ>,
// ) -> ConfDomain {
//     let try_bind = libfabric::domain::DomainBuilder::new(fab, info).build_and_bind(eq, false);
//     match try_bind {
//         Ok(domain) => ConfDomain::Bound(domain),
//         Err(_) => ConfDomain::Unbound(
//             libfabric::domain::DomainBuilder::new(fab, info)
//                 .build()
//                 .unwrap(),
//         ),
//     }
// }

// pub fn ft_alloc_ep_res<E, EQ: ?Sized + 'static + libfabric::SyncSend>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     domain: &libfabric::domain::DomainBase<EQ>,
// ) -> (
//     CqType,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     Option<libfabric::av::AddressVector>,
// ) {
//     let format = if info.caps().is_tagged() {
//         enums::CqFormat::Tagged
//     } else {
//         enums::CqFormat::Context
//     };

//     let tx_cq_builder = CompletionQueueBuilder::new()
//         .size(info.tx_attr().size())
//         .format(format);
//     // .format_ctx();
//     // .build()
//     // .unwrap();

//     let rx_cq_builder = CompletionQueueBuilder::new()
//         .size(info.rx_attr().size())
//         .format(format);

//     let shared_cq_builder = CompletionQueueBuilder::new()
//         .size(info.rx_attr().size() + info.tx_attr().size())
//         .format(format);
//     // .build()
//     // .unwrap();

//     let cq_type = match gl_ctx.comp_method {
//         CompMeth::Spin => {
//             if gl_ctx.options & FT_OPT_CQ_SHARED == 0 {
//                 CqType::Spin(EqCqOpt::Separate(
//                     tx_cq_builder.wait_none().build(domain).unwrap(),
//                     rx_cq_builder.wait_none().build(domain).unwrap(),
//                 ))
//             } else {
//                 CqType::Spin(EqCqOpt::Shared(
//                     shared_cq_builder.wait_none().build(domain).unwrap(),
//                 ))
//             }
//         }
//         CompMeth::Sread => {
//             if gl_ctx.options & FT_OPT_CQ_SHARED == 0 {
//                 CqType::Sread(EqCqOpt::Separate(
//                     tx_cq_builder.build(domain).unwrap(),
//                     rx_cq_builder.build(domain).unwrap(),
//                 ))
//             } else {
//                 CqType::Sread(EqCqOpt::Shared(shared_cq_builder.build(domain).unwrap()))
//             }
//         }
//         CompMeth::WaitSet => todo!(),
//         CompMeth::WaitFd => {
//             if gl_ctx.options & FT_OPT_CQ_SHARED == 0 {
//                 CqType::WaitFd(EqCqOpt::Separate(
//                     tx_cq_builder.wait_fd().build(domain).unwrap(),
//                     rx_cq_builder.wait_fd().build(domain).unwrap(),
//                 ))
//             } else {
//                 CqType::WaitFd(EqCqOpt::Shared(
//                     shared_cq_builder.wait_fd().build(domain).unwrap(),
//                 ))
//             }
//         }
//         CompMeth::Yield => {
//             if gl_ctx.options & FT_OPT_CQ_SHARED == 0 {
//                 CqType::Sread(EqCqOpt::Separate(
//                     tx_cq_builder.wait_yield().build(domain).unwrap(),
//                     rx_cq_builder.wait_yield().build(domain).unwrap(),
//                 ))
//             } else {
//                 CqType::Sread(EqCqOpt::Shared(
//                     shared_cq_builder.wait_yield().build(domain).unwrap(),
//                 ))
//             }
//         }
//     };

//     let tx_cntr = if gl_ctx.options & FT_OPT_TX_CNTR != 0 {
//         Some(CounterBuilder::new().build(domain).unwrap())
//     } else {
//         None
//     };

//     let rx_cntr = if gl_ctx.options & FT_OPT_RX_CNTR != 0 {
//         Some(CounterBuilder::new().build(domain).unwrap())
//     } else {
//         None
//     };

//     let rma_cntr = if gl_ctx.options & FT_OPT_RX_CNTR != 0 && info.caps().is_rma() {
//         Some(CounterBuilder::new().build(domain).unwrap())
//     } else {
//         None
//     };

//     let av = match info.ep_attr().type_() {
//         libfabric::enums::EndpointType::Rdm | libfabric::enums::EndpointType::Dgram => {
//             let av = match info.domain_attr().av_type() {
//                 libfabric::enums::AddressVectorType::Unspec => {
//                     libfabric::av::AddressVectorBuilder::new()
//                 }
//                 _ => libfabric::av::AddressVectorBuilder::new()
//                     .type_(*info.domain_attr().av_type()),
//             }
//             .count(1)
//             .build(domain)
//             .unwrap();
//             Some(av)
//         }
//         _ => None,
//     };

//     (cq_type, tx_cntr, rx_cntr, rma_cntr, av)
// }

// #[allow(clippy::type_complexity)]
// pub fn ft_alloc_active_res<E>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     domain: &ConfDomain,
// ) -> (
//     CqType,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     Endpoint<E>,
//     Option<libfabric::av::AddressVector>,
// ) {
//     let (cq_type, tx_cntr, rx_cntr, rma_cntr, av) = match domain {
//         ConfDomain::Unbound(domain) => ft_alloc_ep_res(info, gl_ctx, domain),
//         ConfDomain::Bound(domain) => ft_alloc_ep_res(info, gl_ctx, domain),
//     };
//     // let (tx_cq, tx_cntr, rx_cq, rx_cntr, rma_cntr, av) = ft_alloc_ep_res(info, gl_ctx, domain);

//     let ep = match domain {
//         ConfDomain::Unbound(domain) => match &cq_type {
//             CqType::Spin(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//             CqType::Sread(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//             CqType::WaitSet(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//             CqType::WaitFd(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//             CqType::WaitYield(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//         },
//         ConfDomain::Bound(domain) => match &cq_type {
//             CqType::Spin(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//             CqType::Sread(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//             CqType::WaitSet(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//             CqType::WaitFd(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//             CqType::WaitYield(eq_cq_opt) => match eq_cq_opt {
//                 EqCqOpt::Shared(scq) => {
//                     EndpointBuilder::new(info).build_with_shared_cq(domain, scq, false)
//                 }
//                 EqCqOpt::Separate(tx_cq, rx_cq) => EndpointBuilder::new(info)
//                     .build_with_separate_cqs(
//                         domain,
//                         tx_cq,
//                         gl_ctx.options & FT_OPT_TX_CQ == 0,
//                         rx_cq,
//                         gl_ctx.options & FT_OPT_RX_CQ == 0,
//                     ),
//             },
//         },
//     }
//     .unwrap();

//     (cq_type, tx_cntr, rx_cntr, rma_cntr, ep, av)
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_prepare_ep<CNTR: ReadCntr + 'static, I, E>(
//     info: &InfoEntry<I>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &mut Endpoint<E>,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     rma_cntr: &Option<Counter<CNTR>>,
// ) {
//     match ep {
//         Endpoint::Connectionless(ep) => {
//             let mut bind_cntr = ep.bind_cntr();

//             if gl_ctx.options & FT_OPT_TX_CNTR != 0 {
//                 bind_cntr.send();
//             }

//             if info.caps().is_rma() || info.caps().is_atomic() {
//                 bind_cntr.write().read();
//             }

//             if let Some(cntr) = tx_cntr {
//                 bind_cntr.cntr(cntr).unwrap();
//             }

//             let mut bind_cntr = ep.bind_cntr();

//             if gl_ctx.options & FT_OPT_RX_CNTR != 0 {
//                 bind_cntr.recv();
//             }

//             if let Some(cntr) = rx_cntr {
//                 bind_cntr.cntr(cntr).unwrap();
//             }

//             if info.caps().is_rma() || info.caps().is_atomic() && info.caps().is_rma_event() {
//                 let mut bind_cntr = ep.bind_cntr();
//                 if info.caps().is_remote_write() {
//                     bind_cntr.remote_write();
//                 }
//                 if info.caps().is_remote_read() {
//                     bind_cntr.remote_read();
//                 }
//                 if let Some(cntr) = rma_cntr {
//                     bind_cntr.cntr(cntr).unwrap();
//                 }
//             }
//         }
//         Endpoint::ConnectionOriented(ep) => {
//             let mut bind_cntr = ep.bind_cntr();

//             if gl_ctx.options & FT_OPT_TX_CNTR != 0 {
//                 bind_cntr.send();
//             }

//             if info.caps().is_rma() || info.caps().is_atomic() {
//                 bind_cntr.write().read();
//             }

//             if let Some(cntr) = tx_cntr {
//                 bind_cntr.cntr(cntr).unwrap();
//             }

//             let mut bind_cntr = ep.bind_cntr();

//             if gl_ctx.options & FT_OPT_RX_CNTR != 0 {
//                 bind_cntr.recv();
//             }

//             if let Some(cntr) = rx_cntr {
//                 bind_cntr.cntr(cntr).unwrap();
//             }

//             if info.caps().is_rma() || info.caps().is_atomic() && info.caps().is_rma_event() {
//                 let mut bind_cntr = ep.bind_cntr();
//                 if info.caps().is_remote_write() {
//                     bind_cntr.remote_write();
//                 }
//                 if info.caps().is_remote_read() {
//                     bind_cntr.remote_read();
//                 }
//                 if let Some(cntr) = rma_cntr {
//                     bind_cntr.cntr(cntr).unwrap();
//                 }
//             }

//             // ep.enable().unwrap();
//         }
//     }
// }

// pub fn ft_complete_connect<E>(
//     pending_ep: ConnectionPendingEndpoint<E>,
//     eq: &impl WaitEq,
// ) -> ConnectedEndpoint<E> {
//     // [TODO] Do not panic, return errors

//     // if let libfabric::eq::EventQueue::Waitable(eq) = eq {

//     if let Ok(event) = eq.sread(-1) {
//         match event {
//             libfabric::eq::Event::Connected(event) => pending_ep.connect_complete(event),
//             _ => panic!("Unexpected Event type received"),
//         }
//     } else {
//         let _err_entry = eq.readerr().unwrap();
//         panic!("{:?}", _err_entry.error())
//     }
// }

// pub fn ft_accept_connection<E>(
//     ep: AcceptPendingEndpoint<E>,
//     _eq: &impl WaitEq,
// ) -> ConnectionPendingEndpoint<E> {
//     ep.accept().unwrap()
//     // match ep {
//     //     EndpointCaps::Msg(ep) => ep.accept().unwrap(),
//     //     EndpointCaps::Tagged(ep) => ep.accept().unwrap(),
//     // }

//     // ft_complete_connect(ep, eq)
// }

// pub fn ft_retrieve_conn_req<E: infocapsoptions::Caps>(
//     eq: &impl WaitEq,
//     _pep: &PassiveEndpoint<E>,
// ) -> InfoEntry<E> {
//     // [TODO] Do not panic, return errors

//     let event = eq.sread(-1).unwrap();

//     if let libfabric::eq::Event::ConnReq(entry) = event {
//         entry.info().unwrap()
//     } else {
//         panic!("Unexpected EventQueueEntry type");
//     }
// }

// pub enum EndpointCaps<M: MsgDefaultCap, T: TagDefaultCap> {
//     ConnectedMsg(ConnectedEndpoint<M>),
//     ConnlessMsg(ConnectionlessEndpoint<M>),
//     ConnectedTagged(ConnectedEndpoint<T>),
//     ConnlessTagged(ConnectionlessEndpoint<T>),
// }

// pub enum PassiveEndpointCaps<M: MsgDefaultCap, T: TagDefaultCap> {
//     Msg(PassiveEndpoint<M>),
//     Tagged(PassiveEndpoint<T>),
// }

// #[allow(clippy::type_complexity)]
// pub fn ft_server_connect<
//     T: ReadEq + WaitEq + 'static,
//     M: infocapsoptions::Caps + MsgDefaultCap + 'static,
//     TT: infocapsoptions::Caps + TagDefaultCap + 'static,
// >(
//     pep: &PassiveEndpointCaps<M, TT>,
//     gl_ctx: &mut TestsGlobalCtx,
//     eq: &libfabric::eq::EventQueue<T>,
//     fab: &fabric::Fabric,
// ) -> (
//     CqType,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     EndpointCaps<M, TT>,
//     Option<libfabric::mr::MemoryRegion>,
// ) {
//     match pep {
//         PassiveEndpointCaps::Msg(pep) => {
//             let new_info = ft_retrieve_conn_req(eq, pep);
//             gl_ctx.tx_ctx = Some(new_info.allocate_context());
//             gl_ctx.rx_ctx = Some(new_info.allocate_context());
//             let domain = ft_open_domain_res(&new_info, fab, eq);
//             let (cq_type, tx_cntr, rx_cntr, rma_cntr, mut ep, _) =
//                 ft_alloc_active_res(&new_info, gl_ctx, &domain);
//             let mr = ft_enable_ep_recv(
//                 &new_info, gl_ctx, &mut ep, &domain, &tx_cntr, &rx_cntr, &rma_cntr,
//             );
//             let ep = match ep {
//                 Endpoint::ConnectionOriented(ep) => ep.enable(eq).unwrap(),
//                 _ => panic!("Unexpected Endpoint Type"),
//             };

//             let ep = match ep {
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(_) => panic!("This should be a server"),
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(ep) => ep,
//             };

//             let peding = ft_accept_connection(ep, eq);
//             let connected_ep = ft_complete_connect(peding, eq);
//             let mut ep = EndpointCaps::ConnectedMsg(connected_ep);
//             ft_ep_recv(
//                 &new_info, gl_ctx, &mut ep, &domain, &cq_type, eq, &None, &tx_cntr, &rx_cntr,
//                 &rma_cntr, &mr,
//             );
//             (cq_type, tx_cntr, rx_cntr, ep, mr)
//         }
//         PassiveEndpointCaps::Tagged(pep) => {
//             let new_info = ft_retrieve_conn_req(eq, pep);
//             gl_ctx.tx_ctx = Some(new_info.allocate_context());
//             gl_ctx.rx_ctx = Some(new_info.allocate_context());
//             let domain = ft_open_domain_res(&new_info, fab, eq);
//             let (cq_type, tx_cntr, rx_cntr, rma_cntr, mut ep, _) =
//                 ft_alloc_active_res(&new_info, gl_ctx, &domain);
//             let mr = ft_enable_ep_recv(
//                 &new_info, gl_ctx, &mut ep, &domain, &tx_cntr, &rx_cntr, &rma_cntr,
//             );
//             let ep = match ep {
//                 Endpoint::ConnectionOriented(ep) => ep.enable(eq).unwrap(),
//                 _ => panic!("Unexpected Endpoint Type"),
//             };

//             let ep = match ep {
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(_) => todo!(),
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(ep) => ep,
//             };

//             let peding = ft_accept_connection(ep, eq);
//             let connected_ep = ft_complete_connect(peding, eq);
//             let mut ep = EndpointCaps::ConnectedTagged(connected_ep);
//             ft_ep_recv(
//                 &new_info, gl_ctx, &mut ep, &domain, &cq_type, eq, &None, &tx_cntr, &rx_cntr,
//                 &rma_cntr, &mr,
//             );
//             (cq_type, tx_cntr, rx_cntr, ep, mr)
//         }
//     }
// }

// pub fn ft_getinfo<T>(
//     hints: InfoHints<T>,
//     node: String,
//     service: String,
//     connected: bool,
//     source: bool,
// ) -> Info<T> {
//     let info = if !connected {
//         hints
//             .enter_ep_attr()
//             .type_(libfabric::enums::EndpointType::Rdm)
//             .leave_ep_attr()
//     } else {
//         hints
//     }
//     .leave_hints();

//     if source {
//         info.source(libfabric::info::ServiceAddress::Service(service))
//     } else {
//         info.service(&service).node(&node)
//     }
//     .get()
//     .unwrap()

//     // let hints = match ep_attr.type_() {
//     //     libfabric::enums::EndpointType::Unspec => {ep_attr.ep_type(libfabric::enums::EndpointType::Rdm); hints.ep_attr(ep_attr)},
//     //     _ => hints ,
//     // };

//     // let info =
//     //     if source {
//     //         Info::new_source(&Version{major: 1, minor:19}, libfabric::info::InfoSourceOpt::Service(service))
//     //     }
//     //     else {
//     //         Info::new(&Version{major: 1, minor:19}).service(&service).node(&node)
//     //     };

//     //  info.hints(&hints).build().unwrap()
// }

// pub fn ft_connect_ep<E>(
//     ep: UnconnectedEndpoint<E>,
//     _eq: &impl WaitEq,
//     addr: &libfabric::ep::Address,
// ) -> ConnectionPendingEndpoint<E> {
//     ep.connect(addr).unwrap()
//     // ft_complete_connect(ep, eq)
// }

// // fn ft_av_insert<T0>(addr: T0, count: size, fi_addr: libfabric::Address, flags: u64) {
// //pub      a
// // }

// pub fn ft_rx_prefix_size<E>(info: &InfoEntry<E>) -> usize {
//     if info.rx_attr().mode().is_msg_prefix() {
//         info.ep_attr().max_msg_size()
//     } else {
//         0
//     }
// }

// pub fn ft_tx_prefix_size<E>(info: &InfoEntry<E>) -> usize {
//     if info.tx_attr().mode().is_msg_prefix() {
//         info.ep_attr().max_msg_size()
//     } else {
//         0
//     }
// }
// pub const WINDOW_SIZE: usize = 64;
// pub const FT_MAX_CTRL_MSG: usize = 1024;
// pub const FT_RMA_SYNC_MSG_BYTES: usize = 4;

// pub fn ft_set_tx_rx_sizes<E>(
//     info: &InfoEntry<E>,
//     max_test_size: usize,
//     tx_size: &mut usize,
//     rx_size: &mut usize,
// ) {
//     *tx_size = max_test_size;
//     if *tx_size > info.ep_attr().max_msg_size() {
//         *tx_size = info.ep_attr().max_msg_size();
//     }
//     println!("FT PREFIX = {}", ft_rx_prefix_size(info));
//     *rx_size = *tx_size + ft_rx_prefix_size(info);
//     *tx_size += ft_tx_prefix_size(info);
// }

// pub fn ft_alloc_msgs<I, E: 'static>(
//     info: &InfoEntry<I>,
//     gl_ctx: &mut TestsGlobalCtx,
//     domain: &ConfDomain,
//     ep: &Endpoint<E>,
// ) -> Option<libfabric::mr::MemoryRegion> {
//     let alignment: usize = 64;
//     ft_set_tx_rx_sizes(
//         info,
//         *gl_ctx.test_sizes.last().unwrap(),
//         &mut gl_ctx.tx_size,
//         &mut gl_ctx.rx_size,
//     );
//     gl_ctx.rx_buf_size = std::cmp::max(gl_ctx.rx_size, FT_MAX_CTRL_MSG) * WINDOW_SIZE;
//     gl_ctx.tx_buf_size = std::cmp::max(gl_ctx.tx_size, FT_MAX_CTRL_MSG) * WINDOW_SIZE;

//     let rma_resv_bytes =
//         FT_RMA_SYNC_MSG_BYTES + std::cmp::max(ft_tx_prefix_size(info), ft_rx_prefix_size(info));
//     gl_ctx.tx_buf_size += rma_resv_bytes;
//     gl_ctx.rx_buf_size += rma_resv_bytes;

//     gl_ctx.buf_size = gl_ctx.rx_buf_size + gl_ctx.tx_buf_size;

//     gl_ctx.buf_size += alignment;
//     gl_ctx.buf.resize(gl_ctx.buf_size, 0);
//     println!("Buf size: {}", gl_ctx.buf_size);
//     gl_ctx.max_msg_size = gl_ctx.tx_size;

//     gl_ctx.rx_buf_index = 0;
//     println!("rx_buf_index: {}", gl_ctx.rx_buf_index);
//     gl_ctx.tx_buf_index = gl_ctx.rx_buf_size;
//     println!("tx_buf_index: {}", gl_ctx.tx_buf_index);

//     gl_ctx.remote_cq_data = ft_init_cq_data(info);

//     ft_reg_mr(info, domain, ep, &mut gl_ctx.buf, 0xC0DE)
// }

// // pub fn ft_alloc_ctx_array(gl_ctx: &mut TestsGlobalCtx) {

// // }

// pub fn ft_ep_recv<
//     EQ: ReadEq + 'static,
//     CNTR: ReadCntr + 'static,
//     E,
//     M: MsgDefaultCap,
//     T: TagDefaultCap,
// >(
//     _info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &mut EndpointCaps<M, T>,
//     _domain: &ConfDomain,
//     cq_type: &CqType,
//     _eq: &libfabric::eq::EventQueue<EQ>,
//     _av: &Option<libfabric::av::AddressVector>,
//     _tx_cntr: &Option<Counter<CNTR>>,
//     _rx_cntr: &Option<Counter<CNTR>>,
//     _rma_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<MemoryRegion>,
// ) {
//     match cq_type {
//         CqType::Spin(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => ft_post_rx(
//                 gl_ctx,
//                 ep,
//                 std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size),
//                 NO_CQ_DATA,
//                 mr,
//                 rx_cq,
//             ),
//         },
//         CqType::Sread(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => ft_post_rx(
//                 gl_ctx,
//                 ep,
//                 std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size),
//                 NO_CQ_DATA,
//                 mr,
//                 rx_cq,
//             ),
//         },
//         CqType::WaitSet(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => ft_post_rx(
//                 gl_ctx,
//                 ep,
//                 std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size),
//                 NO_CQ_DATA,
//                 mr,
//                 rx_cq,
//             ),
//         },
//         CqType::WaitFd(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => ft_post_rx(
//                 gl_ctx,
//                 ep,
//                 std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size),
//                 NO_CQ_DATA,
//                 mr,
//                 rx_cq,
//             ),
//         },
//         CqType::WaitYield(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => ft_post_rx(
//                 gl_ctx,
//                 ep,
//                 std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size),
//                 NO_CQ_DATA,
//                 mr,
//                 rx_cq,
//             ),
//         },
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_enable_ep_recv<CNTR: ReadCntr + 'static, E, T: 'static>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &mut Endpoint<T>,
//     domain: &ConfDomain,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     rma_cntr: &Option<Counter<CNTR>>,
// ) -> Option<libfabric::mr::MemoryRegion> {
    

//     {
//         ft_prepare_ep(info, gl_ctx, ep, tx_cntr, rx_cntr, rma_cntr);
//         ft_alloc_msgs(info, gl_ctx, domain, ep)
//     }
// }

// pub enum InfoWithCaps<M, T> {
//     Msg(InfoEntry<M>),
//     Tagged(InfoEntry<T>),
// }

// #[allow(clippy::type_complexity)]
// pub fn ft_init_fabric<M: MsgDefaultCap + 'static, T: TagDefaultCap + 'static>(
//     hints: HintsCaps<M, T>,
//     gl_ctx: &mut TestsGlobalCtx,
//     node: String,
//     service: String,
//     source: bool,
// ) -> (
//     InfoWithCaps<M, T>,
//     EndpointCaps<M, T>,
//     ConfDomain,
//     CqType,
//     Option<Counter<DefaultCntr>>,
//     Option<Counter<DefaultCntr>>,
//     Option<libfabric::mr::MemoryRegion>,
//     libfabric::av::AddressVector,
// ) {
//     match hints {
//         HintsCaps::Msg(hints) => {
//             let info = ft_getinfo(hints, node.clone(), service.clone(), false, source);
//             let entry = info.into_iter().next().unwrap();

//             gl_ctx.tx_ctx = Some(entry.allocate_context());
//             gl_ctx.rx_ctx = Some(entry.allocate_context());
//             let (_fabric, eq, domain) = ft_open_fabric_res(&entry);
//             let (cq_type, tx_cntr, rx_cntr, rma_ctr, mut ep, av) =
//                 ft_alloc_active_res(&entry, gl_ctx, &domain);
//             let mr = ft_enable_ep_recv(
//                 &entry, gl_ctx, &mut ep, &domain, &tx_cntr, &rx_cntr, &rma_ctr,
//             );
//             let mut ep = EndpointCaps::ConnlessMsg(match ep {
//                 Endpoint::Connectionless(ep) => ep.enable(av.as_ref().unwrap()).unwrap(),
//                 Endpoint::ConnectionOriented(_) => panic!("Unexpected Ep type"),
//             });
//             ft_ep_recv(
//                 &entry, gl_ctx, &mut ep, &domain, &cq_type, &eq, &av, &tx_cntr, &rx_cntr, &rma_ctr,
//                 &mr,
//             );
//             let av = av.unwrap();
//             ft_init_av(
//                 &entry,
//                 gl_ctx,
//                 &av,
//                 &ep,
//                 &cq_type,
//                 &tx_cntr,
//                 &rx_cntr,
//                 &mr,
//                 node.is_empty(),
//             );
//             (
//                 InfoWithCaps::Msg(entry),
//                 ep,
//                 domain,
//                 cq_type,
//                 tx_cntr,
//                 rx_cntr,
//                 mr,
//                 av,
//             )
//         }
//         HintsCaps::Tagged(hints) => {
//             let info = ft_getinfo(hints, node.clone(), service.clone(), false, source);
//             let entry = info.into_iter().next().unwrap();
//             gl_ctx.tx_ctx = Some(entry.allocate_context());
//             gl_ctx.rx_ctx = Some(entry.allocate_context());
//             let (_fabric, eq, domain) = ft_open_fabric_res(&entry);
//             let (cq_type, tx_cntr, rx_cntr, rma_ctr, mut ep, av) =
//                 ft_alloc_active_res(&entry, gl_ctx, &domain);

//             let mr = ft_enable_ep_recv(
//                 &entry, gl_ctx, &mut ep, &domain, &tx_cntr, &rx_cntr, &rma_ctr,
//             );
//             let mut ep = EndpointCaps::ConnlessTagged(match ep {
//                 Endpoint::Connectionless(ep) => ep.enable(av.as_ref().unwrap()).unwrap(),
//                 Endpoint::ConnectionOriented(_) => panic!("Unexpected Ep type"),
//             });
//             ft_ep_recv(
//                 &entry, gl_ctx, &mut ep, &domain, &cq_type, &eq, &av, &tx_cntr, &rx_cntr, &rma_ctr,
//                 &mr,
//             );
//             let av = av.unwrap();
//             ft_init_av(
//                 &entry,
//                 gl_ctx,
//                 &av,
//                 &ep,
//                 &cq_type,
//                 &tx_cntr,
//                 &rx_cntr,
//                 &mr,
//                 node.is_empty(),
//             );
//             (
//                 InfoWithCaps::Tagged(entry),
//                 ep,
//                 domain,
//                 cq_type,
//                 tx_cntr,
//                 rx_cntr,
//                 mr,
//                 av,
//             )
//         }
//     }
// }

// pub fn ft_av_insert(
//     av: &libfabric::av::AddressVector,
//     addr: &Address,
//     options: AVOptions,
// ) -> MappedAddress {
//     let mut added = av
//         .insert(std::slice::from_ref(addr).into(), options)
//         .unwrap();
//     added
//         .pop()
//         .unwrap()
//         .expect("Could not add address to address vector")
// }

// pub const NO_CQ_DATA: u64 = 0;

// macro_rules!  ft_post{
//     ($post_fn:ident, $prog_fn:ident, $cq:ident, $seq:expr, $cq_cntr:expr, $op_str:literal, $ep:ident, $( $x:expr),* ) => {
//         loop {
//             let ret = $ep.$post_fn($($x,)*);
//             if ret.is_ok() {
//                 break;
//             }
//             else if let Err(ref err) = ret {
//                 if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
//                     panic!("Unexpected error!")
//                 }

//             }
//             $prog_fn($cq, $seq, $cq_cntr);
//         }
//         $seq+=1;
//     };
// }

// #[allow(non_camel_case_types)]
// pub enum RmaOp {
//     RMA_WRITE,
//     RMA_WRITEDATA,
//     RMA_READ,
// }

// pub enum SendOp {
//     Send,
//     MsgSend,
//     Inject,
// }
// pub enum RecvOp {
//     Recv,
//     MsgRecv,
// }

// pub enum TagSendOp {
//     TagSend,
//     TagMsgSend,
//     TagInject,
// }

// pub enum TagRecvOp {
//     TagRecv,
//     TagMsgRecv,
// }

// pub fn ft_init_cq_data<E>(info: &InfoEntry<E>) -> u64 {
//     if info.domain_attr().cq_data_size() >= std::mem::size_of::<u64>() {
//         0x0123456789abcdef_u64
//     } else {
//         0x0123456789abcdef & ((1 << (info.domain_attr().cq_data_size() * 8)) - 1)
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_post_rma_inject(
//     gl_ctx: &mut TestsGlobalCtx,
//     rma_op: &RmaOp,
//     offset: usize,
//     size: usize,
//     remote: &RemoteMemAddressInfo,
//     ep: &impl WriteEp,
//     tx_cq: &impl ReadCq,
// ) {
//     let fi_addr = gl_ctx.remote_address.as_ref().unwrap();
//     match rma_op {
//         RmaOp::RMA_WRITE => {
//             let addr = unsafe { remote.mem_address().add(offset) };
//             let key = remote.key();
//             let buf =
//                 &gl_ctx.buf[gl_ctx.tx_buf_index + offset..gl_ctx.tx_buf_index + offset + size];
//             unsafe {
//                 ft_post!(
//                     inject_write_to,
//                     ft_progress,
//                     tx_cq,
//                     gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     "fi_write",
//                     ep,
//                     buf,
//                     fi_addr,
//                     addr,
//                     &key
//                 );
//             }
//         }

//         RmaOp::RMA_WRITEDATA => {
//             let addr = unsafe { remote.mem_address().add(offset) };
//             let key = remote.key();
//             let buf =
//                 &gl_ctx.buf[gl_ctx.tx_buf_index + offset..gl_ctx.tx_buf_index + offset + size];
//             let remote_cq_data = gl_ctx.remote_cq_data;
//             unsafe {
//                 ft_post!(
//                     inject_writedata_to,
//                     ft_progress,
//                     tx_cq,
//                     gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     "fi_write",
//                     ep,
//                     buf,
//                     remote_cq_data,
//                     fi_addr,
//                     addr,
//                     &key
//                 );
//             }
//         }
//         RmaOp::RMA_READ => {
//             panic!("ft_post_rma_inject does not support read");
//         }
//     }

//     gl_ctx.tx_cq_cntr += 1;
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_post_rma(
//     gl_ctx: &mut TestsGlobalCtx,
//     rma_op: &RmaOp,
//     offset: usize,
//     size: usize,
//     remote: &RemoteMemAddressInfo,
//     ep: &impl ReadWriteEp,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     tx_cq: &impl ReadCq,
// ) {
//     let fi_addr = gl_ctx.remote_address.as_ref().unwrap();
//     let mem_addr = unsafe { remote.mem_address().add(offset) };
//     let key = remote.key();
//     let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index + offset..gl_ctx.tx_buf_index + offset + size];
//     let data_desc = Some(mr.as_ref().unwrap().descriptor());
//     match rma_op {
//         RmaOp::RMA_WRITE => unsafe {
//             ft_post!(
//                 write_to,
//                 ft_progress,
//                 tx_cq,
//                 gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 "fi_write",
//                 ep,
//                 buf,
//                 data_desc,
//                 fi_addr,
//                 mem_addr,
//                 &key
//             );
//         },

//         RmaOp::RMA_WRITEDATA => {
//             let remote_cq_data = gl_ctx.remote_cq_data;
//             unsafe {
//                 ft_post!(
//                     writedata_to,
//                     ft_progress,
//                     tx_cq,
//                     gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     "fi_write",
//                     ep,
//                     buf,
//                     data_desc,
//                     remote_cq_data,
//                     fi_addr,
//                     mem_addr,
//                     &key
//                 );
//             }
//         }

//         RmaOp::RMA_READ => unsafe {
//             ft_post!(
//                 read_from,
//                 ft_progress,
//                 tx_cq,
//                 gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 "fi_write",
//                 ep,
//                 buf,
//                 data_desc,
//                 fi_addr,
//                 mem_addr,
//                 &key
//             );
//         },
//     }
// }

// pub fn connected_msg_post(
//     op: SendOp,
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     tx_cq: &impl ReadCq,
//     ep: &impl ConnectedSendEp,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     let desc = mr.as_ref().map(|mr| mr.descriptor());

//     match op {
//         SendOp::MsgSend => {
//             let iov = libfabric::iovec::IoVec::from_slice(base);
//             let flag = libfabric::enums::SendMsgOptions::new().transmit_complete();

//             let msg = libfabric::msg::MsgConnected::from_iov(&iov, desc.as_ref(), None, ctx);
//             let msg_ref = &msg;
//             ft_post!(
//                 sendmsg,
//                 ft_progress,
//                 tx_cq,
//                 *tx_seq,
//                 tx_cq_cntr,
//                 "sendmsg",
//                 ep,
//                 msg_ref,
//                 flag
//             );
//         }
//         SendOp::Send => {
//             if data != NO_CQ_DATA {
//                 ft_post!(
//                     senddata_with_context,
//                     ft_progress,
//                     tx_cq,
//                     *tx_seq,
//                     tx_cq_cntr,
//                     "transmit",
//                     ep,
//                     base,
//                     desc,
//                     data,
//                     ctx
//                 );
//             } else {
//                 ft_post!(
//                     send_with_context,
//                     ft_progress,
//                     tx_cq,
//                     *tx_seq,
//                     tx_cq_cntr,
//                     "transmit",
//                     ep,
//                     base,
//                     desc,
//                     ctx
//                 );
//             }
//         }
//         SendOp::Inject => {
//             ft_post!(
//                 inject,
//                 ft_progress,
//                 tx_cq,
//                 *tx_seq,
//                 tx_cq_cntr,
//                 "inject",
//                 ep,
//                 base
//             );
//         }
//     }
// }

// pub fn conless_msg_post(
//     op: SendOp,
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     tx_cq: &impl ReadCq,
//     ep: &impl SendEp,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     let desc = mr.as_ref().map(|mr| mr.descriptor());
//     match op {
//         SendOp::MsgSend => {
//             let iov = libfabric::iovec::IoVec::from_slice(base);
//             let flag = libfabric::enums::SendMsgOptions::new().transmit_complete();

//             if let Some(fi_addr) = remote_address {
//                 let msg = libfabric::msg::Msg::from_iov(&iov, desc.as_ref(), fi_addr, None, ctx);
//                 let msg_ref = &msg;
//                 ft_post!(
//                     sendmsg_to,
//                     ft_progress,
//                     tx_cq,
//                     *tx_seq,
//                     tx_cq_cntr,
//                     "sendmsg",
//                     ep,
//                     msg_ref,
//                     flag
//                 );
//             }
//         }
//         SendOp::Send => {
//             if let Some(fi_address) = remote_address {
//                 if data != NO_CQ_DATA {
//                     ft_post!(
//                         senddata_to_with_context,
//                         ft_progress,
//                         tx_cq,
//                         *tx_seq,
//                         tx_cq_cntr,
//                         "transmit",
//                         ep,
//                         base,
//                         desc,
//                         data,
//                         fi_address,
//                         ctx
//                     );
//                 } else {
//                     ft_post!(
//                         send_to_with_context,
//                         ft_progress,
//                         tx_cq,
//                         *tx_seq,
//                         tx_cq_cntr,
//                         "transmit",
//                         ep,
//                         base,
//                         desc,
//                         fi_address,
//                         ctx
//                     );
//                 }
//             }
//         }
//         SendOp::Inject => {
//             if let Some(fi_address) = remote_address {
//                 ft_post!(
//                     inject_to,
//                     ft_progress,
//                     tx_cq,
//                     *tx_seq,
//                     tx_cq_cntr,
//                     "inject",
//                     ep,
//                     base,
//                     fi_address
//                 );
//             }
//         }
//     }
// }

// // pub fn msg_post<M: MsgDefaultCap, T: TagDefaultCap, const CONN: bool>(op: SendOp, tx_seq: &mut u64, tx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &Option<MappedAddress>, tx_cq: &impl ReadCq, ep: &EndpointCaps<M, T , CONN>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], data: u64) {
// //     match &ep {
// //         EndpointCaps::Msg(ep) => conless_msg_post(op, tx_seq, tx_cq_cntr, ctx, remote_address, tx_cq, &ep, data_desc, base, data),
// //         // EndpointCaps::Tagged(ep) => connected_tagged_post(op, tx_seq, tx_cq_cntr, ctx, tx_cq, ep, data_desc, base, data),
// //     }
// // }

// pub fn msg_post<M: MsgDefaultCap, T: TagDefaultCap>(
//     op: SendOp,
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     tx_cq: &impl ReadCq,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     match &ep {
//         // pub fn conless_tagged_post<E: TagDefaultCap>(op: TagSendOp, tx_seq: &mut u64, tx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &MappedAddress,  ft_tag: u64, tx_cq: &impl ReadCq, ep: &libfabric::ep::Endpoint<E>, mr: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], data: u64) {
//         EndpointCaps::ConnectedMsg(ep) => {
//             connected_msg_post(op, tx_seq, tx_cq_cntr, ctx, tx_cq, ep, mr, base, data)
//         }
//         EndpointCaps::ConnlessMsg(ep) => conless_msg_post(
//             op,
//             tx_seq,
//             tx_cq_cntr,
//             ctx,
//             remote_address,
//             tx_cq,
//             ep,
//             mr,
//             base,
//             data,
//         ),
//         _ => panic!("Tagged post not supported here"),
//         // pub fn conless_tagged_post<E: TagDefaultCap>(op: TagSendOp, tx_seq: &mut u64, tx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &MappedAddress,  ft_tag: u64, tx_cq: &impl ReadCq, ep: &libfabric::ep::Endpoint<E>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], data: u64) {
//     }
// }

// pub fn msg_post_recv<M: MsgDefaultCap, T: TagDefaultCap>(
//     op: RecvOp,
//     rx_seq: &mut u64,
//     rx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     rx_cq: &impl ReadCq,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let mr_desc = mr.as_ref().map(|mr| mr.descriptor());

//     match ep {
//         EndpointCaps::ConnectedMsg(ep) => match op {
//             RecvOp::MsgRecv => {
//                 todo!()
//             }
//             RecvOp::Recv => {
//                 ft_post!(
//                     recv_with_context,
//                     ft_progress,
//                     rx_cq,
//                     *rx_seq,
//                     rx_cq_cntr,
//                     "receive",
//                     ep,
//                     base,
//                     mr_desc,
//                     ctx
//                 );
//             }
//         },
//         EndpointCaps::ConnlessMsg(ep) => match op {
//             RecvOp::MsgRecv => {
//                 todo!()
//             }
//             RecvOp::Recv => {
//                 if let Some(fi_address) = remote_address.as_ref() {
//                     ft_post!(
//                         recv_from_with_context,
//                         ft_progress,
//                         rx_cq,
//                         *rx_seq,
//                         rx_cq_cntr,
//                         "receive",
//                         ep,
//                         base,
//                         mr_desc,
//                         fi_address,
//                         ctx
//                     );
//                 } else {
//                     ft_post!(
//                         recv_from_any_with_context,
//                         ft_progress,
//                         rx_cq,
//                         *rx_seq,
//                         rx_cq_cntr,
//                         "receive",
//                         ep,
//                         base,
//                         mr_desc,
//                         ctx
//                     );
//                 }
//             }
//         },
//         _ => {
//             panic!("Tagged not supported here");
//         }
//     }
// }

// pub fn tagged_post<M: MsgDefaultCap, T: TagDefaultCap>(
//     op: TagSendOp,
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     _ft_tag: u64,
//     tx_cq: &impl ReadCq,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     match &ep {
//         // pub fn conless_tagged_post<E: TagDefaultCap>(op: TagSendOp, tx_seq: &mut u64, tx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &MappedAddress,  ft_tag: u64, tx_cq: &impl ReadCq, ep: &libfabric::ep::Endpoint<E>, mr: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], data: u64) {
//         EndpointCaps::ConnectedTagged(ep) => connected_tagged_post(
//             op, tx_seq, tx_cq_cntr, ctx, *tx_seq, tx_cq, ep, mr, base, data,
//         ),
//         EndpointCaps::ConnlessTagged(ep) => conless_tagged_post(
//             op,
//             tx_seq,
//             tx_cq_cntr,
//             ctx,
//             remote_address,
//             *tx_seq,
//             tx_cq,
//             ep,
//             mr,
//             base,
//             data,
//         ),
//         _ => panic!("Tagged post not supported here"),
//         // pub fn conless_tagged_post<E: TagDefaultCap>(op: TagSendOp, tx_seq: &mut u64, tx_cq_cntr: &mut u64, ctx : &mut Context, remote_address: &MappedAddress,  ft_tag: u64, tx_cq: &impl ReadCq, ep: &libfabric::ep::Endpoint<E>, data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>, base: &mut [u8], data: u64) {
//     }
// }

// pub fn connected_tagged_post<E: TagDefaultCap>(
//     op: TagSendOp,
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     ft_tag: u64,
//     tx_cq: &impl ReadCq,
//     ep: &ConnectedEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     let flag = libfabric::enums::TaggedSendMsgOptions::new().transmit_complete();
//     let desc = mr.as_ref().map(|mr| mr.descriptor());

//     match op {
//         TagSendOp::TagMsgSend => {
//             let iov = libfabric::iovec::IoVec::from_slice(base);
//             // let mut mem_descs = vec![default_desc()];
//             let msg = libfabric::msg::MsgTaggedConnected::from_iov(
//                 &iov,
//                 desc.as_ref(),
//                 None,
//                 *tx_seq,
//                 None,
//                 ctx,
//             );
//             let msg_ref = &msg;
//             ft_post!(
//                 tsendmsg,
//                 ft_progress,
//                 tx_cq,
//                 *tx_seq,
//                 tx_cq_cntr,
//                 "sendmsg",
//                 ep,
//                 msg_ref,
//                 flag
//             );
//         }
//         TagSendOp::TagSend => {
//             let op_tag = if ft_tag != 0 { ft_tag } else { *tx_seq };

//             if data != NO_CQ_DATA {
//                 ft_post!(
//                     tsenddata_with_context,
//                     ft_progress,
//                     tx_cq,
//                     *tx_seq,
//                     tx_cq_cntr,
//                     "transmit",
//                     ep,
//                     base,
//                     desc,
//                     data,
//                     op_tag,
//                     ctx
//                 );
//             } else {
//                 ft_post!(
//                     tsend_with_context,
//                     ft_progress,
//                     tx_cq,
//                     *tx_seq,
//                     tx_cq_cntr,
//                     "transmit",
//                     ep,
//                     base,
//                     desc,
//                     op_tag,
//                     ctx
//                 );
//             }
//         }
//         TagSendOp::TagInject => {
//             let tag = *tx_seq;
//             ft_post!(
//                 tinject,
//                 ft_progress,
//                 tx_cq,
//                 *tx_seq,
//                 tx_cq_cntr,
//                 "inject",
//                 ep,
//                 base,
//                 tag
//             );
//         }
//     }
// }

// pub fn conless_tagged_post<E: TagDefaultCap>(
//     op: TagSendOp,
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     tx_cq: &impl ReadCq,
//     ep: &ConnectionlessEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     let flag: enums::TferOptions<true, true, false, false, true, false> =
//         libfabric::enums::TaggedSendMsgOptions::new().transmit_complete();
//     let desc = mr.as_ref().map(|mr| mr.descriptor());
//     let fi_address = remote_address.as_ref().unwrap();
//     match op {
//         TagSendOp::TagMsgSend => {
//             let iov = libfabric::iovec::IoVec::from_slice(base);
//             // let mut mem_descs = vec![default_desc()];
//             let msg = libfabric::msg::MsgTagged::from_iov(
//                 &iov,
//                 desc.as_ref(),
//                 remote_address.as_ref().unwrap(),
//                 None,
//                 *tx_seq,
//                 None,
//                 ctx,
//             );
//             let msg_ref = &msg;
//             ft_post!(
//                 tsendmsg_to,
//                 ft_progress,
//                 tx_cq,
//                 *tx_seq,
//                 tx_cq_cntr,
//                 "sendmsg",
//                 ep,
//                 msg_ref,
//                 flag
//             );
//         }
//         TagSendOp::TagSend => {
//             let op_tag = if ft_tag != 0 { ft_tag } else { *tx_seq };
//             if data != NO_CQ_DATA {
//                 ft_post!(
//                     tsenddata_to_with_context,
//                     ft_progress,
//                     tx_cq,
//                     *tx_seq,
//                     tx_cq_cntr,
//                     "transmit",
//                     ep,
//                     base,
//                     desc,
//                     data,
//                     fi_address,
//                     op_tag,
//                     ctx
//                 );
//             } else {
//                 ft_post!(
//                     tsend_to_with_context,
//                     ft_progress,
//                     tx_cq,
//                     *tx_seq,
//                     tx_cq_cntr,
//                     "transmit",
//                     ep,
//                     base,
//                     desc,
//                     fi_address,
//                     op_tag,
//                     ctx
//                 );
//             }
//         }
//         TagSendOp::TagInject => {
//             let tag = *tx_seq;
//             ft_post!(
//                 tinject_to,
//                 ft_progress,
//                 tx_cq,
//                 *tx_seq,
//                 tx_cq_cntr,
//                 "inject",
//                 ep,
//                 base,
//                 fi_address,
//                 tag
//             );
//         }
//     }
// }

// pub fn connected_tagged_post_recv<T: TagDefaultCap>(
//     op: TagRecvOp,
//     rx_seq: &mut u64,
//     rx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     _remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     rx_cq: &impl ReadCq,
//     ep: &ConnectedEndpoint<T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let desc = mr.as_ref().map(|mr| mr.descriptor());
//     match op {
//         TagRecvOp::TagMsgRecv => {
//             todo!()
//         }
//         TagRecvOp::TagRecv => {
//             let op_tag = if ft_tag != 0 { ft_tag } else { *rx_seq };

//             ft_post!(
//                 trecv_with_context,
//                 ft_progress,
//                 rx_cq,
//                 *rx_seq,
//                 rx_cq_cntr,
//                 "receive",
//                 ep,
//                 base,
//                 desc,
//                 op_tag,
//                 None,
//                 ctx
//             );
//         }
//     }
// }

// pub fn connless_tagged_post_recv<T: TagDefaultCap>(
//     op: TagRecvOp,
//     rx_seq: &mut u64,
//     rx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     rx_cq: &impl ReadCq,
//     ep: &ConnectionlessEndpoint<T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let desc = mr.as_ref().map(|mr| mr.descriptor());
//     match op {
//         TagRecvOp::TagMsgRecv => {
//             todo!()
//         }
//         TagRecvOp::TagRecv => {
//             let op_tag = if ft_tag != 0 { ft_tag } else { *rx_seq };
//             let fi_address = remote_address.as_ref().unwrap();
//             ft_post!(
//                 trecv_from_with_context,
//                 ft_progress,
//                 rx_cq,
//                 *rx_seq,
//                 rx_cq_cntr,
//                 "receive",
//                 ep,
//                 base,
//                 desc,
//                 fi_address,
//                 op_tag,
//                 None,
//                 ctx
//             );
//         }
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_post_tx<M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     size: usize,
//     data: u64,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     tx_cq: &impl ReadCq,
// ) {
//     // size += ft_tx_prefix_size(info);
//     let fi_addr = &gl_ctx.remote_address;
//     let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + size];
//     match ep {
//         EndpointCaps::ConnectedMsg(_epp) => {
//             msg_post(
//                 SendOp::Send,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 tx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 data,
//             );
//         }
//         EndpointCaps::ConnectedTagged(_epp) => {
//             tagged_post(
//                 TagSendOp::TagSend,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 gl_ctx.ft_tag,
//                 tx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 data,
//             );
//         }
//         EndpointCaps::ConnlessMsg(_epp) => {
//             msg_post(
//                 SendOp::Send,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 tx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 data,
//             );
//         }
//         EndpointCaps::ConnlessTagged(_epp) => {
//             tagged_post(
//                 TagSendOp::TagSend,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 gl_ctx.ft_tag,
//                 tx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 data,
//             );
//         }
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_tx<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     size: usize,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     tx_cq: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
// ) {
//     match tx_cq {
//         CqType::Spin(eq_type) => match eq_type {
//             EqCqOpt::Separate(tx_cq, _) | EqCqOpt::Shared(tx_cq) => {
//                 ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, mr, tx_cq)
//             }
//         },
//         CqType::Sread(eq_type) => match eq_type {
//             EqCqOpt::Separate(tx_cq, _) | EqCqOpt::Shared(tx_cq) => {
//                 ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, mr, tx_cq)
//             }
//         },
//         CqType::WaitSet(eq_type) => match eq_type {
//             EqCqOpt::Separate(tx_cq, _) | EqCqOpt::Shared(tx_cq) => {
//                 ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, mr, tx_cq)
//             }
//         },
//         CqType::WaitFd(eq_type) => match eq_type {
//             EqCqOpt::Separate(tx_cq, _) | EqCqOpt::Shared(tx_cq) => {
//                 ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, mr, tx_cq)
//             }
//         },
//         CqType::WaitYield(eq_type) => match eq_type {
//             EqCqOpt::Separate(tx_cq, _) | EqCqOpt::Shared(tx_cq) => {
//                 ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, mr, tx_cq)
//             }
//         },
//     }

//     ft_get_tx_comp(gl_ctx, tx_cntr, tx_cq, gl_ctx.tx_seq);
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_post_rx<M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     mut size: usize,
//     _data: u64,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     rx_cq: &impl ReadCq,
// ) {
//     size = std::cmp::max(size, FT_MAX_CTRL_MSG); //+  ft_tx_prefix_size(info);
//     let buf = &mut gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + size];

//     match ep {
//         EndpointCaps::ConnlessMsg(_epp) => {
//             msg_post_recv(
//                 RecvOp::Recv,
//                 &mut gl_ctx.rx_seq,
//                 &mut gl_ctx.rx_cq_cntr,
//                 gl_ctx.rx_ctx.as_mut().unwrap(),
//                 &gl_ctx.remote_address,
//                 rx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnlessTagged(ep) => {
//             connless_tagged_post_recv(
//                 TagRecvOp::TagRecv,
//                 &mut gl_ctx.rx_seq,
//                 &mut gl_ctx.rx_cq_cntr,
//                 gl_ctx.rx_ctx.as_mut().unwrap(),
//                 &gl_ctx.remote_address,
//                 gl_ctx.ft_tag,
//                 rx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnectedMsg(_epp) => {
//             msg_post_recv(
//                 RecvOp::Recv,
//                 &mut gl_ctx.rx_seq,
//                 &mut gl_ctx.rx_cq_cntr,
//                 gl_ctx.rx_ctx.as_mut().unwrap(),
//                 &gl_ctx.remote_address,
//                 rx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnectedTagged(ep) => {
//             connected_tagged_post_recv(
//                 TagRecvOp::TagRecv,
//                 &mut gl_ctx.rx_seq,
//                 &mut gl_ctx.rx_cq_cntr,
//                 gl_ctx.rx_ctx.as_mut().unwrap(),
//                 &gl_ctx.remote_address,
//                 gl_ctx.ft_tag,
//                 rx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_rx<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     _size: usize,
//     data_desc: &Option<libfabric::mr::MemoryRegion>,
//     rx_cq: &CqType,
//     rx_cntr: &Option<libfabric::cntr::Counter<CNTR>>,
// ) {
//     ft_get_rx_comp(gl_ctx, rx_cntr, rx_cq, gl_ctx.rx_seq);
//     match rx_cq {
//         CqType::Spin(cq_type) => match cq_type {
//             EqCqOpt::Shared(rx_cq) | EqCqOpt::Separate(_, rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq)
//             }
//         },
//         CqType::Sread(cq_type) => match cq_type {
//             EqCqOpt::Shared(rx_cq) | EqCqOpt::Separate(_, rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq)
//             }
//         },
//         CqType::WaitSet(cq_type) => match cq_type {
//             EqCqOpt::Shared(rx_cq) | EqCqOpt::Separate(_, rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq)
//             }
//         },
//         CqType::WaitFd(cq_type) => match cq_type {
//             EqCqOpt::Shared(rx_cq) | EqCqOpt::Separate(_, rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq)
//             }
//         },
//         CqType::WaitYield(cq_type) => match cq_type {
//             EqCqOpt::Shared(rx_cq) | EqCqOpt::Separate(_, rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, rx_cq)
//             }
//         },
//     }
// }

// pub fn ft_post_inject<M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     size: usize,
//     tx_cq: &impl ReadCq,
// ) {
//     // size += ft_tx_prefix_size(info);
//     let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + size];
//     let fi_addr = &gl_ctx.remote_address;

//     match ep {
//         EndpointCaps::ConnlessMsg(_epp) => {
//             msg_post(
//                 SendOp::Inject,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 tx_cq,
//                 ep,
//                 &mut None,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnectedMsg(_epp) => {
//             msg_post(
//                 SendOp::Inject,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 tx_cq,
//                 ep,
//                 &mut None,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnlessTagged(_epp) => tagged_post(
//             TagSendOp::TagInject,
//             &mut gl_ctx.tx_seq,
//             &mut gl_ctx.tx_cq_cntr,
//             gl_ctx.tx_ctx.as_mut().unwrap(),
//             fi_addr,
//             gl_ctx.ft_tag,
//             tx_cq,
//             ep,
//             &mut None,
//             buf,
//             NO_CQ_DATA,
//         ),
//         EndpointCaps::ConnectedTagged(_epp) => tagged_post(
//             TagSendOp::TagInject,
//             &mut gl_ctx.tx_seq,
//             &mut gl_ctx.tx_cq_cntr,
//             gl_ctx.tx_ctx.as_mut().unwrap(),
//             fi_addr,
//             gl_ctx.ft_tag,
//             tx_cq,
//             ep,
//             &mut None,
//             buf,
//             NO_CQ_DATA,
//         ),
//     }
//     gl_ctx.tx_cq_cntr += 1;
// }

// pub fn ft_inject<M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     size: usize,
//     tx_cq: &CqType,
// ) {
//     match tx_cq {
//         CqType::Spin(cq_type) => match cq_type {
//             EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                 ft_post_inject(gl_ctx, ep, size, tx_cq)
//             }
//         },
//         CqType::Sread(cq_type) => match cq_type {
//             EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                 ft_post_inject(gl_ctx, ep, size, tx_cq)
//             }
//         },
//         CqType::WaitSet(cq_type) => match cq_type {
//             EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                 ft_post_inject(gl_ctx, ep, size, tx_cq)
//             }
//         },
//         CqType::WaitFd(cq_type) => match cq_type {
//             EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                 ft_post_inject(gl_ctx, ep, size, tx_cq)
//             }
//         },
//         CqType::WaitYield(cq_type) => match cq_type {
//             EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                 ft_post_inject(gl_ctx, ep, size, tx_cq)
//             }
//         },
//     }
// }

// pub fn ft_progress(cq: &impl ReadCq, _total: u64, cq_cntr: &mut u64) {
//     let ret = cq.read(1);
//     match ret {
//         Ok(_) => {
//             *cq_cntr += 1;
//         }
//         Err(ref err) => {
//             if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
//                 ret.unwrap();
//             }
//         }
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_init_av_dst_addr<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     av: &libfabric::av::AddressVector,
//     ep: &EndpointCaps<M, T>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     server: bool,
// ) {
//     if !server {
//         gl_ctx.remote_address = Some(ft_av_insert(
//             av,
//             info.dest_addr().unwrap(),
//             AVOptions::new(),
//         ));
//         let epname = match ep {
//             EndpointCaps::ConnlessMsg(ep) => ep.getname().unwrap(),
//             EndpointCaps::ConnectedMsg(ep) => ep.getname().unwrap(),
//             EndpointCaps::ConnlessTagged(ep) => ep.getname().unwrap(),
//             EndpointCaps::ConnectedTagged(ep) => ep.getname().unwrap(),
//         };
//         let epname_bytes = epname.as_bytes();
//         let len = epname_bytes.len();
//         gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + len].copy_from_slice(epname_bytes);

//         ft_tx(gl_ctx, ep, len, mr, cq_type, tx_cntr);
//         ft_rx(gl_ctx, ep, 1, mr, cq_type, rx_cntr);
//     } else {
//         let mut v = [0_u8; FT_MAX_CTRL_MSG];

//         ft_get_rx_comp(gl_ctx, rx_cntr, cq_type, gl_ctx.rx_seq);
//         v.copy_from_slice(&gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + FT_MAX_CTRL_MSG]);
//         let address = unsafe { Address::from_bytes(&v) };

//         gl_ctx.remote_address = Some(ft_av_insert(av, &address, AVOptions::new()));
//         // if matches!(info.domain_attr().av_type, libfabric::enums::AddressVectorType::Table ) {
//         //     let mut zero = 0;
//         //     ft_av_insert(av, &v, &mut zero, 0);
//         // }
//         // else {
//         //     ft_av_insert(av, &v, &mut gl_ctx.remote_address, 0);
//         // }

//         match cq_type {
//             CqType::Spin(cq_type) => match cq_type {
//                 EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                     ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA, mr, rx_cq)
//                 }
//             },
//             CqType::Sread(cq_type) => match cq_type {
//                 EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                     ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA, mr, rx_cq)
//                 }
//             },
//             CqType::WaitSet(cq_type) => match cq_type {
//                 EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                     ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA, mr, rx_cq)
//                 }
//             },
//             CqType::WaitFd(cq_type) => match cq_type {
//                 EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                     ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA, mr, rx_cq)
//                 }
//             },
//             CqType::WaitYield(cq_type) => match cq_type {
//                 EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                     ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA, mr, rx_cq)
//                 }
//             },
//         }

//         // if matches!(info.domain_attr().av_type, libfabric::enums::AddressVectorType::Table) {
//         //     gl_ctx.remote_address = 0;
//         // }

//         ft_tx(gl_ctx, ep, 1, mr, cq_type, tx_cntr);
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_init_av<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     av: &libfabric::av::AddressVector,
//     ep: &EndpointCaps<M, T>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     server: bool,
// ) {
//     ft_init_av_dst_addr(info, gl_ctx, av, ep, cq_type, tx_cntr, rx_cntr, mr, server);
// }

// pub fn ft_spin_for_comp(cq: &impl ReadCq, curr: &mut u64, total: u64, _timeout: i32, _tag: u64) {
//     while total - *curr > 0 {
//         loop {
//             let err = cq.read(1);
//             match err {
//                 Ok(_) => break,
//                 Err(err) => {
//                     if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
//                         let err_entry = cq.readerr(0).unwrap();

//                         cq.print_error(&err_entry);
//                         panic!("ERROR IN CQ_READ {}", err);
//                     }
//                 }
//             }
//         }
//         *curr += 1;
//     }
// }

// pub fn ft_wait_for_comp(cq: &impl WaitCq, curr: &mut u64, total: u64, _timeout: i32, _tag: u64) {
//     while total - *curr > 0 {
//         let ret = cq.sread(1, -1);
//         if ret.is_ok() {
//             *curr += 1;
//         }
//     }
// }

// // pub fn ft_read_cq(cq: &CqType, curr: &mut u64, total: u64, timeout: i32, tag: u64) {

// //     match cq {
// //         CqType::Spin(cq) => ft_spin_for_comp(cq, curr, total, timeout, tag),
// //         CqType::Sread(cq ) => ft_wait_for_comp(cq, curr, total, timeout, tag),
// //         CqType::WaitSet(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
// //         CqType::WaitFd(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
// //         CqType::WaitYield(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
// //     }
// // }

// pub fn ft_spin_for_cntr<CNTR: ReadCntr>(cntr: &Counter<CNTR>, total: u64) {
//     loop {
//         let cur = cntr.read();
//         if cur >= total {
//             break;
//         }
//     }
// }

// pub fn ft_wait_for_cntr<CNTR: WaitCntr + ReadCntr>(cntr: &Counter<CNTR>, total: u64) {
//     while total > cntr.read() {
//         let ret = cntr.wait(total, -1);
//         if matches!(ret, Ok(())) {
//             break;
//         }
//     }
// }

// // pub fn ft_get_cq_comp(rx_curr: &mut u64, rx_cq: &CqType, total: u64) {
// //     ft_read_cq(rx_cq, rx_curr, total, -1, 0);
// // }

// pub fn ft_get_cntr_comp<CNTR: WaitCntr + ReadCntr>(cntr: &Option<Counter<CNTR>>, total: u64) {
//     if let Some(cntr_v) = cntr {
//         ft_wait_for_cntr(cntr_v, total);
//         //     match cntr_v  {
//         //         Counter::Waitable(cntr) => { ft_wait_for_cntr(cntr, total);}
//         //         Counter::NonWaitable(cntr) => { ft_spin_for_cntr(cntr, total);}
//         //     }
//     }
//     // else {
//     //     panic!("Counter not set");
//     // }
// }

// pub fn ft_get_rx_comp<CNTR: WaitCntr + ReadCntr>(
//     gl_ctx: &mut TestsGlobalCtx,
//     rx_cntr: &Option<Counter<CNTR>>,
//     rx_cq: &CqType,
//     total: u64,
// ) {
//     if gl_ctx.options & FT_OPT_RX_CQ != 0 {
//         match rx_cq {
//             CqType::Spin(cq_type) => match cq_type {
//                 EqCqOpt::Shared(rx_cq) => {
//                     ft_spin_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(_, rx_cq) => {
//                     ft_spin_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//             },
//             CqType::Sread(cq_type) => match cq_type {
//                 EqCqOpt::Shared(rx_cq) => {
//                     ft_wait_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(_, rx_cq) => {
//                     ft_wait_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//             },
//             CqType::WaitSet(cq_type) => match cq_type {
//                 EqCqOpt::Shared(rx_cq) => {
//                     ft_wait_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(_, rx_cq) => {
//                     ft_wait_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//             },
//             CqType::WaitFd(cq_type) => match cq_type {
//                 EqCqOpt::Shared(rx_cq) => {
//                     ft_wait_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(_, rx_cq) => {
//                     ft_wait_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//             },
//             CqType::WaitYield(cq_type) => match cq_type {
//                 EqCqOpt::Shared(rx_cq) => {
//                     ft_wait_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(_, rx_cq) => {
//                     ft_wait_for_comp(rx_cq, &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//                 }
//             },
//         }
//     } else {
//         ft_get_cntr_comp(rx_cntr, total);
//     }
// }

// pub fn ft_get_tx_comp<CNTR: WaitCntr + ReadCntr>(
//     gl_ctx: &mut TestsGlobalCtx,
//     tx_cntr: &Option<Counter<CNTR>>,
//     tx_cq: &CqType,
//     total: u64,
// ) {
//     if gl_ctx.options & FT_OPT_TX_CQ != 0 {
//         match tx_cq {
//             CqType::Spin(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) => {
//                     ft_spin_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(tx_cq, _) => {
//                     ft_spin_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//             },
//             CqType::Sread(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) => {
//                     ft_wait_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(tx_cq, _) => {
//                     ft_wait_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//             },
//             CqType::WaitSet(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) => {
//                     ft_wait_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(tx_cq, _) => {
//                     ft_wait_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//             },
//             CqType::WaitFd(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) => {
//                     ft_wait_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(tx_cq, _) => {
//                     ft_wait_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//             },
//             CqType::WaitYield(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) => {
//                     ft_wait_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//                 EqCqOpt::Separate(tx_cq, _) => {
//                     ft_wait_for_comp(tx_cq, &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//                 }
//             },
//         }
//     } else {
//         ft_get_cntr_comp(tx_cntr, total);
//     }
// }

// pub fn ft_need_mr_reg<E>(info: &InfoEntry<E>) -> bool {
//     if info.caps().is_rma() || info.caps().is_atomic() {
//         true
//     } else {
//         info.domain_attr().mr_mode().is_local()
//     }
// }

// pub fn ft_chek_mr_local_flag<E>(info: &InfoEntry<E>) -> bool {
//     info.mode().is_local_mr() || info.domain_attr().mr_mode().is_local()
// }

// pub fn ft_rma_read_target_allowed(caps: &InfoCapsImpl) -> bool {
//     if caps.is_rma() || caps.is_atomic() {
//         if caps.is_remote_read() {
//             return true;
//         } else {
//             return !(caps.is_read() || caps.is_write() || caps.is_remote_write());
//         }
//     }

//     false
// }

// pub fn ft_rma_write_target_allowed(caps: &InfoCapsImpl) -> bool {
//     if caps.is_rma() || caps.is_atomic() {
//         if caps.is_remote_write() {
//             return true;
//         } else {
//             return !(caps.is_read() || caps.is_write() || caps.is_remote_write());
//         }
//     }

//     false
// }

// pub fn ft_info_to_mr_builder<'a, 'b: 'a, E>(
//     buff: &'b [u8],
//     info: &InfoEntry<E>,
// ) -> libfabric::mr::MemoryRegionBuilder<'a> {
//     let mut mr_builder = libfabric::mr::MemoryRegionBuilder::new(buff, enums::HmemIface::System);

//     if ft_chek_mr_local_flag(info) {
//         if info.caps().is_msg() || info.caps().is_tagged() {
//             let mut temp = info.caps().is_send();
//             if temp {
//                 mr_builder = mr_builder.access_send();
//             }
//             temp |= info.caps().is_recv();
//             if temp {
//                 mr_builder = mr_builder.access_recv();
//             }
//             if !temp {
//                 mr_builder = mr_builder.access_send().access_recv();
//             }
//         }
//     } else if info.caps().is_rma() || info.caps().is_atomic() {
//         if ft_rma_read_target_allowed(info.caps()) {
//             mr_builder = mr_builder.access_remote_read();
//         }
//         if ft_rma_write_target_allowed(info.caps()) {
//             mr_builder = mr_builder.access_remote_write();
//         }
//     }

//     mr_builder
// }

// pub fn ft_reg_mr<I, E: 'static>(
//     info: &InfoEntry<I>,
//     domain: &ConfDomain,
//     ep: &Endpoint<E>,
//     buf: &mut [u8],
//     key: u64,
// ) -> Option<libfabric::mr::MemoryRegion> {
//     if !ft_need_mr_reg(info) {
//         // println!("MR not needed");
//         return None;
//     }
//     // let iov = libfabric::iovec::IoVec::from_slice(buf);
//     // let mut mr_attr = libfabric::mr::MemoryRegionAttr::new().iov(std::slice::from_ref(&iov)).requested_key(key).iface(libfabric::enums::HmemIface::SYSTEM);

//     let mr = match domain {
//         ConfDomain::Unbound(domain) => ft_info_to_mr_builder(buf, info)
//             .requested_key(key)
//             .build(domain)
//             .unwrap(),
//         ConfDomain::Bound(domain) => ft_info_to_mr_builder(buf, info)
//             .requested_key(key)
//             .build(domain)
//             .unwrap(),
//     };

//     let mr = match mr {
//         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,

//         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
//             libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => match ep {
//                 Endpoint::Connectionless(ep) => ep_binding_memory_region.enable(ep),
//                 Endpoint::ConnectionOriented(ep) => ep_binding_memory_region.enable(ep),
//             }
//             .unwrap(),
//             libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
//                 rma_event_memory_region.enable().unwrap()
//             }
//         },
//     };

//     mr.into()
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_sync<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
//     ep: &EndpointCaps<M, T>,
//     gl_ctx: &mut TestsGlobalCtx,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
// ) {
//     ft_tx(gl_ctx, ep, 1, mr, cq_type, tx_cntr);
//     ft_rx(gl_ctx, ep, 1, mr, cq_type, rx_cntr);
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_exchange_keys<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     domain: &ConfDomain,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
// ) -> RemoteMemAddressInfo {
//     // let mut addr ;
//     // let mut key_size = 0;
//     // let mut rma_iov = libfabric::iovec::RmaIoVec::new();

//     // if info.domain_attr().mr_mode.is_raw() {
//     //     addr = mr.address( 0).unwrap(); // [TODO] Change this to return base_addr, key_size
//     // }

//     // let len = std::mem::size_of::<libfabric::iovec::RmaIoVec>();
//     // if key_size >= len - std::mem::size_of_val(&rma_iov.get_key()) {
//     //     panic!("Key size does not fit");
//     // }

//     let mem_info = MemAddressInfo::from_slice(
//         &gl_ctx.buf[..],
//         gl_ctx.rx_buf_index,
//         &mr.as_ref().unwrap().key().unwrap(),
//         info,
//     );
//     // if info.domain_attr().mr_mode().is_basic() || info.domain_attr().mr_mode().is_virt_addr() {
//     //     let addr = gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + ft_rx_prefix_size(info)]
//     //         .as_mut_ptr() as u64;
//     //     rma_iov = rma_iov.address(addr);
//     // }

//     // let key = mr.as_ref().unwrap().key().unwrap();

//     // let key_bytes = key.to_bytes();
//     // let key_len = key_bytes.len();
//     // if key_len > std::mem::size_of::<u64>() {
//     //     panic!("Key size does not fit");
//     // }
//     // let mut key = 0u64;
//     // unsafe {
//     //     std::slice::from_raw_parts_mut(&mut key as *mut u64 as *mut u8, 8)
//     //         .copy_from_slice(&key_bytes)
//     // };
//     // rma_iov = rma_iov.key(key);
//     let mem_info_bytes = mem_info.to_bytes();
//     gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + mem_info_bytes.len()]
//         .copy_from_slice(mem_info_bytes);

//     ft_tx(
//         gl_ctx,
//         ep,
//         mem_info_bytes.len() + ft_tx_prefix_size(info),
//         mr,
//         cq_type,
//         tx_cntr,
//     );
//     ft_get_rx_comp(gl_ctx, rx_cntr, cq_type, gl_ctx.rx_seq);

//     let mem_info = unsafe {
//         MemAddressInfo::from_bytes(
//             &gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + mem_info_bytes.len()],
//         )
//     };

//     let peer_info = match domain {
//         ConfDomain::Unbound(domain) => mem_info.into_remote_info(domain),

//         ConfDomain::Bound(domain) => mem_info.into_remote_info(domain),
//     }
//     .unwrap();

//     match cq_type {
//         CqType::Spin(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq)
//             }
//         },
//         CqType::Sread(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq)
//             }
//         },
//         CqType::WaitSet(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq)
//             }
//         },
//         CqType::WaitFd(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq)
//             }
//         },
//         CqType::WaitYield(cq_type) => match cq_type {
//             EqCqOpt::Separate(_, rx_cq) | EqCqOpt::Shared(rx_cq) => {
//                 ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq)
//             }
//         },
//     }

//     ft_sync(ep, gl_ctx, cq_type, tx_cntr, rx_cntr, mr);

//     peer_info
// }

// pub fn start_server<M: MsgDefaultCap, T: TagDefaultCap>(
//     hints: HintsCaps<M, T>,
//     node: String,
//     service: String,
// ) -> (
//     InfoWithCaps<M, T>,
//     fabric::Fabric,
//     libfabric::eq::EventQueue<EventQueueOptions>,
//     PassiveEndpointCaps<M, T>,
// ) {
//     match hints {
//         HintsCaps::Msg(hints) => {
//             let info = ft_getinfo(hints, node, service, true, true);
//             let entry = info.into_iter().next().unwrap();

//             let fab = libfabric::fabric::FabricBuilder::new()
//                 .build(&entry)
//                 .unwrap();

//             let eq = EventQueueBuilder::new(&fab).build().unwrap();

//             let pep = EndpointBuilder::new(&entry).build_passive(&fab).unwrap();

//             pep.bind(&eq, 0).unwrap();
//             pep.listen().unwrap();

//             (
//                 InfoWithCaps::Msg(entry),
//                 fab,
//                 eq,
//                 PassiveEndpointCaps::Msg(pep),
//             )
//         }
//         HintsCaps::Tagged(hints) => {
//             let info = ft_getinfo(hints, node, service, true, true);
//             let entry = info.into_iter().next().unwrap();

//             let fab = libfabric::fabric::FabricBuilder::new()
//                 .build(&entry)
//                 .unwrap();

//             let eq = EventQueueBuilder::new(&fab).build().unwrap();

//             let pep = EndpointBuilder::new(&entry).build_passive(&fab).unwrap();

//             pep.bind(&eq, 0).unwrap();
//             pep.listen().unwrap();

//             (
//                 InfoWithCaps::Tagged(entry),
//                 fab,
//                 eq,
//                 PassiveEndpointCaps::Tagged(pep),
//             )
//         }
//     }
// }

// #[allow(clippy::type_complexity)]
// pub fn ft_client_connect<M: MsgDefaultCap + 'static, T: TagDefaultCap + 'static>(
//     hints: HintsCaps<M, T>,
//     gl_ctx: &mut TestsGlobalCtx,
//     node: String,
//     service: String,
// ) -> (
//     InfoWithCaps<M, T>,
//     CqType,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     Option<libfabric::cntr::Counter<DefaultCntr>>,
//     EndpointCaps<M, T>,
//     Option<libfabric::mr::MemoryRegion>,
// ) {
//     match hints {
//         HintsCaps::Msg(hints) => {
//             let info = ft_getinfo(hints, node, service, true, false);

//             let entry = info.into_iter().next().unwrap();
//             gl_ctx.tx_ctx = Some(entry.allocate_context());
//             gl_ctx.rx_ctx = Some(entry.allocate_context());

//             let (_fab, eq, domain) = ft_open_fabric_res(&entry);
//             let (cq_type, tx_cntr, rx_cntr, rma_cntr, mut ep, _) =
//                 ft_alloc_active_res(&entry, gl_ctx, &domain);

//             let mr = ft_enable_ep_recv(
//                 &entry, gl_ctx, &mut ep, &domain, &tx_cntr, &rx_cntr, &rma_cntr,
//             );
//             let enable_ep = match ep {
//                 Endpoint::ConnectionOriented(ep) => ep.enable(&eq).unwrap(),
//                 _ => panic!("Unexpected Endpoint Type"),
//             };
//             let pending_conn_ep = match enable_ep {
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(ep) => {
//                     ft_connect_ep(ep, &eq, entry.dest_addr().unwrap())
//                 }
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(_) => {
//                     panic!("This should be a client")
//                 }
//             };

//             let ep = ft_complete_connect(pending_conn_ep, &eq);
//             let mut ep = EndpointCaps::ConnectedMsg(ep);
//             ft_ep_recv(
//                 &entry, gl_ctx, &mut ep, &domain, &cq_type, &eq, &None, &tx_cntr, &rx_cntr,
//                 &rma_cntr, &mr,
//             );

//             (InfoWithCaps::Msg(entry), cq_type, tx_cntr, rx_cntr, ep, mr)
//         }
//         HintsCaps::Tagged(hints) => {
//             let info = ft_getinfo(hints, node, service, true, false);

//             let entry = info.into_iter().next().unwrap();
//             gl_ctx.tx_ctx = Some(entry.allocate_context());
//             gl_ctx.rx_ctx = Some(entry.allocate_context());

//             let (_fab, eq, domain) = ft_open_fabric_res(&entry);
//             let (cq_type, tx_cntr, rx_cntr, rma_cntr, mut ep, _) =
//                 ft_alloc_active_res(&entry, gl_ctx, &domain);
//             let mr = ft_enable_ep_recv(
//                 &entry, gl_ctx, &mut ep, &domain, &tx_cntr, &rx_cntr, &rma_cntr,
//             );

//             let enable_ep = match ep {
//                 Endpoint::ConnectionOriented(ep) => ep.enable(&eq).unwrap(),
//                 _ => panic!("Unexpected Endpoint Type"),
//             };
//             let pending_conn_ep = match enable_ep {
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(ep) => {
//                     ft_connect_ep(ep, &eq, entry.dest_addr().unwrap())
//                 }
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(_) => {
//                     panic!("This should be a client")
//                 }
//             };

//             let ep = ft_complete_connect(pending_conn_ep, &eq);
//             let mut ep = EndpointCaps::ConnectedTagged(ep);
//             ft_ep_recv(
//                 &entry, gl_ctx, &mut ep, &domain, &cq_type, &eq, &None, &tx_cntr, &rx_cntr,
//                 &rma_cntr, &mr,
//             );
//             (
//                 InfoWithCaps::Tagged(entry),
//                 cq_type,
//                 tx_cntr,
//                 rx_cntr,
//                 ep,
//                 mr,
//             )
//         }
//     }

//     // (info, fab, domain, eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, mr, mr_desc)
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_finalize_ep<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
// ) {
//     let base =
//         &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + 4 + ft_tx_prefix_size(info)];

//     match ep {
//         EndpointCaps::ConnectedMsg(_epp) => match cq_type {
//             CqType::Spin(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::Sread(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitSet(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitFd(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitYield(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//         },
//         EndpointCaps::ConnectedTagged(_epp) => match cq_type {
//             CqType::Spin(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::Sread(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitSet(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitFd(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitYield(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//         },
//         EndpointCaps::ConnlessMsg(_epp) => match cq_type {
//             CqType::Spin(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::Sread(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitSet(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitFd(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitYield(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => msg_post(
//                     SendOp::MsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//         },
//         EndpointCaps::ConnlessTagged(_epp) => match cq_type {
//             CqType::Spin(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::Sread(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitSet(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitFd(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//             CqType::WaitYield(cq_type) => match cq_type {
//                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => tagged_post(
//                     TagSendOp::TagMsgSend,
//                     &mut gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                     &gl_ctx.remote_address,
//                     gl_ctx.ft_tag,
//                     tx_cq,
//                     ep,
//                     mr,
//                     base,
//                     NO_CQ_DATA,
//                 ),
//             },
//         },
//     }

//     ft_get_tx_comp(gl_ctx, tx_cntr, cq_type, gl_ctx.tx_seq);
//     ft_get_rx_comp(gl_ctx, rx_cntr, cq_type, gl_ctx.rx_seq);
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_finalize<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
// ) {
//     ft_finalize_ep(info, gl_ctx, ep, mr, cq_type, tx_cntr, rx_cntr);
// }

// // pub fn close_all_pep(fab: libfabric::fabric::Fabric, domain: libfabric::domain::Domain, eq :libfabric::eq::EventQueue, rx_cq: libfabric::cq::CompletionQueue, tx_cq: libfabric::cq::CompletionQueue, ep: libfabric::ep::Endpoint<E>, pep: libfabric::ep::PassiveEndpoint, mr: Option<libfabric::mr::MemoryRegion>) {
// //     ep.close().unwrap();
// //     pep.close().unwrap();
// //     eq.close().unwrap();
// //     tx_cq.close().unwrap();
// //     rx_cq.close().unwrap();
// //     if let Some(mr_val) = mr { mr_val.close().unwrap(); }
// //     domain.close().unwrap();
// //     fab.close().unwrap();
// // }

// // pub fn close_all(fab: &mut libfabric::fabric::Fabric, domain: &mut libfabric::domain::Domain, eq :&mut libfabric::eq::EventQueue, rx_cq: &mut libfabric::cq::CompletionQueue, tx_cq: &mut libfabric::cq::CompletionQueue, tx_cntr: Option<Counter>, rx_cntr: Option<Counter>, ep: &mut libfabric::ep::Endpoint<E>, mr: Option<&mut libfabric::mr::MemoryRegion>, av: Option<&mut libfabric::av::AddressVector>) {

// //     ep.close().unwrap();
// //     eq.close().unwrap();
// //     tx_cq.close().unwrap();
// //     rx_cq.close().unwrap();
// //     if let Some(mr_val) = mr { mr_val.close().unwrap() }
// //     if let Some(av_val) = av { av_val.close().unwrap() }
// //     if let Some(rxcntr_val) = rx_cntr { rxcntr_val.close().unwrap() }
// //     if let Some(txcnr_val) = tx_cntr { txcnr_val.close().unwrap() }
// //     domain.close().unwrap();
// //     fab.close().unwrap();
// // }


impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn pingpong(&self, warmup: usize, iters: usize, server: bool, size: usize) {
        self.sync().unwrap();
        let mut now = Instant::now();
        if !server {
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
    pub fn pingpong_tagged(&self, warmup: usize, iters: usize, server: bool, size: usize) {
        self.sync().unwrap();
        let mut now = Instant::now();
        if !server {
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
    pub fn pingpong_rma(&self, warmup: usize, iters: usize, server: bool, size: usize, window_size: usize) {
        self.sync().unwrap();
        let mut j = 0;
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

// #[allow(clippy::too_many_arguments)]
// pub fn pingpong<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
//     inject_size: usize,
//     gl_ctx: &mut TestsGlobalCtx,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     iters: usize,
//     warmup: usize,
//     size: usize,
//     server: bool,
// ) {
//     // let inject_size = info.tx_attr().get_inject_size();
//     ft_sync(ep, gl_ctx, cq_type, tx_cntr, rx_cntr, mr);
//     let mut now = Instant::now();
//     if !server {
//         for i in 0..warmup + iters {
//             if i == warmup {
//                 now = Instant::now(); // Start timer
//             }

//             if size < inject_size {
//                 ft_inject(gl_ctx, ep, size, cq_type);
//             } else {
//                 ft_tx(gl_ctx, ep, size, mr, cq_type, tx_cntr);
//             }

//             ft_rx(gl_ctx, ep, size, mr, cq_type, rx_cntr);
//         }
//     } else {
//         for i in 0..warmup + iters {
//             if i == warmup {
//                 // Start timer
//             }

//             ft_rx(gl_ctx, ep, size, mr, cq_type, rx_cntr);

//             if size < inject_size {
//                 ft_inject(gl_ctx, ep, size, cq_type);
//             } else {
//                 ft_tx(gl_ctx, ep, size, mr, cq_type, tx_cntr);
//             }
//         }
//     }
//     if size == 1 {
//         println!("bytes iters total time MB/sec usec/xfer Mxfers/sec",);
//     }
//     // println!("Done");
//     // Stop timer
//     let elapsed = now.elapsed();
//     let bytes = iters * size * 2;
//     let usec_per_xfer = elapsed.as_micros() as f64 / iters as f64 / 2_f64;
//     println!(
//         "{} {} {} {} s {} {} {}",
//         size,
//         iters,
//         bytes,
//         elapsed.as_secs(),
//         bytes as f64 / elapsed.as_micros() as f64,
//         usec_per_xfer,
//         1.0 / usec_per_xfer
//     );
//     // print perf data
// }

// #[allow(clippy::too_many_arguments)]
// pub fn bw_tx_comp<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
// ) {
//     ft_get_tx_comp(gl_ctx, tx_cntr, cq_type, gl_ctx.tx_seq);
//     ft_rx(gl_ctx, ep, FT_RMA_SYNC_MSG_BYTES, mr, cq_type, rx_cntr);
// }

// #[allow(clippy::too_many_arguments)]
// pub fn bw_rma_comp<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     op: &RmaOp,
//     ep: &EndpointCaps<M, T>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     server: bool,
// ) {
//     if matches!(op, RmaOp::RMA_WRITEDATA) {
//         if !server {
//             bw_tx_comp(gl_ctx, ep, cq_type, tx_cntr, rx_cntr, mr);
//         }
//     } else {
//         ft_get_tx_comp(gl_ctx, tx_cntr, cq_type, gl_ctx.tx_seq);
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn pingpong_rma<
//     CNTR: WaitCntr,
//     E: RmaCap,
//     M: MsgDefaultCap + RmaDefaultCap,
//     T: TagDefaultCap + RmaDefaultCap,
// >(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     op: RmaOp,
//     remote: &RemoteMemAddressInfo,
//     iters: usize,
//     warmup: usize,
//     size: usize,
//     server: bool,
// ) {
//     let inject_size = info.tx_attr().inject_size();

//     ft_sync(ep, gl_ctx, cq_type, tx_cntr, rx_cntr, mr);
//     let offest_rma_start =
//         FT_RMA_SYNC_MSG_BYTES + std::cmp::max(ft_tx_prefix_size(info), ft_rx_prefix_size(info));

//     let mut now = Instant::now();
//     let mut j = 0;
//     let mut offset = 0;
//     for i in 0..warmup + iters {
//         if i == warmup {
//             now = Instant::now(); // Start timer
//         }
//         if j == 0 {
//             offset = offest_rma_start;
//         }

//         if matches!(&op, RmaOp::RMA_WRITE) {
//             match ep {
//                 EndpointCaps::ConnlessMsg(ep) => {
//                     if size < inject_size {
//                         match cq_type {
//                             CqType::Spin(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                             CqType::Sread(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                             CqType::WaitSet(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                             CqType::WaitFd(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                             CqType::WaitYield(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                         }
//                     } else {
//                         match cq_type {
//                             CqType::Spin(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                             CqType::Sread(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                             CqType::WaitSet(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                             CqType::WaitFd(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                             CqType::WaitYield(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                         }
//                     }
//                 }
//                 EndpointCaps::ConnlessTagged(ep) => {
//                     if size < inject_size {
//                         match cq_type {
//                             CqType::Spin(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                             CqType::Sread(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                             CqType::WaitSet(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                             CqType::WaitFd(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                             CqType::WaitYield(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, tx_cq)
//                                 }
//                             },
//                         }
//                     } else {
//                         match cq_type {
//                             CqType::Spin(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                             CqType::Sread(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                             CqType::WaitSet(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                             CqType::WaitFd(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                             CqType::WaitYield(cq_type) => match cq_type {
//                                 EqCqOpt::Shared(tx_cq) | EqCqOpt::Separate(tx_cq, _) => {
//                                     ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr, tx_cq)
//                                 }
//                             },
//                         }
//                     }
//                 }
//                 _ => panic!("Connected RMA not supported"),
//             }
//         }

//         j += 1;

//         if j == gl_ctx.window_size {
//             bw_rma_comp(gl_ctx, &op, ep, cq_type, tx_cntr, rx_cntr, mr, server);
//             j = 0;
//         }

//         offset += size;
//     }

//     bw_rma_comp(gl_ctx, &op, ep, cq_type, tx_cntr, rx_cntr, mr, server);

//     if size == 1 {
//         println!("bytes iters total time MB/sec usec/xfer Mxfers/sec");
//     }
//     // println!("Done");
//     // Stop timer
//     let elapsed = now.elapsed();
//     let bytes = iters * size * 2;
//     let usec_per_xfer = elapsed.as_micros() as f64 / iters as f64 / 2_f64;
//     println!(
//         "{} {} {} {} s {} {} {}",
//         size,
//         iters,
//         bytes,
//         elapsed.as_secs(),
//         bytes as f64 / elapsed.as_micros() as f64,
//         usec_per_xfer,
//         1.0 / usec_per_xfer
//     );
//     // print perf data
// }

// #[allow(unused_macros)]
// macro_rules! define_test {
//     ($func_name:ident, $async_fname:ident, $body: block) => {

//         #[test]
//         #[ignore]
//         fn $func_name() $body
//     };
// }

// #[allow(unused_imports)]
// pub(crate) use define_test;

// #[allow(unused_macros)]
// macro_rules! call {
//     ($func_name:path, $( $x:expr),* ) => {
//         $func_name($($x,)*)
//     }
// }

// #[allow(unused_imports)]
// pub(crate) use call;

// use super::common;
// use common::gen_info;

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
use libfabric::infocapsoptions::InfoCaps;
use libfabric::mr::MemoryRegionSlice;
use libfabric::mr::MemoryRegionSliceMut;
use libfabric::AsFiType;
use libfabric::RemoteMemAddrSlice;
use libfabric::RemoteMemAddrSliceMut;
use std::cell::RefCell;
use std::ops::Range;
use std::sync::atomic::AtomicUsize;
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
    Context, CqCaps, EqCaps, CntrCaps, MappedAddress, MemAddressInfo, MyRc, RemoteMemAddressInfo,
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

        self.reg_mem.borrow_mut()[..mem_bytes.len()].copy_from_slice(mem_bytes);
        
        self.send(
            0..mem_bytes.len(),
            None,
            false,
        );
        self.recv(
            mem_bytes.len()..2*mem_bytes.len(),
            false,
        );
        
        // self.cq_type.rx_cq().sread(1, -1).unwrap();
        self.wait_rx(1);
        let mem_info = unsafe { MemAddressInfo::from_bytes(&self.reg_mem.borrow()[mem_bytes.len()..2*mem_bytes.len()]) };
        let remote_mem_info = mem_info.into_remote_info(&self.domain).unwrap();
        println!("Remote addr: {:?}, size: {}", remote_mem_info.mem_address().as_ptr(), remote_mem_info.mem_len());

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