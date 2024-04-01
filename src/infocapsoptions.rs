// pub struct NewInfoCaps<MSG, TAG, RMA, ATOMIC, MCAST, COLL, READ, WRITE, RECV, SEND, TRANSMIT, RREAD, RWRITE, RMAEVT> {
//     msg: PhantomData<MSG>,
//     tag: PhantomData<TAG>,
//     rma: PhantomData<RMA>,
//     atomic: PhantomData<ATOMIC>,
//     mcast: PhantomData<MCAST>,
//     coll: PhantomData<COLL>,
//     read: PhantomData<READ>,
//     write: PhantomData<WRITE>,
//     recv: PhantomData<RECV>,
//     send: PhantomData<SEND>,
//     tsmit: PhantomData<TRANSMIT>,
//     rread: PhantomData<RREAD>,
//     rwrite: PhantomData<RWRITE>,
//     rma_event: PhantomData<RMAEVT>,
// }

use crate::InfoCaps;

pub struct On;
pub struct Off;
#[derive(Clone)]
pub struct NewInfoCaps<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool, const MCAST: bool, const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool, const HMEM: bool, const COLL: bool, const XPU: bool> {
    modifiers: InfoCaps,
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
pub trait Capabilities {
    fn get_bitfield() -> u64;
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

impl<const RMA: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool> MsgCap for NewInfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {}
impl<const MSG: bool, const TAG: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool> RmaCap for NewInfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {}
impl<const MSG: bool, const RMA: bool, const ATOMIC: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool> TagCap for NewInfoCaps<MSG, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const MCAST: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool> AtomicCap for NewInfoCaps<MSG, RMA, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const NAMEDRXCTX: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool> McastCap for NewInfoCaps<MSG, RMA, TAG, ATOMIC, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const DRECV: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool> NamedRxCtxCap for NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, true, DRECV, VMSG, HMEM, COLL, XPU> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const VMSG: bool, const HMEM: bool , const COLL: bool , const XPU: bool> DirRecvCap for NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, true, VMSG, HMEM, COLL, XPU> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const HMEM: bool , const COLL: bool , const XPU: bool> VarMsgCap for NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const COLL: bool , const XPU: bool> HmemCap for NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, true, COLL, XPU> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const XPU: bool> CollCap for NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, true, XPU> {}
impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool> XpuCap for NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, true> {}

impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
    
    pub fn get() -> InfoCaps {
        InfoCaps::new()
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
    }
}
impl NewInfoCaps<false, false, false, false, false, false, false, false, false, false, false> {
    pub fn new() -> Self {
        NewInfoCaps::<false, false, false, false, false, false, false, false, false, false, false>{modifiers: InfoCaps::new()}
    }
}

impl<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
 
    pub fn msg(self) -> NewInfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
        NewInfoCaps::<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {modifiers: self.modifiers}
    }

    pub fn is_msg(&self) -> bool {MSG}
    
    pub fn rma(self) -> NewInfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
        NewInfoCaps::<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {modifiers: self.modifiers}
    }
    
    pub fn is_rma(&self) -> bool {RMA}
    
    pub fn tagged(self) -> NewInfoCaps<MSG, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
        NewInfoCaps::<MSG, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {modifiers: self.modifiers}
    }

    pub fn is_tagged(&self) -> bool {TAG}
    
    pub fn atomic(self) -> NewInfoCaps<MSG, RMA, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
        NewInfoCaps::<MSG, RMA, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {modifiers: self.modifiers}
    }

    pub fn is_atomic(&self) -> bool {ATOMIC}
    
    pub fn mcast(self) -> NewInfoCaps<MSG, RMA, TAG, ATOMIC, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
        NewInfoCaps::<MSG, RMA, TAG, ATOMIC, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {modifiers: self.modifiers}
    }

    pub fn is_mcast(&self) -> bool {MCAST}
    
    pub fn named_rx_ctx(self) -> NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, true, DRECV, VMSG, HMEM, COLL, XPU> {
        NewInfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, true, DRECV, VMSG, HMEM, COLL, XPU> {modifiers: self.modifiers}
    }

    pub fn is_named_rx_ctx(&self) -> bool {NAMEDRXCTX}
    
    pub fn directed_recv(self) -> NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, true, VMSG, HMEM, COLL, XPU> {
        NewInfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, true, VMSG, HMEM, COLL, XPU> {modifiers: self.modifiers}
    }

    pub fn is_directed_recv(&self) -> bool {DRECV}
    
    pub fn hmem(self) -> NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, true, COLL, XPU> {
        NewInfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, true, COLL, XPU> {modifiers: self.modifiers}
    }

    pub fn is_hmem(&self) -> bool {HMEM}
    
    pub fn collective(self) -> NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, true, XPU> {
        NewInfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, true, XPU> {modifiers: self.modifiers}
    }

    pub fn is_collective(&self) -> bool {COLL}
    
    pub fn xpu(self) -> NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, true> {
        NewInfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, true> {modifiers: self.modifiers}
    }

    pub fn is_xpu(&self) -> bool {XPU}

}

