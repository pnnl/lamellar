use std::{marker::PhantomData, os::fd::BorrowedFd, rc::Rc, cell::RefCell};
use crate::{av::{AddressVector, AddressVectorImpl}, cntr::Counter, enums::{HmemP2p, TransferOptions}, ep::{BaseEndpointImpl, Endpoint, ActiveEndpointImpl, Address}, eq::{EventQueue, ReadEq}, fid::{AsFid, self, AsRawFid, OwnedEpFid, RawFid, AsTypedFid, EpRawFid, AsRawTypedFid}, BindImpl, cq::ReadCq};

pub struct Receive;
pub struct Transmit;

pub(crate) struct XContextBaseImpl<T> {
    pub(crate) c_ep: OwnedEpFid,
    phantom: PhantomData<T>,
    _sync_rcs: RefCell<Vec<Rc<dyn crate::BindImpl>>>,
}

pub struct XContextBase<T> {
    pub(crate) inner: Rc<XContextBaseImpl<T>>
}

impl<T: 'static> XContextBase<T> {

    pub fn getname<T0>(&self) -> Result<Address, crate::error::Error> {
        self.inner.getname()
    }

    pub fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        self.inner.buffered_limit()
    }

    pub fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        self.inner.set_buffered_limit(size)
    }

    pub fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        self.inner.buffered_min()
    }

    pub fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        self.inner.set_buffered_min(size)
    }

    pub fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        self.inner.cm_data_size()
    }

    pub fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        self.inner.set_cm_data_size(size)
    }

    pub fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        self.inner.min_multi_recv()
    }

    pub fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        self.inner.set_min_multi_recv(size)
    }

    pub fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {
        self.inner.set_hmem_p2p(hmem)
    }

    pub fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        self.inner.hmem_p2p()
    }

    pub fn xpu_trigger(&self) -> Result<libfabric_sys::fi_trigger_xpu, crate::error::Error> {
        self.inner.xpu_trigger()
    }

    pub fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        self.inner.set_cuda_api_permitted(permitted)
    }

    pub fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        self.inner.wait_fd()
    }

    pub fn enable(&self) -> Result<(), crate::error::Error> {
        self.inner.enable()
    }

    pub fn cancel(&self) -> Result<(), crate::error::Error> {
        self.inner.cancel()
    }

    pub fn cancel_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        self.inner.cancel_with_context(context)
    }

    pub fn tx_size_left(&self) -> Result<usize, crate::error::Error> {
        self.inner.tx_size_left()
    }

    pub fn getpeer(&self) -> Result<Address, crate::error::Error> {
        self.inner.getpeer()
    }

    pub fn connect_with<P>(&self, addr: &Address, param: &[P]) -> Result<(), crate::error::Error> {
        self.inner.connect_with(addr, param)
    }

    pub fn connect(&self, addr: &Address) -> Result<(), crate::error::Error> {
        self.inner.connect(addr)
    }

    pub fn accept_with<P>(&self, param: &[P]) -> Result<(), crate::error::Error> {
        self.inner.accept_with(param)
    }

    pub fn accept(&self) -> Result<(), crate::error::Error> {
        self.inner.accept()
    }

    pub fn shutdown(&self) -> Result<(), crate::error::Error> {
        self.inner.shutdown()
    }
}

impl<T> ActiveEndpointImpl for XContextBase<T> {}

impl<T> ActiveEndpointImpl for XContextBaseImpl<T> {}

impl<T> BaseEndpointImpl for XContextBaseImpl<T> {}

impl<T> AsFid for XContextBase<T> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.inner.as_fid()
    }    
}

impl<T> AsFid for XContextBaseImpl<T> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.c_ep.as_fid()
    }    
}

impl<T> AsRawFid for XContextBase<T> {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }    
}

impl<T> AsRawFid for XContextBaseImpl<T> {
    fn as_raw_fid(&self) -> RawFid {
        self.c_ep.as_raw_fid()
    }    
}

impl<T> AsTypedFid<EpRawFid> for XContextBase<T> {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<EpRawFid>{
        self.inner.as_typed_fid()
    }
}

impl<T> AsTypedFid<EpRawFid> for XContextBaseImpl<T> {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<EpRawFid> {
        self.c_ep.as_typed_fid()
    }
}

