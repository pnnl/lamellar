use std::marker::PhantomData;

use super::{
    cq::AsyncReadCq,
    ep::{AsyncRxEp, AsyncTxEp}, eq::AsyncReadEq,
};
use crate::{
    cntr::{Counter, ReadCntr}, enums::TransferOptions, ep::{Connected, Connectionless, EndpointBase, EndpointImplBase, EpState}, fid::{AsRawFid, AsRawTypedFid, AsTypedFid, OwnedEpFid}, xcontext::{
        MsgOrder, Receive, RxAttr, RxCompOrder, RxContextBase, Transmit, TxAttr, TxCompOrder, TxContextBase, XContextBase, XContextBaseImpl
    }, Context, MyOnceCell, MyRc
};

pub(crate) type TxContextImplBase<I, STATE, CQ> = XContextBaseImpl<Transmit, I, STATE, CQ>;
pub(crate) type TxContextImpl<I, STATE> = XContextBaseImpl<Transmit, I, STATE, dyn AsyncReadCq>;

pub type TxContext<EP, STATE> = TxContextBase<EP, STATE, dyn AsyncReadCq>;
pub type ConnectedTxContext<EP> = TxContext<EP, Connected>;
pub type ConnlessTxContext<EP> = TxContext<EP, Connectionless>;

impl<I: 'static, STATE: EpState> TxContextImpl<I, STATE> {
    pub(crate) fn new(
        parent_ep: &MyRc<EndpointImplBase<I, dyn AsyncReadEq, dyn AsyncReadCq>>,
        index: i32,
        attr: TxAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<TxContextImpl<I, STATE>, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_tx_context(
                parent_ep.as_typed_fid_mut().as_raw_typed_fid(),
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
            Ok(TxContextImpl::<I, STATE> {
                #[cfg(not(any(
                    feature = "threading-domain",
                    feature = "threading-completion",
                    feature = "threading-endpoint"
                )))]
                c_ep: OwnedEpFid::from(c_ep),
                #[cfg(feature = "threading-completion")]
                c_ep: EpCompletionOwnedTypedFid::from(c_ep),
                #[cfg(feature = "threading-endpoint")]
                c_ep: XContextOwnedTypedFid::from(c_ep, parent_ep.fid().typed_fid.clone()),
                #[cfg(feature = "threading-domain")]
                c_ep: OwnedEpFid::from(c_ep, parent_ep.fid().domain.clone()),
                phantom: PhantomData,
                cq: MyOnceCell::new(),
                cntr: MyOnceCell::new(),
                _parent_ep: parent_ep.clone(),
                xphantom: PhantomData,
                iphantom: PhantomData,
            })
        }
    }
}

impl<I: 'static, STATE: EpState> TxContext<I, STATE> {
    pub(crate) fn new(
        ep: &EndpointBase<EndpointImplBase<I, dyn AsyncReadEq, dyn AsyncReadCq>, STATE>,
        index: i32,
        attr: TxAttr,
        context: Option<&mut Context>,
    ) -> Result<TxContext<I, STATE>, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: XContextBase {
                inner: MyRc::new(TxContextImpl::new(&ep.inner, index, attr, c_void)?),
            }
        })
    }
}

impl<I, STATE: EpState> AsyncTxEp for TxContext<I, STATE> {
    fn retrieve_tx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.inner.inner.cq.get().unwrap()
    }
}

impl<I, STATE: EpState> AsyncRxEp for RxContext<I, STATE> {
    fn retrieve_rx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.inner.inner.cq.get().unwrap()
    }
}

pub struct TxIncompleteBindCq<'a, I, STATE: EpState> {
    pub(crate) ep: &'a TxContextImplBase<I, STATE, dyn AsyncReadCq>,
    pub(crate) flags: u64,
}