impl<const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
    pub fn send(self) -> Self {
        Self {
            modifiers: self.modifiers.send()
        }       
    }
    
    pub fn recv(self) -> Self {
        Self {
            modifiers: self.modifiers.recv()
        }       
    }
}

impl<const RMA: bool, const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
    
    pub fn send(self) -> Self {
        Self {
            modifiers: self.modifiers.send()
        }       
    }
    
    pub fn recv(self) -> Self {
        Self {
            modifiers: self.modifiers.recv()
        }       
    }    
}

impl<const RMA: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<false, RMA, false, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
    
    pub fn send(self) -> Self {
        Self {
            modifiers: self.modifiers.send()
        }       
    }
    
    pub fn recv(self) -> Self {
        Self {
            modifiers: self.modifiers.recv()
        }       
    }    
}

impl<const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<false, false, false, false, true, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
    
    pub fn send(self) -> Self {
        Self {
            modifiers: self.modifiers.send()
        }       
    }
    
    pub fn recv(self) -> Self {
        Self {
            modifiers: self.modifiers.recv()
        }       
    }    
}

impl<const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, false, HMEM, COLL, XPU> {
    
    pub fn variable_msg(self) -> NewInfoCaps<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU> {
        NewInfoCaps::<true, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU> {
            modifiers: self.modifiers
        }       
    }
}

impl<const RMA: bool, const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, false, HMEM, COLL, XPU> {
    
    pub fn variable_msg(self) -> NewInfoCaps<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU> {
        NewInfoCaps::<false, RMA, true, ATOMIC, MCAST, NAMEDRXCTX, DRECV, true, HMEM, COLL, XPU> {
            modifiers: self.modifiers
        }       
    }
}

impl<const MSG: bool, const TAG: bool, const ATOMIC: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<MSG, true, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
    
    pub fn write(self) -> Self {
        Self {
            modifiers: self.modifiers.write()
        }       
    }
    
    pub fn read(self) -> Self {
        Self {
            modifiers: self.modifiers.read()
        }       
    }    

    pub fn remote_write(self, gen_event: bool) -> Self {
        Self {
            modifiers: if gen_event {self.modifiers.rma_event().remote_write()} else { self.modifiers.remote_write()}
        }       
    }
    
    pub fn remote_read(self, gen_event: bool) -> Self {
        Self {
            modifiers: if gen_event {self.modifiers.rma_event().remote_read()} else { self.modifiers.remote_write()}
        }       
    }    
}

impl<const MSG: bool, const TAG: bool, const MCAST: bool , const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool , const HMEM: bool , const COLL: bool, const XPU: bool> NewInfoCaps<MSG, false, TAG, true, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU> {
    
    pub fn write(self) -> Self {
        Self {
            modifiers: self.modifiers.write()
        }       
    }
    
    pub fn read(self) -> Self {
        Self {
            modifiers: self.modifiers.read()
        }       
    }    
    
    pub fn remote_write(self, gen_event: bool) -> Self {
        Self {
            modifiers: if gen_event {self.modifiers.rma_event().remote_write()} else { self.modifiers.remote_write()}
        }       
    }
    
    pub fn remote_read(self, gen_event: bool) -> Self {
        Self {
            modifiers: if gen_event {self.modifiers.rma_event().remote_read()} else { self.modifiers.remote_write()}
        }       
    }    
}


impl Default for NewInfoCaps<false, false, false, false, false, false, false, false, false, false, false> {
    fn default() -> Self {
        Self::new()
    }
}

pub type InfoCapsComm<const MSG: bool, const RMA: bool, const TAG: bool , const ATOMIC: bool, const MCAST: bool, const COLL: bool> = NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, false, false, false, false, COLL, false>;

impl<const MSG: bool, const RMA: bool, const TAG: bool, const ATOMIC: bool, const MCAST: bool, const NAMEDRXCTX: bool, const DRECV: bool, const VMSG: bool, const HMEM: bool, const COLL: bool, const XPU: bool> Capabilities for NewInfoCaps<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU>{
    fn get_bitfield() -> u64 {
        NewInfoCaps::<MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU>::get().bitfield
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
}

// macro_rules! from_info_caps {
//     () => {
        
//     };
// }

// impl Default for NewInfoCaps<Off,Off,Off,Off,Off,Off,Off,Off,Off,Off,Off> {
//     fn default() -> Self {
//         Self::new()
//     }
// }