impl<T> AsRawTypedFid for XContextBase<T> {
    type Output = EpRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl<T> AsRawTypedFid for XContextBaseImpl<T> {
    type Output = EpRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_ep.as_raw_typed_fid()
    }  
}

pub type TransmitContext = XContextBase<Transmit>; 
type TransmitContextImpl = XContextBaseImpl<Transmit>; 

impl TransmitContextImpl {

    pub(crate) fn new<T0>(ep: &impl ActiveEndpointImpl, index: i32, mut attr: TxAttr, context: Option<&mut T0>) -> Result<TransmitContextImpl, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let err = 
            if let Some(ctx) = context {
                unsafe{ libfabric_sys::inlined_fi_tx_context(ep.as_raw_typed_fid(), index, attr.get_mut(), &mut c_ep, (ctx as *mut T0).cast())}
            }
            else {
                unsafe{ libfabric_sys::inlined_fi_tx_context(ep.as_raw_typed_fid(), index, attr.get_mut(), &mut c_ep, std::ptr::null_mut())}
            };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    c_ep: OwnedEpFid::from(c_ep), 
                    phantom: PhantomData,
                    _sync_rcs: RefCell::new(Vec::new()),
                })
        }
    }

    pub(crate) fn bind<T: crate::BindImpl + AsFid + 'static>(&self, res: &Rc<T>, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.as_raw_typed_fid(), res.as_fid().as_raw_fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            self._sync_rcs.borrow_mut().push(res.clone());
            Ok(())
        }
    } 

    pub(crate) fn bind_cq(&self) -> TxIncompleteBindCq {
        TxIncompleteBindCq { ep: self, flags: 0}
    }

    pub(crate) fn bind_cntr(&self) -> TxIncompleteBindCntr {
        TxIncompleteBindCntr { ep: self, flags: 0}
    }

    pub(crate) fn bind_eq<EQ: BindImpl + fid::AsFid + 'static>(&self, eq: &Rc<EQ>) -> Result<(), crate::error::Error>  {
        
        self.bind(eq, 0)
    }

    pub(crate) fn bind_av(&self, av: &Rc<AddressVectorImpl>) -> Result<(), crate::error::Error> {
    
        self.bind(av, 0)
    }
}

impl TransmitContext {
    pub(crate) fn new<T0>(ep: &impl ActiveEndpointImpl, index: i32, attr: TxAttr, context: Option<&mut T0>) -> Result<TransmitContext, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new(TransmitContextImpl::new(ep, index, attr, context)?)
            }
        )
    }
    
    pub fn bind_cq(&self) -> TxIncompleteBindCq {
        self.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> TxIncompleteBindCntr {
        self.inner.bind_cntr()
    }

    pub fn bind_eq<T: ReadCq + BindImpl + 'static + fid::AsFid>(&self, eq: &EventQueue<T>) -> Result<(), crate::error::Error>  {
        self.inner.bind_eq(&eq.inner)
    }

    pub fn bind_av(&mut self, av: &AddressVector) -> Result<(), crate::error::Error> {
        self.inner.bind_av(&av.inner)
    }
}

pub struct TransmitContextBuilder<'a, T, E> {
    tx_attr: TxAttr,
    index: i32,
    ep: &'a Endpoint<E>,
    ctx: Option<&'a mut T>,
}


impl<'a> TransmitContextBuilder<'a, (), ()> {
    pub fn new<E>(ep: &'a Endpoint<E>, index: i32) -> TransmitContextBuilder<'a, (), E> {
        TransmitContextBuilder::<(), E> {
            tx_attr: TxAttr::new(),
            index,
            ep,
            ctx: None,
        }
    }
}

impl <'a, T, E> TransmitContextBuilder<'a, T, E> {
    
    // pub fn caps(mut self, caps: TxCaps) -> Self {
    //     self.tx_attr.caps(caps);
    //     self
    // }

    pub fn mode(mut self, mode: crate::enums::Mode) -> Self {
        self.tx_attr.mode(mode);
        self
    }

    pub fn set_transmit_options(mut self, ops: TransferOptions) -> Self {
        ops.transmit();
        self.tx_attr.op_flags(ops);
        self
    }

    pub fn msg_order(mut self, msg_order: MsgOrder) -> Self {
        self.tx_attr.msg_order(msg_order);
        self
    }

    pub fn comp_order(mut self, comp_order: TxCompOrder) -> Self {
        self.tx_attr.comp_order(comp_order);
        self
    }

