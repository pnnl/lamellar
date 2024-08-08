use std::os::fd::BorrowedFd;

use libfabric_sys::{FI_RECV, FI_TRANSMIT};

macro_rules! gen_enum {
    ($name: ident, $type_: ty, $(($var: ident, $val: path)),*) => {
        #[derive(Clone, Copy, Debug)]
        pub enum $name {
            $($var,)*
        }

        impl $name {
            #[allow(dead_code)]
            pub(crate) fn as_raw(&self) -> $type_ {
                match self {
                    $($name::$var => $val,)*
                }
            }

            #[allow(dead_code)]
            pub(crate) fn from_raw(value: $type_) -> $name {
                $(if value == $val {return $name::$var})*
                else {
                    panic!("Invalid value {}", value);
                }
            }
        }
    };
}

gen_enum!(Op, u32, 
    (Min, libfabric_sys::fi_op_FI_MIN),
    (Max, libfabric_sys::fi_op_FI_MAX),
    (Sum, libfabric_sys::fi_op_FI_SUM),
    (Prod, libfabric_sys::fi_op_FI_PROD),
    (Lor, libfabric_sys::fi_op_FI_LOR),
    (Land, libfabric_sys::fi_op_FI_LAND),
    (Bor, libfabric_sys::fi_op_FI_BOR),
    (Bar, libfabric_sys::fi_op_FI_BAND),
    (Lxor, libfabric_sys::fi_op_FI_LXOR),
    (Bxor, libfabric_sys::fi_op_FI_BXOR),
    (AtomicRead, libfabric_sys::fi_op_FI_ATOMIC_READ),
    (AtomicWrite, libfabric_sys::fi_op_FI_ATOMIC_WRITE),
    (Cswap, libfabric_sys::fi_op_FI_CSWAP),
    (CswapNe, libfabric_sys::fi_op_FI_CSWAP_NE),
    (CswapLe, libfabric_sys::fi_op_FI_CSWAP_LE),
    (CswapLt, libfabric_sys::fi_op_FI_CSWAP_LT),
    (CswapGe, libfabric_sys::fi_op_FI_CSWAP_GE),
    (CswapGt, libfabric_sys::fi_op_FI_CSWAP_GT),
    (Mswap, libfabric_sys::fi_op_FI_MSWAP),
    (AtomicOpLast, libfabric_sys::fi_op_FI_ATOMIC_OP_LAST),
    (Noop, libfabric_sys::fi_op_FI_NOOP)
);


gen_enum!(CollectiveOp, u32,
    (Barrier,libfabric_sys::fi_collective_op_FI_BARRIER),
    (Broadcast,libfabric_sys::fi_collective_op_FI_BROADCAST),
    (AllToAll,libfabric_sys::fi_collective_op_FI_ALLTOALL),
    (AllReduce,libfabric_sys::fi_collective_op_FI_ALLREDUCE),
    (AllGather,libfabric_sys::fi_collective_op_FI_ALLGATHER),
    (ReduceScatter,libfabric_sys::fi_collective_op_FI_REDUCE_SCATTER),
    (Reduce,libfabric_sys::fi_collective_op_FI_REDUCE),
    (Scatter,libfabric_sys::fi_collective_op_FI_SCATTER),
    (Gather,libfabric_sys::fi_collective_op_FI_GATHER)
);

