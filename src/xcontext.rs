use std::marker::PhantomData;

use crate::{
    cntr::{Counter, ReadCntr},
    cq::ReadCq,
    enums::{Mode, TrafficClass, TransferOptions},
    ep::{ActiveEndpoint, BaseEndpoint, EndpointBase, EndpointImplBase},
    eq::ReadEq,
    fid::{self, AsFid, AsRawFid, AsRawTypedFid, AsTypedFid, EpRawFid, OwnedEpFid, RawFid},
    Context, MyOnceCell, MyRc,
};

pub struct Receive;
pub struct Transmit;
//================== XContext Template ==================//
pub(crate) struct XContextBaseImpl<T, CQ: ?Sized> {
    pub(crate) c_ep: OwnedEpFid,
    phantom: PhantomData<T>,
    pub(crate) cq: MyOnceCell<MyRc<CQ>>,
    pub(crate) cntr: MyOnceCell<MyRc<dyn ReadCntr>>,
}

pub struct XContextBase<T, CQ: ?Sized> {
    pub(crate) inner: MyRc<XContextBaseImpl<T, CQ>>,
}

//================== XContext Trait Implementations ==================//
impl<T, CQ> ActiveEndpoint for XContextBase<T, CQ> {}
impl<T, CQ> ActiveEndpoint for XContextBaseImpl<T, CQ> {}
impl<T, CQ> BaseEndpoint for XContextBaseImpl<T, CQ> {}

impl<T, CQ> AsFid for XContextBase<T, CQ> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.inner.as_fid()
    }
}

impl<T, CQ> AsFid for XContextBaseImpl<T, CQ> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.c_ep.as_fid()
    }
}

impl<T, CQ> AsRawFid for XContextBase<T, CQ> {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}

impl<T, CQ> AsRawFid for XContextBaseImpl<T, CQ> {
    fn as_raw_fid(&self) -> RawFid {
        self.c_ep.as_raw_fid()
    }
}

impl<T, CQ> AsTypedFid<EpRawFid> for XContextBase<T, CQ> {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<EpRawFid> {
        self.inner.as_typed_fid()
    }
}

impl<T, CQ> AsTypedFid<EpRawFid> for XContextBaseImpl<T, CQ> {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<EpRawFid> {
        self.c_ep.as_typed_fid()
    }
}

impl<T, CQ: ?Sized> AsRawTypedFid for XContextBase<T, CQ> {
    type Output = EpRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl<T, CQ: ?Sized> AsRawTypedFid for XContextBaseImpl<T, CQ> {
    type Output = EpRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_ep.as_raw_typed_fid()
    }
}

//================== TxContext ==================//
pub type TxContextBase<CQ> = XContextBase<Transmit, CQ>;
pub type TxContext = XContextBase<Transmit, dyn ReadCq>;
pub(crate) type TxContextImplBase<CQ> = XContextBaseImpl<Transmit, CQ>;
pub(crate) type TxContextImpl = XContextBaseImpl<Transmit, dyn ReadCq>;

impl<CQ: ?Sized> TxContextImplBase<CQ> {
    pub(crate) fn new(
        ep: &impl ActiveEndpoint,
        index: i32,
        attr: TxAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<TxContextImplBase<CQ>, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_tx_context(
                ep.as_raw_typed_fid(),
                index,
                &mut attr.get(),
                &mut c_ep,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                c_ep: OwnedEpFid::from(c_ep),
                phantom: PhantomData,
                cq: MyOnceCell::new(),
                cntr: MyOnceCell::new(),
            })
        }
    }

    pub(crate) fn bind_cntr_<T: ReadCntr + AsFid + 'static>(
        &self,
        res: &MyRc<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_raw_typed_fid(),
                res.as_fid().as_raw_fid(),
                flags,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self.cntr.set(res.clone()).is_err() {
                panic!("TransmitContext already bound to a Counter");
            }
            Ok(())
        }
    }
}