    pub fn inject_size(mut self, size: usize) -> Self {
        self.tx_attr.inject_size(size);
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.tx_attr.size(size);
        self
    }

    pub fn iov_limit(mut self, iov_limit: usize) -> Self {
        self.tx_attr.iov_limit(iov_limit);
        self
    }

    pub fn rma_iov_limit(mut self, rma_iov_limit: usize) -> Self {
        self.tx_attr.rma_iov_limit(rma_iov_limit);
        self
    }

    pub fn tclass(mut self, class: crate::enums::TClass) -> Self {
        self.tx_attr.tclass(class);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> TransmitContextBuilder<'a, T, E> {
        TransmitContextBuilder {
            tx_attr: self.tx_attr,
            index: self.index,
            ep: self.ep,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<TransmitContext, crate::error::Error> {
        TransmitContext::new(self.ep, self.index, self.tx_attr, self.ctx)
    }
}

pub type ReceiveContext = XContextBase<Receive>; 
type ReceiveContextImpl = XContextBaseImpl<Receive>; 

impl ReceiveContextImpl {

    pub(crate) fn new<T0>(ep: &impl ActiveEndpointImpl, index: i32, mut attr: RxAttr, context: Option<&mut T0>) -> Result<ReceiveContextImpl, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let err = 
            if let Some(ctx) = context {
                unsafe{ libfabric_sys::inlined_fi_rx_context(ep.as_raw_typed_fid(), index, attr.get_mut(), &mut c_ep, (ctx as *mut T0).cast())}
            }
            else {
                unsafe{ libfabric_sys::inlined_fi_rx_context(ep.as_raw_typed_fid(), index, attr.get_mut(), &mut c_ep, std::ptr::null_mut())}
            };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    c_ep: OwnedEpFid::from(c_ep), 
                    phantom: PhantomData,
                    _sync_rcs: RefCell::new(Vec::new()),
                })
        }
    }

    pub(crate) fn bind<T:?Sized + crate::Bind + AsRawFid>(&self, res: &T, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.as_raw_typed_fid(), res.as_raw_fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    } 

    pub(crate) fn bind_cq(&self) -> RxIncompleteBindCq {
        RxIncompleteBindCq { ep: self, flags: 0}
    }

    pub(crate) fn bind_cntr(&self) -> RxIncompleteBindCntr {
        RxIncompleteBindCntr { ep: self, flags: 0}
    }

    pub(crate) fn bind_eq<T: ReadEq + BindImpl + 'static>(&self, eq: &EventQueue<T>) -> Result<(), crate::error::Error>  {
        
        self.bind(eq, 0)
    }

    pub(crate) fn bind_av(&self, av: &AddressVector) -> Result<(), crate::error::Error> {
    
        self.bind(av, 0)
    }
}

impl ReceiveContext {
    pub(crate) fn new<T0>(ep: &impl ActiveEndpointImpl, index: i32, attr: RxAttr, context: Option<&mut T0>) -> Result<ReceiveContext, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new(ReceiveContextImpl::new(ep, index, attr, context)?)
            }
        )
    }

    pub fn bind_cq(&self) -> RxIncompleteBindCq {
        self.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> RxIncompleteBindCntr {
        self.inner.bind_cntr()
    }

    pub fn bind_eq<T: ReadEq + BindImpl + 'static>(&self, eq: &EventQueue<T>) -> Result<(), crate::error::Error>  {
        self.inner.bind_eq(eq)
    }

    pub fn bind_av(&self, av: &AddressVector) -> Result<(), crate::error::Error> {
        self.inner.bind_av(av)
    }
    

}


pub struct ReceiveContextBuilder<'a, T, E> {
    rx_attr: RxAttr,
    index: i32,
    ep: &'a Endpoint<E>,
    ctx: Option<&'a mut T>,
}

impl<'a> ReceiveContextBuilder<'a, (), ()> {
    pub fn new<E>(ep: &'a Endpoint<E>, index: i32) -> ReceiveContextBuilder<'a, (), E> {
        ReceiveContextBuilder::<(), E> {
            rx_attr: RxAttr::new(),
            index,
            ep,
            ctx: None,
        }
    }
}

impl<'a, T, E> ReceiveContextBuilder<'a, T, E> {

    // pub fn caps(&mut self, caps: RxCaps) -> &mut Self {
    //     self.rx_attr.caps(caps);
    //     self
    // }