gen_enum!(CqFormat, u32,
    (Unspec,libfabric_sys::fi_cq_format_FI_CQ_FORMAT_UNSPEC),
    (Context,libfabric_sys::fi_cq_format_FI_CQ_FORMAT_CONTEXT),
    (Msg,libfabric_sys::fi_cq_format_FI_CQ_FORMAT_MSG),
    (Data,libfabric_sys::fi_cq_format_FI_CQ_FORMAT_DATA),
    (Tagged,libfabric_sys::fi_cq_format_FI_CQ_FORMAT_TAGGED)
);

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

    pub(crate) fn as_raw(&self) -> u32 {
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

gen_enum!(WaitObj2, u32, 
    (Unspec, libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC),
    (Fd, libfabric_sys::fi_wait_obj_FI_WAIT_FD),
    (MutexCond, libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND),
    (Yield, libfabric_sys::fi_wait_obj_FI_WAIT_YIELD),
    (PollFd, libfabric_sys::fi_wait_obj_FI_WAIT_POLLFD)
);

gen_enum!(WaitCond, u32, 
    (None, libfabric_sys::fi_cq_wait_cond_FI_CQ_COND_NONE),
    (Threshold, libfabric_sys::fi_cq_wait_cond_FI_CQ_COND_THRESHOLD)
);

gen_enum!(HmemIface, libfabric_sys::fi_hmem_iface, 
    (System, libfabric_sys::fi_hmem_iface_FI_HMEM_SYSTEM), 
    (Cuda, libfabric_sys::fi_hmem_iface_FI_HMEM_CUDA), 
    (Rocr, libfabric_sys::fi_hmem_iface_FI_HMEM_ROCR), 
    (Ze, libfabric_sys::fi_hmem_iface_FI_HMEM_ZE), 
    (Neuron, libfabric_sys::fi_hmem_iface_FI_HMEM_NEURON), 
    (SynapseAi, libfabric_sys::fi_hmem_iface_FI_HMEM_SYNAPSEAI)
);

gen_enum!(EndpointOptName, libfabric_sys::_bindgen_ty_20, 
    (MinMultiRecv, libfabric_sys::FI_OPT_MIN_MULTI_RECV),
    (CmDataSize, libfabric_sys::FI_OPT_CM_DATA_SIZE),
    (BufferedMin, libfabric_sys::FI_OPT_BUFFERED_MIN),
    (BufferedLimit, libfabric_sys::FI_OPT_BUFFERED_LIMIT),
    (SendBufSize, libfabric_sys::FI_OPT_SEND_BUF_SIZE),
    (RecvBufSize, libfabric_sys::FI_OPT_RECV_BUF_SIZE),
    (TxSize, libfabric_sys::FI_OPT_TX_SIZE),
    (RxSize, libfabric_sys::FI_OPT_RX_SIZE),
    (FiHmemP2p, libfabric_sys::FI_OPT_FI_HMEM_P2P),
    (XpuTrigger, libfabric_sys::FI_OPT_XPU_TRIGGER),
    (CudaApiPermitted, libfabric_sys::FI_OPT_CUDA_API_PERMITTED)
);

gen_enum!(EndpointOptLevel, libfabric_sys::_bindgen_ty_19, 
    (Endpoint, libfabric_sys::FI_OPT_ENDPOINT)
);

gen_enum!(EndpointType, libfabric_sys::fi_ep_type, 
    (Unspec, libfabric_sys::fi_ep_type_FI_EP_UNSPEC),
    (Msg, libfabric_sys::fi_ep_type_FI_EP_MSG),
    (Dgram, libfabric_sys::fi_ep_type_FI_EP_DGRAM),
    (Rdm, libfabric_sys::fi_ep_type_FI_EP_RDM),
    (SockStream, libfabric_sys::fi_ep_type_FI_EP_SOCK_STREAM),
    (SockDgram, libfabric_sys::fi_ep_type_FI_EP_SOCK_DGRAM)
);

gen_enum!(HmemP2p, libfabric_sys::_bindgen_ty_21, 
    (Enabled,libfabric_sys::FI_HMEM_P2P_ENABLED),
    (Required,libfabric_sys::FI_HMEM_P2P_REQUIRED),
    (Preferred,libfabric_sys::FI_HMEM_P2P_PREFERRED),
    (Disabled,libfabric_sys::FI_HMEM_P2P_DISABLED)
);

gen_enum!(ControlOpt, libfabric_sys::_bindgen_ty_7,
    (GetFidFlag, libfabric_sys::FI_GETFIDFLAG),
    (SetFidFlag, libfabric_sys::FI_SETFIDFLAG),
    (GetOpsFlag, libfabric_sys::FI_GETOPSFLAG),
    (SetOpsFlag, libfabric_sys::FI_SETOPSFLAG),
    (Alias, libfabric_sys::FI_ALIAS),
    (GetWait, libfabric_sys::FI_GETWAIT),
    (Enable, libfabric_sys::FI_ENABLE),
    (Backlog, libfabric_sys::FI_BACKLOG),
    (GetRawMr, libfabric_sys::FI_GET_RAW_MR),
    (MapRawMr, libfabric_sys::FI_MAP_RAW_MR),
    (UnmapKey, libfabric_sys::FI_UNMAP_KEY),
    (QueueWork, libfabric_sys::FI_QUEUE_WORK),
    (CancelWork, libfabric_sys::FI_CANCEL_WORK),
    (FlushWork, libfabric_sys::FI_FLUSH_WORK),
    (Refresh, libfabric_sys::FI_REFRESH),
    (Dup, libfabric_sys::FI_DUP),
    (GetWaitObj, libfabric_sys::FI_GETWAITOBJ),
    (GetVal, libfabric_sys::FI_GET_VAL),
    (SetVal, libfabric_sys::FI_SET_VAL),
    (ExportFid, libfabric_sys::FI_EXPORT_FID)
);

gen_enum!(AddressVectorType, libfabric_sys::fi_av_type,
    (Unspec,libfabric_sys::fi_av_type_FI_AV_UNSPEC),
    (Map,libfabric_sys::fi_av_type_FI_AV_MAP),
    (Table,libfabric_sys::fi_av_type_FI_AV_TABLE)
);

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

#[derive(Clone, Copy, Debug)]
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

    pub(crate) fn from_raw(val: u64) -> Self {
        Self {
            c_flags: val,
        }
    }

    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
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
}