impl<EP, STATE: EpState> TxContextImpl<EP, STATE> {
    pub(crate) fn bind_cq(&self) -> TxIncompleteBindCq<EP, STATE> {
        TxIncompleteBindCq { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cntr(&self) -> TxIncompleteBindCntr<EP, STATE> {
        TxIncompleteBindCntr { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cq_<T: AsyncReadCq + AsRawFid + 'static>(
        &self,
        res: &MyRc<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_typed_fid().as_raw_typed_fid(),
                res.as_raw_fid(),
                flags,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self.cq.set(res.clone()).is_err() {
                panic!("TxContext already bound to a CompletionQueueu");
            }
            Ok(())
        }
    }
}

impl<EP, STATE: EpState> AsyncTxEp for TxContextImpl<EP, STATE> {
    fn retrieve_tx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.cq.get().unwrap()
    }
}

impl<EP, STATE: EpState> TxContext<EP, STATE> {
    pub fn bind_cq(&self) -> TxIncompleteBindCq<EP, STATE> {
        self.inner.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> TxIncompleteBindCntr<EP, STATE> {
        self.inner.inner.bind_cntr()
    }
}

impl<'a, I, STATE: EpState> TxIncompleteBindCq<'a, I, STATE> {
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

    pub fn cq<T: AsyncReadCq + AsRawFid + 'static>(
        &mut self,
        cq: &crate::cq::CompletionQueue<T>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cq_(&cq.inner, self.flags)
    }
}

pub struct TxIncompleteBindCntr<'a, I, STATE: EpState> {
    pub(crate) ep: &'a TxContextImpl<I, STATE>,
    pub(crate) flags: u64,
}

impl<'a, I: 'static, STATE: EpState> TxIncompleteBindCntr<'a, I, STATE> {
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
        cntr: &Counter<impl AsRawFid + ReadCntr + 'static>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cntr_(&cntr.inner, self.flags)
    }
}

pub type RxContext<EP, STATE> = RxContextBase<EP, STATE, dyn AsyncReadCq>;
pub(crate) type RxContextImpl<I, STATE> = XContextBaseImpl<Receive, I, STATE, dyn AsyncReadCq>;


impl<I: 'static, STATE:EpState> RxContextImpl<I, STATE> {
    pub(crate) fn new(
        parent_ep: &MyRc<EndpointImplBase<I, dyn AsyncReadEq, dyn AsyncReadCq>>,
        index: i32,
        attr: RxAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_rx_context(
                parent_ep.as_typed_fid_mut().as_raw_typed_fid(),
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
                #[cfg(not(any(
                    feature = "threading-domain",
                    feature = "threading-completion",
                    feature = "threading-endpoint"
                )))]
                c_ep: OwnedEpFid::from(c_ep),
                #[cfg(feature = "threading-completion")]
                c_ep: EpCompletionOwnedTypedFid::from(c_ep),
                #[cfg(feature = "threading-domain")]
                c_ep: OwnedEpFid::from(c_ep, parent_ep.fid().domain.clone()),
                #[cfg(feature = "threading-endpoint")]
                c_ep: XContextOwnedTypedFid::from(c_ep, parent_ep.fid().typed_fid.clone()),
                phantom: PhantomData,
                xphantom: PhantomData,
                iphantom: PhantomData,
                cq: MyOnceCell::new(),
                cntr: MyOnceCell::new(),
                _parent_ep: parent_ep.clone(),
            })
        }
    }
}

impl<EP, STATE: EpState> RxContextImpl<EP, STATE> {
    pub(crate) fn bind_cq(&self) -> RxIncompleteBindCq<EP, STATE> {
        RxIncompleteBindCq { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cntr(&self) -> RxIncompleteBindCntr<EP, STATE> {
        RxIncompleteBindCntr { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cq_<T: AsyncReadCq + AsRawFid + 'static>(
        &self,
        res: &MyRc<T>,
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_ep_bind(
                self.as_typed_fid().as_raw_typed_fid(),
                res.as_raw_fid(),
                flags,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self.cq.set(res.clone()).is_err() {
                panic!("TxContext already bound to a CompletionQueueu");
            }
            Ok(())
        }
    }
}

impl<I: 'static, STATE: EpState> RxContext<I, STATE> {
    pub(crate) fn new(
        ep: &EndpointBase<EndpointImplBase<I, dyn AsyncReadEq, dyn AsyncReadCq>, STATE>,
        index: i32,
        attr: RxAttr,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: XContextBase {
                inner: MyRc::new(RxContextImpl::new(&ep.inner, index, attr, c_void)?),
            },
        })
    }
}