    pub fn mode(mut self, mode: crate::enums::Mode) -> Self {
        self.rx_attr.mode(mode);
        self
    }


    pub fn msg_order(mut self, msg_order: MsgOrder) -> Self {
        self.rx_attr.msg_order(msg_order);
        self
    }

    pub fn comp_order(mut self, comp_order: RxCompOrder) -> Self {
        self.rx_attr.comp_order(comp_order);
        self
    }

    pub fn total_buffered_recv(mut self, total_buffered_recv: usize) -> Self {
        self.rx_attr.total_buffered_recv(total_buffered_recv);
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.rx_attr.size(size);
        self
    }

    pub fn iov_limit(mut self, iov_limit: usize) -> Self {
        self.rx_attr.iov_limit(iov_limit);
        self
    }

    pub fn set_receive_options(mut self, ops: TransferOptions) -> Self {
        ops.recv();
        self.rx_attr.op_flags(ops);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> ReceiveContextBuilder<'a, T, E> {
        ReceiveContextBuilder {
            rx_attr: self.rx_attr,
            index: self.index,
            ep: self.ep,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<ReceiveContext, crate::error::Error> {
        ReceiveContext::new(self.ep, self.index, self.rx_attr, self.ctx)
    }
}

pub struct TxIncompleteBindCq<'a> {
    pub(crate) ep: &'a TransmitContextImpl,
    pub(crate) flags: u64,
}

impl<'a> TxIncompleteBindCq<'a> {

    pub fn transmit(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_TRANSMIT as u64;

            self
        }
        else {
            self.flags |= libfabric_sys::FI_TRANSMIT as u64;

            self
        }
    }

    pub fn cq<T: ReadCq + BindImpl + AsFid + 'static>(&mut self, cq: &crate::cq::CompletionQueue<T>) -> Result<(), crate::error::Error> {
        self.ep.bind(&cq.inner, self.flags)
    }
}

pub struct TxIncompleteBindCntr<'a> {
    pub(crate) ep: &'a TransmitContextImpl,
    pub(crate) flags: u64,
}

impl<'a> TxIncompleteBindCntr<'a> {

    pub fn remote_write(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_REMOTE_WRITE as u64;

        self
    }

    pub fn send(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_SEND as u64;

        self
    }

    pub fn write(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_WRITE as u64;

        self
    }

    pub fn cntr<T: crate::cntroptions::CntrConfig + 'static>(&self, cntr: &Counter<T>) -> Result<(), crate::error::Error> {
        self.ep.bind(&cntr.inner, self.flags)
    }
}

pub struct TxCaps {
    c_flags: u64,
}

impl TxCaps {

    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }

    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    crate::enums::gen_set_get_flag!(message, is_message, libfabric_sys::FI_MSG as u64);
    crate::enums::gen_set_get_flag!(rma, is_rma, libfabric_sys::FI_RMA as u64);
    crate::enums::gen_set_get_flag!(tagged, is_tagged, libfabric_sys::FI_TAGGED as u64);
    crate::enums::gen_set_get_flag!(atomic, is_atomic, libfabric_sys::FI_ATOMIC as u64);
    crate::enums::gen_set_get_flag!(read, is_read, libfabric_sys::FI_READ as u64);
    crate::enums::gen_set_get_flag!(write, is_write, libfabric_sys::FI_WRITE as u64);
    crate::enums::gen_set_get_flag!(send, is_send, libfabric_sys::FI_SEND as u64);
    crate::enums::gen_set_get_flag!(hmem, is_hmem, libfabric_sys::FI_HMEM);
    crate::enums::gen_set_get_flag!(trigger, is_trigger, libfabric_sys::FI_TRIGGER as u64);
    crate::enums::gen_set_get_flag!(fence, is_fence, libfabric_sys::FI_FENCE as u64);
    crate::enums::gen_set_get_flag!(multicast, is_multicast, libfabric_sys::FI_MULTICAST as u64);
    crate::enums::gen_set_get_flag!(rma_pmem, is_rma_pmem, libfabric_sys::FI_RMA_PMEM);
    crate::enums::gen_set_get_flag!(named_rx_ctx, is_named_rx_ctx, libfabric_sys::FI_NAMED_RX_CTX);
    crate::enums::gen_set_get_flag!(collective, is_collective, libfabric_sys::FI_COLLECTIVE as u64);
    // crate::enums::gen_set_get_flag!(xpu, is_xpu, libfabric_sys::FI_XPU as u64);
}