impl Into<u64> for Mode {
    fn into(self) -> u64 {
        self.c_flags
    }
}


impl Default for Mode {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct MrMode {
    c_flags: u32
}

impl MrMode {
    
    pub fn new() -> Self {
        Self {c_flags: 0}
    }

    pub(crate) fn from_raw(value: u32) -> Self {
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

    pub fn as_raw(&self) -> u32 {
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
    pub(crate) fn from_raw(value: u32) -> Self {
        Self {c_flags: value}
    }

    gen_set_get_flag!(send, is_send, libfabric_sys::FI_SEND);
    gen_set_get_flag!(recv, is_recv, libfabric_sys::FI_RECV);
    gen_set_get_flag!(read, is_read, libfabric_sys::FI_READ);
    gen_set_get_flag!(write, is_write, libfabric_sys::FI_WRITE);
    gen_set_get_flag!(remote_read, is_remote_read, libfabric_sys::FI_REMOTE_READ);
    gen_set_get_flag!(remote_write, is_remote_write, libfabric_sys::FI_REMOTE_WRITE);
    gen_set_get_flag!(collective, is_collective, libfabric_sys::FI_COLLECTIVE);

    pub fn as_raw(&self) -> u32 {
        self.c_flags
    }
}

impl Default for MrAccess {
    fn default() -> Self {
        Self::new()
    }
}

gen_enum!(Progress, libfabric_sys::fi_progress,
    (Unspec, libfabric_sys::fi_progress_FI_PROGRESS_UNSPEC),
    (Auto, libfabric_sys::fi_progress_FI_PROGRESS_AUTO),
    (Manual, libfabric_sys::fi_progress_FI_PROGRESS_MANUAL)
);

gen_enum!(Threading, libfabric_sys::fi_threading,
    (Unspec, libfabric_sys::fi_threading_FI_THREAD_UNSPEC),
    (Safe, libfabric_sys::fi_threading_FI_THREAD_SAFE),
    (Fid, libfabric_sys::fi_threading_FI_THREAD_FID),
    (Domain, libfabric_sys::fi_threading_FI_THREAD_DOMAIN),
    (Completion, libfabric_sys::fi_threading_FI_THREAD_COMPLETION),
    (Endpoint, libfabric_sys::fi_threading_FI_THREAD_ENDPOINT)
);

gen_enum!(ResourceMgmt, libfabric_sys::fi_resource_mgmt,
    (Unspec, libfabric_sys::fi_resource_mgmt_FI_RM_UNSPEC),
    (Disabled, libfabric_sys::fi_resource_mgmt_FI_RM_DISABLED),
    (Enabled, libfabric_sys::fi_resource_mgmt_FI_RM_ENABLED)
);

gen_enum!(CounterEvents, libfabric_sys::fi_cntr_events,
    (Comp, libfabric_sys::fi_cntr_events_FI_CNTR_EVENTS_COMP)
);

gen_enum!(TrafficClass, libfabric_sys::_bindgen_ty_5, 
    (Unspec,libfabric_sys::FI_TC_UNSPEC),
    (Dscp,libfabric_sys::FI_TC_DSCP),
    (Label,libfabric_sys::FI_TC_LABEL),
    (BestEffort,libfabric_sys::FI_TC_BEST_EFFORT),
    (LowLatency,libfabric_sys::FI_TC_LOW_LATENCY),
    (DedicatedAccess,libfabric_sys::FI_TC_DEDICATED_ACCESS),
    (BulkData,libfabric_sys::FI_TC_BULK_DATA),
    (Scavenger,libfabric_sys::FI_TC_SCAVENGER),
    (NetworkCtrl,libfabric_sys::FI_TC_NETWORK_CTRL)
);

gen_enum!(AddressFormat, libfabric_sys::_bindgen_ty_3,
    (Unspec, libfabric_sys::FI_FORMAT_UNSPEC),
    (SockAddr, libfabric_sys::FI_SOCKADDR),
    (SockaddrIn, libfabric_sys::FI_SOCKADDR_IN),
    (SockaddrIn6, libfabric_sys::FI_SOCKADDR_IN6),
    (SockaddrIb, libfabric_sys::FI_SOCKADDR_IB),
    (Psmx, libfabric_sys::FI_ADDR_PSMX),
    (Gni, libfabric_sys::FI_ADDR_GNI),
    (Bgq, libfabric_sys::FI_ADDR_BGQ),
    (Mlx, libfabric_sys::FI_ADDR_MLX),
    (Str, libfabric_sys::FI_ADDR_STR),
    (Psmx2, libfabric_sys::FI_ADDR_PSMX2),
    (IbUd, libfabric_sys::FI_ADDR_IB_UD),
    (Efa, libfabric_sys::FI_ADDR_EFA),
    (Psmx3, libfabric_sys::FI_ADDR_PSMX3),
    (Opx, libfabric_sys::FI_ADDR_OPX),
    (Cxi, libfabric_sys::FI_ADDR_CXI),
    (Ucx, libfabric_sys::FI_ADDR_UCX)
);

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

    pub(crate) fn as_raw(&self) -> u64 {
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

    pub(crate) fn as_raw(&self) -> u64 {
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

#[derive(Clone,Copy, Debug)]
pub struct TransferOptions {
    c_flags: u32,
}

impl TransferOptions {
    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(val: u32) -> Self {
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

    pub(crate) fn as_raw(&self) -> libfabric_sys::_bindgen_ty_3 {
        self.c_flags
    }
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self::new()
    }
}

gen_enum!(ParamType, libfabric_sys::fi_param_type,
    (String, libfabric_sys::fi_param_type_FI_PARAM_STRING),
    (Int, libfabric_sys::fi_param_type_FI_PARAM_INT),
    (Bool, libfabric_sys::fi_param_type_FI_PARAM_BOOL),
    (SizeT, libfabric_sys::fi_param_type_FI_PARAM_SIZE_T)
);

gen_enum!(Protocol, libfabric_sys::_bindgen_ty_4, 

    (Unspec, libfabric_sys::FI_PROTO_UNSPEC),
    (RdmaCmIbRc, libfabric_sys::FI_PROTO_RDMA_CM_IB_RC),
    (Iwarp, libfabric_sys::FI_PROTO_IWARP),
    (IbUd, libfabric_sys::FI_PROTO_IB_UD),
    (Psmx, libfabric_sys::FI_PROTO_PSMX),
    (Udp, libfabric_sys::FI_PROTO_UDP),
    (SockTcp, libfabric_sys::FI_PROTO_SOCK_TCP),
    (Mxm, libfabric_sys::FI_PROTO_MXM),
    (IwarpRdm, libfabric_sys::FI_PROTO_IWARP_RDM),
    (IbRdm, libfabric_sys::FI_PROTO_IB_RDM),
    (Gni, libfabric_sys::FI_PROTO_GNI),
    (Rxm, libfabric_sys::FI_PROTO_RXM),
    (Rxd, libfabric_sys::FI_PROTO_RXD),
    (Mlx, libfabric_sys::FI_PROTO_MLX),
    (NetworkDirect, libfabric_sys::FI_PROTO_NETWORKDIRECT),
    (Psmx2, libfabric_sys::FI_PROTO_PSMX2),
    (Shm, libfabric_sys::FI_PROTO_SHM),
    (Mrail, libfabric_sys::FI_PROTO_MRAIL),
    (Rstream, libfabric_sys::FI_PROTO_RSTREAM),
    (RdmaCmIbXrc, libfabric_sys::FI_PROTO_RDMA_CM_IB_XRC),
    (Efa, libfabric_sys::FI_PROTO_EFA),
    (Psmx3, libfabric_sys::FI_PROTO_PSMX3),
    (RxmTcp, libfabric_sys::FI_PROTO_RXM_TCP),
    (Opx, libfabric_sys::FI_PROTO_OPX),
    (Cxi, libfabric_sys::FI_PROTO_CXI),
    (Xnet, libfabric_sys::FI_PROTO_XNET),
    (Coll, libfabric_sys::FI_PROTO_COLL),
    (Ucx, libfabric_sys::FI_PROTO_UCX),
    (Sm2, libfabric_sys::FI_PROTO_SM2)
);

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

#[derive(Clone)]
pub struct DomainCaps {
    c_flags: u64,
}

impl DomainCaps {
    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    pub(crate) fn from_raw(value: u64) -> Self {
        DomainCaps {
            c_flags: value,
        }
    }   

    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }   

    gen_set_get_flag!(directed_recv, is_directed_recv, libfabric_sys::FI_DIRECTED_RECV);
    gen_set_get_flag!(av_user_id, is_av_user_id, libfabric_sys::FI_AV_USER_ID);
    gen_set_get_flag!(local_comm, is_local_comm, libfabric_sys::FI_LOCAL_COMM);
    gen_set_get_flag!(remote_comm, is_remote_comm, libfabric_sys::FI_REMOTE_COMM);
    gen_set_get_flag!(shared_av, is_shared_av, libfabric_sys::FI_SHARED_AV);
}


impl Into<u64> for DomainCaps {
    fn into(self) -> u64 {
        self.c_flags
    }
}

pub struct CompletionFlags {
    c_flags: u64,
}

impl CompletionFlags {
    pub(crate) fn from_raw(c_flags: u64) -> Self {
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
    
    pub(crate) fn as_raw(&self) -> u64 {
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

    pub(crate) fn as_raw(&self) -> u64 {
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