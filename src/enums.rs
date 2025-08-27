use std::os::fd::BorrowedFd;

use libfabric_sys::{FI_RECV, FI_TRANSMIT};

macro_rules! gen_enum {
    ($(#[$attr:meta])* $name: ident, $type_: ty, $(($var: ident, $val: expr)),*) => {
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

gen_enum!(
    /// A traffic class for network packets.
    Dscp,
    u8,
    (CS0 ,0),
    (CS1 ,8),
    (CS2 ,16),
    (CS3 ,24),
    (CS4 ,32),
    (CS5 ,40),
    (CS6 ,48),
    (CS7 ,56),
    (AF11, 10),
    (AF12, 12),
    (AF13, 14),
    (AF21, 18),
    (AF22, 20),
    (AF23, 22),
    (AF31, 26),
    (AF32, 28),
    (AF33, 30),
    (AF41, 34),
    (AF42, 36),
    (AF43, 38),
    (EF, 46),
    (VoiceAdmit, 44),
    (LowerEffort, 1)
);


impl From<TrafficClass> for Dscp {
    fn from(tc: TrafficClass) -> Self {
        unsafe {Self::from_raw(libfabric_sys::inlined_fi_tc_dscp_get(tc.as_raw()))}
    }
}

impl From<Dscp> for TrafficClass {
    fn from(dscp: Dscp) -> Self {
        unsafe {Self::from_raw(libfabric_sys::inlined_fi_tc_dscp_set(dscp.as_raw()))}
    }
}

gen_enum!(
    /// An enumeration of atomic operations.
    AtomicOp,
    u32,
    (Min, libfabric_sys::fi_op_FI_MIN),
    (Max, libfabric_sys::fi_op_FI_MAX),
    (Sum, libfabric_sys::fi_op_FI_SUM),
    (Prod, libfabric_sys::fi_op_FI_PROD),
    (Lor, libfabric_sys::fi_op_FI_LOR),
    (Land, libfabric_sys::fi_op_FI_LAND),
    (Bor, libfabric_sys::fi_op_FI_BOR),
    (Band, libfabric_sys::fi_op_FI_BAND),
    (Lxor, libfabric_sys::fi_op_FI_LXOR),
    (Bxor, libfabric_sys::fi_op_FI_BXOR),
    (AtomicWrite, libfabric_sys::fi_op_FI_ATOMIC_WRITE)
);
gen_enum!(
    /// An enumeration of fetch-and-operate atomic operations.
    FetchAtomicOp,
    u32,
    (Min, libfabric_sys::fi_op_FI_MIN),
    (Max, libfabric_sys::fi_op_FI_MAX),
    (Sum, libfabric_sys::fi_op_FI_SUM),
    (Prod, libfabric_sys::fi_op_FI_PROD),
    (Lor, libfabric_sys::fi_op_FI_LOR),
    (Land, libfabric_sys::fi_op_FI_LAND),
    (Bor, libfabric_sys::fi_op_FI_BOR),
    (Band, libfabric_sys::fi_op_FI_BAND),
    (Lxor, libfabric_sys::fi_op_FI_LXOR),
    (Bxor, libfabric_sys::fi_op_FI_BXOR),
    (AtomicRead, libfabric_sys::fi_op_FI_ATOMIC_READ),
    (AtomicWrite, libfabric_sys::fi_op_FI_ATOMIC_WRITE)
);

gen_enum!(
    /// An enumeration of compare-and-swap atomic operations.
    CompareAtomicOp,
    u32,
    (Cswap, libfabric_sys::fi_op_FI_CSWAP),
    (CswapNe, libfabric_sys::fi_op_FI_CSWAP_NE),
    (CswapLe, libfabric_sys::fi_op_FI_CSWAP_LE),
    (CswapLt, libfabric_sys::fi_op_FI_CSWAP_LT),
    (CswapGe, libfabric_sys::fi_op_FI_CSWAP_GE),
    (CswapGt, libfabric_sys::fi_op_FI_CSWAP_GT),
    (Mswap, libfabric_sys::fi_op_FI_MSWAP)
);

gen_enum!(
    /// An enumeration of collective atomic operations. Can be the operation of a reduce for example
    CollAtomicOp,
    u32,
    (Min, libfabric_sys::fi_op_FI_MIN),
    (Max, libfabric_sys::fi_op_FI_MAX),
    (Sum, libfabric_sys::fi_op_FI_SUM),
    (Prod, libfabric_sys::fi_op_FI_PROD),
    (Lor, libfabric_sys::fi_op_FI_LOR),
    (Land, libfabric_sys::fi_op_FI_LAND),
    (Bor, libfabric_sys::fi_op_FI_BOR),
    (Band, libfabric_sys::fi_op_FI_BAND),
    (Lxor, libfabric_sys::fi_op_FI_LXOR),
    (Bxor, libfabric_sys::fi_op_FI_BXOR),
    (AtomicWrite, libfabric_sys::fi_op_FI_ATOMIC_WRITE),
    (AtomicRead, libfabric_sys::fi_op_FI_ATOMIC_READ),
    (Noop, libfabric_sys::fi_op_FI_NOOP)
);

/// A trait for atomic operations that can be converted to their raw representation.
/// 
/// Used as a bound for functions that accept atomic operations.
pub trait AtomicOperation {
    fn as_raw(&self) -> u32;
}

impl AtomicOperation for AtomicOp {
    fn as_raw(&self) -> u32 {
        self.as_raw()
    }
}

impl AtomicOperation for FetchAtomicOp {
    fn as_raw(&self) -> u32 {
        self.as_raw()
    }
}
impl AtomicOperation for CompareAtomicOp {
    fn as_raw(&self) -> u32 {
        self.as_raw()
    }
}

// impl AtomicOperation for CompareAtomicOp {
//     fn as_raw(&self) -> u32 {
//         self.as_raw()
//     }
// }

gen_enum!(
    /// An enumeration of collective operations.
    CollectiveOp,
    u32,
    (Barrier, libfabric_sys::fi_collective_op_FI_BARRIER),
    (Broadcast, libfabric_sys::fi_collective_op_FI_BROADCAST),
    (AllToAll, libfabric_sys::fi_collective_op_FI_ALLTOALL),
    (AllReduce, libfabric_sys::fi_collective_op_FI_ALLREDUCE),
    (AllGather, libfabric_sys::fi_collective_op_FI_ALLGATHER),
    (
        ReduceScatter,
        libfabric_sys::fi_collective_op_FI_REDUCE_SCATTER
    ),
    (Reduce, libfabric_sys::fi_collective_op_FI_REDUCE),
    (Scatter, libfabric_sys::fi_collective_op_FI_SCATTER),
    (Gather, libfabric_sys::fi_collective_op_FI_GATHER)
);

gen_enum!(
    /// An enumeration of completion queue formats.
    CqFormat,
    u32,
    (Unspec, libfabric_sys::fi_cq_format_FI_CQ_FORMAT_UNSPEC),
    (Context, libfabric_sys::fi_cq_format_FI_CQ_FORMAT_CONTEXT),
    (Msg, libfabric_sys::fi_cq_format_FI_CQ_FORMAT_MSG),
    (Data, libfabric_sys::fi_cq_format_FI_CQ_FORMAT_DATA),
    (Tagged, libfabric_sys::fi_cq_format_FI_CQ_FORMAT_TAGGED)
);

gen_enum!(
    /// An enumeration of primitive data types.
    /// Corresponds to `fi_datatype` in libfabric.
    DataType,
    libfabric_sys::fi_datatype,
    (Int8, libfabric_sys::fi_datatype_FI_INT8),
    (Uint8, libfabric_sys::fi_datatype_FI_UINT8),
    (Int16, libfabric_sys::fi_datatype_FI_INT16),
    (Uint16, libfabric_sys::fi_datatype_FI_UINT16),
    (Int32, libfabric_sys::fi_datatype_FI_INT32),
    (Uint32, libfabric_sys::fi_datatype_FI_UINT32),
    (Int64, libfabric_sys::fi_datatype_FI_INT64),
    (Uint64, libfabric_sys::fi_datatype_FI_UINT64),
    (Float, libfabric_sys::fi_datatype_FI_FLOAT),
    (Double, libfabric_sys::fi_datatype_FI_DOUBLE),
    (FloatComplex, libfabric_sys::fi_datatype_FI_FLOAT_COMPLEX),
    (DoubleComplex, libfabric_sys::fi_datatype_FI_DOUBLE_COMPLEX),
    (LongDouble, libfabric_sys::fi_datatype_FI_LONG_DOUBLE),
    (
        LongDoubleComplex,
        libfabric_sys::fi_datatype_FI_LONG_DOUBLE_COMPLEX
    ),
    (DatatypeLast, libfabric_sys::fi_datatype_FI_DATATYPE_LAST),
    (Int128, libfabric_sys::fi_datatype_FI_INT128),
    (Uint28, libfabric_sys::fi_datatype_FI_UINT128),
    (Void, libfabric_sys::fi_datatype_FI_VOID)
);

gen_enum!(
    /// An enumeration of type.
    /// Corresponds to `fi_type` in libfabric.
    Type,
    libfabric_sys::fi_type,
    (Info, libfabric_sys::fi_type_FI_TYPE_INFO),
    (EpType, libfabric_sys::fi_type_FI_TYPE_EP_TYPE),
    (Caps, libfabric_sys::fi_type_FI_TYPE_CAPS),
    (OpFlags, libfabric_sys::fi_type_FI_TYPE_OP_FLAGS),
    (AddrFormat, libfabric_sys::fi_type_FI_TYPE_ADDR_FORMAT),
    (TxAttr, libfabric_sys::fi_type_FI_TYPE_TX_ATTR),
    (RxAttr, libfabric_sys::fi_type_FI_TYPE_RX_ATTR),
    (EpAttr, libfabric_sys::fi_type_FI_TYPE_EP_ATTR),
    (DomainAttr, libfabric_sys::fi_type_FI_TYPE_DOMAIN_ATTR),
    (FabricAttr, libfabric_sys::fi_type_FI_TYPE_FABRIC_ATTR),
    (Threading, libfabric_sys::fi_type_FI_TYPE_THREADING),
    (Progress, libfabric_sys::fi_type_FI_TYPE_PROGRESS),
    (Protocol, libfabric_sys::fi_type_FI_TYPE_PROTOCOL),
    (MsgOrder, libfabric_sys::fi_type_FI_TYPE_MSG_ORDER),
    (Mode, libfabric_sys::fi_type_FI_TYPE_MODE),
    (AvType, libfabric_sys::fi_type_FI_TYPE_AV_TYPE),
    (AtomicType, libfabric_sys::fi_type_FI_TYPE_ATOMIC_TYPE),
    (AtomicOp, libfabric_sys::fi_type_FI_TYPE_ATOMIC_OP),
    (Version, libfabric_sys::fi_type_FI_TYPE_VERSION),
    (EqEvent, libfabric_sys::fi_type_FI_TYPE_EQ_EVENT),
    (CqEventFlags, libfabric_sys::fi_type_FI_TYPE_CQ_EVENT_FLAGS),
    (MrMode, libfabric_sys::fi_type_FI_TYPE_MR_MODE),
    (OpType, libfabric_sys::fi_type_FI_TYPE_OP_TYPE),
    (Fid, libfabric_sys::fi_type_FI_TYPE_FID),
    (CollectiveOp, libfabric_sys::fi_type_FI_TYPE_COLLECTIVE_OP),
    (HmemIface, libfabric_sys::fi_type_FI_TYPE_HMEM_IFACE),
    (CqFormat, libfabric_sys::fi_type_FI_TYPE_CQ_FORMAT),
    (LogLevel, libfabric_sys::fi_type_FI_TYPE_LOG_LEVEL),
    (LogSubsys, libfabric_sys::fi_type_FI_TYPE_LOG_SUBSYS),
    (AvAttr, libfabric_sys::fi_type_FI_TYPE_AV_ATTR),
    (CqAttr, libfabric_sys::fi_type_FI_TYPE_CQ_ATTR),
    (MrAttr, libfabric_sys::fi_type_FI_TYPE_MR_ATTR),
    (CntrAttr, libfabric_sys::fi_type_FI_TYPE_CNTR_ATTR),
    (CqErrEntry, libfabric_sys::fi_type_FI_TYPE_CQ_ERR_ENTRY)
);

/// A profile data type, which can be either a primitive data type or a defined type.
pub enum ProfileDataType {
    Primitive(DataType),
    Defined(Type),
}

#[derive(Copy, Clone)]
/// An enumeration of wait object types.
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

gen_enum!(
    WaitObj2,
    u32,
    (Unspec, libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC),
    (Fd, libfabric_sys::fi_wait_obj_FI_WAIT_FD),
    (MutexCond, libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND),
    (Yield, libfabric_sys::fi_wait_obj_FI_WAIT_YIELD),
    (PollFd, libfabric_sys::fi_wait_obj_FI_WAIT_POLLFD)
);

gen_enum!(
    /// An enumeration of wait conditions for completion queues.
    WaitCond,
    u32,
    (None, libfabric_sys::fi_cq_wait_cond_FI_CQ_COND_NONE),
    (
        Threshold,
        libfabric_sys::fi_cq_wait_cond_FI_CQ_COND_THRESHOLD
    )
);

/// An enumeration of memory registration interfaces.
#[derive(Clone, Copy, Debug)]
pub enum HmemIface {
    System,
    Cuda(i32),
    Rocr(i32),
    Ze(i32, i32),
    Neuron(i32),
    SynapseAi(i32),
}
impl HmemIface {
    #[allow(dead_code)]
    pub(crate) fn as_raw(&self) -> libfabric_sys::fi_hmem_iface {
        match self {
            HmemIface::System => libfabric_sys::fi_hmem_iface_FI_HMEM_SYSTEM,
            HmemIface::Cuda(_) => libfabric_sys::fi_hmem_iface_FI_HMEM_CUDA,
            HmemIface::Rocr(_) => libfabric_sys::fi_hmem_iface_FI_HMEM_ROCR,
            HmemIface::Ze(_, _) => libfabric_sys::fi_hmem_iface_FI_HMEM_ZE,
            HmemIface::Neuron(_) => libfabric_sys::fi_hmem_iface_FI_HMEM_NEURON,
            HmemIface::SynapseAi(_) => libfabric_sys::fi_hmem_iface_FI_HMEM_SYNAPSEAI,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(
        value: libfabric_sys::fi_hmem_iface,
        id: i32,
        additional_id: i32,
    ) -> HmemIface {
        if value == libfabric_sys::fi_hmem_iface_FI_HMEM_SYSTEM {
            HmemIface::System
        } else if value == libfabric_sys::fi_hmem_iface_FI_HMEM_CUDA {
            HmemIface::Cuda(id)
        } else if value == libfabric_sys::fi_hmem_iface_FI_HMEM_ROCR {
            HmemIface::Rocr(id)
        } else if value == libfabric_sys::fi_hmem_iface_FI_HMEM_ZE {
            HmemIface::Ze(id, additional_id)
        } else if value == libfabric_sys::fi_hmem_iface_FI_HMEM_NEURON {
            HmemIface::Neuron(id)
        } else if value == libfabric_sys::fi_hmem_iface_FI_HMEM_SYNAPSEAI {
            HmemIface::SynapseAi(id)
        } else {
            panic!("Invalid value {}", value);
        }
    }
}

gen_enum!(
    /// An enumeration of endpoint options.
    /// Corresponds to `FI_OPT_` values in libfabric.
    EndpointOptName,
    libfabric_sys::_bindgen_ty_20,
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

gen_enum!(
    /// An enumeration of endpoint option levels.
    /// 
    /// Corresponds to `FI_OPT_ENDPOINT` in libfabric.
    EndpointOptLevel,
    libfabric_sys::_bindgen_ty_19,
    (Endpoint, libfabric_sys::FI_OPT_ENDPOINT)
);

gen_enum!(
    /// An enumeration of endpoint types.
    /// 
    /// Corresponds to `fi_ep_type` in libfabric.
    EndpointType,
    libfabric_sys::fi_ep_type,
    (Unspec, libfabric_sys::fi_ep_type_FI_EP_UNSPEC),
    (Msg, libfabric_sys::fi_ep_type_FI_EP_MSG),
    (Dgram, libfabric_sys::fi_ep_type_FI_EP_DGRAM),
    (Rdm, libfabric_sys::fi_ep_type_FI_EP_RDM)
);

gen_enum!(
    /// An enumeration of host memory peer-to-peer support levels.
    /// 
    /// Corresponds to `FI_HMEM_P2P_` values in libfabric.
    HmemP2p,
    libfabric_sys::_bindgen_ty_21,
    (Enabled, libfabric_sys::FI_HMEM_P2P_ENABLED),
    (Required, libfabric_sys::FI_HMEM_P2P_REQUIRED),
    (Preferred, libfabric_sys::FI_HMEM_P2P_PREFERRED),
    (Disabled, libfabric_sys::FI_HMEM_P2P_DISABLED)
);

gen_enum!(
    ControlOpt,
    libfabric_sys::_bindgen_ty_7,
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

gen_enum!(
    /// An enumeration of address vector types.
    AddressVectorType,
    libfabric_sys::fi_av_type,
    (Unspec, libfabric_sys::fi_av_type_FI_AV_UNSPEC),
    (Map, libfabric_sys::fi_av_type_FI_AV_MAP),
    (Table, libfabric_sys::fi_av_type_FI_AV_TABLE)
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

use crate::trigger::{TriggerThreshold, TriggerXpu};

#[derive(Clone, Copy, Debug)]
/// Represents the mode flags for an endpoint, domain, or fabric.
/// 
/// Corresponds to `mode` field in different structs in libfabric.
pub struct Mode {
    c_flags: u64,
}

impl Mode {
    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    pub fn all() -> Self {
        Self { c_flags: !0 }
    }

    pub(crate) fn from_raw(val: u64) -> Self {
        Self { c_flags: val }
    }

    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    gen_set_get_flag!(context, is_context, libfabric_sys::FI_CONTEXT);
    gen_set_get_flag!(msg_prefix, is_msg_prefix, libfabric_sys::FI_MSG_PREFIX);
    gen_set_get_flag!(async_iov, is_async_iov, libfabric_sys::FI_ASYNC_IOV);
    gen_set_get_flag!(rx_cq_data, is_rx_cq_data, libfabric_sys::FI_RX_CQ_DATA);
    gen_set_get_flag!(local_mr, is_local_mr, libfabric_sys::FI_LOCAL_MR);
    gen_set_get_flag!(
        #[deprecated]
        notify_flags_only,
        #[deprecated]
        is_notify_flags_only,
        libfabric_sys::FI_NOTIFY_FLAGS_ONLY as u64
    );
    gen_set_get_flag!(
        #[deprecated]
        restricted_comp,
        #[deprecated]
        is_restricted_comp,
        libfabric_sys::FI_RESTRICTED_COMP as u64
    );
    gen_set_get_flag!(context2, is_context2, libfabric_sys::FI_CONTEXT2);
    gen_set_get_flag!(
        buffered_recv,
        is_buffered_recv,
        libfabric_sys::FI_BUFFERED_RECV
    );
}

impl From<Mode> for u64 {
    fn from(val: Mode) -> Self {
        val.c_flags
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::new()
    }
}


///  Memory registration mode flags.
/// 
/// Corresponds to `FI_MR_` values in libfabric.
#[derive(Clone, Copy, Debug)]
pub struct MrMode {
    c_flags: u32,
}

impl MrMode {
    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    pub(crate) fn from_raw(value: u32) -> Self {
        Self { c_flags: value }
    }

    pub fn is_unspec(&self) -> bool {
        self.c_flags == libfabric_sys::fi_mr_mode_FI_MR_UNSPEC
    }

    pub fn inverse(mut self) -> Self {
        self.c_flags = !self.c_flags;

        self
    }

    gen_set_get_flag!(basic, is_basic, libfabric_sys::fi_mr_mode_FI_MR_BASIC);
    gen_set_get_flag!(
        scalable,
        is_scalable,
        libfabric_sys::fi_mr_mode_FI_MR_SCALABLE
    );
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

#[derive(Clone, Copy, Debug)]
/// Memory registration options.
pub struct MrRegOpt {
    c_flags: u64,
}

impl MrRegOpt {
    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(value: u64) -> Self {
        Self { c_flags: value }
    }

    pub fn as_raw(&self) -> u64 {
        self.c_flags
    }

    gen_set_get_flag!(rma_event, is_rma_event, libfabric_sys::FI_RMA_EVENT);
    gen_set_get_flag!(rma_pmem, is_rma_pmem, libfabric_sys::FI_RMA_PMEM);
    gen_set_get_flag!(
        hmem_device_only,
        is_hmem_device_only,
        libfabric_sys::FI_HMEM_DEVICE_ONLY
    );
    gen_set_get_flag!(
        hmem_host_alloc,
        is_hmem_host_alloc,
        libfabric_sys::FI_HMEM_HOST_ALLOC
    );
}

impl Default for MrRegOpt {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
/// Memory registration access flags.
pub struct MrAccess {
    c_flags: u32,
}

impl MrAccess {
    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(value: u32) -> Self {
        Self { c_flags: value }
    }

    gen_set_get_flag!(send, is_send, libfabric_sys::FI_SEND);
    gen_set_get_flag!(recv, is_recv, libfabric_sys::FI_RECV);
    gen_set_get_flag!(read, is_read, libfabric_sys::FI_READ);
    gen_set_get_flag!(write, is_write, libfabric_sys::FI_WRITE);
    gen_set_get_flag!(remote_read, is_remote_read, libfabric_sys::FI_REMOTE_READ);
    gen_set_get_flag!(
        remote_write,
        is_remote_write,
        libfabric_sys::FI_REMOTE_WRITE
    );
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
gen_enum!(
    /// An enumeration of authentication key types.
    UserId, 
    u64, 
    (AuthKey, libfabric_sys::FI_AUTH_KEY)
);

gen_enum!(
    /// An enumeration of progress modes.
    Progress,
    libfabric_sys::fi_progress,
    (Unspec, libfabric_sys::fi_progress_FI_PROGRESS_UNSPEC),
    (Auto, libfabric_sys::fi_progress_FI_PROGRESS_AUTO),
    (Manual, libfabric_sys::fi_progress_FI_PROGRESS_MANUAL)
);

gen_enum!(
    /// An enumeration of threading models.
    Threading,
    libfabric_sys::fi_threading,
    (Unspec, libfabric_sys::fi_threading_FI_THREAD_UNSPEC),
    (Safe, libfabric_sys::fi_threading_FI_THREAD_SAFE),
    (Fid, libfabric_sys::fi_threading_FI_THREAD_FID),
    (Domain, libfabric_sys::fi_threading_FI_THREAD_DOMAIN),
    (Completion, libfabric_sys::fi_threading_FI_THREAD_COMPLETION),
    (Endpoint, libfabric_sys::fi_threading_FI_THREAD_ENDPOINT)
);

gen_enum!(
    /// An enumeration of resource management modes.
    ResourceMgmt,
    libfabric_sys::fi_resource_mgmt,
    (Unspec, libfabric_sys::fi_resource_mgmt_FI_RM_UNSPEC),
    (Disabled, libfabric_sys::fi_resource_mgmt_FI_RM_DISABLED),
    (Enabled, libfabric_sys::fi_resource_mgmt_FI_RM_ENABLED)
);

gen_enum!(
    /// An enumeration of counter event types.
    CounterEvents,
    libfabric_sys::fi_cntr_events,
    (Comp, libfabric_sys::fi_cntr_events_FI_CNTR_EVENTS_COMP),
    (Bytes, libfabric_sys::fi_cntr_events_FI_CNTR_EVENTS_BYTES)
);

gen_enum!(
    /// An enumeration of traffic classes for network packets.
    TrafficClass,
    libfabric_sys::_bindgen_ty_5,
    (Unspec, libfabric_sys::FI_TC_UNSPEC),
    (Dscp, libfabric_sys::FI_TC_DSCP),
    (Label, libfabric_sys::FI_TC_LABEL),
    (BestEffort, libfabric_sys::FI_TC_BEST_EFFORT),
    (LowLatency, libfabric_sys::FI_TC_LOW_LATENCY),
    (DedicatedAccess, libfabric_sys::FI_TC_DEDICATED_ACCESS),
    (BulkData, libfabric_sys::FI_TC_BULK_DATA),
    (Scavenger, libfabric_sys::FI_TC_SCAVENGER),
    (NetworkCtrl, libfabric_sys::FI_TC_NETWORK_CTRL)
);

gen_enum!(
    /// An enumeration of address formats.
    AddressFormat,
    libfabric_sys::_bindgen_ty_3,
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

#[derive(Clone)]
/// An enumeration of trigger events.
pub enum TriggerEvent<'a> {
    Threshold(TriggerThreshold<'a>),
    Xpu(TriggerXpu),
}

impl<'a> TriggerEvent<'a> {
    pub fn as_raw(
        &mut self,
    ) -> (
        libfabric_sys::fi_trigger_event,
        libfabric_sys::fi_triggered_context__bindgen_ty_1,
    ) {
        match self {
            Self::Threshold(thold) => (
                libfabric_sys::fi_trigger_event_FI_TRIGGER_THRESHOLD,
                libfabric_sys::fi_triggered_context__bindgen_ty_1 {
                    threshold: thold.c_thold,
                },
            ),
            Self::Xpu(xpu) => (
                libfabric_sys::fi_trigger_event_FI_TRIGGER_XPU,
                libfabric_sys::fi_triggered_context__bindgen_ty_1 { xpu: xpu.as_raw() },
            ),
        }
    }

    pub fn as_raw2(
        &mut self,
    ) -> (
        libfabric_sys::fi_trigger_event,
        libfabric_sys::fi_triggered_context2__bindgen_ty_1,
    ) {
        match self {
            Self::Threshold(thold) => (
                libfabric_sys::fi_trigger_event_FI_TRIGGER_THRESHOLD,
                libfabric_sys::fi_triggered_context2__bindgen_ty_1 {
                    threshold: thold.c_thold,
                },
            ),
            Self::Xpu(xpu) => (
                libfabric_sys::fi_trigger_event_FI_TRIGGER_XPU,
                libfabric_sys::fi_triggered_context2__bindgen_ty_1 { xpu: xpu.as_raw() },
            ),
        }
    }
}
//     (Threshold, libfabric_sys::fi_trigger_event_FI_TRIGGER_THRESHOLD),
//     (Xpu, libfabric_sys::fi_trigger_event_FI_TRIGGER_XPU)
// );

#[derive(Clone, Copy, Debug)]
/// Address vector options.
pub struct AVOptions {
    c_flags: u64,
}

impl AVOptions {
    /// Create a new [AVOptions] object with the default configuration.
    pub fn new() -> Self {
        Self { c_flags: 0 }
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
pub struct TferOptions<
    const OUT: bool,
    const MSG: bool,
    const RMA: bool,
    const DATA: bool,
    const TAGGED: bool,
    const ATOMIC: bool,
> {
    c_flags: u64,
}

impl<
        const OUT: bool,
        const MSG: bool,
        const RMA: bool,
        const DATA: bool,
        const TAGGED: bool,
        const ATOMIC: bool,
    > TferOptions<OUT, MSG, RMA, DATA, TAGGED, ATOMIC>
{
    // All transfer types
    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    gen_set_get_flag!(
        completion,
        is_completion,
        libfabric_sys::FI_COMPLETION as u64
    );
    gen_set_get_flag!(more, is_more, libfabric_sys::FI_MORE as u64);
}

impl<
        const OUT: bool,
        const MSG: bool,
        const RMA: bool,
        const DATA: bool,
        const TAGGED: bool,
        const ATOMIC: bool,
    > Default for TferOptions<OUT, MSG, RMA, DATA, TAGGED, ATOMIC>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const OUT: bool, const MSG: bool> TferOptions<OUT, MSG, false, false, false, true> {
    // All atomic transfers
    gen_set_get_flag!(tagged, is_tagged, libfabric_sys::FI_TAGGED as u64);
}

impl<const OUT: bool> TferOptions<OUT, true, false, false, false, true> {
    // All atomic msg transfers
    gen_set_get_flag!(inject, is_inject, libfabric_sys::FI_INJECT as u64);
}

impl<
        const MSG: bool,
        const RMA: bool,
        const DATA: bool,
        const TAGGED: bool,
        const ATOMIC: bool,
    > TferOptions<true, MSG, RMA, DATA, TAGGED, ATOMIC>
{
    // All transmits
    gen_set_get_flag!(fence, is_fence, libfabric_sys::FI_FENCE as u64);
}

impl<const RMA: bool, const DATA: bool, const TAGGED: bool>
    TferOptions<true, true, RMA, DATA, TAGGED, false>
{
    // Only data transmits (no msg)
    gen_set_get_flag!(
        remote_cq_data,
        is_remote_cq_data,
        libfabric_sys::FI_REMOTE_CQ_DATA as u64
    );
}

impl<const RMA: bool, const TAGGED: bool> TferOptions<true, true, RMA, false, TAGGED, false> {
    // Only msg transmits (no data)
    gen_set_get_flag!(inject, is_inject, libfabric_sys::FI_INJECT as u64);
    gen_set_get_flag!(
        inject_complete,
        is_inject_complete,
        libfabric_sys::FI_INJECT_COMPLETE as u64
    );
    gen_set_get_flag!(
        transmit_complete,
        is_transmit_complete,
        libfabric_sys::FI_TRANSMIT_COMPLETE as u64
    );
    gen_set_get_flag!(
        delivery_complete,
        is_delivery_complete,
        libfabric_sys::FI_DELIVERY_COMPLETE as u64
    );
    // gen_set_get_flag!(remote_cq_data, is_remote_cq_data, libfabric_sys::FI_REMOTE_CQ_DATA as u64);
}

impl TferOptions<true, true, false, false, true, false> {
    // Only tagged msg transmits (no data)
    gen_set_get_flag!(
        match_complete,
        is_match_complete,
        libfabric_sys::FI_MATCH_COMPLETE as u64
    );
}

impl TferOptions<true, true, true, false, false, false> {
    // Only RMA msg transmits (no data)
    gen_set_get_flag!(
        commit_complete,
        is_commit_complete,
        libfabric_sys::FI_COMMIT_COMPLETE as u64
    );
}

impl<const MSG: bool, const DATA: bool> TferOptions<true, MSG, false, DATA, false, false> {
    // Non-RMA or Tagged transmits
    gen_set_get_flag!(multicast, is_multicast, libfabric_sys::FI_MULTICAST as u64);
}

impl<const MSG: bool> TferOptions<false, MSG, false, false, false, false> {
    // All Posted Receive Operations (i.e. recv, recvmsg)
    gen_set_get_flag!(claim, is_claim, libfabric_sys::FI_CLAIM);
    gen_set_get_flag!(discard, is_discard, libfabric_sys::FI_DISCARD);
    gen_set_get_flag!(
        multi_recv,
        is_multi_recv,
        libfabric_sys::FI_MULTI_RECV as u64
    );
    gen_set_get_flag!(auth_key, is_auth_key, libfabric_sys::FI_AUTH_KEY);
}

impl TferOptions<false, true, false, false, true, false> {
    // Only tagged Posted Receive Operations
    gen_set_get_flag!(peek, is_peek, libfabric_sys::FI_PEEK as u64);
    gen_set_get_flag!(claim, is_claim, libfabric_sys::FI_CLAIM);
    gen_set_get_flag!(discard, is_discard, libfabric_sys::FI_DISCARD);
}

// pub type SendOptions = TferOptions<true, false, false, false, false>;
/// Send message options
pub type SendMsgOptions = TferOptions<true, true, false, false, false, false>;
// pub type SendDataOptions = TferOptions<true, true, false, true, false>;
// pub type TaggedSendOptions = TferOptions<true, false, false, false, true>;
/// Tagged send message options
pub type TaggedSendMsgOptions = TferOptions<true, true, false, false, true, false>;
// pub type TaggedSendDataOptions = TferOptions<true, false, false, true, true>;

// pub type WriteOptions = TferOptions<true, false, true, false, false>;
/// Write message options
pub type WriteMsgOptions = TferOptions<true, true, true, false, false, false>;
// pub type WriteDataOptions = TferOptions<true, false, true, true, false>;

// pub type RecvOptions = TferOptions<false, false, false, false, false>;
/// Receive message options
pub type RecvMsgOptions = TferOptions<false, true, false, false, false, false>;
// pub type TaggedRecvOptions = TferOptions<false, false, false, false, true>;
/// Tagged receive message options
pub type TaggedRecvMsgOptions = TferOptions<false, true, false, false, true, false>;

// pub type ReadOptions = TferOptions<false, false, true, false, false>;
/// Read message options
pub type ReadMsgOptions = TferOptions<false, true, true, false, false, false>;

// pub type AtomicOptions = TferOptions<true, false, true, false, false, true>;
/// Atomic message options
pub type AtomicMsgOptions = TferOptions<true, true, false, false, false, true>;

// pub type AtomicFetchOptions = TferOptions<true, false, true, false, false, true>;
/// Atomic fetch message options
pub type AtomicFetchMsgOptions = TferOptions<true, true, false, false, false, true>;

/// Collective message options
pub type CollectiveOptions = AtomicMsgOptions;

#[derive(Clone, Copy, Debug)]
pub struct TransferOptions {
    c_flags: u32,
}

impl TransferOptions {
    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    #[allow(dead_code)]
    pub(crate) fn from_raw(val: u32) -> Self {
        Self { c_flags: val }
    }

    pub(crate) fn transmit(mut self) -> Self {
        self.c_flags |= FI_TRANSMIT;
        self
    }

    pub(crate) fn recv(mut self) -> Self {
        self.c_flags |= FI_RECV;
        self
    }

    gen_set_get_flag!(
        commit_complete,
        is_commit_complete,
        libfabric_sys::FI_COMMIT_COMPLETE
    );
    gen_set_get_flag!(completion, is_completion, libfabric_sys::FI_COMPLETION);
    gen_set_get_flag!(
        delivery_complete,
        is_delivery_complete,
        libfabric_sys::FI_DELIVERY_COMPLETE
    );
    gen_set_get_flag!(inject, is_inject, libfabric_sys::FI_INJECT);
    gen_set_get_flag!(
        inject_complete,
        is_inject_complete,
        libfabric_sys::FI_INJECT_COMPLETE
    );
    gen_set_get_flag!(multicast, is_multicast, libfabric_sys::FI_MULTICAST);
    gen_set_get_flag!(multi_recv, is_multi_recv, libfabric_sys::FI_MULTI_RECV);
    gen_set_get_flag!(
        transmit_complete,
        is_transmit_complete,
        libfabric_sys::FI_TRANSMIT_COMPLETE
    );

    pub(crate) fn as_raw(&self) -> libfabric_sys::_bindgen_ty_3 {
        self.c_flags
    }
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self::new()
    }
}

gen_enum!(
    /// An enumeration of parameter types.
    ParamType,
    libfabric_sys::fi_param_type,
    (String, libfabric_sys::fi_param_type_FI_PARAM_STRING),
    (Int, libfabric_sys::fi_param_type_FI_PARAM_INT),
    (Bool, libfabric_sys::fi_param_type_FI_PARAM_BOOL),
    (SizeT, libfabric_sys::fi_param_type_FI_PARAM_SIZE_T)
);

gen_enum!(
    /// An enumeration of protocol types.
    Protocol,
    libfabric_sys::_bindgen_ty_4,
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
    (CxiRnNR, libfabric_sys::FI_PROTO_CXI_RNR),
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

/// Domain capability flags.
#[derive(Clone, Debug)]
pub struct DomainCaps {
    c_flags: u64,
}

impl DomainCaps {
    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    pub(crate) fn from_raw(value: u64) -> Self {
        DomainCaps { c_flags: value }
    }

    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    gen_set_get_flag!(
        directed_recv,
        is_directed_recv,
        libfabric_sys::FI_DIRECTED_RECV
    );
    gen_set_get_flag!(av_user_id, is_av_user_id, libfabric_sys::FI_AV_USER_ID);
    gen_set_get_flag!(local_comm, is_local_comm, libfabric_sys::FI_LOCAL_COMM);
    gen_set_get_flag!(remote_comm, is_remote_comm, libfabric_sys::FI_REMOTE_COMM);
    gen_set_get_flag!(shared_av, is_shared_av, libfabric_sys::FI_SHARED_AV);
}

impl From<DomainCaps> for u64 {
    fn from(val: DomainCaps) -> Self {
        val.c_flags
    }
}

/// Completion flags for operations.
#[derive(Clone, Copy, Debug)]
pub struct CompletionFlags {
    c_flags: u64,
}

impl CompletionFlags {
    pub(crate) fn from_raw(c_flags: u64) -> Self {
        Self { c_flags }
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

#[derive(Clone, Copy, Debug)]
/// Address vector set options.
pub struct AVSetOptions {
    c_flags: u64,
}

impl AVSetOptions {
    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    gen_set_get_flag!(universe, is_universe, libfabric_sys::FI_UNIVERSE);
    gen_set_get_flag!(barrier_set, is_barrier_set, libfabric_sys::FI_BARRIER_SET);
    gen_set_get_flag!(
        broadcast_set,
        is_broadcast_set,
        libfabric_sys::FI_BROADCAST_SET
    );
    gen_set_get_flag!(
        alltoall_set,
        is_alltoall_set,
        libfabric_sys::FI_ALLTOALL_SET
    );
    gen_set_get_flag!(
        allreduce_set,
        is_allreduce_set,
        libfabric_sys::FI_ALLREDUCE_SET
    );
    gen_set_get_flag!(
        allgather_set,
        is_allgather_set,
        libfabric_sys::FI_ALLGATHER_SET
    );
    gen_set_get_flag!(
        reduce_scatter_set,
        is_reduce_scatter_set,
        libfabric_sys::FI_REDUCE_SCATTER_SET
    );
    gen_set_get_flag!(reduce_set, is_reduce_set, libfabric_sys::FI_REDUCE_SET);
    gen_set_get_flag!(scatter_set, is_scatter_set, libfabric_sys::FI_SCATTER_SET);
    gen_set_get_flag!(gather_set, is_gather_set, libfabric_sys::FI_GATHER_SET);
}

impl Default for AVSetOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
/// Options for joining a multicast group.
pub struct JoinOptions {
    c_flags: u64,
}

impl JoinOptions {
    pub fn new() -> Self {
        Self { c_flags: 0 }
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