impl Default for TxCaps {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MsgOrder {
    c_flags: u64,
}

impl MsgOrder {

    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }

    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    crate::enums::gen_set_get_flag!(atomic_rar, is_atomic_rar, libfabric_sys::FI_ORDER_ATOMIC_RAR);
    crate::enums::gen_set_get_flag!(atomic_raw, is_atomic_raw, libfabric_sys::FI_ORDER_ATOMIC_RAW);
    crate::enums::gen_set_get_flag!(atomic_war, is_atomic_war, libfabric_sys::FI_ORDER_ATOMIC_WAR);
    crate::enums::gen_set_get_flag!(atomic_waw, is_atomic_waw, libfabric_sys::FI_ORDER_ATOMIC_WAW);
    crate::enums::gen_set_get_flag!(rar, is_rar, libfabric_sys::FI_ORDER_RAR as u64);
    crate::enums::gen_set_get_flag!(ras, is_ras, libfabric_sys::FI_ORDER_RAS as u64);
    crate::enums::gen_set_get_flag!(raw, is_raw, libfabric_sys::FI_ORDER_RAW as u64);
    crate::enums::gen_set_get_flag!(sar, is_sar, libfabric_sys::FI_ORDER_SAR as u64);
    crate::enums::gen_set_get_flag!(sas, is_sas, libfabric_sys::FI_ORDER_SAS as u64);
    crate::enums::gen_set_get_flag!(saw, is_saw, libfabric_sys::FI_ORDER_SAW as u64);
    crate::enums::gen_set_get_flag!(war, is_war, libfabric_sys::FI_ORDER_WAR as u64);
    crate::enums::gen_set_get_flag!(was, is_was, libfabric_sys::FI_ORDER_WAS as u64);
    crate::enums::gen_set_get_flag!(waw, is_waw, libfabric_sys::FI_ORDER_WAW as u64);
    crate::enums::gen_set_get_flag!(rma_rar, is_rma_rar, libfabric_sys::FI_ORDER_RMA_RAR);
    crate::enums::gen_set_get_flag!(rma_raw, is_rma_raw, libfabric_sys::FI_ORDER_RMA_RAW);
    crate::enums::gen_set_get_flag!(rma_war, is_rma_war, libfabric_sys::FI_ORDER_RMA_WAR);
    crate::enums::gen_set_get_flag!(rma_waw, is_rma_waw, libfabric_sys::FI_ORDER_RMA_WAW);
}

impl Default for MsgOrder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TxCompOrder {
    c_flags: u64,
}


impl TxCompOrder {

    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }

    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    crate::enums::gen_set_get_flag!(strict, is_strict, libfabric_sys::FI_ORDER_STRICT as u64);
}

impl Default for TxCompOrder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct TxAttr {
    c_attr: libfabric_sys::fi_tx_attr,
}

impl TxAttr {

    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_tx_attr {
            caps: 0,
            mode: 0,
            op_flags: 0,
            msg_order: 0,
            comp_order: 0,
            inject_size: 0,
            size: 0,
            iov_limit: 0,
            rma_iov_limit: 0,
            tclass: 0,
        };

