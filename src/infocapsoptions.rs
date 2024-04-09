use std::ops::Add;

use crate::InfoCapsImpl;

pub struct On;
pub struct Off;
#[derive(Clone)]
pub struct InfoCaps<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool, const MCAST: bool, const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool, const HMEM: bool, const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> {
}

type IsMsg = InfoCaps<true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false>  ;
type IsRma = InfoCaps<false, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false>  ;
type IsTag = InfoCaps<false, false, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false>  ;
type IsAtomic = InfoCaps<false, false, false, true, false, false, false, false, false, false, false, false, false, false, false, false, false> ;
type IsMcast = InfoCaps<false, false, false, false, true, false, false, false, false, false, false, false, false, false, false, false, false> ;
type IsNamedRxCtx = InfoCaps<false, false, true, false, false, true, false, false, false, false, false, false, false, false, false, false, false> ;
type IsDRecv = InfoCaps<false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false, false> ;
type IsVMsg = InfoCaps<false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false> ;
type IsHmem = InfoCaps<false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false> ;
type IsColl = InfoCaps<false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false> ;
type IsXpu = InfoCaps<false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false> ;

pub const IS_MSG: IsMsg = IsMsg::get(); 
pub const IS_RMA: IsRma = IsRma::get(); 
pub const IS_TAG: IsTag = IsTag::get(); 
pub const IS_ATOMIC: IsAtomic = IsAtomic::get(); 
pub const IS_MCAST: IsMcast = IsMcast::get(); 
pub const IS_NAMEDRXCTX: IsNamedRxCtx = IsNamedRxCtx::get(); 
pub const IS_DRECV: IsDRecv = IsDRecv::get(); 
pub const IS_VMSG: IsVMsg = IsVMsg::get(); 
pub const IS_HMEM: IsHmem = IsHmem::get(); 
pub const IS_COLL: IsColl = IsColl::get(); 
pub const IS_XPU: IsXpu = IsXpu::get(); 


impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsMsg> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsMsg) -> Self::Output {
        InfoCaps::<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsRma> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsRma) -> Self::Output {
        InfoCaps::<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsTag> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsTag) -> Self::Output {
        InfoCaps::<MSG, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsAtomic> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsAtomic) -> Self::Output {
        InfoCaps::<MSG, RMA, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsMcast> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, TAG, ATOMIC, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsMcast) -> Self::Output {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsNamedRxCtx> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, true, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsNamedRxCtx) -> Self::Output {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, true, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsDRecv> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, true, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsDRecv) -> Self::Output {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, true, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsVMsg> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsVMsg) -> Self::Output {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsHmem> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, true, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsHmem) -> Self::Output {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, true, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsColl> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, true, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsColl) -> Self::Output {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, true, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool>  Add<IsXpu> for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    type Output = InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, true, SEND, RECV, WRITE, READ, RWRITE, RREAD>;

    fn add(self, _rhs: IsXpu) -> Self::Output {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, true, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
}

pub trait MsgCap {}
pub trait RmaCap {}
pub trait TagCap {}
pub trait AtomicCap {}
pub trait McastCap {}
pub trait NamedRxCtxCap {}
pub trait DirRecvCap {}
pub trait VarMsgCap {}
pub trait HmemCap {}
pub trait CollCap {}
pub trait XpuCap {}
pub trait SendMod {}
pub trait RecvMod {}
pub trait WriteMod {}
pub trait ReadMod {}
pub trait RWriteMod {}
pub trait RReadMod {}

pub trait AtomicDefaultCap : AtomicCap + ReadMod + WriteMod {}
impl<T> AtomicDefaultCap for T where T: AtomicCap + ReadMod + WriteMod {}
pub trait AtomicReadOnlyCap : AtomicCap + ReadMod{}
impl<T> AtomicReadOnlyCap for T where T: AtomicCap + ReadMod {}

pub trait AtomicWriteOnlyCap : AtomicCap + WriteMod {}
impl<T> AtomicWriteOnlyCap for T where T: AtomicCap + WriteMod {}

pub trait MsgDefaultCap : MsgCap + SendMod + RecvMod {}
impl<T> MsgDefaultCap for T where T: MsgCap + SendMod + RecvMod {}

pub trait MsgRecvOnlyCap : MsgCap + RecvMod {}
impl<T> MsgRecvOnlyCap for T where T: MsgCap + RecvMod {}
pub trait MsgSendOnlyCap : MsgCap + SendMod {}
impl<T> MsgSendOnlyCap for T where T: MsgCap + SendMod {}

pub trait TagDefaultCap : TagCap + SendMod + RecvMod {}
impl<T> TagDefaultCap for T where T: TagCap + SendMod + RecvMod {}

pub trait TagRecvOnlyCap : TagCap + RecvMod {}
impl<T> TagRecvOnlyCap for T where T: TagCap + RecvMod {}

pub trait TagSendOnlyCap : TagCap + SendMod {}
impl<T> TagSendOnlyCap for T where T: TagCap + SendMod {}

pub trait RmaDefaultCap: RmaCap + ReadMod + WriteMod {}
impl<T> RmaDefaultCap for T where T: RmaCap + ReadMod + WriteMod{}

pub trait RmaReadOnlyCap: RmaCap + ReadMod {}
impl<T> RmaReadOnlyCap for T where T: RmaCap + ReadMod{}

pub trait RmaWriteOnlyCap: RmaCap + WriteMod {}
impl<T> RmaWriteOnlyCap for T where T: RmaCap + WriteMod{}



pub trait Caps {
    fn bitfield() -> u64;
    fn is_msg() -> bool;
    fn is_rma() -> bool;
    fn is_tagged() -> bool;
    fn is_atomic() -> bool;
    fn is_mcast() -> bool;
    fn is_named_rx_ctx() -> bool;
    fn is_directed_recv() -> bool;
    fn is_hmem() -> bool;
    fn is_collective() -> bool;
    fn is_xpu() -> bool;

}

impl<const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> MsgCap for InfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> RmaCap for InfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> TagCap for InfoCaps<MSG, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> AtomicCap for InfoCaps<MSG, RMA, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> McastCap for InfoCaps<MSG, RMA, TAG, ATOMIC, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> NamedRxCtxCap for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, true, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> DirRecvCap for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, true, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> VarMsgCap for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> HmemCap for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, true, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> CollCap for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, true, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> XpuCap for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, true, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> 
SendMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> 
SendMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, false, false, WRITE, READ, RWRITE, RREAD> {}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> 
WriteMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, true, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const RWRITE: bool, const RREAD: bool> 
WriteMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, false, false, RWRITE, RREAD> {}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const RWRITE: bool, const RREAD: bool> 
ReadMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, true, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const RWRITE: bool, const RREAD: bool> 
ReadMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, false, false, RWRITE, RREAD> {}


impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RREAD: bool> 
RWriteMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, true, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool> 
RWriteMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, false, false> {}

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool> 
RReadMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, true> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool> 
RReadMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, false, false> {}



impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const SEND: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> 
RecvMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> 
RecvMod for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, false, false, WRITE, READ, RWRITE, RREAD> {}


impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    
    pub fn inner() -> InfoCapsImpl {
        InfoCapsImpl::new()
        .msg_if(MSG)
        .rma_if(RMA)
        .tagged_if(TAG)
        .atomic_if(ATOMIC)
        .multicast_if(MCAST)
        .named_rx_ctx_if(NAMEDRXCTX)
        .directed_recv_if(DRECV)
        .variable_msg_if(VMSG)
        .hmem_if(HMEM)
        .collective_if(COLL)
        .send_if(SEND)
        .recv_if(RECV)
        .write_if(WRITE)
        .read_if(READ)
        .remote_write_if(RWRITE)
        .remote_read_if(RREAD)
    }
}

impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    const fn get() -> Self {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>{}
    }
}

impl InfoCaps<false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false> {
    pub const fn new() -> Self {
        InfoCaps::<false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false>{}
    }
}

impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
 
    pub fn msg(self) -> InfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_msg(&self) -> bool {MSG}
    
    pub fn rma(self) -> InfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }
    
    pub fn is_rma(&self) -> bool {RMA}
    
    pub fn tagged(self) -> InfoCaps<MSG, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_tagged(&self) -> bool {TAG}
    
    pub fn atomic(self) -> InfoCaps<MSG, RMA, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, RMA, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_atomic(&self) -> bool {ATOMIC}
    
    pub fn mcast(self) -> InfoCaps<MSG, RMA, TAG, ATOMIC, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_mcast(&self) -> bool {MCAST}
    
    pub fn named_rx_ctx(self) -> InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, true, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, true, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_named_rx_ctx(&self) -> bool {NAMEDRXCTX}
    
    pub fn directed_recv(self) -> InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, true, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, true, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_directed_recv(&self) -> bool {DRECV}
    
    pub fn hmem(self) -> InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, true, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, true, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_hmem(&self) -> bool {HMEM}
    
    pub fn collective(self) -> InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, true, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, true, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_collective(&self) -> bool {COLL}
    
    pub fn xpu(self) -> InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, true, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, true, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}
    }

    pub fn is_xpu(&self) -> bool {XPU}

}

