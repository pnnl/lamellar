// use libfabric::async_::conn_ep::AcceptPendingEndpoint;
// use libfabric::domain::NoEventQueue;
// use libfabric::{
//     async_::{
//         av::{AddressVector, AddressVectorBuilder},
//         comm::{message::AsyncSendEp, tagged::AsyncTagSendEp},
//         cq::{CompletionQueue, CompletionQueueBuilder},
//         ep::{Endpoint, EndpointBuilder, PassiveEndpoint},
//         eq::{AsyncReadEq, EventQueue, EventQueueBuilder},
//     },
//     cntr::Counter,
//     cq::{ReadCq, WaitCq},
//     domain::{DomainBase, DomainBuilder},
//     enums::{self, AVOptions},
//     ep::Address,
//     fabric::{self, Fabric},
//     info::{Info, InfoCapsImpl, InfoEntry, InfoHints},
//     infocapsoptions::{self, MsgDefaultCap, TagDefaultCap},
//     mr::{MemoryRegion, MemoryRegionBuilder},
//     Context, MappedAddress,
// };
// use libfabric::{
//     async_::{
//         comm::{message::ConnectedAsyncSendEp, rma::AsyncWriteEp, tagged::ConnectedAsyncTagSendEp},
//         conn_ep::{ConnectedEndpoint, UnconnectedEndpoint},
//         connless_ep::ConnectionlessEndpoint,
//     },
//     cntr::{ReadCntr, WaitCntr},
//     comm::{
//         message::{ConnectedRecvEp, ConnectedSendEp, RecvEp, SendEp},
//         rma::{ReadEp, WriteEp},
//         tagged::{ConnectedTagRecvEp, TagRecvEp},
//     },
//     ep::BaseEndpoint,
//     infocapsoptions::RmaDefaultCap,
// };
// use std::time::Instant;

// pub enum CompMeth {
//     // Spin,
//     // Sread,
//     // WaitSet,
//     // Yield,
//     WaitFd,
// }
// pub type EventQueueOptions = libfabric::async_eq_caps_type!(libfabric::EqCaps::WRITE);
// pub type CounterOptions = libfabric::cntr_caps_type!(libfabric::CntrCaps::WAIT);

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
// use libfabric::{FabInfoCaps, MemAddressInfo, RemoteMemAddressInfo};

// pub type MsgRma = libfabric::info_caps_type!(FabInfoCaps::MSG, FabInfoCaps::RMA);
// pub type MsgTagRma =
//     libfabric::info_caps_type!(FabInfoCaps::MSG, FabInfoCaps::TAG, FabInfoCaps::RMA);
// pub type CqAsync = libfabric::async_cq_caps_type!();

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
// pub enum EpCqType {
//     Shared(CompletionQueue<CqAsync>),
//     Separate(CompletionQueue<CqAsync>, CompletionQueue<CqAsync>),
// }

// impl EpCqType {
//     fn rx_cq(&self) -> &CompletionQueue<CqAsync> {
//         match self {
//             EpCqType::Shared(rx_cq) | EpCqType::Separate(_, rx_cq) => rx_cq,
//         }
//     }
//     fn tx_cq(&self) -> &CompletionQueue<CqAsync> {
//         match self {
//             EpCqType::Shared(tx_cq) | EpCqType::Separate(tx_cq, _) => tx_cq,
//         }
//     }
// }

// pub enum CqType {
//     // Spin(CompletionQueue<Options<libfabric::cqoptions::WaitNone, libfabric::cqoptions::Off>>),
//     // Sread(CompletionQueue<Options<libfabric::cqoptions::WaitNoRetrieve, libfabric::cqoptions::Off>>),
//     // WaitSet(CompletionQueue<Options<libfabric::cqoptions::WaitNoRetrieve, libfabric::cqoptions::Off>>),
//     WaitFd(EpCqType),
//     // WaitYield(CompletionQueue<Options<libfabric::cqoptions::WaitNoRetrieve, libfabric::cqoptions::Off>>),
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
//             comp_method: CompMeth::WaitFd,
//             tx_ctx: None,
//             rx_ctx: None,
//             options: FT_OPT_RX_CQ | FT_OPT_TX_CQ,
//         }
//     }
// }

// impl Default for TestsGlobalCtx {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// pub fn ft_open_fabric_res<E>(
//     info: &InfoEntry<E>,
// ) -> (
//     Fabric,
//     EventQueue<EventQueueOptions>,
//     DomainBase<NoEventQueue>,
// ) {
//     let fab = libfabric::fabric::FabricBuilder::new().build(info).unwrap();
//     let eq = EventQueueBuilder::new(&fab).write().build().unwrap();
//     let domain = ft_open_domain_res(info, &fab);
//     // domain.bind_eq(&eq, true).unwrap();
//     (fab, eq, domain)
// }

// pub fn ft_open_domain_res<E>(
//     info: &InfoEntry<E>,
//     fab: &fabric::Fabric,
// ) -> DomainBase<NoEventQueue> {
//     DomainBuilder::new(fab, info).build().unwrap()
// }

// pub fn ft_alloc_ep_res<E, EQ: AsyncReadEq + 'static>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     domain: &DomainBase<NoEventQueue>,
//     eq: &EventQueue<EQ>,
// ) -> (
//     CqType,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     Option<AddressVector>,
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
//         // CompMeth::Spin => {
//         //     (CqType::Spin(tx_cq_builder.wait_none().build().unwrap()), CqType::Spin(rx_cq_builder.wait_none().build().unwrap()))
//         // },
//         // CompMeth::Sread => {
//         //     (CqType::Sread(tx_cq_builder.build().unwrap()), CqType::Sread(rx_cq_builder.build().unwrap()))
//         // },
//         // CompMeth::WaitSet => todo!(),
//         CompMeth::WaitFd => {
//             if gl_ctx.options & FT_OPT_CQ_SHARED == 0 {
//                 CqType::WaitFd(EpCqType::Separate(
//                     tx_cq_builder.build(domain).unwrap(),
//                     rx_cq_builder.build(domain).unwrap(),
//                 ))
//             } else {
//                 CqType::WaitFd(EpCqType::Shared(shared_cq_builder.build(domain).unwrap()))
//             }
//         } // CompMeth::Yield => {
//           //     (CqType::WaitYield(tx_cq_builder.wait_yield().build().unwrap()), CqType::WaitYield(rx_cq_builder.wait_yield().build().unwrap()))
//           // },
//     };

//     let tx_cntr = if gl_ctx.options & FT_OPT_TX_CNTR != 0 {
//         todo!();
//         // Some(CounterBuilder::new(domain).build().unwrap())
//     } else {
//         None
//     };

//     let rx_cntr = if gl_ctx.options & FT_OPT_RX_CNTR != 0 {
//         todo!();
//         // Some(CounterBuilder::new(domain).build().unwrap())
//     } else {
//         None
//     };

//     let rma_cntr = if gl_ctx.options & FT_OPT_RX_CNTR != 0 && info.caps().is_rma() {
//         todo!();
//         // Some(CounterBuilder::new(domain).build().unwrap())
//     } else {
//         None
//     };

//     let av = match info.ep_attr().type_() {
//         libfabric::enums::EndpointType::Rdm | libfabric::enums::EndpointType::Dgram => {
//             let av = match info.domain_attr().av_type() {
//                 libfabric::enums::AddressVectorType::Unspec => AddressVectorBuilder::new(eq),
//                 _ => AddressVectorBuilder::new(eq).type_(info.domain_attr().av_type().clone()),
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
// pub fn ft_alloc_active_res<E, EQ: AsyncReadEq + 'static>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     domain: &DomainBase<NoEventQueue>,
//     eq: &EventQueue<EQ>,
// ) -> (
//     CqType,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     Endpoint<E>,
//     Option<AddressVector>,
// ) {
//     let (cq_type, tx_cntr, rx_cntr, rma_cntr, av) = ft_alloc_ep_res(info, gl_ctx, domain, eq);
//     let ep = match &cq_type {
//         CqType::WaitFd(eq_cq_opt) => match eq_cq_opt {
//             EpCqType::Shared(scq) => EndpointBuilder::new(info).build_with_shared_cq(domain, scq),
//             EpCqType::Separate(tx_cq, rx_cq) => {
//                 EndpointBuilder::new(info).build_with_separate_cqs(domain, tx_cq, rx_cq)
//             }
//         },
//     }
//     .unwrap();
//     (cq_type, tx_cntr, rx_cntr, rma_cntr, ep, av)
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_prepare_ep<CNTR: WaitCntr + 'static, I, E>(
//     info: &InfoEntry<I>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &Endpoint<E>,
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
//         }
//     }
// }

