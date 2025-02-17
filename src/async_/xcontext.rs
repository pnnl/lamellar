use super::{
    cq::AsyncReadCq,
    ep::{AsyncRxEp, AsyncTxEp},
};
use crate::{
    cntr::{Counter, ReadCntr},
    ep::EpState,
    fid::{AsRawFid, AsRawTypedFid, AsTypedFid, EpRawFid},
    xcontext::{
        Receive, ReceiveContextBuilder, Transmit, TxContextBuilder, XContextBase, XContextBaseImpl,
    },
    MyRc,
};

pub type TransmitContext = XContextBase<Transmit, dyn AsyncReadCq>;
pub(crate) type TransmitContextImpl = XContextBaseImpl<Transmit, dyn AsyncReadCq>;
impl TransmitContextImpl {
    pub(crate) fn bind_cq(&self) -> TxIncompleteBindCq {
        TxIncompleteBindCq { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cntr(&self) -> TxIncompleteBindCntr {
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
                panic!("TransmitContext already bound to a CompletionQueueu");
            }
            Ok(())
        }
    }
}

impl AsyncTxEp for TransmitContext {
    fn retrieve_tx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.inner.retrieve_tx_cq()
    }
}

impl AsyncTxEp for TransmitContextImpl {
    fn retrieve_tx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.cq.get().unwrap()
    }
}

impl TransmitContext {
    pub fn bind_cq(&self) -> TxIncompleteBindCq {
        self.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> TxIncompleteBindCntr {
        self.inner.bind_cntr()
    }
}

pub struct TxIncompleteBindCq<'a> {
    pub(crate) ep: &'a TransmitContextImpl,
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

    pub fn cq<T: AsyncReadCq + AsRawFid + 'static>(
        &mut self,
        cq: &crate::cq::CompletionQueue<T>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cq_(&cq.inner, self.flags)
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

    pub fn cntr(
        &self,
        cntr: &Counter<impl AsRawFid + ReadCntr + 'static>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cntr_(&cntr.inner, self.flags)
    }
}

pub type ReceiveContext = XContextBase<Receive, dyn AsyncReadCq>;
pub(crate) type ReceiveContextImpl = XContextBaseImpl<Receive, dyn AsyncReadCq>;

impl ReceiveContextImpl {
    pub(crate) fn bind_cq(&self) -> RxIncompleteBindCq {
        RxIncompleteBindCq { ep: self, flags: 0 }
    }

    pub(crate) fn bind_cntr(&self) -> RxIncompleteBindCntr {
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
                panic!("TransmitContext already bound to a CompletionQueueu");
            }
            Ok(())
        }
    }
}

impl ReceiveContext {
    pub fn bind_cq(&self) -> RxIncompleteBindCq {
        self.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> RxIncompleteBindCntr {
        self.inner.bind_cntr()
    }
}

impl AsyncRxEp for ReceiveContext {
    fn retrieve_rx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.inner.retrieve_rx_cq()
    }
}

impl AsyncRxEp for ReceiveContextImpl {
    fn retrieve_rx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.cq.get().unwrap()
    }
}

impl<'a, E: 'static, STATE: EpState> ReceiveContextBuilder<'a, E, STATE> {
    pub fn build_async(self) -> Result<ReceiveContext, crate::error::Error> {
        ReceiveContext::new(self.ep.inner.clone(), self.index, self.rx_attr, self.ctx)
    }
}

impl<'a, E: AsRawTypedFid<Output = EpRawFid> + 'static, STATE: EpState>
    TxContextBuilder<'a, E, STATE>
{
    pub fn build_async(self) -> Result<TransmitContext, crate::error::Error> {
        TransmitContext::new(self.ep.inner.clone(), self.index, self.tx_attr, self.ctx)
    }
}

pub struct RxIncompleteBindCq<'a> {
    pub(crate) ep: &'a ReceiveContextImpl,
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

    pub fn cq<T: AsyncReadCq + 'static + AsRawFid>(
        &self,
        cq: &crate::cq::CompletionQueue<T>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cq_(&cq.inner, self.flags)
    }
}

pub struct RxIncompleteBindCntr<'a> {
    pub(crate) ep: &'a ReceiveContextImpl,
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
        cntr: &Counter<impl AsRawFid + ReadCntr + 'static>,
    ) -> Result<(), crate::error::Error> {
        self.ep.bind_cntr_(&cntr.inner, self.flags)
    }
}