impl<EP, STATE: EpState> RxContext<EP, STATE> {
    pub fn bind_cq(&self) -> RxIncompleteBindCq<EP, STATE> {
        self.inner.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> RxIncompleteBindCntr<EP, STATE> {
        self.inner.inner.bind_cntr()
    }
}

impl<EP, STATE: EpState> AsyncRxEp for RxContextImpl<EP, STATE> {
    fn retrieve_rx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.cq.get().unwrap()
    }
}

pub struct RxContextBuilder<'a, I, STATE: EpState> {
    pub(crate) rx_attr: RxAttr,
    pub(crate) index: i32,
    pub(crate) ep: &'a EndpointBase<EndpointImplBase<I, dyn AsyncReadEq, dyn AsyncReadCq>, STATE>,
    pub(crate) ctx: Option<&'a mut Context>,
}


impl<'a, STATE: EpState> RxContextBuilder<'a, (), STATE> {
    pub fn new<I>(
        ep: &'a EndpointBase<EndpointImplBase<I, dyn AsyncReadEq, dyn AsyncReadCq>, STATE>,
        index: i32,
    ) -> RxContextBuilder<'a, I, STATE> {
        RxContextBuilder::<I, STATE> {
            rx_attr: RxAttr::new(),
            index,
            ep,
            ctx: None,
        }
    }
}

impl<'a, I: 'static , STATE: EpState>
    RxContextBuilder<'a, I, STATE>
{
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

    pub fn context(self, ctx: &'a mut Context) -> RxContextBuilder<'a, I, STATE> {
        RxContextBuilder {
            rx_attr: self.rx_attr,
            index: self.index,
            ep: self.ep,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<RxContext<I, STATE>, crate::error::Error> {
        RxContext::new(self.ep, self.index, self.rx_attr, self.ctx)
    }
}


pub struct TxContextBuilder<'a, I, STATE: EpState> {
    pub(crate) tx_attr: TxAttr,
    pub(crate) index: i32,
    pub(crate) ep: &'a EndpointBase<EndpointImplBase<I, dyn AsyncReadEq, dyn AsyncReadCq>, STATE>,
    pub(crate) ctx: Option<&'a mut Context>,
}

impl<'a, STATE: EpState> TxContextBuilder<'a, (), STATE> {
    pub fn new<I>(
        ep: &'a EndpointBase<EndpointImplBase<I, dyn AsyncReadEq, dyn AsyncReadCq>, STATE>,
        index: i32,
    ) -> TxContextBuilder<'a, I, STATE> {
        TxContextBuilder::<I, STATE> {
            tx_attr: TxAttr::new(),
            index,
            ep,
            ctx: None,
        }
    }
}

impl<'a, I: 'static, STATE: EpState>
    TxContextBuilder<'a, I, STATE>
{

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

    pub fn context(self, ctx: &'a mut Context) -> TxContextBuilder<'a, I, STATE> {
        TxContextBuilder {
            tx_attr: self.tx_attr,
            index: self.index,
            ep: self.ep,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<TxContext<I, STATE>, crate::error::Error> {
        TxContext::new(self.ep, self.index, self.tx_attr, self.ctx)
    }
}

pub struct RxIncompleteBindCq<'a, EP, STATE: EpState> {
    pub(crate) ep: &'a RxContextImpl<EP, STATE>,
    pub(crate) flags: u64,
}

impl<'a, EP, STATE: EpState> RxIncompleteBindCq<'a, EP, STATE> {
    pub fn recv(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_RECV as u64;

            self
        } else {
            self.flags |= libfabric_sys::FI_RECV as u64;

            self
        }
    }

    pub fn cq<T: AsyncReadCq + 'static + AsRawFid>(
        &self,
        cq: &crate::cq::CompletionQueue<T>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cq_(&cq.inner, self.flags)
    }
}

pub struct RxIncompleteBindCntr<'a, EP, STATE: EpState> {
    pub(crate) ep: &'a RxContextImpl<EP, STATE>,
    pub(crate) flags: u64,
}

impl<'a, I: 'static, STATE: EpState> RxIncompleteBindCntr<'a, I, STATE> {
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
        cntr: &Counter<impl AsRawFid + ReadCntr + 'static>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cntr_(&cntr.inner, self.flags)
    }
}