impl<const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    pub fn send(self) -> InfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {}       
    }
    
    pub fn recv(self) -> InfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {}       
    }
}

impl<const RMA: bool, const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    
    pub fn send(self) -> InfoCaps<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {}       
    }
    
    pub fn recv(self) -> InfoCaps<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {}       
    }    
}

impl<const RMA: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<false, RMA, false, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    
    pub fn send(self) -> InfoCaps<false, RMA, false, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<false, RMA, false, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {}       
    }
    
    pub fn recv(self) -> InfoCaps<false, RMA, false, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<false, RMA, false, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {}       
    }    
}

impl<const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<false, false, false, false, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    
    pub fn send(self) ->  InfoCaps<false, false, false, false, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {
         InfoCaps::<false, false, false, false, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, true, RECV, WRITE, READ, RWRITE, RREAD> {}       
    }
    
    pub fn recv(self) ->  InfoCaps<false, false, false, false, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {
         InfoCaps::<false, false, false, false, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, true, WRITE, READ, RWRITE, RREAD> {}       
    }    
}

impl<const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, false, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    
    pub fn variable_msg(self) -> InfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}       
    }
}

impl<const RMA: bool, const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, false, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    
    pub fn variable_msg(self) -> InfoCaps<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
        InfoCaps::<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {}       
    }
}

impl<const MSG: bool, const TAG: bool, const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    
    pub fn write(self) -> InfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, true, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, true, READ, RWRITE, RREAD> {}       
    }
    
    pub fn read(self) -> InfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, true, RWRITE, RREAD> {
        InfoCaps::<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, true, RWRITE, RREAD> {}       
    }    

    pub fn remote_write(self, gen_event: bool) -> InfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, true, RREAD> {
        InfoCaps::<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, true, RREAD> {}       
    }
    
    pub fn remote_read(self, gen_event: bool) -> InfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, true> {
        InfoCaps::<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, true> {}       
    }    
}

impl<const MSG: bool, const TAG: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> InfoCaps<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD> {
    
    pub fn write(self) -> InfoCaps<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, true, READ, RWRITE, RREAD> {
        InfoCaps::<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, true, READ, RWRITE, RREAD> {}       
    }
    
    pub fn read(self) -> InfoCaps<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, true, RWRITE, RREAD> {
        InfoCaps::<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, true, RWRITE, RREAD> {}       
    }    
    
    pub fn remote_write(self, gen_event: bool) -> InfoCaps<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, true, RREAD>  {
        InfoCaps::<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, true, RREAD>  {}       
    }
    
    pub fn remote_read(self, gen_event: bool) -> InfoCaps<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, true>  {
        InfoCaps::<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, true>  {}       
    }    
}


impl Default for InfoCaps<false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false> {
    fn default() -> Self {
        Self::new()
    }
}

// pub type InfoCapsComm<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool, const COLL: bool> = InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, false, false, false, false, COLL, false>;

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool, const MCAST: bool, const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool, const HMEM: bool, const COLL: bool, const XPU: bool, const SEND: bool, const RECV: bool, const WRITE: bool, const READ: bool, const RWRITE: bool, const RREAD: bool> Caps for InfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>{
    fn bitfield() -> u64 {
        InfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD>::inner().bitfield
    }

    fn is_msg() -> bool {
        MSG
    }
    
    fn is_rma() -> bool {
        RMA
    }
    
    fn is_tagged() -> bool {
        TAG
    }
    
    fn is_atomic() -> bool {
        ATOMIC
    }
    
    fn is_mcast() -> bool {
        MCAST
    }
    
    fn is_named_rx_ctx() -> bool {
        NAMEDRXCTX
    }
    
    fn is_directed_recv() -> bool {
        DRECV
    }
    
    fn is_hmem() -> bool {
        HMEM
    }
    
    fn is_collective() -> bool {
        COLL
    }
    
    fn is_xpu() -> bool {
        XPU
    }
    
    // fn is_send() -> bool {
    //     SEND
    // }
    
    // fn is_recv() -> bool {
    //     RECV
    // }
    
    // fn is_write() -> bool {
    //     WRITE
    // }
    
    // fn is_read() -> bool {
    //     READ
    // }
}


