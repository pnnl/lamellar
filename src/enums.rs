use std::os::fd::BorrowedFd;

use libfabric_sys::{FI_RECV, FI_TRANSMIT};

pub enum Op {
    Min,
    Max,
    Sum,
    Prod,
    Lor,
    Land,
    Bor,
    Bar,
    Lxor,
    Bxor,
    AtomicRead,
    AtomicWrite,
    Cswap,
    CswapNe,
    CswapLe,
    CswapLt,
    CswapGe,
    CswapGt,
    Mswap,
    AtomicOpLast,
    Noop,
}

impl Op {
    pub(crate) fn get_value(&self) -> u32  {
        match self {
            Op::Min => libfabric_sys::fi_op_FI_MIN,
            Op::Max => libfabric_sys::fi_op_FI_MAX,
            Op::Sum => libfabric_sys::fi_op_FI_SUM,
            Op::Prod => libfabric_sys::fi_op_FI_PROD,
            Op::Lor => libfabric_sys::fi_op_FI_LOR,
            Op::Land => libfabric_sys::fi_op_FI_LAND,
            Op::Bor => libfabric_sys::fi_op_FI_BOR,
            Op::Bar => libfabric_sys::fi_op_FI_BAND,
            Op::Lxor => libfabric_sys::fi_op_FI_LXOR,
            Op::Bxor => libfabric_sys::fi_op_FI_BXOR,
            Op::AtomicRead => libfabric_sys::fi_op_FI_ATOMIC_READ,
            Op::AtomicWrite => libfabric_sys::fi_op_FI_ATOMIC_WRITE,
            Op::Cswap => libfabric_sys::fi_op_FI_CSWAP,
            Op::CswapNe => libfabric_sys::fi_op_FI_CSWAP_NE,
            Op::CswapLe => libfabric_sys::fi_op_FI_CSWAP_LE,
            Op::CswapLt => libfabric_sys::fi_op_FI_CSWAP_LT,
            Op::CswapGe => libfabric_sys::fi_op_FI_CSWAP_GE,
            Op::CswapGt => libfabric_sys::fi_op_FI_CSWAP_GT,
            Op::Mswap => libfabric_sys::fi_op_FI_MSWAP,
            Op::AtomicOpLast => libfabric_sys::fi_op_FI_ATOMIC_OP_LAST,
            Op::Noop  => libfabric_sys::fi_op_FI_NOOP,
        }
    }
}

pub enum CollectiveOp {
    Barrier,
    Broadcast,
    AllToAll,
    AllReduce,
    AllGather,
    ReduceScatter,
    Reduce,
    Scatter,
    Gather,
}