impl TxContextImpl {
    pub(crate) fn bind_cq(&self) -> TxIncompleteBindCq {
        TxIncompleteBindCq { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cntr(&self) -> TxIncompleteBindCntr {
        TxIncompleteBindCntr { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cq_<T: ReadCq + AsFid + 'static>(
        &self,
        res: &MyRc<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_raw_typed_fid(),
                res.as_fid().as_raw_fid(),
                flags,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self.cq.set(res.clone()).is_err() {
                panic!("TransmitContext already bound to a CompletionQueueu");
            }
            Ok(())
        }
    }
}

impl<CQ: ?Sized> TxContextBase<CQ> {
    pub(crate) fn new(
        ep: &impl ActiveEndpoint,
        index: i32,
        attr: TxAttr,
        context: Option<&mut Context>,
    ) -> Result<TxContextBase<CQ>, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(TxContextImplBase::<CQ>::new(ep, index, attr, c_void)?),
        })
    }
}

impl TxContext {
    pub fn bind_cq(&self) -> TxIncompleteBindCq {
        self.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> TxIncompleteBindCntr {
        self.inner.bind_cntr()
    }
}

//================== TxContext Builder ==================//
pub struct TxContextBuilder<'a, E, const CONN: bool> {
    pub(crate) tx_attr: TxAttr,
    pub(crate) index: i32,
    pub(crate) ep: &'a EndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>, CONN>,
    pub(crate) ctx: Option<&'a mut Context>,
}

impl<'a, const CONN: bool> TxContextBuilder<'a, (), CONN> {
    pub fn new<E>(
        ep: &'a EndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>, CONN>,
        index: i32,
    ) -> TxContextBuilder<'a, E, CONN> {
        TxContextBuilder::<E, CONN> {
            tx_attr: TxAttr::new(),
            index,
            ep,
            ctx: None,
        }
    }
}

impl<'a, E: AsRawTypedFid<Output = EpRawFid>, const CONN: bool> TxContextBuilder<'a, E, CONN> {
    // pub fn caps(mut self, caps: TxCaps) -> Self {
    //     self.tx_attr.caps(caps);
    //     self
    // }

    pub fn mode(mut self, mode: crate::enums::Mode) -> Self {
        self.tx_attr.set_mode(mode);
        self
    }

    pub fn set_transmit_options(mut self, ops: TransferOptions) -> Self {
        ops.transmit();
        self.tx_attr.set_op_flags(ops);
        self
    }

    pub fn msg_order(mut self, msg_order: MsgOrder) -> Self {
        self.tx_attr.set_msg_order(msg_order);
        self
    }

    pub fn comp_order(mut self, comp_order: TxCompOrder) -> Self {
        self.tx_attr.set_comp_order(comp_order);
        self
    }

    pub fn inject_size(mut self, size: usize) -> Self {
        self.tx_attr.set_inject_size(size);
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.tx_attr.set_size(size);
        self
    }

    pub fn iov_limit(mut self, iov_limit: usize) -> Self {
        self.tx_attr.set_iov_limit(iov_limit);
        self
    }

    pub fn rma_iov_limit(mut self, rma_iov_limit: usize) -> Self {
        self.tx_attr.set_rma_iov_limit(rma_iov_limit);
        self
    }

    pub fn tclass(mut self, class: crate::enums::TrafficClass) -> Self {
        self.tx_attr.set_traffic_class(class);
        self
    }