        Self { c_attr }        
    }

    pub(crate) fn from(c_tx_attr_ptr: *mut libfabric_sys::fi_tx_attr) -> Self {
        let c_attr = unsafe { *c_tx_attr_ptr };

        Self { c_attr }
    }
    
    pub fn caps(&mut self, caps: TxCaps) -> &mut Self {
        self.c_attr.caps = caps.get_value();
        self
    }

    pub fn mode(&mut self, mode: crate::enums::Mode) -> &mut Self {
        self.c_attr.mode = mode.get_value();
        self
    }

    pub fn op_flags(&mut self, tfer: crate::enums::TransferOptions) -> &mut Self {
        self.c_attr.op_flags = tfer.get_value().into();
        self
    }

    pub fn msg_order(&mut self, msg_order: MsgOrder) -> &mut Self {
        self.c_attr.msg_order = msg_order.get_value();
        self
    }

    pub fn comp_order(&mut self, comp_order: TxCompOrder) -> &mut Self {
        self.c_attr.comp_order = comp_order.get_value();
        self
    }

    pub fn inject_size(&mut self, size: usize) -> &mut Self {
        self.c_attr.inject_size = size;
        self
    }

    pub fn size(&mut self, size: usize) -> &mut Self {
        self.c_attr.size = size;
        self
    }

    pub fn iov_limit(&mut self, iov_limit: usize) -> &mut Self {
        self.c_attr.iov_limit = iov_limit;
        self
    }

    pub fn rma_iov_limit(&mut self, rma_iov_limit: usize) -> &mut Self {
        self.c_attr.rma_iov_limit = rma_iov_limit;
        self
    }

    pub fn tclass(&mut self, class: crate::enums::TClass) -> &mut Self {
        self.c_attr.tclass = class.get_value();
        self
    }

    pub fn get_caps(&self) -> u64 {
        self.c_attr.caps
    }

    pub fn get_mode(&self) -> crate::enums::Mode {
        crate::enums::Mode::from_value(self.c_attr.mode)
    }

    pub fn get_op_flags(&self) -> u64 {
        self.c_attr.op_flags
    }

    pub fn get_msg_order(&self) -> u64 {
        self.c_attr.msg_order
    }

    pub fn get_comp_order(&self) -> u64 {
        self.c_attr.comp_order
    }

    pub fn get_inject_size(&self) -> usize {
        self.c_attr.inject_size
    }

    pub fn get_size(&self) -> usize {
        self.c_attr.size
    }
    
    pub fn get_iov_limit(&self) -> usize {
        self.c_attr.iov_limit
    }

    pub fn get_rma_iov_limit(&self) -> usize {
        self.c_attr.rma_iov_limit
    }

    pub fn get_tclass(&self) -> u32 {
        self.c_attr.tclass
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_tx_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_tx_attr {
        &mut self.c_attr
    }
}

impl Default for TxAttr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RxIncompleteBindCq<'a> {
    pub(crate) ep: &'a  ReceiveContextImpl,
    pub(crate) flags: u64,
}

impl<'a> RxIncompleteBindCq<'a> {

    pub fn recv(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_RECV  as u64 ;
        
            self
        }
        else {
            self.flags |= libfabric_sys::FI_RECV as u64;

            self
        }
    }

    pub fn cq<T: ReadCq + BindImpl + AsRawFid + 'static>(&self, cq: &crate::cq::CompletionQueue<T>) -> Result<(), crate::error::Error> {
        self.ep.bind(cq, self.flags)
    }
}

pub struct RxIncompleteBindCntr<'a> {
    pub(crate) ep: &'a  ReceiveContextImpl,
    pub(crate) flags: u64,
}

impl<'a> RxIncompleteBindCntr<'a> {
    
    pub fn read(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_READ as u64;

        self
    }

    pub fn recv(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_RECV as u64;

        self
    }

    pub fn remote_read(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_REMOTE_READ as u64;

        self
    }

    pub fn cntr<T: crate::cntroptions::CntrConfig + 'static>(&mut self, cntr: &Counter<T>) -> Result<(), crate::error::Error> {
        self.ep.bind(cntr, self.flags)
    }
}


#[derive(Clone)]
pub struct RxAttr {
    c_attr: libfabric_sys::fi_rx_attr,
}

impl RxAttr {
    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_rx_attr {
            caps: 0,
            mode: 0,
            op_flags: 0,
            msg_order: 0,
            comp_order: 0,
            total_buffered_recv: 0,
            size: 0,
            iov_limit: 0,
        };