impl CollectiveOp {
    pub(crate) fn get_value(&self) -> u32 {
        match self {

            CollectiveOp::Barrier => libfabric_sys::fi_collective_op_FI_BARRIER,
            CollectiveOp::Broadcast => libfabric_sys::fi_collective_op_FI_BROADCAST,
            CollectiveOp::AllToAll => libfabric_sys::fi_collective_op_FI_ALLTOALL,
            CollectiveOp::AllReduce => libfabric_sys::fi_collective_op_FI_ALLREDUCE,
            CollectiveOp::AllGather => libfabric_sys::fi_collective_op_FI_ALLGATHER,
            CollectiveOp::ReduceScatter => libfabric_sys::fi_collective_op_FI_REDUCE_SCATTER,
            CollectiveOp::Reduce => libfabric_sys::fi_collective_op_FI_REDUCE,
            CollectiveOp::Scatter => libfabric_sys::fi_collective_op_FI_SCATTER,
            CollectiveOp::Gather => libfabric_sys::fi_collective_op_FI_GATHER,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CqFormat {
    UNSPEC,
    CONTEXT,
    MSG,
    DATA,
    TAGGED,
}

impl CqFormat {
    #[allow(dead_code)]
    pub(crate) fn from_value(value: libfabric_sys::fi_cq_format) -> Self {
        
        if value == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_UNSPEC {
            CqFormat::UNSPEC
        }
        else if value == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_CONTEXT {
            CqFormat::CONTEXT
        }
        else if value == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_MSG {
            CqFormat::MSG
        }
        else if value == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_DATA {
            CqFormat::DATA
        }
        else if value == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_TAGGED {
            CqFormat::TAGGED
        }
        else {
            CqFormat::UNSPEC
        }
    }
    
    pub(crate) fn get_value(&self) -> u32 {
        match self {   
            CqFormat::UNSPEC => libfabric_sys::fi_cq_format_FI_CQ_FORMAT_UNSPEC,
            CqFormat::CONTEXT => libfabric_sys::fi_cq_format_FI_CQ_FORMAT_CONTEXT,
            CqFormat::MSG => libfabric_sys::fi_cq_format_FI_CQ_FORMAT_MSG,
            CqFormat::DATA => libfabric_sys::fi_cq_format_FI_CQ_FORMAT_DATA,
            CqFormat::TAGGED => libfabric_sys::fi_cq_format_FI_CQ_FORMAT_TAGGED,
        }
    }
}

#[derive(Copy, Clone)]
pub enum WaitObj<'a> {
    None,
    Unspec,
    Set(&'a crate::sync::WaitSet),
    Fd,
    MutexCond,
    Yield,
    PollFd,
}

impl<'a> WaitObj<'a> {

    pub(crate) fn get_value(&self) -> u32 {
        match self {
            WaitObj::None => libfabric_sys::fi_wait_obj_FI_WAIT_NONE,
            WaitObj::Unspec => libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
            WaitObj::Set(_) => libfabric_sys::fi_wait_obj_FI_WAIT_SET,
            WaitObj::Fd => libfabric_sys::fi_wait_obj_FI_WAIT_FD,
            WaitObj::MutexCond => libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND,
            WaitObj::Yield => libfabric_sys::fi_wait_obj_FI_WAIT_YIELD,
            WaitObj::PollFd => libfabric_sys::fi_wait_obj_FI_WAIT_POLLFD,
        }
    }
}


#[derive(Copy, Clone)]
pub enum WaitObj2 {
    Unspec,
    Fd,
    MutexCond,
    Yield,
    PollFd,
}

impl WaitObj2 {

    pub(crate) fn get_value(&self) -> u32 {
        match self {
            WaitObj2::Unspec => libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
            WaitObj2::Fd => libfabric_sys::fi_wait_obj_FI_WAIT_FD,
            WaitObj2::MutexCond => libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND,
            WaitObj2::Yield => libfabric_sys::fi_wait_obj_FI_WAIT_YIELD,
            WaitObj2::PollFd => libfabric_sys::fi_wait_obj_FI_WAIT_POLLFD,
        }
    }
}

pub enum WaitCond {
    None,
    Threshold,
}

impl WaitCond {

    pub(crate) fn get_value(&self) -> u32 {
        match self {
            WaitCond::None => libfabric_sys::fi_cq_wait_cond_FI_CQ_COND_NONE,
            WaitCond::Threshold => libfabric_sys::fi_cq_wait_cond_FI_CQ_COND_THRESHOLD,
        }
    }
}

pub enum HmemIface {
    SYSTEM,
    CUDA,
    ROCR,
    ZE,
    NEURON,
    SYNAPSEAI,
}


impl HmemIface {
    pub fn get_value(&self) -> libfabric_sys::fi_hmem_iface {
        match self {
            HmemIface::SYSTEM => libfabric_sys::fi_hmem_iface_FI_HMEM_SYSTEM, 
            HmemIface::CUDA => libfabric_sys::fi_hmem_iface_FI_HMEM_CUDA, 
            HmemIface::ROCR => libfabric_sys::fi_hmem_iface_FI_HMEM_ROCR, 
            HmemIface::ZE => libfabric_sys::fi_hmem_iface_FI_HMEM_ZE, 
            HmemIface::NEURON => libfabric_sys::fi_hmem_iface_FI_HMEM_NEURON, 
            HmemIface::SYNAPSEAI => libfabric_sys::fi_hmem_iface_FI_HMEM_SYNAPSEAI, 
        }
    }
}

pub enum EndpointOptName {
    MinMultiRecv,
    CmDataSize,
    BufferedMin,
    BufferedLimit,
    SendBufSize,
    RecvBufSize,
    TxSize,
    RxSize,
    FiHmemP2p,
    XpuTrigger,
    CudaApiPermitted,    
}

impl EndpointOptName {
    
    pub fn get_value(&self) -> libfabric_sys::_bindgen_ty_20{
        match self {
            EndpointOptName::MinMultiRecv => libfabric_sys::FI_OPT_MIN_MULTI_RECV,
            EndpointOptName::CmDataSize => libfabric_sys::FI_OPT_CM_DATA_SIZE,
            EndpointOptName::BufferedMin => libfabric_sys::FI_OPT_BUFFERED_MIN,
            EndpointOptName::BufferedLimit => libfabric_sys::FI_OPT_BUFFERED_LIMIT,
            EndpointOptName::SendBufSize => libfabric_sys::FI_OPT_SEND_BUF_SIZE,
            EndpointOptName::RecvBufSize => libfabric_sys::FI_OPT_RECV_BUF_SIZE,
            EndpointOptName::TxSize => libfabric_sys::FI_OPT_TX_SIZE,
            EndpointOptName::RxSize => libfabric_sys::FI_OPT_RX_SIZE,
            EndpointOptName::FiHmemP2p => libfabric_sys::FI_OPT_FI_HMEM_P2P,
            EndpointOptName::XpuTrigger => libfabric_sys::FI_OPT_XPU_TRIGGER,
            EndpointOptName::CudaApiPermitted => libfabric_sys::FI_OPT_CUDA_API_PERMITTED,    
        }
    }
}

pub enum EndpointOptLevel {
    Endpoint,
}

impl EndpointOptLevel {

    pub fn get_value(&self) -> libfabric_sys::_bindgen_ty_19 {
        match self {
            EndpointOptLevel::Endpoint => libfabric_sys::FI_OPT_ENDPOINT,
        }
    }
}

pub enum EndpointType {
    Unspec,
    Msg,
    Dgram,
    Rdm,
    SockStream,
    SockDgram,    
}

impl EndpointType {
    pub fn get_value(&self) -> libfabric_sys::fi_ep_type {
        match self {
            EndpointType::Unspec => libfabric_sys::fi_ep_type_FI_EP_UNSPEC, 
            EndpointType::Msg => libfabric_sys::fi_ep_type_FI_EP_MSG, 
            EndpointType::Dgram => libfabric_sys::fi_ep_type_FI_EP_DGRAM, 
            EndpointType::Rdm => libfabric_sys::fi_ep_type_FI_EP_RDM, 
            EndpointType::SockStream => libfabric_sys::fi_ep_type_FI_EP_SOCK_STREAM, 
            EndpointType::SockDgram => libfabric_sys::fi_ep_type_FI_EP_SOCK_DGRAM,        
        }
    }

    pub fn from(val: libfabric_sys::fi_ep_type ) -> Self { // [TODO] Handle errors
        
        match val {
            libfabric_sys::fi_ep_type_FI_EP_UNSPEC => EndpointType::Unspec,
            libfabric_sys::fi_ep_type_FI_EP_MSG => EndpointType::Msg,
            libfabric_sys::fi_ep_type_FI_EP_DGRAM => EndpointType::Dgram,
            libfabric_sys::fi_ep_type_FI_EP_RDM => EndpointType::Rdm,
            libfabric_sys::fi_ep_type_FI_EP_SOCK_STREAM => EndpointType::SockStream,
            libfabric_sys::fi_ep_type_FI_EP_SOCK_DGRAM => EndpointType::SockDgram,
            _ => panic!("Endpoint type flag not valid {}", val)
        }
    }
}

pub enum HmemP2p {
    Enabled,
    Required,
    Preferred,
    Disabled,
}

impl HmemP2p {

    pub fn get_value(&self) -> libfabric_sys::_bindgen_ty_21 {
        match self { 
            HmemP2p::Enabled => libfabric_sys::FI_HMEM_P2P_ENABLED,
            HmemP2p::Required => libfabric_sys::FI_HMEM_P2P_REQUIRED,
            HmemP2p::Preferred => libfabric_sys::FI_HMEM_P2P_PREFERRED,
            HmemP2p::Disabled => libfabric_sys::FI_HMEM_P2P_DISABLED, 
        }
    }

    pub fn from_value(val: u32) -> Self {

        if val == libfabric_sys::FI_HMEM_P2P_ENABLED {
            HmemP2p::Enabled
        }
        else if val == libfabric_sys::FI_HMEM_P2P_REQUIRED {
            HmemP2p::Required
        }
        else if val == libfabric_sys::FI_HMEM_P2P_PREFERRED {
            HmemP2p::Preferred
        }
        else if val == libfabric_sys::FI_HMEM_P2P_DISABLED {
            HmemP2p::Disabled
        }
        else {
            panic!("Unexpected HmemP2p value")
        }
    }
}

pub enum ControlOpt {
    GetFidFlag,
    SetFidFlag,
    GetOpsFlag,
    SetOpsFlag,
    Alias,
    GetWait,
    Enable,
    Backlog,
    GetRawMr,
    MapRawMr,
    UnmapKey,
    QueueWork,
    CancelWork,
    FlushWork,
    Refresh,
    Dup,
    GetWaitObj,
    GetVal,
    SetVal,
    ExportFid,
}

impl ControlOpt {

    #[allow(dead_code)]
    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_7 {
        match self {

            ControlOpt::GetFidFlag =>     libfabric_sys::FI_GETFIDFLAG,
            ControlOpt::SetFidFlag =>     libfabric_sys::FI_SETFIDFLAG,
            ControlOpt::GetOpsFlag =>     libfabric_sys::FI_GETOPSFLAG,
            ControlOpt::SetOpsFlag =>     libfabric_sys::FI_SETOPSFLAG,
            ControlOpt::Alias =>     libfabric_sys::FI_ALIAS,
            ControlOpt::GetWait =>     libfabric_sys::FI_GETWAIT,
            ControlOpt::Enable =>     libfabric_sys::FI_ENABLE,
            ControlOpt::Backlog =>     libfabric_sys::FI_BACKLOG,
            ControlOpt::GetRawMr =>     libfabric_sys::FI_GET_RAW_MR,
            ControlOpt::MapRawMr =>     libfabric_sys::FI_MAP_RAW_MR,
            ControlOpt::UnmapKey =>     libfabric_sys::FI_UNMAP_KEY,
            ControlOpt::QueueWork =>     libfabric_sys::FI_QUEUE_WORK,
            ControlOpt::CancelWork =>     libfabric_sys::FI_CANCEL_WORK,
            ControlOpt::FlushWork =>     libfabric_sys::FI_FLUSH_WORK,
            ControlOpt::Refresh =>     libfabric_sys::FI_REFRESH,
            ControlOpt::Dup =>     libfabric_sys::FI_DUP,
            ControlOpt::GetWaitObj =>     libfabric_sys::FI_GETWAITOBJ,
            ControlOpt::GetVal =>     libfabric_sys::FI_GET_VAL,
            ControlOpt::SetVal =>     libfabric_sys::FI_SET_VAL,
            ControlOpt::ExportFid =>     libfabric_sys::FI_EXPORT_FID,  
        }
    }
}

pub enum AddressVectorType {
    Unspec,
    Map,
    Table,    
}

impl AddressVectorType {
    pub(crate) fn from_value(value: libfabric_sys::fi_av_type) -> Self {
        if value == Self::Unspec.get_value() {
            Self::Unspec
        }
        else if value == Self::Map.get_value(){
            Self::Map
        }
        else if value == Self::Table.get_value() {
            Self::Table
        }
        else {
            panic!("Unexpected value for AddressVectorType");
        }
    }

    pub(crate) fn get_value(&self) -> libfabric_sys::fi_av_type {
        
        match self {
            AddressVectorType::Unspec => libfabric_sys::fi_av_type_FI_AV_UNSPEC, 
            AddressVectorType::Map => libfabric_sys::fi_av_type_FI_AV_MAP, 
            AddressVectorType::Table => libfabric_sys::fi_av_type_FI_AV_TABLE, 
        }
    }
}

macro_rules! gen_set_get_flag {
    ($(#[$attr0:meta])* $set_method_name:ident, $(#[$attr1:meta])? $get_method_name:ident, $flag:expr) => {

        $(#[$attr0])*
        pub fn $set_method_name(mut self) -> Self {
            self.c_flags |= $flag;
            
            self
        }

        $(#[$attr1])*
        pub fn $get_method_name(&self) -> bool {
            self.c_flags & $flag != 0
        } 
    };
}

macro_rules! gen_get_flag {
    ($get_method_name:ident, $flag:expr) => {

        pub fn $get_method_name(&self) -> bool {
            self.c_flags & $flag != 0
        } 
    };
}

pub(crate) use gen_set_get_flag;

pub struct Mode {
    c_flags: u64
}

impl Mode {

    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    pub fn all() -> Self {
        Self {c_flags: !0}
    }

    pub(crate) fn from_value(value: u64) -> Self {
        Self {c_flags: value}
    }

    gen_set_get_flag!(context, is_context, libfabric_sys::FI_CONTEXT);
    gen_set_get_flag!(msg_prefix, is_msg_prefix, libfabric_sys::FI_MSG_PREFIX);
    gen_set_get_flag!(async_iov, is_async_iov, libfabric_sys::FI_ASYNC_IOV);
    gen_set_get_flag!(rx_cq_data, is_rx_cq_data, libfabric_sys::FI_RX_CQ_DATA);
    gen_set_get_flag!(local_mr, is_local_mr, libfabric_sys::FI_LOCAL_MR);
    gen_set_get_flag!(notify_flags_only, is_notify_flags_only, libfabric_sys::FI_NOTIFY_FLAGS_ONLY);
    gen_set_get_flag!(restricted_comp, is_restricted_comp, libfabric_sys::FI_RESTRICTED_COMP);
    gen_set_get_flag!(context2, is_context2, libfabric_sys::FI_CONTEXT2);
    gen_set_get_flag!(buffered_recv, is_buffered_recv, libfabric_sys::FI_BUFFERED_RECV);


    pub fn get_value(&self) -> u64 {
        self.c_flags
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MrMode {
    c_flags: u32
}

impl MrMode {
    
    pub fn new() -> Self {
        Self {c_flags: 0}
    }

    pub(crate) fn from_value(value: u32) -> Self {
        Self {c_flags: value}
    }

    pub fn is_unspec(&self) -> bool {
        self.c_flags == libfabric_sys::fi_mr_mode_FI_MR_UNSPEC
    }
    
    pub fn inverse(mut self) -> Self {
        self.c_flags = ! self.c_flags;

        self
    }

    gen_set_get_flag!(basic, is_basic, libfabric_sys::fi_mr_mode_FI_MR_BASIC);
    gen_set_get_flag!(scalable, is_scalable, libfabric_sys::fi_mr_mode_FI_MR_SCALABLE);
    gen_set_get_flag!(local, is_local, libfabric_sys::FI_MR_LOCAL);
    gen_set_get_flag!(raw, is_raw, libfabric_sys::FI_MR_RAW);
    gen_set_get_flag!(virt_addr, is_virt_addr, libfabric_sys::FI_MR_VIRT_ADDR);
    gen_set_get_flag!(allocated, is_allocated, libfabric_sys::FI_MR_ALLOCATED);
    gen_set_get_flag!(prov_key, is_prov_key, libfabric_sys::FI_MR_PROV_KEY);
    gen_set_get_flag!(mmu_notify, is_mmu_notify, libfabric_sys::FI_MR_MMU_NOTIFY);
    gen_set_get_flag!(rma_event, is_rma_event, libfabric_sys::FI_MR_RMA_EVENT);
    gen_set_get_flag!(endpoint, is_endpoint, libfabric_sys::FI_MR_ENDPOINT);
    gen_set_get_flag!(hmem, is_hmem, libfabric_sys::FI_MR_HMEM);
    gen_set_get_flag!(collective, is_collective, libfabric_sys::FI_MR_COLLECTIVE);

    pub fn get_value(&self) -> u32 {
        self.c_flags
    }

}

impl Default for MrMode {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MrAccess {
    c_flags: u32,
}

impl MrAccess {
    pub fn new() -> Self {
        Self{c_flags: 0}
    }

    #[allow(dead_code)]
    pub(crate) fn from_value(value: u32) -> Self {
        Self {c_flags: value}
    }

    gen_set_get_flag!(send, is_send, libfabric_sys::FI_SEND);
    gen_set_get_flag!(recv, is_recv, libfabric_sys::FI_RECV);
    gen_set_get_flag!(read, is_read, libfabric_sys::FI_READ);
    gen_set_get_flag!(write, is_write, libfabric_sys::FI_WRITE);
    gen_set_get_flag!(remote_read, is_remote_read, libfabric_sys::FI_REMOTE_READ);
    gen_set_get_flag!(remote_write, is_remote_write, libfabric_sys::FI_REMOTE_WRITE);
    gen_set_get_flag!(collective, is_collective, libfabric_sys::FI_COLLECTIVE);

    pub fn get_value(&self) -> u32 {
        self.c_flags
    }
}

impl Default for MrAccess {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Progress {
    Unspec,
    Auto,
    Manual,    
}

impl Progress {
    pub fn get_value(&self) -> libfabric_sys::fi_progress {
        match self {
            Progress::Unspec => libfabric_sys::fi_progress_FI_PROGRESS_UNSPEC,
            Progress::Auto => libfabric_sys::fi_progress_FI_PROGRESS_AUTO,
            Progress::Manual => libfabric_sys::fi_progress_FI_PROGRESS_MANUAL,
        }
    }

    pub fn from_value(val: libfabric_sys::fi_progress) -> Self {

        if val == libfabric_sys::fi_progress_FI_PROGRESS_AUTO {
            Progress::Auto
        }
        else if val == libfabric_sys::fi_progress_FI_PROGRESS_MANUAL {
            Progress::Manual
        }
        else {
            Progress::Unspec
        }
    }
}

pub enum Threading {
    Unspec,
    Safe,
    Fid,
    Domain,
    Completion,
    Endpoint,
}

impl Threading {
    pub fn get_value(&self) -> libfabric_sys::fi_threading {
        match self {
            Threading::Unspec => libfabric_sys::fi_threading_FI_THREAD_UNSPEC,
            Threading::Safe => libfabric_sys::fi_threading_FI_THREAD_SAFE,
            Threading::Fid => libfabric_sys::fi_threading_FI_THREAD_FID,
            Threading::Domain => libfabric_sys::fi_threading_FI_THREAD_DOMAIN,
            Threading::Completion => libfabric_sys::fi_threading_FI_THREAD_COMPLETION,
            Threading::Endpoint => libfabric_sys::fi_threading_FI_THREAD_ENDPOINT,
        }
    }
}

pub enum ResourceMgmt {
    Unspec,
    Disabled,
    Enabled,
}

impl ResourceMgmt {
    pub fn get_value(&self) -> libfabric_sys::fi_resource_mgmt {
        match self {
            ResourceMgmt::Unspec => libfabric_sys::fi_resource_mgmt_FI_RM_UNSPEC,
            ResourceMgmt::Disabled => libfabric_sys::fi_resource_mgmt_FI_RM_DISABLED,
            ResourceMgmt::Enabled => libfabric_sys::fi_resource_mgmt_FI_RM_ENABLED,
        }
    }
}

pub enum EpType {
    Unspec,
    Msg,
    Dgram,
    Rdm,
    SockStream,
    SockDgram,
}

impl EpType {
    pub fn get_value(&self) -> libfabric_sys::fi_ep_type {

        match self {
            EpType::Unspec => libfabric_sys::fi_ep_type_FI_EP_UNSPEC,
            EpType::Msg => libfabric_sys::fi_ep_type_FI_EP_MSG,
            EpType::Dgram => libfabric_sys::fi_ep_type_FI_EP_DGRAM,
            EpType::Rdm => libfabric_sys::fi_ep_type_FI_EP_RDM,
            EpType::SockStream => libfabric_sys::fi_ep_type_FI_EP_SOCK_STREAM,
            EpType::SockDgram => libfabric_sys::fi_ep_type_FI_EP_SOCK_DGRAM,
        }
    }
}

pub enum CounterEvents {
    Comp,
}

impl CounterEvents {
    pub fn get_value(&self) -> libfabric_sys::fi_cntr_events {
        match self {
            CounterEvents::Comp => libfabric_sys::fi_cntr_events_FI_CNTR_EVENTS_COMP,
        }
    }
}

pub enum TClass {
    Unspec,
    Dscp,
    Label,
    BestEffort,
    LowLatency,
    DedicatedAccess,
    BulkData,
    Scavenger,
    NetworkCtrl,
}

impl TClass {
    pub fn get_value(&self) -> libfabric_sys::_bindgen_ty_5 {
        
        match self {
            TClass::Unspec => libfabric_sys::FI_TC_UNSPEC,
            TClass::Dscp => libfabric_sys::FI_TC_DSCP,
            TClass::Label => libfabric_sys::FI_TC_LABEL,
            TClass::BestEffort => libfabric_sys::FI_TC_BEST_EFFORT,
            TClass::LowLatency => libfabric_sys::FI_TC_LOW_LATENCY,
            TClass::DedicatedAccess => libfabric_sys::FI_TC_DEDICATED_ACCESS,
            TClass::BulkData => libfabric_sys::FI_TC_BULK_DATA,
            TClass::Scavenger => libfabric_sys::FI_TC_SCAVENGER,
            TClass::NetworkCtrl => libfabric_sys::FI_TC_NETWORK_CTRL,
        }
    }
}

pub enum AddressFormat {
    Unspec,
    SockAddr,
    SockaddrIn,
    SockaddrIn6,
    SockaddrIb,
    Psmx,
    Gni,
    Bgq,
    Mlx,
    Str,
    Psmx2,
    IbUd,
    Efa,
    Psmx3,
    Opx,
    Cxi,
    Ucx,
}

impl AddressFormat {
    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_3 {

        match self {
            AddressFormat::Unspec => libfabric_sys::FI_FORMAT_UNSPEC,
            AddressFormat::SockAddr => libfabric_sys::FI_SOCKADDR,
            AddressFormat::SockaddrIn => libfabric_sys::FI_SOCKADDR_IN,
            AddressFormat::SockaddrIn6 => libfabric_sys::FI_SOCKADDR_IN6,
            AddressFormat::SockaddrIb => libfabric_sys::FI_SOCKADDR_IB,
            AddressFormat::Psmx => libfabric_sys::FI_ADDR_PSMX,
            AddressFormat::Gni => libfabric_sys::FI_ADDR_GNI,
            AddressFormat::Bgq => libfabric_sys::FI_ADDR_BGQ,
            AddressFormat::Mlx => libfabric_sys::FI_ADDR_MLX,
            AddressFormat::Str => libfabric_sys::FI_ADDR_STR,
            AddressFormat::Psmx2 => libfabric_sys::FI_ADDR_PSMX2,
            AddressFormat::IbUd => libfabric_sys::FI_ADDR_IB_UD,
            AddressFormat::Efa => libfabric_sys::FI_ADDR_EFA,
            AddressFormat::Psmx3 => libfabric_sys::FI_ADDR_PSMX3,
            AddressFormat::Opx => libfabric_sys::FI_ADDR_OPX,
            AddressFormat::Cxi => libfabric_sys::FI_ADDR_CXI,
            AddressFormat::Ucx => libfabric_sys::FI_ADDR_UCX,
        }
    }
}

pub struct AVOptions {
    c_flags: u64,
} 

impl AVOptions {
    
    /// Create a new [AVOptions] object with the default configuration.
    pub fn new() -> Self {
        Self{
            c_flags: 0,
        }
    }

    gen_set_get_flag!(
        /// Hint to the provider that more insertion requests will follow, allowing the provider to aggregate insertion requests if desired.
        /// 
        /// Corresponds to setting the bitflag `FI_MORE`.
        more, 
        /// Check if the `FI_MORE` bitflag is set.
        is_more, libfabric_sys::FI_MORE as u64);
    
    gen_set_get_flag!(
        /// This flag applies to synchronous insertions only, and is used to retrieve error details of failed insertions.alloc
        /// 
        /// Corrsponds to setting the bitflag `FI_SYNC_ERR`.
        sync_err, 
        /// Check if the `FI_SYNC_ERR` bitflag is set.
        is_sync_err, libfabric_sys::FI_SYNC_ERR);
    gen_set_get_flag!(user_id, is_user_id, libfabric_sys::FI_AV_USER_ID);

    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }
}

impl Default for AVOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct TferOptions<const OUT: bool, const MSG: bool, const RMA: bool, const DATA: bool, const TAGGED: bool, const ATOMIC: bool> {
    c_flags: u64,
}



impl<const OUT: bool, const MSG: bool, const RMA: bool, const DATA: bool, const TAGGED: bool, const ATOMIC: bool> TferOptions<OUT, MSG, RMA, DATA, TAGGED, ATOMIC> { // All transfer types
    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }

    gen_set_get_flag!(completion, is_completion, libfabric_sys::FI_COMPLETION as u64);
    gen_set_get_flag!(more, is_more, libfabric_sys::FI_MORE as u64);
}

impl<const OUT: bool, const MSG: bool, const RMA: bool, const DATA: bool, const TAGGED: bool, const ATOMIC: bool> Default for TferOptions<OUT, MSG, RMA, DATA, TAGGED, ATOMIC> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const OUT: bool, const MSG: bool> TferOptions<OUT, MSG, false, false, false, true> { // All atomic transfers
    gen_set_get_flag!(tagged, is_tagged, libfabric_sys::FI_TAGGED as u64);
}


impl<const MSG: bool, const RMA: bool, const DATA: bool, const TAGGED: bool, const ATOMIC: bool> TferOptions<true, MSG, RMA, DATA, TAGGED, ATOMIC> { // All transmits
    gen_set_get_flag!(fence, is_fence, libfabric_sys::FI_FENCE as u64);
}

impl<const RMA: bool, const TAGGED: bool> TferOptions<true, false, RMA, true, TAGGED, false> { // Only data transmits (no msg)
    gen_set_get_flag!(remote_cq_data, is_remote_cq_data, libfabric_sys::FI_REMOTE_CQ_DATA as u64);
}



impl<const RMA: bool, const TAGGED: bool> TferOptions<true, true, RMA, false, TAGGED, false> { // Only msg transmits (no data)
    gen_set_get_flag!(inject, is_inject, libfabric_sys::FI_INJECT as u64);
    gen_set_get_flag!(inject_complete, is_inject_complete, libfabric_sys::FI_INJECT_COMPLETE as u64);
    gen_set_get_flag!(transmit_complete, is_transmit_complete, libfabric_sys::FI_TRANSMIT_COMPLETE as u64);
    gen_set_get_flag!(delivery_complete, is_delivery_complete, libfabric_sys::FI_DELIVERY_COMPLETE as u64);
    gen_set_get_flag!(remote_cq_data, is_remote_cq_data, libfabric_sys::FI_REMOTE_CQ_DATA as u64);
}



impl TferOptions<true, true, false, false, true, false> { // Only tagged msg transmits (no data)
    gen_set_get_flag!(match_complete, is_match_complete, libfabric_sys::FI_MATCH_COMPLETE as u64);
}

impl TferOptions<true, true, true, false, false, false> { // Only RMA msg transmits (no data)
    gen_set_get_flag!(commit_complete, is_commit_complete, libfabric_sys::FI_COMMIT_COMPLETE as u64);
}

impl<const MSG: bool, const DATA: bool> TferOptions<true, MSG, false, DATA, false, false> { // Non-RMA or Tagged transmits
    gen_set_get_flag!(multicast, is_multicast, libfabric_sys::FI_MULTICAST as u64);
}

impl<const MSG: bool> TferOptions<false, MSG, false, false, false, false> { // All Posted Receive Operations (i.e. recv, recvmsg)
    gen_set_get_flag!(claim, is_claim, libfabric_sys::FI_CLAIM);
    gen_set_get_flag!(discard, is_discard, libfabric_sys::FI_DISCARD);
    gen_set_get_flag!(multi_recv, is_multi_recv, libfabric_sys::FI_MULTI_RECV as u64);
    
}

impl TferOptions<false, true, false, false, true, false> { // Only tagged Posted Receive Operations
    gen_set_get_flag!(peek, is_peek, libfabric_sys::FI_PEEK as u64);
    gen_set_get_flag!(claim, is_claim, libfabric_sys::FI_CLAIM);
    gen_set_get_flag!(discard, is_discard, libfabric_sys::FI_DISCARD);
}

// pub type SendOptions = TferOptions<true, false, false, false, false>;
pub type SendMsgOptions = TferOptions<true, true, false, false, false, false>;
// pub type SendDataOptions = TferOptions<true, true, false, true, false>;
// pub type TaggedSendOptions = TferOptions<true, false, false, false, true>;
pub type TaggedSendMsgOptions = TferOptions<true, true, false, false, true, false>;
// pub type TaggedSendDataOptions = TferOptions<true, false, false, true, true>;

// pub type WriteOptions = TferOptions<true, false, true, false, false>;
pub type WriteMsgOptions = TferOptions<true, true, true, false, false, false>;
// pub type WriteDataOptions = TferOptions<true, false, true, true, false>;


// pub type RecvOptions = TferOptions<false, false, false, false, false>;
pub type RecvMsgOptions = TferOptions<false, true, false, false, false, false>;
// pub type TaggedRecvOptions = TferOptions<false, false, false, false, true>;
pub type TaggedRecvMsgOptions = TferOptions<false, true, false, false, true, false>;

// pub type ReadOptions = TferOptions<false, false, true, false, false>;
pub type ReadMsgOptions = TferOptions<false, true, true, false, false, false>;

// pub type AtomicOptions = TferOptions<true, false, true, false, false, true>;
pub type AtomicMsgOptions = TferOptions<true, true, true, false, false, true>;

// pub type AtomicFetchOptions = TferOptions<true, false, true, false, false, true>;
pub type AtomicFetchMsgOptions = TferOptions<true, true, true, false, false, true>;

pub type CollectiveOptions = AtomicMsgOptions;

#[derive(Clone,Copy)]
pub struct TransferOptions {
    c_flags: u32,
}

impl TransferOptions {
    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    pub(crate) fn from_value(val: u32) -> Self {
        Self {
            c_flags: val,
        }
    }

    pub(crate) fn transmit(mut self) -> Self {
        self.c_flags |= FI_TRANSMIT;
        self
    }

    pub(crate) fn recv(mut self) -> Self {
        self.c_flags |= FI_RECV;
        self
    }

    gen_set_get_flag!(commit_complete, is_commit_complete, libfabric_sys::FI_COMMIT_COMPLETE);
    gen_set_get_flag!(completion, is_completion, libfabric_sys::FI_COMPLETION);
    gen_set_get_flag!(delivery_complete, is_delivery_complete, libfabric_sys::FI_DELIVERY_COMPLETE);
    gen_set_get_flag!(inject, is_inject, libfabric_sys::FI_INJECT);
    gen_set_get_flag!(inject_complete, is_inject_complete, libfabric_sys::FI_INJECT_COMPLETE);
    gen_set_get_flag!(multicast, is_multicast, libfabric_sys::FI_MULTICAST);
    gen_set_get_flag!(multi_recv, is_multi_recv, libfabric_sys::FI_MULTI_RECV);
    gen_set_get_flag!(transmit_complete, is_transmit_complete, libfabric_sys::FI_TRANSMIT_COMPLETE);

    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_3 {
        self.c_flags
    }

    // pub(crate) fn from_value(value: u32) -> Self {

    // }
    //     match self {
    //         TransferOptions::COMMIT_COMPLETE => libfabric_sys::FI_COMMIT_COMPLETE,
    //         TransferOptions::COMPLETION => libfabric_sys::FI_COMPLETION,
    //         TransferOptions::DELIVERY_COMPLETE => libfabric_sys::FI_DELIVERY_COMPLETE,
    //         TransferOptions::INJECT => libfabric_sys::FI_INJECT,
    //         TransferOptions::INJECT_COMPLETE => libfabric_sys::FI_INJECT_COMPLETE,
    //         TransferOptions::MULTICAST => libfabric_sys::FI_MULTICAST,
    //         TransferOptions::MULTI_RECV => libfabric_sys::FI_MULTI_RECV,
    //         TransferOptions::TRANSMIT_COMPLETE => libfabric_sys::FI_TRANSMIT_COMPLETE,
    //     }
    // }    
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self::new()
    }
}



pub enum ParamType {
    String,
    Int,
    Bool,
    SizeT, 
}

impl ParamType {
    #[allow(dead_code)]
    pub(crate) fn get_value(&self) -> libfabric_sys::fi_param_type {

        match self {
            ParamType::String => libfabric_sys::fi_param_type_FI_PARAM_STRING,
            ParamType::Int => libfabric_sys::fi_param_type_FI_PARAM_INT,
            ParamType::Bool => libfabric_sys::fi_param_type_FI_PARAM_BOOL,
            ParamType::SizeT => libfabric_sys::fi_param_type_FI_PARAM_SIZE_T,
        }
    }
}

pub enum Protocol {
    Gni,
    IbRdm,
    IbUd,
    IWARP,
    IwarpRdm,
    NetworkDirect,
    Psmx,
    Psmx2,
    Psmx3,
    RdmaCmIbRc,
    Rxd,
    Udp,
    Unspec,
}

impl Protocol {

    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_4 {
        
        match self {
            Protocol::Gni => libfabric_sys::FI_PROTO_GNI,
            Protocol::IbRdm => libfabric_sys::FI_PROTO_IB_RDM,
            Protocol::IbUd => libfabric_sys::FI_PROTO_IB_UD,
            Protocol::IWARP => libfabric_sys::FI_PROTO_IWARP,
            Protocol::IwarpRdm => libfabric_sys::FI_PROTO_IWARP_RDM,
            Protocol::NetworkDirect => libfabric_sys::FI_PROTO_NETWORKDIRECT,
            Protocol::Psmx => libfabric_sys::FI_PROTO_PSMX,
            Protocol::Psmx2 => libfabric_sys::FI_PROTO_PSMX2,
            Protocol::Psmx3 => libfabric_sys::FI_PROTO_PSMX3,
            Protocol::RdmaCmIbRc => libfabric_sys::FI_PROTO_RDMA_CM_IB_RC,
            Protocol::Rxd => libfabric_sys::FI_PROTO_RXD,
            Protocol::Udp => libfabric_sys::FI_PROTO_UDP,
            Protocol::Unspec => libfabric_sys::FI_PROTO_UNSPEC,
        }
    }
}

/// Encapsulates the possible values returned by a call to `Counter/EventQueue/CompletionQueue::wait_object`
pub enum WaitObjType<'a> {
    MutexCond(libfabric_sys::fi_mutex_cond),
    Fd(BorrowedFd<'a>),
    Unspec,
}

pub enum WaitObjType2<'a> {
    MutexCond(libfabric_sys::fi_mutex_cond),
    Fd(BorrowedFd<'a>),
    PollFd(libfabric_sys::fi_wait_pollfd),
    Yield,
    Unspec,
}

pub enum DomainCaps {
    LocalComm,
    RemoteComm,
    SharedAv,
}

impl DomainCaps {
    pub(crate) fn get_value(&self) -> u64 {
        match self {
            DomainCaps::LocalComm => libfabric_sys::FI_LOCAL_COMM,
            DomainCaps::RemoteComm => libfabric_sys::FI_REMOTE_COMM,
            DomainCaps::SharedAv => libfabric_sys::FI_SHARED_AV,
        }
    }
}

pub struct CompletionFlags {
    c_flags: u64,
}

impl CompletionFlags {
    pub(crate) fn from_value(c_flags: u64) -> Self {
        Self{
            c_flags
        }
    }

    gen_get_flag!(is_send, libfabric_sys::FI_SEND as u64);
    gen_get_flag!(is_recv, libfabric_sys::FI_RECV as u64);
    gen_get_flag!(is_rma, libfabric_sys::FI_RMA as u64);
    gen_get_flag!(is_atomic, libfabric_sys::FI_ATOMIC as u64);
    gen_get_flag!(is_msg, libfabric_sys::FI_MSG as u64);
    gen_get_flag!(is_tagged, libfabric_sys::FI_TAGGED as u64);
    gen_get_flag!(is_multicast, libfabric_sys::FI_MULTICAST as u64);
    gen_get_flag!(is_read, libfabric_sys::FI_READ as u64);
    gen_get_flag!(is_write, libfabric_sys::FI_WRITE as u64);
    gen_get_flag!(is_remote_read, libfabric_sys::FI_REMOTE_READ as u64);
    gen_get_flag!(is_remote_write, libfabric_sys::FI_REMOTE_WRITE as u64);
    gen_get_flag!(is_remote_cq_data, libfabric_sys::FI_REMOTE_CQ_DATA as u64);
    gen_get_flag!(is_multi_recv, libfabric_sys::FI_MULTI_RECV as u64);
    gen_get_flag!(is_more, libfabric_sys::FI_MORE as u64);
    gen_get_flag!(is_claim, libfabric_sys::FI_CLAIM);
}

pub struct AVSetOptions {
    c_flags: u64,
}

impl AVSetOptions {
    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }
    
    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }

    gen_set_get_flag!(universe, is_universe, libfabric_sys::FI_UNIVERSE);
    gen_set_get_flag!(barrier_set, is_barrier_set, libfabric_sys::FI_BARRIER_SET);
    gen_set_get_flag!(broadcast_set, is_broadcast_set, libfabric_sys::FI_BROADCAST_SET);
    gen_set_get_flag!(alltoall_set, is_alltoall_set, libfabric_sys::FI_ALLTOALL_SET);
    gen_set_get_flag!(allreduce_set, is_allreduce_set, libfabric_sys::FI_ALLREDUCE_SET);
    gen_set_get_flag!(allgather_set, is_allgather_set, libfabric_sys::FI_ALLGATHER_SET);
    gen_set_get_flag!(reduce_scatter_set, is_reduce_scatter_set, libfabric_sys::FI_REDUCE_SCATTER_SET);
    gen_set_get_flag!(reduce_set, is_reduce_set, libfabric_sys::FI_REDUCE_SET);
    gen_set_get_flag!(scatter_set, is_scatter_set, libfabric_sys::FI_SCATTER_SET);
    gen_set_get_flag!(gather_set, is_gather_set, libfabric_sys::FI_GATHER_SET);
}

impl Default for AVSetOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JoinOptions {
    c_flags: u64,
}

impl JoinOptions {
    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }

    gen_set_get_flag!(send, is_send, libfabric_sys::FI_SEND as u64);
    gen_set_get_flag!(receive, is_receive, libfabric_sys::FI_SEND as u64);
}

impl Default for JoinOptions {
    fn default() -> Self {
        Self::new()
    }
}