use std::clone;

#[allow(non_camel_case_types)]
pub enum Op {
    MIN,
    MAX,
    SUM,
    PROD,
    LOR,
    LAND,
    BOR,
    BAND,
    LXOR,
    BXOR,
    ATOMIC_READ,
    ATOMIC_WRITE,
    CSWAP,
    CSWAP_NE,
    CSWAP_LE,
    CSWAP_LT,
    CSWAP_GE,
    CSWAP_GT,
    MSWAP,
    ATOMIC_OP_LAST,
    NOOP,
}

impl Op {
    pub(crate) fn get_value(&self) -> u32  {
        match self {
            Op::MIN => libfabric_sys::fi_op_FI_MIN,
            Op::MAX => libfabric_sys::fi_op_FI_MAX,
            Op::SUM => libfabric_sys::fi_op_FI_SUM,
            Op::PROD => libfabric_sys::fi_op_FI_PROD,
            Op::LOR => libfabric_sys::fi_op_FI_LOR,
            Op::LAND => libfabric_sys::fi_op_FI_LAND,
            Op::BOR => libfabric_sys::fi_op_FI_BOR,
            Op::BAND => libfabric_sys::fi_op_FI_BAND,
            Op::LXOR => libfabric_sys::fi_op_FI_LXOR,
            Op::BXOR => libfabric_sys::fi_op_FI_BXOR,
            Op::ATOMIC_READ => libfabric_sys::fi_op_FI_ATOMIC_READ,
            Op::ATOMIC_WRITE => libfabric_sys::fi_op_FI_ATOMIC_WRITE,
            Op::CSWAP => libfabric_sys::fi_op_FI_CSWAP,
            Op::CSWAP_NE => libfabric_sys::fi_op_FI_CSWAP_NE,
            Op::CSWAP_LE => libfabric_sys::fi_op_FI_CSWAP_LE,
            Op::CSWAP_LT => libfabric_sys::fi_op_FI_CSWAP_LT,
            Op::CSWAP_GE => libfabric_sys::fi_op_FI_CSWAP_GE,
            Op::CSWAP_GT => libfabric_sys::fi_op_FI_CSWAP_GT,
            Op::MSWAP => libfabric_sys::fi_op_FI_MSWAP,
            Op::ATOMIC_OP_LAST => libfabric_sys::fi_op_FI_ATOMIC_OP_LAST,
            Op::NOOP  => libfabric_sys::fi_op_FI_NOOP,
        }
    }
}

#[allow(non_camel_case_types)]
pub enum CollectiveOp {
    BARRIER,
    BROADCAST,
    ALLTOALL,
    ALLREDUCE,
    ALLGATHER,
    REDUCE_SCATTER,
    REDUCE,
    SCATTER,
    GATHER,
}