        Self { c_attr }
    }

    pub(crate) fn from(c_rx_attr: *mut libfabric_sys::fi_rx_attr) -> Self {
        let c_attr = unsafe { *c_rx_attr };

        Self { c_attr }
    }

    pub fn caps(&mut self, caps: RxCaps) -> &mut Self {
        self.c_attr.caps = caps.get_value();
        self
    }

    pub fn mode(&mut self, mode: crate::enums::Mode) -> &mut Self {
        self.c_attr.mode = mode.get_value();
        self
    }


    pub fn msg_order(&mut self, msg_order: MsgOrder) -> &mut Self {
        self.c_attr.msg_order = msg_order.get_value();
        self
    }

    pub fn comp_order(&mut self, comp_order: RxCompOrder) -> &mut Self {
        self.c_attr.comp_order = comp_order.get_value();
        self
    }

    pub fn total_buffered_recv(&mut self, total_buffered_recv: usize) -> &mut Self {
        self.c_attr.total_buffered_recv = total_buffered_recv;
        self
    }

    pub fn size(&mut self, size: usize) -> &mut Self {
        self.c_attr.size = size;
        self
    }

    pub fn iov_limit(&mut self, iov_limit: usize) -> &mut Self {
        self.c_attr.iov_limit = iov_limit;
        self
    }

    pub fn op_flags(&mut self, tfer: crate::enums::TransferOptions) -> &mut Self {
        self.c_attr.op_flags = tfer.get_value().into();
        self
    }

    pub fn get_caps(&self) -> u64 {
        self.c_attr.caps
    }

    pub fn get_mode(&self) -> crate::enums::Mode {
        crate::enums::Mode::from_value(self.c_attr.mode)
    }

    pub fn get_op_flags(&self) -> u64 {
        self.c_attr.op_flags
    }

    pub fn get_msg_order(&self) -> u64 {
        self.c_attr.msg_order
    }

    pub fn get_comp_order(&self) -> u64 {
        self.c_attr.comp_order
    }

    pub fn get_size(&self) -> usize {
        self.c_attr.size
    }

    pub fn get_iov_limit(&self) -> usize {
        self.c_attr.iov_limit
    }

    pub fn get_total_buffered_recv(&self) -> usize {
        self.c_attr.total_buffered_recv
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_rx_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_rx_attr {
        &mut self.c_attr
    }
}

impl Default for RxAttr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RxCaps {
    c_flags: u64,
}

impl RxCaps {

    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }

    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    crate::enums::gen_set_get_flag!(message, is_message, libfabric_sys::FI_MSG as u64);
    crate::enums::gen_set_get_flag!(vabiable_message, is_vabiable_message, libfabric_sys::FI_VARIABLE_MSG);
    crate::enums::gen_set_get_flag!(rma, is_rma, libfabric_sys::FI_RMA as u64);
    crate::enums::gen_set_get_flag!(rma_event, is_rma_event, libfabric_sys::FI_RMA_EVENT);
    crate::enums::gen_set_get_flag!(tagged, is_tagged, libfabric_sys::FI_TAGGED as u64);
    crate::enums::gen_set_get_flag!(atomic, is_atomic, libfabric_sys::FI_ATOMIC as u64);
    crate::enums::gen_set_get_flag!(remote_read, is_remote_read, libfabric_sys::FI_REMOTE_READ as u64);
    crate::enums::gen_set_get_flag!(remote_write, is_remote_write, libfabric_sys::FI_REMOTE_WRITE as u64);
    crate::enums::gen_set_get_flag!(recv, is_recv, libfabric_sys::FI_RECV as u64);
    crate::enums::gen_set_get_flag!(directed_recv, is_directed_recv, libfabric_sys::FI_DIRECTED_RECV);
    crate::enums::gen_set_get_flag!(hmem, is_hmem, libfabric_sys::FI_HMEM);
    crate::enums::gen_set_get_flag!(trigger, is_trigger, libfabric_sys::FI_TRIGGER as u64);
    crate::enums::gen_set_get_flag!(rma_pmem, is_rma_pmem, libfabric_sys::FI_RMA_PMEM);
    crate::enums::gen_set_get_flag!(multi_recv, is_multi_recv, libfabric_sys::FI_MULTI_RECV as u64);
    crate::enums::gen_set_get_flag!(source, is_source, libfabric_sys::FI_SOURCE);
    crate::enums::gen_set_get_flag!(source_err, is_source_err, libfabric_sys::FI_SOURCE_ERR);
    crate::enums::gen_set_get_flag!(collective, is_collective, libfabric_sys::FI_COLLECTIVE as u64);
}

impl Default for RxCaps {
    fn default() -> Self {
        Self::new()
    }
}


pub struct RxCompOrder {
    c_flags: u64,
}

impl RxCompOrder {

    pub(crate) fn get_value(&self) -> u64 {
        self.c_flags
    }

    pub fn new() -> Self {
        Self {
            c_flags: 0,
        }
    }

    crate::enums::gen_set_get_flag!(strict, is_strict, libfabric_sys::FI_ORDER_STRICT as u64);
    crate::enums::gen_set_get_flag!(data, is_data, libfabric_sys::FI_ORDER_DATA as u64);
}

impl Default for RxCompOrder {
    fn default() -> Self {
        Self::new()
    }
}