// //     println!("Checking for Connected");
// //     if let Ok(event) = task::block_on( async {eq.read_async().await}) {
// //         if let libfabric::eq::Event::Connected(_) = event {
// //             println!("Connected retrieved");
// //         }
// //         else {
// //             panic!("Unexpected Event Type");
// //         }
// //     }
// //     else {
// //         let err_entry = eq.readerr().unwrap();
// //         panic!("{}\n", eq.strerror(&err_entry));

// //     }
// // }

// pub async fn ft_accept_connection<EQ: AsyncReadEq, E>(
//     ep: AcceptPendingEndpoint<E>,
//     _eq: &EventQueue<EQ>,
// ) -> ConnectedEndpoint<E> {
//     ep.accept_async().await.unwrap()
// }

// pub async fn ft_retrieve_conn_req<E: infocapsoptions::Caps>(
//     pep: &PassiveEndpoint<E>,
// ) -> InfoEntry<E> {
//     // [TODO] Do not panic, return errors

//     let listener = pep.listen_async().unwrap();
//     let event = listener.next().await;

//     if let libfabric::eq::Event::ConnReq(entry) = event.unwrap() {
//         entry.info::<E>().unwrap()
//     } else {
//         panic!("Unexpected EventQueueEntry type")
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
// pub async fn ft_server_connect<
//     T: AsyncReadEq + 'static,
//     M: infocapsoptions::Caps + MsgDefaultCap + 'static,
//     TT: infocapsoptions::Caps + TagDefaultCap + 'static,
// >(
//     pep: &PassiveEndpointCaps<M, TT>,
//     gl_ctx: &mut TestsGlobalCtx,
//     eq: &EventQueue<T>,
//     fab: &fabric::Fabric,
// ) -> (
//     CqType,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     EndpointCaps<M, TT>,
//     Option<MemoryRegion>,
// ) {
//     match pep {
//         PassiveEndpointCaps::Msg(pep) => {
//             let new_info = ft_retrieve_conn_req(pep).await;
//             gl_ctx.tx_ctx = Some(new_info.allocate_context());
//             gl_ctx.rx_ctx = Some(new_info.allocate_context());
//             let domain = ft_open_domain_res(&new_info, fab);
//             let (cq_type, tx_cntr, rx_cntr, rma_cntr, ep, _) =
//                 ft_alloc_active_res(&new_info, gl_ctx, &domain, eq);
//             let mr = ft_enable_ep_recv(
//                 &new_info, gl_ctx, &ep, &domain, &tx_cntr, &rx_cntr, &rma_cntr,
//             );
//             let ep = match ep {
//                 Endpoint::Connectionless(_) => panic!("Expected Connected Endpoint"),
//                 Endpoint::ConnectionOriented(ep) => ep.enable(&eq).unwrap(),
//             };
//             let pending_ep = match ep {
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(_ep) => {
//                     panic!("This should be a server")
//                 }
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(ep) => ep,
//             };

//             let ep = ft_accept_connection(pending_ep, eq).await;
//             let mut ep = EndpointCaps::ConnectedMsg(ep);
//             ft_ep_recv(
//                 &new_info, gl_ctx, &mut ep, &domain, &cq_type, eq, &None, &tx_cntr, &rx_cntr,
//                 &rma_cntr, &mr,
//             );
//             (cq_type, tx_cntr, rx_cntr, ep, mr)
//         }
//         PassiveEndpointCaps::Tagged(pep) => {
//             let new_info = ft_retrieve_conn_req(pep).await;
//             gl_ctx.tx_ctx = Some(new_info.allocate_context());
//             gl_ctx.rx_ctx = Some(new_info.allocate_context());
//             let domain = ft_open_domain_res(&new_info, fab);
//             let (cq_type, tx_cntr, rx_cntr, rma_cntr, mut ep, _) =
//                 ft_alloc_active_res(&new_info, gl_ctx, &domain, eq);
//             let mr = ft_enable_ep_recv(
//                 &new_info, gl_ctx, &mut ep, &domain, &tx_cntr, &rx_cntr, &rma_cntr,
//             );
//             let ep = match ep {
//                 Endpoint::Connectionless(_) => panic!("Expected Connected Endpoint"),
//                 Endpoint::ConnectionOriented(ep) => ep.enable(&eq).unwrap(),
//             };

//             let pending_ep = match ep {
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(_) => {
//                     panic!("This should be a server")
//                 }
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(ep) => ep,
//             };

//             let ep = ft_accept_connection(pending_ep, eq).await;
//             let mut ep = EndpointCaps::ConnectedTagged(ep);
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
//     //         Info::new_source(libfabric::info::InfoSourceOpt::Service(service))
//     //     }
//     //     else {
//     //         Info::new().service(&service).node(&node)
//     //     };

//     //  info.hints(&hints).build().unwrap()
// }

// pub async fn ft_connect_ep<T: AsyncReadEq, E>(
//     ep: UnconnectedEndpoint<E>,
//     eq: &EventQueue<T>,
//     addr: &libfabric::ep::Address,
// ) -> ConnectedEndpoint<E> {
//     let res = ep.connect_async(addr).await;
//     match res {
//         Err(error) => match error.kind {
//             libfabric::error::ErrorKind::ErrorInEventQueue(err_entry) => {
//                 println!("Event Queue error: {}", err_entry.error());
//                 panic!("Provider error: {}", eq.strerror(&err_entry));
//             }
//             _ => panic!("{:?}", error),
//         },
//         Ok(conn_ep) => conn_ep,
//     }
// }

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
//     domain: &DomainBase<NoEventQueue>,
//     ep: &Endpoint<E>,
// ) -> Option<MemoryRegion> {
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

// pub fn ft_ep_recv<
//     EQ: AsyncReadEq + 'static,
//     CNTR: ReadCntr + 'static,
//     E,
//     M: MsgDefaultCap,
//     T: TagDefaultCap,
// >(
//     _info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &mut EndpointCaps<M, T>,
//     _domain: &DomainBase<NoEventQueue>,
//     cq_type: &CqType,
//     _eq: &EventQueue<EQ>,
//     _av: &Option<AddressVector>,
//     _tx_cntr: &Option<Counter<CNTR>>,
//     _rx_cntr: &Option<Counter<CNTR>>,
//     _rma_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<MemoryRegion>,
// ) {
//     match cq_type {
//         CqType::WaitFd(cq_type) => ft_post_rx(
//             gl_ctx,
//             ep,
//             std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size),
//             NO_CQ_DATA,
//             mr,
//             cq_type.rx_cq(),
//         ),
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_enable_ep_recv<CNTR: WaitCntr + 'static, E, T: 'static>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &Endpoint<T>,
//     domain: &DomainBase<NoEventQueue>,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     rma_cntr: &Option<Counter<CNTR>>,
// ) -> Option<MemoryRegion> {
//     let mr = {
//         ft_prepare_ep(info, gl_ctx, ep, tx_cntr, rx_cntr, rma_cntr);
//         ft_alloc_msgs(info, gl_ctx, domain, ep)
//     };
//     // match cq_type {
//     //     CqType::WaitFd(cq_type) => ft_post_rx(
//     //         gl_ctx,
//     //         ep,
//     //         std::cmp::max(FT_MAX_CTRL_MSG, gl_ctx.rx_size),
//     //         NO_CQ_DATA,
//     //         &mut data_desc,
//     //         cq_type.rx_cq(),
//     //     ),
//     // }

//     mr
// }

// pub enum InfoWithCaps<M, T> {
//     Msg(InfoEntry<M>),
//     Tagged(InfoEntry<T>),
// }

// #[allow(clippy::type_complexity)]
// pub async fn ft_init_fabric<M: MsgDefaultCap + 'static, T: TagDefaultCap + 'static>(
//     hints: HintsCaps<M, T>,
//     gl_ctx: &mut TestsGlobalCtx,
//     node: String,
//     service: String,
//     source: bool,
// ) -> (
//     InfoWithCaps<M, T>,
//     EndpointCaps<M, T>,
//     DomainBase<NoEventQueue>,
//     CqType,
//     Option<Counter<CounterOptions>>,
//     Option<Counter<CounterOptions>>,
//     Option<MemoryRegion>,
//     AddressVector,
// ) {
//     match hints {
//         HintsCaps::Msg(hints) => {
//             let info = ft_getinfo(hints, node.clone(), service.clone(), false, source);
//             let entry = info.into_iter().next().unwrap();
//             gl_ctx.tx_ctx = Some(entry.allocate_context());
//             gl_ctx.rx_ctx = Some(entry.allocate_context());
//             let (_fabric, eq, domain) = ft_open_fabric_res(&entry);
//             let (cq_type, tx_cntr, rx_cntr, rma_ctr, ep, av) =
//                 ft_alloc_active_res(&entry, gl_ctx, &domain, &eq);
//             let mr = ft_enable_ep_recv(&entry, gl_ctx, &ep, &domain, &tx_cntr, &rx_cntr, &rma_ctr);
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
//             )
//             .await;
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
//             let (cq_type, tx_cntr, rx_cntr, rma_ctr, ep, av) =
//                 ft_alloc_active_res(&entry, gl_ctx, &domain, &eq);
//             // let mut ep = EndpointCaps::Tagged(ep);

//             let mr = ft_enable_ep_recv(&entry, gl_ctx, &ep, &domain, &tx_cntr, &rx_cntr, &rma_ctr);
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
//             )
//             .await;
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
//     // (info, fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, eq, mr, av, mr_desc)
// }

// pub async fn ft_av_insert<E>(
//     info: &InfoEntry<E>,
//     av: &AddressVector,
//     addr: &Address,
//     options: AVOptions,
// ) -> MappedAddress {
//     let mut ctx = info.allocate_context();

//     let (_, mut added) = av
//         .insert_async(std::slice::from_ref(addr), options, &mut ctx)
//         .await
//         .unwrap();
//     added
//         .pop()
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

// macro_rules!  ft_post_async{
//     ($post_fn:ident, $op_str:literal, $ep:ident, $( $x:expr),* ) => {
//         loop {
//             let ret = $ep.$post_fn($($x,)*).await;
//             if ret.is_ok() {
//                 break;
//             }
//             else if let Err(ref err) = ret {
//                 if let libfabric::error::ErrorKind::ErrorInCompletionQueue(ref q_error)  = err.kind{
//                     println!("{:?}", q_error.error())
//                 }
//                 if !matches!(err.kind, libfabric::error::ErrorKind::TryAgain) {
//                     ret.unwrap();
//                     panic!("Unexpected error!")
//                 }
//             }
//         }
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
// }
// pub enum RecvOp {
//     Recv,
//     MsgRecv,
// }

// pub enum TagSendOp {
//     TagSend,
//     TagMsgSend,
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
// pub fn ft_post_rma_inject<CQ: ReadCq>(
//     gl_ctx: &mut TestsGlobalCtx,
//     rma_op: &RmaOp,
//     offset: usize,
//     size: usize,
//     remote: &RemoteMemAddressInfo,
//     ep: &impl WriteEp,
//     tx_cq: &CompletionQueue<CQ>,
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
// pub async fn ft_post_rma<CQ: ReadCq, E: RmaDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     rma_op: &RmaOp,
//     offset: usize,
//     size: usize,
//     remote: &RemoteMemAddressInfo,
//     ep: &ConnectionlessEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     tx_cq: &CompletionQueue<CQ>,
// ) {
//     let mr_desc = Some(mr.as_ref().unwrap().descriptor());

//     let fi_addr = gl_ctx.remote_address.as_ref().unwrap();
//     match rma_op {
//         RmaOp::RMA_WRITE => {
//             let addr = unsafe { remote.mem_address().add(offset) };
//             let key = remote.key();
//             let buf =
//                 &gl_ctx.buf[gl_ctx.tx_buf_index + offset..gl_ctx.tx_buf_index + offset + size];
//             // unsafe{ ft_post!(write, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_write", ep, buf, data_desc, fi_addr, addr, key); }
//             unsafe {
//                 ep.write_to_async(
//                     buf,
//                     mr_desc,
//                     fi_addr,
//                     addr,
//                     &key,
//                     &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 )
//                 .await
//                 .unwrap();
//             }
//         }

//         RmaOp::RMA_WRITEDATA => {
//             let addr = unsafe { remote.mem_address().add(offset) };
//             let key = remote.key();
//             let buf =
//                 &gl_ctx.buf[gl_ctx.tx_buf_index + offset..gl_ctx.tx_buf_index + offset + size];
//             let remote_cq_data = gl_ctx.remote_cq_data;
//             // unsafe{ ft_post!(writedata, ft_progress, tx_cq, gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, "fi_write", ep, buf, data_desc, remote_cq_data, fi_addr, addr, key); }
//             unsafe {
//                 ep.writedata_to_async(
//                     buf,
//                     mr_desc,
//                     remote_cq_data,
//                     fi_addr,
//                     addr,
//                     &key,
//                     gl_ctx.tx_ctx.as_mut().unwrap(),
//                 )
//                 .await
//                 .unwrap();
//             }
//         }

//         RmaOp::RMA_READ => {
//             let addr = unsafe { remote.mem_address().add(offset) };
//             let key = remote.key();
//             let buf =
//                 &mut gl_ctx.buf[gl_ctx.tx_buf_index + offset..gl_ctx.tx_buf_index + offset + size];
//             let _remote_cq_data = gl_ctx.remote_cq_data;
//             unsafe {
//                 ft_post!(
//                     read_from,
//                     ft_progress,
//                     tx_cq,
//                     gl_ctx.tx_seq,
//                     &mut gl_ctx.tx_cq_cntr,
//                     "fi_write",
//                     ep,
//                     buf,
//                     mr_desc,
//                     fi_addr,
//                     addr,
//                     &key
//                 );
//             }
//         }
//     }
// }

// pub fn connected_msg_post_inject<CQ: ReadCq, E: MsgDefaultCap>(
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     _ctx: &mut Context,
//     _remote_address: &Option<MappedAddress>,
//     tx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectedEndpoint<E>,
//     _data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     ft_post!(
//         inject,
//         ft_progress,
//         tx_cq,
//         *tx_seq,
//         tx_cq_cntr,
//         "inject",
//         ep,
//         base
//     );
// }

// pub fn connless_msg_post_inject<CQ: ReadCq, E: MsgDefaultCap>(
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     _ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     tx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectionlessEndpoint<E>,
//     _data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let fi_address = remote_address.as_ref().unwrap();
//     ft_post!(
//         inject_to,
//         ft_progress,
//         tx_cq,
//         *tx_seq,
//         tx_cq_cntr,
//         "inject",
//         ep,
//         base,
//         fi_address
//     );
// }

// pub async fn connless_msg_post<CQ: ReadCq, E: MsgDefaultCap>(
//     op: SendOp,
//     _tx_seq: &mut u64,
//     _tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     _tx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectionlessEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     let desc = if let Some(mr) = mr {
//         Some(mr.descriptor())
//     } else {
//         None
//     };
//     match op {
//         SendOp::MsgSend => {
//             let iov = libfabric::iovec::IoVec::from_slice(base);
//             let flag = libfabric::enums::SendMsgOptions::new().transmit_complete();

//             let fi_addr = remote_address.as_ref().unwrap();
//             let mut msg = libfabric::msg::Msg::from_iov(&iov, desc.as_ref(), fi_addr, None, ctx);
//             let msg_ref = &mut msg;
//             ft_post_async!(sendmsg_to_async, "sendmsg", ep, msg_ref, flag);
//         }
//         SendOp::Send => {
//             if let Some(fi_address) = remote_address {
//                 if data != NO_CQ_DATA {
//                     ft_post_async!(
//                         senddata_to_async,
//                         "",
//                         ep,
//                         base,
//                         desc,
//                         data,
//                         fi_address,
//                         ctx
//                     )
//                 } else {
//                     ft_post_async!(send_to_async, "", ep, base, desc, fi_address, ctx)
//                 }
//             }
//         }
//     }
// }

// pub async fn connected_msg_post<CQ: ReadCq, E: MsgDefaultCap>(
//     op: SendOp,
//     _tx_seq: &mut u64,
//     _tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     _remote_address: &Option<MappedAddress>,
//     _tx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectedEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     let desc = if let Some(mr) = mr {
//         Some(mr.descriptor())
//     } else {
//         None
//     };
//     match op {
//         SendOp::MsgSend => {
//             let iov = libfabric::iovec::IoVec::from_slice(base);
//             let flag = libfabric::enums::SendMsgOptions::new().transmit_complete();

//             let mut msg = libfabric::msg::MsgConnected::from_iov(&iov, desc.as_ref(), None, ctx);
//             let msg_ref = &mut msg;
//             ft_post_async!(sendmsg_async, "sendmsg", ep, msg_ref, flag);
//         }
//         SendOp::Send => {
//             if data != NO_CQ_DATA {
//                 ft_post_async!(senddata_async, "", ep, base, desc, data, ctx)
//             } else {
//                 ft_post_async!(send_async, "", ep, base, desc, ctx)
//             }
//         }
//     }
// }

// pub fn connected_msg_post_recv<CQ: ReadCq, E: MsgDefaultCap>(
//     op: RecvOp,
//     rx_seq: &mut u64,
//     rx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     _remote_address: &Option<MappedAddress>,
//     rx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectedEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let desc = if let Some(mr) = mr {
//         Some(mr.descriptor())
//     } else {
//         None
//     };

//     match op {
//         RecvOp::MsgRecv => {
//             todo!()
//         }
//         RecvOp::Recv => {
//             ft_post!(
//                 recv_with_context,
//                 ft_progress,
//                 rx_cq,
//                 *rx_seq,
//                 rx_cq_cntr,
//                 "receive",
//                 ep,
//                 base,
//                 desc,
//                 ctx
//             );
//         }
//     }
// }

// pub fn connless_msg_post_recv<CQ: ReadCq, E: MsgDefaultCap>(
//     op: RecvOp,
//     rx_seq: &mut u64,
//     rx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     rx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectionlessEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let desc = if let Some(mr) = mr {
//         Some(mr.descriptor())
//     } else {
//         None
//     };

//     match op {
//         RecvOp::MsgRecv => {
//             todo!()
//         }
//         RecvOp::Recv => {
//             if let Some(fi_address) = remote_address.as_ref() {
//                 ft_post!(
//                     recv_from_with_context,
//                     ft_progress,
//                     rx_cq,
//                     *rx_seq,
//                     rx_cq_cntr,
//                     "receive",
//                     ep,
//                     base,
//                     desc,
//                     fi_address,
//                     ctx
//                 );
//             } else {
//                 ft_post!(
//                     recv_from_any_with_context,
//                     ft_progress,
//                     rx_cq,
//                     *rx_seq,
//                     rx_cq_cntr,
//                     "receive",
//                     ep,
//                     base,
//                     desc,
//                     ctx
//                 );
//             }
//         }
//     }
// }

// pub fn connected_tagged_post_inject<CQ: ReadCq>(
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     _ctx: &mut Context,
//     _remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     tx_cq: &CompletionQueue<CQ>,
//     ep: &impl ConnectedAsyncTagSendEp,
//     _data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let tag = ft_tag;

//     ft_post!(
//         tinject,
//         ft_progress,
//         tx_cq,
//         *tx_seq,
//         tx_cq_cntr,
//         "inject",
//         ep,
//         base,
//         tag
//     );
// }

// pub fn connless_tagged_post_inject<CQ: ReadCq>(
//     tx_seq: &mut u64,
//     tx_cq_cntr: &mut u64,
//     _ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     tx_cq: &CompletionQueue<CQ>,
//     ep: &impl AsyncTagSendEp,
//     _data_desc: &mut Option<libfabric::mr::MemoryRegionDesc>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let tag = ft_tag;
//     let fi_address = remote_address.as_ref().unwrap();
//     ft_post!(
//         tinject_to,
//         ft_progress,
//         tx_cq,
//         *tx_seq,
//         tx_cq_cntr,
//         "inject",
//         ep,
//         base,
//         fi_address,
//         tag
//     );
// }

// pub async fn connless_tagged_post<CQ: ReadCq, E: TagDefaultCap>(
//     op: TagSendOp,
//     _tx_seq: &mut u64,
//     _tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     _tx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectionlessEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     let op_tag = ft_tag;
//     let flag = libfabric::enums::TaggedSendMsgOptions::new().transmit_complete();
//     let mr_desc = if let Some(mr) = mr {
//         Some(mr.descriptor())
//     } else {
//         None
//     };

//     match op {
//         TagSendOp::TagMsgSend => {
//             let iov = libfabric::iovec::IoVec::from_slice(base);
//             let fi_address = remote_address.as_ref().unwrap();
//             let mut msg = libfabric::msg::MsgTagged::from_iov(
//                 &iov,
//                 mr_desc.as_ref(),
//                 fi_address,
//                 None,
//                 op_tag,
//                 None,
//                 ctx,
//             );
//             let msg_ref = &mut msg;
//             ep.tsendmsg_to_async(msg_ref, flag).await.unwrap();
//         }
//         TagSendOp::TagSend => {
//             if let Some(fi_address) = remote_address {
//                 if data != NO_CQ_DATA {
//                     ft_post_async!(
//                         tsend_to_async,
//                         "transmit",
//                         ep,
//                         base,
//                         mr_desc,
//                         fi_address,
//                         op_tag,
//                         ctx
//                     );
//                 } else {
//                     ep.tsend_to_async(base, mr_desc, fi_address, op_tag, ctx)
//                         .await
//                         .unwrap();
//                 }
//             }
//         }
//     }
// }

// pub async fn connected_tagged_post<CQ: ReadCq, E: TagDefaultCap>(
//     op: TagSendOp,
//     _tx_seq: &mut u64,
//     _tx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     _remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     _tx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectedEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     data: u64,
// ) {
//     let op_tag = ft_tag;
//     let flag = libfabric::enums::TaggedSendMsgOptions::new().transmit_complete();
//     let mr_desc = if let Some(mr) = mr {
//         Some(mr.descriptor())
//     } else {
//         None
//     };

//     match op {
//         TagSendOp::TagMsgSend => {
//             let iov = libfabric::iovec::IoVec::from_slice(base);
//             let mut msg = libfabric::msg::MsgTaggedConnected::from_iov(
//                 &iov,
//                 mr_desc.as_ref(),
//                 None,
//                 op_tag,
//                 None,
//                 ctx,
//             );
//             let msg_ref = &mut msg;
//             ep.tsendmsg_async(msg_ref, flag).await.unwrap();
//         }
//         TagSendOp::TagSend => {
//             if data != NO_CQ_DATA {
//                 ep.tsenddata_async(base, mr_desc, data, op_tag, ctx)
//                     .await
//                     .unwrap();
//             } else {
//                 ep.tsenddata_async(base, mr_desc, data, op_tag, ctx)
//                     .await
//                     .unwrap();
//             }
//         }
//     }
// }

// pub fn connected_tagged_post_recv<CQ: ReadCq, E: TagDefaultCap>(
//     op: TagRecvOp,
//     rx_seq: &mut u64,
//     rx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     _remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     rx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectedEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let desc = if let Some(mr) = mr {
//         Some(mr.descriptor())
//     } else {
//         None
//     };

//     match op {
//         TagRecvOp::TagMsgRecv => {
//             todo!()
//         }
//         TagRecvOp::TagRecv => {
//             // let op_tag = if ft_tag != 0 {ft_tag} else {*rx_seq};
//             let op_tag = ft_tag;
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
// pub fn connless_tagged_post_recv<CQ: ReadCq, E: TagDefaultCap>(
//     op: TagRecvOp,
//     rx_seq: &mut u64,
//     rx_cq_cntr: &mut u64,
//     ctx: &mut Context,
//     remote_address: &Option<MappedAddress>,
//     ft_tag: u64,
//     rx_cq: &CompletionQueue<CQ>,
//     ep: &ConnectionlessEndpoint<E>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     base: &mut [u8],
//     _data: u64,
// ) {
//     let desc = if let Some(mr) = mr {
//         Some(mr.descriptor())
//     } else {
//         None
//     };

//     match op {
//         TagRecvOp::TagMsgRecv => {
//             todo!()
//         }
//         TagRecvOp::TagRecv => {
//             // let op_tag = if ft_tag != 0 {ft_tag} else {*rx_seq};
//             let op_tag = ft_tag;
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
// pub async fn ft_post_tx<CQ: ReadCq, M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     size: usize,
//     data: u64,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     tx_cq: &CompletionQueue<CQ>,
// ) {
//     // size += ft_tx_prefix_size(info);
//     let fi_addr = &gl_ctx.remote_address;
//     let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + size];
//     match ep {
//         EndpointCaps::ConnectedMsg(ep) => {
//             connected_msg_post(
//                 SendOp::Send,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 tx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 data,
//             )
//             .await;
//         }
//         EndpointCaps::ConnlessMsg(ep) => {
//             connless_msg_post(
//                 SendOp::Send,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 tx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 data,
//             )
//             .await;
//         }
//         EndpointCaps::ConnectedTagged(ep) => {
//             connected_tagged_post(
//                 TagSendOp::TagSend,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 gl_ctx.ft_tag,
//                 tx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 data,
//             )
//             .await;
//         }
//         EndpointCaps::ConnlessTagged(ep) => {
//             connless_tagged_post(
//                 TagSendOp::TagSend,
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 gl_ctx.ft_tag,
//                 tx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 data,
//             )
//             .await;
//         }
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub async fn ft_tx<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     size: usize,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     cq_type: &CqType,
//     _tx_cntr: &Option<Counter<CNTR>>,
// ) {
//     match cq_type {
//         // CqType::Spin(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq).await},
//         // CqType::Sread(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq).await},
//         // CqType::WaitSet(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq).await},
//         CqType::WaitFd(cq_type) => {
//             ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, mr, cq_type.tx_cq()).await
//         } // CqType::WaitYield(tx_cq) => {ft_post_tx(gl_ctx, ep, size, NO_CQ_DATA, data_desc, tx_cq).await},
//     }

//     // ft_get_tx_comp(gl_ctx, tx_cntr, tx_cq, gl_ctx.tx_seq);
// }

// #[allow(clippy::too_many_arguments)]
// pub fn ft_post_rx<CQ: ReadCq, M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     mut size: usize,
//     _data: u64,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     rx_cq: &CompletionQueue<CQ>,
// ) {
//     size = std::cmp::max(size, FT_MAX_CTRL_MSG); //+  ft_tx_prefix_size(info);
//     let buf = &mut gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + size];

//     match ep {
//         EndpointCaps::ConnectedMsg(ep) => {
//             connected_msg_post_recv(
//                 RecvOp::Recv,
//                 &mut gl_ctx.rx_seq,
//                 &mut gl_ctx.rx_cq_cntr,
//                 &mut gl_ctx.rx_ctx.as_mut().unwrap(),
//                 &gl_ctx.remote_address,
//                 rx_cq,
//                 ep,
//                 mr,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnlessMsg(ep) => {
//             connless_msg_post_recv(
//                 RecvOp::Recv,
//                 &mut gl_ctx.rx_seq,
//                 &mut gl_ctx.rx_cq_cntr,
//                 &mut gl_ctx.rx_ctx.as_mut().unwrap(),
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
//                 &mut gl_ctx.rx_ctx.as_mut().unwrap(),
//                 &gl_ctx.remote_address,
//                 gl_ctx.ft_tag,
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
//                 &mut gl_ctx.rx_ctx.as_mut().unwrap(),
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
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     cq_type: &CqType,
//     rx_cntr: &Option<libfabric::cntr::Counter<CNTR>>,
// ) {
//     ft_get_rx_comp(gl_ctx, rx_cntr, cq_type, gl_ctx.rx_seq);
//     match cq_type {
//         // CqType::Spin(cq_type) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, cq_type.rx_cq()),
//         // CqType::Sread(cq_type) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, cq_type.rx_cq()),
//         // CqType::WaitSet(cq_type) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, cq_type.rx_cq()),
//         CqType::WaitFd(cq_type) => {
//             ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, cq_type.rx_cq())
//         } // CqType::WaitYield(cq_type) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, data_desc, cq_type.rx_cq()),
//     }
// }

// pub fn ft_post_inject<CQ: ReadCq, M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     size: usize,
//     tx_cq: &CompletionQueue<CQ>,
// ) {
//     // size += ft_tx_prefix_size(info);
//     let buf = &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + size];
//     let fi_addr = &gl_ctx.remote_address;

//     match ep {
//         EndpointCaps::ConnectedMsg(ep) => {
//             connected_msg_post_inject(
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 tx_cq,
//                 ep,
//                 &mut None,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnlessMsg(ep) => {
//             connless_msg_post_inject(
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 tx_cq,
//                 ep,
//                 &mut None,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnectedTagged(ep) => {
//             connected_tagged_post_inject(
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 gl_ctx.ft_tag,
//                 tx_cq,
//                 ep,
//                 &mut None,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//         EndpointCaps::ConnlessTagged(ep) => {
//             connless_tagged_post_inject(
//                 &mut gl_ctx.tx_seq,
//                 &mut gl_ctx.tx_cq_cntr,
//                 &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                 fi_addr,
//                 gl_ctx.ft_tag,
//                 tx_cq,
//                 ep,
//                 &mut None,
//                 buf,
//                 NO_CQ_DATA,
//             );
//         }
//     }
//     // gl_ctx.tx_cq_cntr += 1;
// }

// pub fn ft_inject<M: MsgDefaultCap, T: TagDefaultCap>(
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     size: usize,
//     cq_type: &CqType,
// ) {
//     match cq_type {
//         // CqType::Spin(cq_type) => {ft_post_inject(gl_ctx, ep, size, cq_type.tx_cq());},
//         // CqType::Sread(cq_type) => {ft_post_inject(gl_ctx, ep, size, cq_type.tx_cq());},
//         // CqType::WaitSet(cq_type) => {ft_post_inject(gl_ctx, ep, size, cq_type.tx_cq());},
//         CqType::WaitFd(cq_type) => {
//             ft_post_inject(gl_ctx, ep, size, cq_type.tx_cq());
//         } // CqType::WaitYield(cq_type) => {ft_post_inject(gl_ctx, ep, size, cq_type.tx_cq());},
//     }
// }

// pub fn ft_progress<CQ: ReadCq>(cq: &CompletionQueue<CQ>, _total: u64, cq_cntr: &mut u64) {
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
// pub async fn ft_init_av_dst_addr<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     av: &AddressVector,
//     ep: &EndpointCaps<M, T>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     server: bool,
// ) {
//     if !server {
//         gl_ctx.remote_address =
//             Some(ft_av_insert(info, av, &info.dest_addr().unwrap(), AVOptions::new()).await);
//         let epname = match ep {
//             EndpointCaps::ConnectedMsg(ep) => ep.getname().unwrap(),
//             EndpointCaps::ConnlessMsg(ep) => ep.getname().unwrap(),
//             EndpointCaps::ConnectedTagged(ep) => ep.getname().unwrap(),
//             EndpointCaps::ConnlessTagged(ep) => ep.getname().unwrap(),
//         };
//         let epname_bytes = epname.as_bytes();
//         let len = epname_bytes.len();
//         gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + len].copy_from_slice(epname_bytes);
//         ft_tx(gl_ctx, ep, len, mr, cq_type, tx_cntr).await;
//         ft_rx(gl_ctx, ep, 1, mr, cq_type, rx_cntr);
//     } else {
//         let mut v = [0_u8; FT_MAX_CTRL_MSG];

//         ft_get_rx_comp(gl_ctx, rx_cntr, cq_type, gl_ctx.rx_seq);
//         v.copy_from_slice(&gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + FT_MAX_CTRL_MSG]);
//         let address = unsafe { Address::from_bytes(&v) };

//         gl_ctx.remote_address = Some(ft_av_insert(info, av, &address, AVOptions::new()).await);
//         // if matches!(info.domain_attr().get_av_type()(), libfabric::enums::AddressVectorType::Table ) {
//         //     let mut zero = 0;
//         //     ft_av_insert(av, &v, &mut zero, 0);
//         // }
//         // else {
//         //     ft_av_insert(av, &v, &mut gl_ctx.remote_address, 0);
//         // }

//         match cq_type {
//             // CqType::Spin(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
//             // CqType::Sread(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
//             // CqType::WaitSet(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
//             CqType::WaitFd(cq_type) => ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA, mr, cq_type.rx_cq()), // CqType::WaitYield(rx_cq) => {ft_post_rx(gl_ctx, ep, 1, NO_CQ_DATA,  data_desc, rx_cq)},
//         }

//         // if matches!(info.domain_attr().get_av_type()(), libfabric::enums::AddressVectorType::Table) {
//         //     gl_ctx.remote_address = 0;
//         // }

//         ft_tx(gl_ctx, ep, 1, mr, cq_type, tx_cntr).await;
//     }
// }

// #[allow(clippy::too_many_arguments)]
// pub async fn ft_init_av<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     av: &AddressVector,
//     ep: &EndpointCaps<M, T>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     server: bool,
// ) {
//     ft_init_av_dst_addr(info, gl_ctx, av, ep, cq_type, tx_cntr, rx_cntr, mr, server).await;
// }

// pub fn ft_spin_for_comp<CQ: ReadCq>(
//     cq: &CompletionQueue<CQ>,
//     curr: &mut u64,
//     total: u64,
//     _timeout: i32,
//     _tag: u64,
// ) {
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

// pub fn ft_wait_for_comp<CQ: ReadCq + WaitCq>(
//     cq: &CompletionQueue<CQ>,
//     curr: &mut u64,
//     total: u64,
//     _timeout: i32,
//     _tag: u64,
// ) {
//     while total - *curr > 0 {
//         let ret = cq.sread(1, -1);
//         if ret.is_ok() {
//             *curr += 1;
//         }
//     }
// }

// // pub fn ft_read_cq(cq: &CqType, curr: &mut u64, total: u64, timeout: i32, tag: u64) {

// //     match cq {
// //         // CqType::Spin(cq) => ft_spin_for_comp(cq, curr, total, timeout, tag),
// //         // CqType::Sread(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
// //         // CqType::WaitSet(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
// //         CqType::WaitFd(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
// //         // CqType::WaitYield(cq) => ft_wait_for_comp(cq, curr, total, timeout, tag),
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

// pub fn ft_wait_for_cntr<CNTR: WaitCntr>(cntr: &Counter<CNTR>, total: u64) {
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

// pub fn ft_get_cntr_comp<CNTR: WaitCntr>(cntr: &Option<Counter<CNTR>>, total: u64) {
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

// pub fn ft_get_rx_comp<CNTR: WaitCntr>(
//     gl_ctx: &mut TestsGlobalCtx,
//     rx_cntr: &Option<Counter<CNTR>>,
//     cq_type: &CqType,
//     total: u64,
// ) {
//     if gl_ctx.options & FT_OPT_RX_CQ != 0 {
//         match cq_type {
//             CqType::WaitFd(cq_type) => {
//                 ft_wait_for_comp(cq_type.rx_cq(), &mut gl_ctx.rx_cq_cntr, total, -1, 0)
//             }
//         }
//     } else {
//         ft_get_cntr_comp(rx_cntr, total);
//     }
// }

// pub fn ft_get_tx_comp<CNTR: WaitCntr>(
//     gl_ctx: &mut TestsGlobalCtx,
//     tx_cntr: &Option<Counter<CNTR>>,
//     cq_type: &CqType,
//     total: u64,
// ) {
//     if gl_ctx.options & FT_OPT_TX_CQ != 0 {
//         match cq_type {
//             CqType::WaitFd(cq_type) => {
//                 ft_wait_for_comp(cq_type.tx_cq(), &mut gl_ctx.tx_cq_cntr, total, -1, 0)
//             }
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
//     _domain: &'a DomainBase<NoEventQueue>,
//     buff: &'b [u8],
//     info: &InfoEntry<E>,
// ) -> MemoryRegionBuilder<'a> {
//     let mut mr_builder = MemoryRegionBuilder::new(buff, libfabric::enums::HmemIface::System);

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
//     domain: &DomainBase<NoEventQueue>,
//     ep: &Endpoint<E>,
//     buf: &mut [u8],
//     key: u64,
// ) -> Option<MemoryRegion> {
//     if !ft_need_mr_reg(info) {
//         println!("MR not needed");
//         return None;
//     }
//     // let iov = libfabric::iovec::IoVec::from_slice(buf);
//     // let mut mr_attr = libfabric::mr::MemoryRegionAttr::new().iov(std::slice::from_ref(&iov)).requested_key(key).iface(libfabric::enums::HmemIface::System);

//     let mr = ft_info_to_mr_builder(domain, buf, info)
//         .requested_key(key)
//         .build(domain)
//         .unwrap();
//     // let (_event, mr) = task::block_on(async {mr_buidler.build_async().await}).unwrap();

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

//     if info.domain_attr().mr_mode().is_endpoint() {
//         todo!();
//         // mr.bind_ep(ep).unwrap();
//     }

//     mr.into()
// }

// #[allow(clippy::too_many_arguments)]
// pub async fn ft_sync<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
//     ep: &EndpointCaps<M, T>,
//     gl_ctx: &mut TestsGlobalCtx,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
// ) {
//     // println!("TX SEQ: {},  TX_CTR: {}", gl_ctx.tx_seq, gl_ctx.tx_cq_cntr);
//     // println!("RX SEQ: {},  RX_CTR: {}", gl_ctx.rx_seq, gl_ctx.rx_cq_cntr);
//     ft_tx(gl_ctx, ep, 1, mr, cq_type, tx_cntr).await;
//     ft_rx(gl_ctx, ep, 1, mr, cq_type, rx_cntr);
// }

// #[allow(clippy::too_many_arguments)]
// pub async fn ft_exchange_keys<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     // mr: &mut MemoryRegion,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     domain: &DomainBase<NoEventQueue>,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
// ) -> RemoteMemAddressInfo {
//     // let mut addr ;
//     // let mut key_size = 0;
//     // let mut rma_iov = libfabric::iovec::RmaIoVec::new();

//     // if info.domain_attr().mr_mode().is_raw() {
//     //     addr = mr.address( 0).unwrap(); // [TODO] Change this to return base_addr, key_size
//     // }

//     // let len = std::mem::size_of::<libfabric::iovec::RmaIoVec>();
//     // // if key_size >= len - std::mem::size_of_val(&rma_iov.get_key()) {
//     // //     panic!("Key size does not fit");
//     // // }

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
//     // let mut key = 0u64;
//     // unsafe {
//     //     std::slice::from_raw_parts_mut(&mut key as *mut u64 as *mut u8, 8)
//     //         .copy_from_slice(&key_bytes)
//     // };

//     let mem_info = MemAddressInfo::from_slice(
//         &gl_ctx.buf[..],
//         gl_ctx.rx_buf_index,
//         &mr.as_ref().unwrap().key().unwrap(),
//         info,
//     );
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
//     )
//     .await;
//     ft_get_rx_comp(gl_ctx, rx_cntr, cq_type, gl_ctx.rx_seq);

//     let mem_info = unsafe {
//         MemAddressInfo::from_bytes(
//             &gl_ctx.buf[gl_ctx.rx_buf_index..gl_ctx.rx_buf_index + mem_info_bytes.len()],
//         )
//     };

//     let peer_info = { mem_info.into_remote_info(domain).unwrap() };

//     match cq_type {
//         // CqType::Spin(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq),
//         // CqType::Sread(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq),
//         // CqType::WaitSet(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq),
//         CqType::WaitFd(cq_type) => {
//             ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, cq_type.rx_cq())
//         } // CqType::WaitYield(rx_cq) => ft_post_rx(gl_ctx, ep, gl_ctx.rx_size, NO_CQ_DATA, mr, rx_cq),
//     }

//     ft_sync(ep, gl_ctx, cq_type, tx_cntr, rx_cntr, mr).await;
//     peer_info
// }

// pub fn start_server<M: MsgDefaultCap, T: TagDefaultCap>(
//     hints: HintsCaps<M, T>,
//     node: String,
//     service: String,
// ) -> (
//     InfoWithCaps<M, T>,
//     fabric::Fabric,
//     EventQueue<EventQueueOptions>,
//     PassiveEndpointCaps<M, T>,
// ) {
//     match hints {
//         HintsCaps::Msg(hints) => {
//             let info = ft_getinfo(hints, node, service, true, true);
//             let entry = info.into_iter().next().unwrap();

//             let fab = libfabric::fabric::FabricBuilder::new()
//                 .build(&entry)
//                 .unwrap();

//             let eq = EventQueueBuilder::new(&fab).write().build().unwrap();

//             let pep = EndpointBuilder::new(&entry).build_passive(&fab).unwrap();

//             pep.bind(&eq, 0).unwrap();
//             // pep.listen().unwrap();

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

//             let eq = EventQueueBuilder::new(&fab).write().build().unwrap();

//             let pep = EndpointBuilder::new(&entry).build_passive(&fab).unwrap();

//             pep.bind(&eq, 0).unwrap();
//             // pep.listen().unwrap();

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
// pub async fn ft_client_connect<M: MsgDefaultCap + 'static, T: TagDefaultCap + 'static>(
//     hints: HintsCaps<M, T>,
//     gl_ctx: &mut TestsGlobalCtx,
//     node: String,
//     service: String,
// ) -> (
//     InfoWithCaps<M, T>,
//     CqType,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     Option<libfabric::cntr::Counter<CounterOptions>>,
//     EndpointCaps<M, T>,
//     Option<MemoryRegion>,
// ) {
//     match hints {
//         HintsCaps::Msg(hints) => {
//             let info = ft_getinfo(hints, node, service, true, false);

//             let entry = info.into_iter().next().unwrap();
//             gl_ctx.tx_ctx = Some(entry.allocate_context());
//             gl_ctx.rx_ctx = Some(entry.allocate_context());

//             let (_fab, eq, domain) = ft_open_fabric_res(&entry);
//             let (cq_type, tx_cntr, rx_cntr, rma_cntr, ep, _) =
//                 ft_alloc_active_res(&entry, gl_ctx, &domain, &eq);

//             let mr = ft_enable_ep_recv(&entry, gl_ctx, &ep, &domain, &tx_cntr, &rx_cntr, &rma_cntr);

//             let ep = match ep {
//                 Endpoint::ConnectionOriented(ep) => ep.enable(&eq).unwrap(),
//                 _ => panic!("Unexpected Endpoint Type"),
//             };

//             let pending_ep = match ep {
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(ep) => ep,
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(_ep) => {
//                     panic!("This should be a client")
//                 }
//             };

//             let ep = ft_connect_ep(pending_ep, &eq, &entry.dest_addr().as_ref().unwrap()).await;

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
//             let (cq_type, tx_cntr, rx_cntr, rma_cntr, ep, _) =
//                 ft_alloc_active_res(&entry, gl_ctx, &domain, &eq);

//             let mr = ft_enable_ep_recv(&entry, gl_ctx, &ep, &domain, &tx_cntr, &rx_cntr, &rma_cntr);
//             let ep = match ep {
//                 Endpoint::ConnectionOriented(ep) => ep.enable(&eq).unwrap(),
//                 _ => panic!("Unexpected Endpoint Type"),
//             };

//             let pending_ep = match ep {
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::Unconnected(ep) => ep,
//                 libfabric::conn_ep::EnabledConnectionOrientedEndpoint::AcceptPending(_) => {
//                     panic!("This should be a client")
//                 }
//             };

//             let ep = ft_connect_ep(pending_ep, &eq, &entry.dest_addr().as_ref().unwrap()).await;
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

//     // (info, fab, domain, eq, cq_type, tx_cntr, rx_cntr, ep, mr, mr_desc)
// }

// #[allow(clippy::too_many_arguments)]
// pub async fn ft_finalize_ep<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
//     cq_type: &CqType,
//     _tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
// ) {
//     println!("Finalizing {}", gl_ctx.rx_seq);
//     let base =
//         &mut gl_ctx.buf[gl_ctx.tx_buf_index..gl_ctx.tx_buf_index + 4 + ft_tx_prefix_size(info)];

//     match ep {
//         EndpointCaps::ConnectedMsg(ep) => {
//             match cq_type {
//                 // CqType::Spin(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, data_desc, base, NO_CQ_DATA).await},
//                 // CqType::Sread(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, data_desc, base, NO_CQ_DATA).await},
//                 // CqType::WaitSet(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, data_desc, base, NO_CQ_DATA).await},
//                 CqType::WaitFd(cq_type) => {
//                     connected_msg_post(
//                         SendOp::MsgSend,
//                         &mut gl_ctx.tx_seq,
//                         &mut gl_ctx.tx_cq_cntr,
//                         &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                         &gl_ctx.remote_address,
//                         cq_type.tx_cq(),
//                         ep,
//                         mr,
//                         base,
//                         NO_CQ_DATA,
//                     )
//                     .await
//                 } // CqType::WaitYield(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//             }
//         }
//         EndpointCaps::ConnlessMsg(ep) => {
//             match cq_type {
//                 // CqType::Spin(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 // CqType::Sread(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 // CqType::WaitSet(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx, &gl_ctx.remote_address, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 CqType::WaitFd(cq_type) => {
//                     connless_msg_post(
//                         SendOp::MsgSend,
//                         &mut gl_ctx.tx_seq,
//                         &mut gl_ctx.tx_cq_cntr,
//                         &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                         &gl_ctx.remote_address,
//                         cq_type.tx_cq(),
//                         ep,
//                         mr,
//                         base,
//                         NO_CQ_DATA,
//                     )
//                     .await
//                 } // CqType::WaitYield(tx_cq) => {msg_post(SendOp::MsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//             }
//         }
//         EndpointCaps::ConnectedTagged(ep) => {
//             match cq_type {
//                 // CqType::Spin(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 // CqType::Sread(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 // CqType::WaitSet(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 CqType::WaitFd(cq_type) => {
//                     connected_tagged_post(
//                         TagSendOp::TagMsgSend,
//                         &mut gl_ctx.tx_seq,
//                         &mut gl_ctx.tx_cq_cntr,
//                         &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                         &gl_ctx.remote_address,
//                         gl_ctx.ft_tag,
//                         cq_type.tx_cq(),
//                         ep,
//                         mr,
//                         base,
//                         NO_CQ_DATA,
//                     )
//                     .await
//                 } // CqType::WaitYield(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//             }
//         }
//         EndpointCaps::ConnlessTagged(ep) => {
//             match cq_type {
//                 // CqType::Spin(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 // CqType::Sread(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 // CqType::WaitSet(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, mr, base, NO_CQ_DATA).await},
//                 CqType::WaitFd(cq_type) => {
//                     connless_tagged_post(
//                         TagSendOp::TagMsgSend,
//                         &mut gl_ctx.tx_seq,
//                         &mut gl_ctx.tx_cq_cntr,
//                         &mut gl_ctx.tx_ctx.as_mut().unwrap(),
//                         &gl_ctx.remote_address,
//                         gl_ctx.ft_tag,
//                         cq_type.tx_cq(),
//                         ep,
//                         mr,
//                         base,
//                         NO_CQ_DATA,
//                     )
//                     .await
//                 } // CqType::WaitYield(tx_cq) => {tagged_post(TagSendOp::TagMsgSend, &mut gl_ctx.tx_seq, &mut gl_ctx.tx_cq_cntr, &mut gl_ctx.tx_ctx.as_mut().unwrap(), &gl_ctx.remote_address, gl_ctx.ft_tag, tx_cq, ep, data_desc, base, NO_CQ_DATA).await},
//             }
//         }
//     }

//     // ft_get_tx_comp(gl_ctx, tx_cntr, tx_cq, gl_ctx.tx_seq);
//     ft_get_rx_comp(gl_ctx, rx_cntr, cq_type, gl_ctx.rx_seq);
//     println!("Done Finalizing");
// }

// #[allow(clippy::too_many_arguments)]
// pub async fn ft_finalize<CNTR: WaitCntr, E, M: MsgDefaultCap, T: TagDefaultCap>(
//     info: &InfoEntry<E>,
//     gl_ctx: &mut TestsGlobalCtx,
//     ep: &EndpointCaps<M, T>,
//     cq_type: &CqType,
//     tx_cntr: &Option<Counter<CNTR>>,
//     rx_cntr: &Option<Counter<CNTR>>,
//     mr: &Option<libfabric::mr::MemoryRegion>,
// ) {
//     ft_finalize_ep(info, gl_ctx, ep, mr, cq_type, tx_cntr, rx_cntr).await;
// }

// #[allow(clippy::too_many_arguments)]
// pub async fn pingpong<CNTR: WaitCntr, M: MsgDefaultCap, T: TagDefaultCap>(
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
//     ft_sync(ep, gl_ctx, cq_type, tx_cntr, rx_cntr, mr).await;

//     let mut now = Instant::now();
//     if !server {
//         for i in 0..warmup + iters {
//             if i == warmup {
//                 now = Instant::now(); // Start timer
//             }
//             if size < inject_size {
//                 ft_inject(gl_ctx, ep, size, cq_type);
//             } else {
//                 ft_tx(gl_ctx, ep, size, mr, cq_type, tx_cntr).await;
//             }

//             ft_rx(gl_ctx, ep, size, mr, cq_type, rx_cntr);
//         }
//     } else {
//         for i in 0..warmup + iters {
//             if i == warmup {
//                 now = Instant::now(); // Start timer
//             }

//             ft_rx(gl_ctx, ep, size, mr, cq_type, rx_cntr);

//             if size < inject_size {
//                 ft_inject(gl_ctx, ep, size, cq_type); // Should return immediately
//             } else {
//                 ft_tx(gl_ctx, ep, size, mr, cq_type, tx_cntr).await;
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
// pub async fn pingpong_rma<
//     CNTR: WaitCntr,
//     E,
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

//     ft_sync(ep, gl_ctx, cq_type, tx_cntr, rx_cntr, mr).await;
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
//                             // CqType::Spin(cq_type) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, cq_type.tx_cq()),
//                             // CqType::Sread(cq_type) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, cq_type.tx_cq()),
//                             // CqType::WaitSet(cq_type) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, cq_type.tx_cq()),
//                             CqType::WaitFd(cq_type) => ft_post_rma_inject(
//                                 gl_ctx,
//                                 &op,
//                                 offset,
//                                 size,
//                                 remote,
//                                 ep,
//                                 cq_type.tx_cq(),
//                             ),
//                             // CqType::WaitYield(cq_type) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, cq_type.tx_cq()),
//                         };
//                     } else {
//                         match cq_type {
//                             // CqType::Spin(cq_type) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), cq_type),
//                             // CqType::Sread(cq_type) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), cq_type),
//                             // CqType::WaitSet(cq_type) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), cq_type),
//                             CqType::WaitFd(cq_type) => ft_post_rma(
//                                 gl_ctx,
//                                 &op,
//                                 offset,
//                                 size,
//                                 remote,
//                                 ep,
//                                 mr,
//                                 cq_type.tx_cq(),
//                             ),
//                             // CqType::WaitYield(cq_type) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), cq_type),
//                         }
//                         .await;
//                     }
//                 }
//                 EndpointCaps::ConnlessTagged(ep) => {
//                     if size < inject_size {
//                         match cq_type {
//                             // CqType::Spin(cq_type) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, cq_type.tx_cq()),
//                             // CqType::Sread(cq_type) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, cq_type.tx_cq()),
//                             // CqType::WaitSet(cq_type) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, cq_type.tx_cq()),
//                             CqType::WaitFd(cq_type) => ft_post_rma_inject(
//                                 gl_ctx,
//                                 &op,
//                                 offset,
//                                 size,
//                                 remote,
//                                 ep,
//                                 cq_type.tx_cq(),
//                             ),
//                             // CqType::WaitYield(cq_type) => ft_post_rma_inject(gl_ctx, &op, offset, size, remote, ep, cq_type.tx_cq()),
//                         };
//                     } else {
//                         match cq_type {
//                             // CqType::Spin(cq_type) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), cq_type),
//                             // CqType::Sread(cq_type) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), cq_type),
//                             // CqType::WaitSet(cq_type) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), cq_type),
//                             CqType::WaitFd(cq_type) => ft_post_rma(
//                                 gl_ctx,
//                                 &op,
//                                 offset,
//                                 size,
//                                 remote,
//                                 ep,
//                                 mr,
//                                 cq_type.tx_cq(),
//                             ),
//                             // CqType::WaitYield(cq_type) => ft_post_rma(gl_ctx, &op, offset, size, remote, ep, mr_desc.as_mut().unwrap(), cq_type),
//                         }
//                         .await;
//                     }
//                 }
//                 _ => panic!("Connected Endpoints not handled for RMA"),
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

//         #[cfg(feature= "use-async-std")]
//         #[test]
//         #[ignore]
//         fn $func_name() {
//             async_std::task::block_on(async {$async_fname().await});
//         }

//         #[cfg(feature= "use-tokio")]
//         #[tokio::test]
//         #[ignore]
//         async fn $func_name() {
//             $async_fname().await;
//         }

//         async fn $async_fname() $body
//     };
// }

// #[allow(unused_imports)]
// pub(crate) use define_test;

// #[allow(unused_macros)]
// macro_rules! call {
//     ($func_name:path, $( $x:expr),* ) => {
//         $func_name($($x,)*).await
//     }
// }

// #[allow(unused_imports)]
// pub(crate) use call;


use std::cell::RefCell;

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
use libfabric::mr::{EpBindingMemoryRegion, MemoryRegionBuilder, MemoryRegionDesc, MemoryRegionKey};
use libfabric::enums::AVOptions;
use libfabric::ep::{Address, BaseEndpoint};
use libfabric::fabric::FabricBuilder;
use libfabric::mr::MemoryRegion;
use libfabric::info::{Info, InfoEntry};
use libfabric::async_::cq::{CompletionQueue, CompletionQueueBuilder};
use libfabric::msg::{Msg, MsgAtomic, MsgAtomicConnected, MsgCompareAtomic, MsgCompareAtomicConnected, MsgConnected, MsgConnectedMut, MsgFetchAtomic, MsgFetchAtomicConnected, MsgMut, MsgRma, MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut, MsgTagged, MsgTaggedConnected, MsgTaggedConnectedMut, MsgTaggedMut};
use libfabric::{AsFiType, Context, EqCaps, MappedAddress, MemAddressInfo, MyRc, RemoteMemAddrSlice, RemoteMemAddrSliceMut, RemoteMemAddressInfo};


pub type SpinCq = libfabric::async_cq_caps_type!(CqCaps::FD);
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


impl<I: MsgDefaultCap + 'static> Ofi<I> {
    pub fn send<T: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc>,
        data: Option<u64>,
        ctx: &mut Context,
    ) {
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



pub fn gen_info<I: Caps + MsgDefaultCap + 'static>(ep_type: EndpointType, caps: I, shared_cq: bool, server: bool, ip: &str, name: &str) -> Ofi<I> {
    Ofi::new(
            {
                let info = Info::new(&libfabric::info::libfabric_version())
                    .enter_hints()
                    .enter_ep_attr()
                    // .tx_ctx_cnt(1)
                    // .rx_ctx_cnt(1)
                    .type_(ep_type)
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
                if server {
                    info.source(libfabric::info::ServiceAddress::Service("9222".to_owned()))
                        .get()
                        .unwrap()
                        .into_iter()
                        .next()
                        .unwrap()
                } else {
                    info.node(ip.strip_suffix("\n").unwrap_or(&ip))
                        .service("9222")
                        .get()
                        .unwrap()
                        .into_iter()
                        .next()
                        .unwrap()
                }
            },
            shared_cq,
            server,
            name,
        )
        .unwrap()
}



pub fn handshake<I: Caps + MsgDefaultCap + 'static>(
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
    let ip = "172.17.110.".to_string() + &hostname;

    let info = gen_info(ep_type, caps, false, server, &ip, name);
    info
}


pub fn handshake_connectionless<I: MsgDefaultCap + Caps + 'static>(
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

    let info = gen_info(
        ep_type,
        caps,
        false,
        server,
        ip.strip_suffix("\n").unwrap_or(&ip),
        name
    );

    info
}

pub fn enable_ep_mr<E: 'static>(ep: &MyEndpoint<E>, mr: EpBindingMemoryRegion) -> MemoryRegion {
    match ep {
        MyEndpoint::Connected(ep) => mr.enable(ep).unwrap(),
        MyEndpoint::Connectionless(ep) => mr.enable(ep).unwrap(),
    }
}