    pub fn context(self, ctx: &'a mut Context) -> TxContextBuilder<'a, E, CONN> {
        TxContextBuilder {
            tx_attr: self.tx_attr,
            index: self.index,
            ep: self.ep,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<TxContextBase<dyn ReadCq>, crate::error::Error> {
        TxContextBase::new(self.ep, self.index, self.tx_attr, self.ctx)
    }
}

//================== TxContext Attribute ==================//
#[derive(Clone, Debug)]
pub struct TxAttr {
    caps: TxCaps,
    mode: Mode,
    op_flags: TransferOptions,
    msg_order: MsgOrder,
    comp_order: TxCompOrder,
    inject_size: usize,
    size: usize,
    iov_limit: usize,
    rma_iov_limit: usize,
    traffic_class: TrafficClass,
}

impl TxAttr {
    pub(crate) fn new() -> Self {
        Self {
            caps: TxCaps::new(),
            mode: Mode::new(),
            op_flags: TransferOptions::new(),
            msg_order: MsgOrder::new(),
            comp_order: TxCompOrder::new(),
            traffic_class: TrafficClass::Unspec,
            inject_size: 0,
            iov_limit: 0,
            size: 0,
            rma_iov_limit: 0,
        }
    }

    pub(crate) fn from_raw_ptr(c_tx_attr_ptr: *const libfabric_sys::fi_tx_attr) -> Self {
        assert!(!c_tx_attr_ptr.is_null());
        Self {
            caps: TxCaps::from_raw(unsafe { *c_tx_attr_ptr }.caps),
            mode: Mode::from_raw(unsafe { *c_tx_attr_ptr }.mode),
            op_flags: TransferOptions::from_raw(unsafe { *c_tx_attr_ptr }.op_flags as u32),
            msg_order: MsgOrder::from_raw(unsafe { *c_tx_attr_ptr }.msg_order),
            comp_order: TxCompOrder::from_raw(unsafe { *c_tx_attr_ptr }.comp_order),
            inject_size: unsafe { *c_tx_attr_ptr }.inject_size,
            iov_limit: unsafe { *c_tx_attr_ptr }.iov_limit,
            size: unsafe { *c_tx_attr_ptr }.size,
            rma_iov_limit: unsafe { *c_tx_attr_ptr }.rma_iov_limit,
            traffic_class: TrafficClass::from_raw(unsafe { *c_tx_attr_ptr }.tclass),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_caps(&mut self, caps: TxCaps) -> &mut Self {
        self.caps = caps;
        self
    }

    pub(crate) fn set_mode(&mut self, mode: crate::enums::Mode) -> &mut Self {
        self.mode = mode;
        self
    }

    pub(crate) fn set_op_flags(&mut self, tfer: crate::enums::TransferOptions) -> &mut Self {
        self.op_flags = tfer;
        self
    }

    pub(crate) fn set_msg_order(&mut self, msg_order: MsgOrder) -> &mut Self {
        self.msg_order = msg_order;
        self
    }

    pub(crate) fn set_comp_order(&mut self, comp_order: TxCompOrder) -> &mut Self {
        self.comp_order = comp_order;
        self
    }

    pub(crate) fn set_inject_size(&mut self, size: usize) -> &mut Self {
        self.inject_size = size;
        self
    }

    pub(crate) fn set_size(&mut self, size: usize) -> &mut Self {
        self.size = size;
        self
    }

    pub(crate) fn set_iov_limit(&mut self, iov_limit: usize) -> &mut Self {
        self.iov_limit = iov_limit;
        self
    }

    pub(crate) fn set_rma_iov_limit(&mut self, rma_iov_limit: usize) -> &mut Self {
        self.rma_iov_limit = rma_iov_limit;
        self
    }

    pub(crate) fn set_traffic_class(&mut self, class: crate::enums::TrafficClass) -> &mut Self {
        self.traffic_class = class;
        self
    }

    pub fn caps(&self) -> &TxCaps {
        &self.caps
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn op_flags(&self) -> &TransferOptions {
        &self.op_flags
    }

    pub fn msg_order(&self) -> &MsgOrder {
        &self.msg_order
    }

    pub fn comp_order(&self) -> &TxCompOrder {
        &self.comp_order
    }

    pub fn inject_size(&self) -> usize {
        self.inject_size
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn iov_limit(&self) -> usize {
        self.iov_limit
    }

    pub fn rma_iov_limit(&self) -> usize {
        self.rma_iov_limit
    }

    pub fn traffic_class(&self) -> &TrafficClass {
        &self.traffic_class
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn get(&self) -> libfabric_sys::fi_tx_attr {
        libfabric_sys::fi_tx_attr {
            caps: self.caps.as_raw(),
            mode: self.mode.as_raw(),
            op_flags: self.op_flags.as_raw() as u64,
            msg_order: self.msg_order.as_raw(),
            comp_order: self.comp_order.as_raw(),
            tclass: self.traffic_class.as_raw(),
            inject_size: self.inject_size,
            size: self.size,
            iov_limit: self.iov_limit,
            rma_iov_limit: self.rma_iov_limit,
        }
    }
}

// impl Default for TxAttr {
//     fn default() -> Self {
//         Self::new()
//     }
// }

//================== RxContext ==================//
pub type RxContext = XContextBase<Receive, dyn ReadCq>;
pub type RxContextBase<CQ> = XContextBase<Receive, CQ>;
pub(crate) type RxContextImpl = XContextBaseImpl<Receive, dyn ReadCq>;
pub(crate) type RxContextImplBase<CQ> = XContextBaseImpl<Receive, CQ>;

impl<CQ: ?Sized> RxContextImplBase<CQ> {
    pub(crate) fn new(
        ep: &impl ActiveEndpoint,
        index: i32,
        attr: RxAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_rx_context(
                ep.as_raw_typed_fid(),
                index,
                &mut attr.get(),
                &mut c_ep,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                c_ep: OwnedEpFid::from(c_ep),
                phantom: PhantomData,
                cq: MyOnceCell::new(),
                cntr: MyOnceCell::new(),
            })
        }
    }

    pub(crate) fn bind_cntr_<T: ReadCntr + AsFid + 'static>(
        &self,
        res: &MyRc<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_raw_typed_fid(),
                res.as_fid().as_raw_fid(),
                flags,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self.cntr.set(res.clone()).is_err() {
                panic!("TransmitContext already bound to a CompletionQueueu");
            }
            Ok(())
        }
    }
}

impl RxContextImplBase<dyn ReadCq> {
    pub(crate) fn bind_cq(&self) -> RxIncompleteBindCq {
        RxIncompleteBindCq { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cntr(&self) -> RxIncompleteBindCntr {
        RxIncompleteBindCntr { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cq_<T: ReadCq + AsFid + 'static>(
        &self,
        res: &MyRc<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_raw_typed_fid(),
                res.as_fid().as_raw_fid(),
                flags,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self.cq.set(res.clone()).is_err() {
                panic!("TransmitContext already bound to a CompletionQueueu");
            }
            Ok(())
        }
    }
}

impl<CQ: ?Sized> RxContextBase<CQ> {
    pub(crate) fn new(
        ep: &impl ActiveEndpoint,
        index: i32,
        attr: RxAttr,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(RxContextImplBase::new(ep, index, attr, c_void)?),
        })
    }
}

impl RxContextBase<dyn ReadCq> {
    pub fn bind_cq(&self) -> RxIncompleteBindCq {
        self.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> RxIncompleteBindCntr {
        self.inner.bind_cntr()
    }
}

//================== RxContext Builder ==================//
pub struct ReceiveContextBuilder<'a, E, const CONN: bool> {
    pub(crate) rx_attr: RxAttr,
    pub(crate) index: i32,
    pub(crate) ep: &'a EndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>, CONN>,
    pub(crate) ctx: Option<&'a mut Context>,
}

impl<'a, const CONN: bool> ReceiveContextBuilder<'a, (), CONN> {
    pub fn new<E>(
        ep: &'a EndpointBase<EndpointImplBase<E, dyn ReadEq, dyn ReadCq>, CONN>,
        index: i32,
    ) -> ReceiveContextBuilder<'a, E, CONN> {
        ReceiveContextBuilder::<E, CONN> {
            rx_attr: RxAttr::new(),
            index,
            ep,
            ctx: None,
        }
    }
}

impl<'a, E: AsRawTypedFid<Output = EpRawFid>, const CONN: bool> ReceiveContextBuilder<'a, E, CONN> {
    // pub fn caps(&mut self, caps: RxCaps) -> &mut Self {
    //     self.rx_attr.caps(caps);
    //     self
    // }

    pub fn mode(mut self, mode: crate::enums::Mode) -> Self {
        self.rx_attr.set_mode(mode);
        self
    }

    pub fn msg_order(mut self, msg_order: MsgOrder) -> Self {
        self.rx_attr.set_msg_order(msg_order);
        self
    }

    pub fn comp_order(mut self, comp_order: RxCompOrder) -> Self {
        self.rx_attr.set_comp_order(comp_order);
        self
    }

    pub fn total_buffered_recv(mut self, total_buffered_recv: usize) -> Self {
        self.rx_attr.set_total_buffered_recv(total_buffered_recv);
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.rx_attr.set_size(size);
        self
    }

    pub fn iov_limit(mut self, iov_limit: usize) -> Self {
        self.rx_attr.set_iov_limit(iov_limit);
        self
    }

    pub fn set_receive_options(mut self, ops: TransferOptions) -> Self {
        ops.recv();
        self.rx_attr.set_op_flags(ops);
        self
    }

    pub fn context(self, ctx: &'a mut Context) -> ReceiveContextBuilder<'a, E, CONN> {
        ReceiveContextBuilder {
            rx_attr: self.rx_attr,
            index: self.index,
            ep: self.ep,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<RxContext, crate::error::Error> {
        RxContext::new(self.ep, self.index, self.rx_attr, self.ctx)
    }
}

//================== RxContext Attribute ==================//
// #[derive(Clone)]
// pub struct RxAttr {
//     c_attr: libfabric_sys::fi_rx_attr,
// }

#[derive(Clone, Debug)]
pub struct RxAttr {
    caps: RxCaps,
    mode: Mode,
    op_flags: TransferOptions,
    msg_order: MsgOrder,
    comp_order: RxCompOrder,
    total_buffered_recv: usize,
    size: usize,
    iov_limit: usize,
}

impl RxAttr {
    pub(crate) fn new() -> Self {
        Self {
            caps: RxCaps::new(),
            mode: Mode::new(),
            op_flags: TransferOptions::new(),
            msg_order: MsgOrder::new(),
            comp_order: RxCompOrder::new(),
            total_buffered_recv: 0,
            iov_limit: 0,
            size: 0,
        }
    }

    pub(crate) fn from_raw_ptr(c_rx_attr_ptr: *const libfabric_sys::fi_rx_attr) -> Self {
        assert!(!c_rx_attr_ptr.is_null());
        Self {
            caps: RxCaps::from_raw(unsafe { *c_rx_attr_ptr }.caps),
            mode: Mode::from_raw(unsafe { *c_rx_attr_ptr }.mode),
            op_flags: TransferOptions::from_raw(unsafe { *c_rx_attr_ptr }.op_flags as u32),
            msg_order: MsgOrder::from_raw(unsafe { *c_rx_attr_ptr }.msg_order),
            comp_order: RxCompOrder::from_raw(unsafe { *c_rx_attr_ptr }.comp_order),
            total_buffered_recv: unsafe { *c_rx_attr_ptr }.total_buffered_recv,
            iov_limit: unsafe { *c_rx_attr_ptr }.iov_limit,
            size: unsafe { *c_rx_attr_ptr }.size,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_caps(&mut self, caps: RxCaps) -> &mut Self {
        self.caps = caps;
        self
    }

    pub(crate) fn set_mode(&mut self, mode: crate::enums::Mode) -> &mut Self {
        self.mode = mode;
        self
    }

    pub(crate) fn set_op_flags(&mut self, tfer: crate::enums::TransferOptions) -> &mut Self {
        self.op_flags = tfer;
        self
    }

    pub(crate) fn set_msg_order(&mut self, msg_order: MsgOrder) -> &mut Self {
        self.msg_order = msg_order;
        self
    }

    pub(crate) fn set_comp_order(&mut self, comp_order: RxCompOrder) -> &mut Self {
        self.comp_order = comp_order;
        self
    }

    pub(crate) fn set_total_buffered_recv(&mut self, total_buffered_recv: usize) -> &mut Self {
        self.total_buffered_recv = total_buffered_recv;
        self
    }

    pub(crate) fn set_size(&mut self, size: usize) -> &mut Self {
        self.size = size;
        self
    }

    pub(crate) fn set_iov_limit(&mut self, iov_limit: usize) -> &mut Self {
        self.iov_limit = iov_limit;
        self
    }

    pub fn caps(&self) -> &RxCaps {
        &self.caps
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn op_flags(&self) -> &TransferOptions {
        &self.op_flags
    }

    pub fn msg_order(&self) -> &MsgOrder {
        &self.msg_order
    }

    pub fn comp_order(&self) -> &RxCompOrder {
        &self.comp_order
    }

    pub fn total_buffered_recv(&self) -> usize {
        self.total_buffered_recv
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn iov_limit(&self) -> usize {
        self.iov_limit
    }

    pub(crate) unsafe fn get(&self) -> libfabric_sys::fi_rx_attr {
        libfabric_sys::fi_rx_attr {
            caps: self.caps.as_raw(),
            mode: self.mode.as_raw(),
            op_flags: self.op_flags.as_raw() as u64,
            msg_order: self.msg_order.as_raw(),
            comp_order: self.comp_order.as_raw(),
            total_buffered_recv: self.total_buffered_recv,
            size: self.size,
            iov_limit: self.iov_limit,
        }
    }
}

// impl RxAttr {
//     pub fn new() -> Self {
//         let c_attr = libfabric_sys::fi_rx_attr {
//             caps: 0,
//             mode: 0,
//             op_flags: 0,
//             msg_order: 0,
//             comp_order: 0,
//             total_buffered_recv: 0,
//             size: 0,
//             iov_limit: 0,
//         };

//         Self { c_attr }
//     }

//     pub(crate) fn from(c_rx_attr: *mut libfabric_sys::fi_rx_attr) -> Self {
//         let c_attr = unsafe { *c_rx_attr };

//         Self { c_attr }
//     }

//     pub fn caps(&mut self, caps: RxCaps) -> &mut Self {
//         self.c_attr.caps = caps.get_value();
//         self
//     }

//     pub fn mode(&mut self, mode: crate::enums::Mode) -> &mut Self {
//         self.c_attr.mode = mode.into();
//         self
//     }

//     pub fn msg_order(&mut self, msg_order: MsgOrder) -> &mut Self {
//         self.c_attr.msg_order = msg_order.get_value();
//         self
//     }

//     pub fn comp_order(&mut self, comp_order: RxCompOrder) -> &mut Self {
//         self.c_attr.comp_order = comp_order.get_value();
//         self
//     }

//     pub fn total_buffered_recv(&mut self, total_buffered_recv: usize) -> &mut Self {
//         self.c_attr.total_buffered_recv = total_buffered_recv;
//         self
//     }

//     pub fn size(&mut self, size: usize) -> &mut Self {
//         self.c_attr.size = size;
//         self
//     }

//     pub fn iov_limit(&mut self, iov_limit: usize) -> &mut Self {
//         self.c_attr.iov_limit = iov_limit;
//         self
//     }

//     pub fn op_flags(&mut self, tfer: crate::enums::TransferOptions) -> &mut Self {
//         self.c_attr.op_flags = tfer.get_value().into();
//         self
//     }

//     pub fn get_caps(&self) -> u64 {
//         self.c_attr.caps
//     }

//     pub fn get_mode(&self) -> crate::enums::Mode {
//         crate::enums::Mode::from_value(self.c_attr.mode)
//     }

//     pub fn get_op_flags(&self) -> u64 {
//         self.c_attr.op_flags
//     }

//     pub fn get_msg_order(&self) -> u64 {
//         self.c_attr.msg_order
//     }

//     pub fn get_comp_order(&self) -> u64 {
//         self.c_attr.comp_order
//     }

//     pub fn get_size(&self) -> usize {
//         self.c_attr.size
//     }

//     pub fn get_iov_limit(&self) -> usize {
//         self.c_attr.iov_limit
//     }

//     pub fn get_total_buffered_recv(&self) -> usize {
//         self.c_attr.total_buffered_recv
//     }

//     #[allow(dead_code)]
//     pub(crate) fn get(&self) -> *const libfabric_sys::fi_rx_attr {
//         &self.c_attr
//     }

//     pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_rx_attr {
//         &mut self.c_attr
//     }
// }

impl Default for RxAttr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TxIncompleteBindCq<'a> {
    pub(crate) ep: &'a TxContextImplBase<dyn ReadCq>,
    pub(crate) flags: u64,
}

impl<'a> TxIncompleteBindCq<'a> {
    pub fn transmit(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |=
                libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_TRANSMIT as u64;

            self
        } else {
            self.flags |= libfabric_sys::FI_TRANSMIT as u64;

            self
        }
    }

    pub fn cq<T: ReadCq + AsFid + 'static>(
        &mut self,
        cq: &crate::cq::CompletionQueue<T>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cq_(&cq.inner, self.flags)
    }
}

pub struct TxIncompleteBindCntr<'a> {
    pub(crate) ep: &'a TxContextImplBase<dyn ReadCq>,
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

    pub fn cntr(
        &self,
        cntr: &Counter<impl ReadCntr + AsFid + 'static>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cntr_(&cntr.inner, self.flags)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TxCaps {
    c_flags: u64,
}

impl TxCaps {
    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    pub(crate) fn from_raw(value: u64) -> Self {
        Self { c_flags: value }
    }

    pub fn new() -> Self {
        Self { c_flags: 0 }
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
    crate::enums::gen_set_get_flag!(
        named_rx_ctx,
        is_named_rx_ctx,
        libfabric_sys::FI_NAMED_RX_CTX
    );
    crate::enums::gen_set_get_flag!(
        collective,
        is_collective,
        libfabric_sys::FI_COLLECTIVE as u64
    );
    // crate::enums::gen_set_get_flag!(xpu, is_xpu, libfabric_sys::FI_XPU as u64);
}

impl Default for TxCaps {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MsgOrder {
    c_flags: u64,
}

impl MsgOrder {
    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    pub(crate) fn from_raw(value: u64) -> Self {
        Self { c_flags: value }
    }

    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    crate::enums::gen_set_get_flag!(
        atomic_rar,
        is_atomic_rar,
        libfabric_sys::FI_ORDER_ATOMIC_RAR
    );
    crate::enums::gen_set_get_flag!(
        atomic_raw,
        is_atomic_raw,
        libfabric_sys::FI_ORDER_ATOMIC_RAW
    );
    crate::enums::gen_set_get_flag!(
        atomic_war,
        is_atomic_war,
        libfabric_sys::FI_ORDER_ATOMIC_WAR
    );
    crate::enums::gen_set_get_flag!(
        atomic_waw,
        is_atomic_waw,
        libfabric_sys::FI_ORDER_ATOMIC_WAW
    );
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

#[derive(Clone, Copy, Debug)]
pub struct TxCompOrder {
    c_flags: u64,
}

impl TxCompOrder {
    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    pub(crate) fn from_raw(value: u64) -> Self {
        Self { c_flags: value }
    }

    crate::enums::gen_set_get_flag!(strict, is_strict, libfabric_sys::FI_ORDER_STRICT as u64);
}

impl Default for TxCompOrder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RxIncompleteBindCq<'a> {
    pub(crate) ep: &'a RxContextImpl,
    pub(crate) flags: u64,
}

impl<'a> RxIncompleteBindCq<'a> {
    pub fn recv(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_RECV as u64;

            self
        } else {
            self.flags |= libfabric_sys::FI_RECV as u64;

            self
        }
    }

    pub fn cq<T: ReadCq + AsFid + 'static>(
        &self,
        cq: &crate::cq::CompletionQueue<T>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cq_(&cq.inner, self.flags)
    }
}

pub struct RxIncompleteBindCntr<'a> {
    pub(crate) ep: &'a RxContextImpl,
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

    pub fn cntr(
        &mut self,
        cntr: &Counter<impl AsFid + ReadCntr + 'static>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cntr_(&cntr.inner, self.flags)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RxCaps {
    c_flags: u64,
}

impl RxCaps {
    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    pub(crate) fn from_raw(value: u64) -> Self {
        Self { c_flags: value }
    }

    crate::enums::gen_set_get_flag!(message, is_message, libfabric_sys::FI_MSG as u64);
    crate::enums::gen_set_get_flag!(
        vabiable_message,
        is_vabiable_message,
        libfabric_sys::FI_VARIABLE_MSG
    );
    crate::enums::gen_set_get_flag!(rma, is_rma, libfabric_sys::FI_RMA as u64);
    crate::enums::gen_set_get_flag!(rma_event, is_rma_event, libfabric_sys::FI_RMA_EVENT);
    crate::enums::gen_set_get_flag!(tagged, is_tagged, libfabric_sys::FI_TAGGED as u64);
    crate::enums::gen_set_get_flag!(atomic, is_atomic, libfabric_sys::FI_ATOMIC as u64);
    crate::enums::gen_set_get_flag!(
        remote_read,
        is_remote_read,
        libfabric_sys::FI_REMOTE_READ as u64
    );
    crate::enums::gen_set_get_flag!(
        remote_write,
        is_remote_write,
        libfabric_sys::FI_REMOTE_WRITE as u64
    );
    crate::enums::gen_set_get_flag!(recv, is_recv, libfabric_sys::FI_RECV as u64);
    crate::enums::gen_set_get_flag!(
        directed_recv,
        is_directed_recv,
        libfabric_sys::FI_DIRECTED_RECV
    );
    crate::enums::gen_set_get_flag!(hmem, is_hmem, libfabric_sys::FI_HMEM);
    crate::enums::gen_set_get_flag!(trigger, is_trigger, libfabric_sys::FI_TRIGGER as u64);
    crate::enums::gen_set_get_flag!(rma_pmem, is_rma_pmem, libfabric_sys::FI_RMA_PMEM);
    crate::enums::gen_set_get_flag!(
        multi_recv,
        is_multi_recv,
        libfabric_sys::FI_MULTI_RECV as u64
    );
    crate::enums::gen_set_get_flag!(source, is_source, libfabric_sys::FI_SOURCE);
    crate::enums::gen_set_get_flag!(source_err, is_source_err, libfabric_sys::FI_SOURCE_ERR);
    crate::enums::gen_set_get_flag!(
        collective,
        is_collective,
        libfabric_sys::FI_COLLECTIVE as u64
    );
}

impl Default for RxCaps {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RxCompOrder {
    c_flags: u64,
}

impl RxCompOrder {
    pub(crate) fn as_raw(&self) -> u64 {
        self.c_flags
    }

    pub(crate) fn from_raw(value: u64) -> Self {
        Self { c_flags: value }
    }

    pub fn new() -> Self {
        Self { c_flags: 0 }
    }

    crate::enums::gen_set_get_flag!(strict, is_strict, libfabric_sys::FI_ORDER_STRICT as u64);
    crate::enums::gen_set_get_flag!(data, is_data, libfabric_sys::FI_ORDER_DATA as u64);
}

impl Default for RxCompOrder {
    fn default() -> Self {
        Self::new()
    }
}