impl CollectiveOp {
    pub(crate) fn get_value(&self) -> u32 {
        match self {

            CollectiveOp::BARRIER => libfabric_sys::fi_collective_op_FI_BARRIER,
            CollectiveOp::BROADCAST => libfabric_sys::fi_collective_op_FI_BROADCAST,
            CollectiveOp::ALLTOALL => libfabric_sys::fi_collective_op_FI_ALLTOALL,
            CollectiveOp::ALLREDUCE => libfabric_sys::fi_collective_op_FI_ALLREDUCE,
            CollectiveOp::ALLGATHER => libfabric_sys::fi_collective_op_FI_ALLGATHER,
            CollectiveOp::REDUCE_SCATTER => libfabric_sys::fi_collective_op_FI_REDUCE_SCATTER,
            CollectiveOp::REDUCE => libfabric_sys::fi_collective_op_FI_REDUCE,
            CollectiveOp::SCATTER => libfabric_sys::fi_collective_op_FI_SCATTER,
            CollectiveOp::GATHER => libfabric_sys::fi_collective_op_FI_GATHER,
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
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum WaitObj {
    NONE,
    UNSPEC,
    SET,
    FD,
    MUTEX_COND,
    YIELD,
    POLLFD,
}

impl WaitObj {

    pub(crate) fn get_value(&self) -> u32 {
        match self {
            WaitObj::NONE => libfabric_sys::fi_wait_obj_FI_WAIT_NONE,
            WaitObj::UNSPEC => libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
            WaitObj::SET => libfabric_sys::fi_wait_obj_FI_WAIT_SET,
            WaitObj::FD => libfabric_sys::fi_wait_obj_FI_WAIT_FD,
            WaitObj::MUTEX_COND => libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND,
            WaitObj::YIELD => libfabric_sys::fi_wait_obj_FI_WAIT_YIELD,
            WaitObj::POLLFD => libfabric_sys::fi_wait_obj_FI_WAIT_POLLFD,
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum WaitObj2 {
    UNSPEC,
    FD,
    MUTEX_COND,
    YIELD,
    POLLFD,
}

impl WaitObj2 {

    pub(crate) fn get_value(&self) -> u32 {
        match self {
            WaitObj2::UNSPEC => libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
            WaitObj2::FD => libfabric_sys::fi_wait_obj_FI_WAIT_FD,
            WaitObj2::MUTEX_COND => libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND,
            WaitObj2::YIELD => libfabric_sys::fi_wait_obj_FI_WAIT_YIELD,
            WaitObj2::POLLFD => libfabric_sys::fi_wait_obj_FI_WAIT_POLLFD,
        }
    }
}

pub enum WaitCond {
    NONE,
    THRESHOLD,
}

impl WaitCond {

    pub(crate) fn get_value(&self) -> u32 {
        match self {
            WaitCond::NONE => libfabric_sys::fi_cq_wait_cond_FI_CQ_COND_NONE,
            WaitCond::THRESHOLD => libfabric_sys::fi_cq_wait_cond_FI_CQ_COND_THRESHOLD,
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

#[allow(non_camel_case_types)]
pub enum EndpointOptName {
    MIN_MULTI_RECV,
    CM_DATA_SIZE,
    BUFFERED_MIN,
    BUFFERED_LIMIT,
    SEND_BUF_SIZE,
    RECV_BUF_SIZE,
    TX_SIZE,
    RX_SIZE,
    FI_HMEM_P2P,
    XPU_TRIGGER,
    CUDA_API_PERMITTED,    
}

impl EndpointOptName {
    
    pub fn get_value(&self) -> libfabric_sys::_bindgen_ty_20{
        match self {
            EndpointOptName::MIN_MULTI_RECV => libfabric_sys::FI_OPT_MIN_MULTI_RECV,
            EndpointOptName::CM_DATA_SIZE => libfabric_sys::FI_OPT_CM_DATA_SIZE,
            EndpointOptName::BUFFERED_MIN => libfabric_sys::FI_OPT_BUFFERED_MIN,
            EndpointOptName::BUFFERED_LIMIT => libfabric_sys::FI_OPT_BUFFERED_LIMIT,
            EndpointOptName::SEND_BUF_SIZE => libfabric_sys::FI_OPT_SEND_BUF_SIZE,
            EndpointOptName::RECV_BUF_SIZE => libfabric_sys::FI_OPT_RECV_BUF_SIZE,
            EndpointOptName::TX_SIZE => libfabric_sys::FI_OPT_TX_SIZE,
            EndpointOptName::RX_SIZE => libfabric_sys::FI_OPT_RX_SIZE,
            EndpointOptName::FI_HMEM_P2P => libfabric_sys::FI_OPT_FI_HMEM_P2P,
            EndpointOptName::XPU_TRIGGER => libfabric_sys::FI_OPT_XPU_TRIGGER,
            EndpointOptName::CUDA_API_PERMITTED => libfabric_sys::FI_OPT_CUDA_API_PERMITTED,    
        }
    }
}

#[allow(non_camel_case_types)]
pub enum EndpointOptLevel {
    ENDPOINT,
}

impl EndpointOptLevel {

    pub fn get_value(&self) -> libfabric_sys::_bindgen_ty_19 {
        match self {
            EndpointOptLevel::ENDPOINT => libfabric_sys::FI_OPT_ENDPOINT,
        }
    }
}

#[allow(non_camel_case_types)]
pub enum EndpointType {
    UNSPEC,
    MSG,
    DGRAM,
    RDM,
    SOCK_STREAM,
    SOCK_DGRAM,    
}

impl EndpointType {
    pub fn get_value(&self) -> libfabric_sys::fi_ep_type {
        match self {
            EndpointType::UNSPEC => libfabric_sys::fi_ep_type_FI_EP_UNSPEC, 
            EndpointType::MSG => libfabric_sys::fi_ep_type_FI_EP_MSG, 
            EndpointType::DGRAM => libfabric_sys::fi_ep_type_FI_EP_DGRAM, 
            EndpointType::RDM => libfabric_sys::fi_ep_type_FI_EP_RDM, 
            EndpointType::SOCK_STREAM => libfabric_sys::fi_ep_type_FI_EP_SOCK_STREAM, 
            EndpointType::SOCK_DGRAM => libfabric_sys::fi_ep_type_FI_EP_SOCK_DGRAM,        
        }
    }

    pub fn from(val: libfabric_sys::fi_ep_type ) -> Self { // [TODO] Handle errors
        
        match val {
            libfabric_sys::fi_ep_type_FI_EP_UNSPEC => EndpointType::UNSPEC,
            libfabric_sys::fi_ep_type_FI_EP_MSG => EndpointType::MSG,
            libfabric_sys::fi_ep_type_FI_EP_DGRAM => EndpointType::DGRAM,
            libfabric_sys::fi_ep_type_FI_EP_RDM => EndpointType::RDM,
            libfabric_sys::fi_ep_type_FI_EP_SOCK_STREAM => EndpointType::SOCK_STREAM,
            libfabric_sys::fi_ep_type_FI_EP_SOCK_DGRAM => EndpointType::SOCK_DGRAM,
            _ => panic!("Endpoint type flag not valid {}", val)
        }
    }
}

pub enum HmemP2p {
    ENABLED,
    REQUIRED,
    PREFERRED,
    DISABLED,
}

impl HmemP2p {

    pub fn get_value(&self) -> libfabric_sys::_bindgen_ty_21 {
        match self { 
            HmemP2p::ENABLED => libfabric_sys::FI_HMEM_P2P_ENABLED,
            HmemP2p::REQUIRED => libfabric_sys::FI_HMEM_P2P_REQUIRED,
            HmemP2p::PREFERRED => libfabric_sys::FI_HMEM_P2P_PREFERRED,
            HmemP2p::DISABLED => libfabric_sys::FI_HMEM_P2P_DISABLED, 
        }
    }
}

#[allow(non_camel_case_types)]
pub enum ControlOpt {
    GETFIDFLAG,
    SETFIDFLAG,
    GETOPSFLAG,
    SETOPSFLAG,
    ALIAS,
    GETWAIT,
    ENABLE,
    BACKLOG,
    GET_RAW_MR,
    MAP_RAW_MR,
    UNMAP_KEY,
    QUEUE_WORK,
    CANCEL_WORK,
    FLUSH_WORK,
    REFRESH,
    DUP,
    GETWAITOBJ,
    GET_VAL,
    SET_VAL,
    EXPORT_FID,
}

impl ControlOpt {
    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_7 {
        match self {

            ControlOpt::GETFIDFLAG =>     libfabric_sys::FI_GETFIDFLAG,
            ControlOpt::SETFIDFLAG =>     libfabric_sys::FI_SETFIDFLAG,
            ControlOpt::GETOPSFLAG =>     libfabric_sys::FI_GETOPSFLAG,
            ControlOpt::SETOPSFLAG =>     libfabric_sys::FI_SETOPSFLAG,
            ControlOpt::ALIAS =>     libfabric_sys::FI_ALIAS,
            ControlOpt::GETWAIT =>     libfabric_sys::FI_GETWAIT,
            ControlOpt::ENABLE =>     libfabric_sys::FI_ENABLE,
            ControlOpt::BACKLOG =>     libfabric_sys::FI_BACKLOG,
            ControlOpt::GET_RAW_MR =>     libfabric_sys::FI_GET_RAW_MR,
            ControlOpt::MAP_RAW_MR =>     libfabric_sys::FI_MAP_RAW_MR,
            ControlOpt::UNMAP_KEY =>     libfabric_sys::FI_UNMAP_KEY,
            ControlOpt::QUEUE_WORK =>     libfabric_sys::FI_QUEUE_WORK,
            ControlOpt::CANCEL_WORK =>     libfabric_sys::FI_CANCEL_WORK,
            ControlOpt::FLUSH_WORK =>     libfabric_sys::FI_FLUSH_WORK,
            ControlOpt::REFRESH =>     libfabric_sys::FI_REFRESH,
            ControlOpt::DUP =>     libfabric_sys::FI_DUP,
            ControlOpt::GETWAITOBJ =>     libfabric_sys::FI_GETWAITOBJ,
            ControlOpt::GET_VAL =>     libfabric_sys::FI_GET_VAL,
            ControlOpt::SET_VAL =>     libfabric_sys::FI_SET_VAL,
            ControlOpt::EXPORT_FID =>     libfabric_sys::FI_EXPORT_FID,  
        }
    }
}

pub enum AddressVectorType {
    UNSPEC,
    MAP,
    TABLE,    
}

impl AddressVectorType {
    pub(crate) fn from_value(value: libfabric_sys::fi_av_type) -> Self {
        if value == Self::UNSPEC.get_value() {
            Self::UNSPEC
        }
        else if value == Self::MAP.get_value(){
            Self::MAP
        }
        else if value == Self::TABLE.get_value() {
            Self::TABLE
        }
        else {
            panic!("Unexpected value for AddressVectorType");
        }
    }

    pub fn get_value(&self) -> libfabric_sys::fi_av_type {
        
        match self {
            AddressVectorType::UNSPEC => libfabric_sys::fi_av_type_FI_AV_UNSPEC, 
            AddressVectorType::MAP => libfabric_sys::fi_av_type_FI_AV_MAP, 
            AddressVectorType::TABLE => libfabric_sys::fi_av_type_FI_AV_TABLE, 
        }
    }
}

macro_rules! gen_set_get_flag {
    ($set_method_name:ident, $get_method_name:ident, $flag:expr) => {

        pub fn $set_method_name(mut self) -> Self {
            self.c_flags |= $flag;
            
            self
        }

        pub fn $get_method_name(&self) -> bool {
            self.c_flags & $flag != 0
        } 
    };
}

pub struct ModeBuilder;


impl ModeBuilder {

    pub fn context(self) -> Mode {
        Mode {c_flags: libfabric_sys::FI_CONTEXT}
    }
    
    pub fn msg_prefix(self) -> Mode {
        Mode {c_flags: libfabric_sys::FI_MSG_PREFIX}
    }


    pub fn async_iov(self) -> Mode {

        Mode{c_flags: libfabric_sys::FI_ASYNC_IOV}
    }
    pub fn rx_cq_data(self) -> Mode {

        Mode{c_flags: libfabric_sys::FI_RX_CQ_DATA}
    }
    pub fn local_mr(self) -> Mode {

        Mode{c_flags: libfabric_sys::FI_LOCAL_MR}
    }
    pub fn notify_flags_only(self) -> Mode {

        Mode{c_flags: libfabric_sys::FI_NOTIFY_FLAGS_ONLY}
    }
    pub fn restricted_comp(self) -> Mode {

        Mode{c_flags: libfabric_sys::FI_RESTRICTED_COMP}
    }
    pub fn context2(self) -> Mode {

        Mode{c_flags: libfabric_sys::FI_CONTEXT2}
    }
    pub fn buffered_recv(self) -> Mode {

        Mode{c_flags: libfabric_sys::FI_BUFFERED_RECV}
    }
    
}


pub struct Mode {
    c_flags: u64
}

impl Mode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ModeBuilder {
        ModeBuilder
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

pub struct MrMode {
    c_flags: u32
}


// pub fn msg(self) -> Self  { Self { bitfield: self.bitfield | libfabric_sys::FI_MSG as u64 } }
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

#[allow(non_camel_case_types)]
pub enum Progress {
    UNSPEC,
    AUTO,
    MANUAL,    
}

impl Progress {
    pub fn get_value(&self) -> libfabric_sys::fi_progress {
        match self {
            Progress::UNSPEC => libfabric_sys::fi_progress_FI_PROGRESS_UNSPEC,
            Progress::AUTO => libfabric_sys::fi_progress_FI_PROGRESS_AUTO,
            Progress::MANUAL => libfabric_sys::fi_progress_FI_PROGRESS_MANUAL,
        }
    }

    pub fn from_value(val: libfabric_sys::fi_progress) -> Self {

        if val == libfabric_sys::fi_progress_FI_PROGRESS_AUTO {
            Progress::AUTO
        }
        else if val == libfabric_sys::fi_progress_FI_PROGRESS_MANUAL {
            Progress::MANUAL
        }
        else {
            Progress::UNSPEC
        }
    }
}

#[allow(non_camel_case_types)]
pub enum Threading {
    UNSPEC,
    SAFE,
    FID,
    DOMAIN,
    COMPLETION,
    ENDPOINT,
}

impl Threading {
    pub fn get_value(&self) -> libfabric_sys::fi_threading {
        match self {
            Threading::UNSPEC => libfabric_sys::fi_threading_FI_THREAD_UNSPEC,
            Threading::SAFE => libfabric_sys::fi_threading_FI_THREAD_SAFE,
            Threading::FID => libfabric_sys::fi_threading_FI_THREAD_FID,
            Threading::DOMAIN => libfabric_sys::fi_threading_FI_THREAD_DOMAIN,
            Threading::COMPLETION => libfabric_sys::fi_threading_FI_THREAD_COMPLETION,
            Threading::ENDPOINT => libfabric_sys::fi_threading_FI_THREAD_ENDPOINT,
        }
    }
}

#[allow(non_camel_case_types)]
pub enum ResourceMgmt {
    UNSPEC,
    DISABLED,
    ENABLED,
}

impl ResourceMgmt {
    pub fn get_value(&self) -> libfabric_sys::fi_resource_mgmt {
        match self {
            ResourceMgmt::UNSPEC => libfabric_sys::fi_resource_mgmt_FI_RM_UNSPEC,
            ResourceMgmt::DISABLED => libfabric_sys::fi_resource_mgmt_FI_RM_DISABLED,
            ResourceMgmt::ENABLED => libfabric_sys::fi_resource_mgmt_FI_RM_ENABLED,
        }
    }
}

#[allow(non_camel_case_types)]
pub enum EpType {
    UNSPEC,
    MSG,
    DGRAM,
    RDM,
    SOCK_STREAM,
    SOCK_DGRAM,
}

impl EpType {
    pub fn get_value(&self) -> libfabric_sys::fi_ep_type {

        match self {
            EpType::UNSPEC => libfabric_sys::fi_ep_type_FI_EP_UNSPEC,
            EpType::MSG => libfabric_sys::fi_ep_type_FI_EP_MSG,
            EpType::DGRAM => libfabric_sys::fi_ep_type_FI_EP_DGRAM,
            EpType::RDM => libfabric_sys::fi_ep_type_FI_EP_RDM,
            EpType::SOCK_STREAM => libfabric_sys::fi_ep_type_FI_EP_SOCK_STREAM,
            EpType::SOCK_DGRAM => libfabric_sys::fi_ep_type_FI_EP_SOCK_DGRAM,
        }
    }
}

pub enum CounterEvents {
    COMP,
}

impl CounterEvents {
    pub fn get_value(&self) -> libfabric_sys::fi_cntr_events {
        match self {
            CounterEvents::COMP => libfabric_sys::fi_cntr_events_FI_CNTR_EVENTS_COMP,
        }
    }
}

#[allow(non_camel_case_types)]
pub enum TClass {
    UNSPEC,
    DSCP,
    LABEL,
    BEST_EFFORT,
    LOW_LATENCY,
    DEDICATED_ACCESS,
    BULK_DATA,
    SCAVENGER,
    NETWORK_CTRL,
}

impl TClass {
    pub fn get_value(&self) -> libfabric_sys::_bindgen_ty_5 {
        
        match self {
            TClass::UNSPEC => libfabric_sys::FI_TC_UNSPEC,
            TClass::DSCP => libfabric_sys::FI_TC_DSCP,
            TClass::LABEL => libfabric_sys::FI_TC_LABEL,
            TClass::BEST_EFFORT => libfabric_sys::FI_TC_BEST_EFFORT,
            TClass::LOW_LATENCY => libfabric_sys::FI_TC_LOW_LATENCY,
            TClass::DEDICATED_ACCESS => libfabric_sys::FI_TC_DEDICATED_ACCESS,
            TClass::BULK_DATA => libfabric_sys::FI_TC_BULK_DATA,
            TClass::SCAVENGER => libfabric_sys::FI_TC_SCAVENGER,
            TClass::NETWORK_CTRL => libfabric_sys::FI_TC_NETWORK_CTRL,
        }
    }
}

#[allow(non_camel_case_types)]
pub enum AddressFormat {
    UNSPEC,
    SOCKADDR,
    SOCKADDR_IN,
    SOCKADDR_IN6,
    SOCKADDR_IB,
    PSMX,
    GNI,
    BGQ,
    MLX,
    STR,
    PSMX2,
    IB_UD,
    EFA,
    PSMX3,
    OPX,
    CXI,
    UCX,
}

impl AddressFormat {
    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_3 {

        match self {
            AddressFormat::UNSPEC => libfabric_sys::FI_FORMAT_UNSPEC,
            AddressFormat::SOCKADDR => libfabric_sys::FI_SOCKADDR,
            AddressFormat::SOCKADDR_IN => libfabric_sys::FI_SOCKADDR_IN,
            AddressFormat::SOCKADDR_IN6 => libfabric_sys::FI_SOCKADDR_IN6,
            AddressFormat::SOCKADDR_IB => libfabric_sys::FI_SOCKADDR_IB,
            AddressFormat::PSMX => libfabric_sys::FI_ADDR_PSMX,
            AddressFormat::GNI => libfabric_sys::FI_ADDR_GNI,
            AddressFormat::BGQ => libfabric_sys::FI_ADDR_BGQ,
            AddressFormat::MLX => libfabric_sys::FI_ADDR_MLX,
            AddressFormat::STR => libfabric_sys::FI_ADDR_STR,
            AddressFormat::PSMX2 => libfabric_sys::FI_ADDR_PSMX2,
            AddressFormat::IB_UD => libfabric_sys::FI_ADDR_IB_UD,
            AddressFormat::EFA => libfabric_sys::FI_ADDR_EFA,
            AddressFormat::PSMX3 => libfabric_sys::FI_ADDR_PSMX3,
            AddressFormat::OPX => libfabric_sys::FI_ADDR_OPX,
            AddressFormat::CXI => libfabric_sys::FI_ADDR_CXI,
            AddressFormat::UCX => libfabric_sys::FI_ADDR_UCX,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
pub enum TransferOptions {
    COMMIT_COMPLETE,
    COMPLETION,
    DELIVERY_COMPLETE,
    INJECT,
    INJECT_COMPLETE,
    MULTICAST,
    MULTI_RECV,
    TRANSMIT_COMPLETE,
}

impl TransferOptions {
    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_3 {

        match self {
            TransferOptions::COMMIT_COMPLETE => libfabric_sys::FI_COMMIT_COMPLETE,
            TransferOptions::COMPLETION => libfabric_sys::FI_COMPLETION,
            TransferOptions::DELIVERY_COMPLETE => libfabric_sys::FI_DELIVERY_COMPLETE,
            TransferOptions::INJECT => libfabric_sys::FI_INJECT,
            TransferOptions::INJECT_COMPLETE => libfabric_sys::FI_INJECT_COMPLETE,
            TransferOptions::MULTICAST => libfabric_sys::FI_MULTICAST,
            TransferOptions::MULTI_RECV => libfabric_sys::FI_MULTI_RECV,
            TransferOptions::TRANSMIT_COMPLETE => libfabric_sys::FI_TRANSMIT_COMPLETE,
        }
    }    
}

#[allow(non_camel_case_types)]
pub enum Event {
    NOTIFY,
    CONNREQ,
    CONNECTED,
    SHUTDOWN,
    MR_COMPLETE,
    AV_COMPLETE,
    JOIN_COMPLETE,
}

impl Event{

    #[allow(dead_code)]
    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_18 {

        match self {
            Event::NOTIFY => libfabric_sys::FI_NOTIFY,
            Event::CONNREQ => libfabric_sys::FI_CONNREQ,
            Event::CONNECTED => libfabric_sys::FI_CONNECTED,
            Event::SHUTDOWN => libfabric_sys::FI_SHUTDOWN,
            Event::MR_COMPLETE => libfabric_sys::FI_MR_COMPLETE,
            Event::AV_COMPLETE => libfabric_sys::FI_AV_COMPLETE,
            Event::JOIN_COMPLETE => libfabric_sys::FI_JOIN_COMPLETE,
        }
    } 

    pub(crate) fn from_value(val: u32) -> Self {

        if val == libfabric_sys::FI_NOTIFY {
            Event::NOTIFY
        }
        else if  val == libfabric_sys::FI_CONNREQ {
            Event::CONNREQ
        }
        else if val == libfabric_sys::FI_CONNECTED {
            Event::CONNECTED
        }
        else if val == libfabric_sys::FI_SHUTDOWN {
            Event::SHUTDOWN
        }
        else if val == libfabric_sys::FI_MR_COMPLETE {
            Event::MR_COMPLETE
        }
        else if val == libfabric_sys::FI_AV_COMPLETE {
            Event::AV_COMPLETE
        }
        else if val == libfabric_sys::FI_JOIN_COMPLETE {
            Event::JOIN_COMPLETE
        }
        else {
            panic!("Unexpected value for Event")
        }
    }
}

#[allow(non_camel_case_types)]
pub enum ParamType {
    STRING,
    INT,
    BOOL,
    SIZE_T, 
}

impl ParamType {
    #[allow(dead_code)]
    pub(crate) fn get_value(&self) -> libfabric_sys::fi_param_type {

        match self {
            ParamType::STRING => libfabric_sys::fi_param_type_FI_PARAM_STRING,
            ParamType::INT => libfabric_sys::fi_param_type_FI_PARAM_INT,
            ParamType::BOOL => libfabric_sys::fi_param_type_FI_PARAM_BOOL,
            ParamType::SIZE_T => libfabric_sys::fi_param_type_FI_PARAM_SIZE_T,
        }
    }
}
