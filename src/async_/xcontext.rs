use super::{
    cq::AsyncReadCq,
    ep::{AsyncRxEp, AsyncTxEp},
};
use crate::{
    cntr::{Counter, ReadCntr},
    ep::{ActiveEndpoint, EpState},
    fid::{AsRawFid, AsRawTypedFid, AsTypedFid, EpRawFid},
    xcontext::{
        Receive, RxContextBase, RxContextBuilder, Transmit, TxContextBase, TxContextBuilder, XContextBase, XContextBaseImpl
    },
    MyRc,
};
pub type TxContext<EP, STATE> = TxContextBase<EP, STATE, dyn AsyncReadCq>;

pub(crate) type TxContextImpl<EP, STATE> = XContextBaseImpl<Transmit, EP, STATE, dyn AsyncReadCq>;

impl<EP: ActiveEndpoint, STATE: EpState> TxContextImpl<EP, STATE> {
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

impl<EP: ActiveEndpoint, STATE: EpState> AsyncTxEp for TxContext<EP, STATE> {
    fn retrieve_tx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.inner.inner.retrieve_tx_cq()
    }
}

impl<EP: ActiveEndpoint, STATE: EpState> AsyncTxEp for TxContextImpl<EP, STATE> {
    fn retrieve_tx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.cq.get().unwrap()
    }
}

impl<EP: ActiveEndpoint, STATE: EpState> TxContext<EP, STATE> {
    pub fn bind_cq(&self) -> TxIncompleteBindCq<EP, STATE> {
        self.inner.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> TxIncompleteBindCntr<EP, STATE> {
        self.inner.inner.bind_cntr()
    }
}

pub struct TxIncompleteBindCq<'a, EP: ActiveEndpoint, STATE: EpState> {
    pub(crate) ep: &'a TxContextImpl<EP, STATE>,
    pub(crate) flags: u64,
}

impl<'a, EP: ActiveEndpoint, STATE: EpState> TxIncompleteBindCq<'a, EP, STATE> {
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

pub struct TxIncompleteBindCntr<'a, EP: ActiveEndpoint, STATE: EpState> {
    pub(crate) ep: &'a TxContextImpl<EP, STATE>,
    pub(crate) flags: u64,
}

impl<'a, EP: ActiveEndpoint, STATE: EpState> TxIncompleteBindCntr<'a, EP, STATE> {
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
pub(crate) type RxContextImpl<EP, STATE> = XContextBaseImpl<Receive, EP, STATE, dyn AsyncReadCq>;

impl<EP: ActiveEndpoint, STATE: EpState> RxContextImpl<EP, STATE> {
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

impl<EP: ActiveEndpoint, STATE: EpState> RxContext<EP, STATE> {
    pub fn bind_cq(&self) -> RxIncompleteBindCq<EP, STATE> {
        self.inner.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> RxIncompleteBindCntr<EP, STATE> {
        self.inner.inner.bind_cntr()
    }
}

impl<EP: ActiveEndpoint, STATE: EpState> AsyncRxEp for RxContext<EP, STATE> {
    fn retrieve_rx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.inner.inner.retrieve_rx_cq()
    }
}

impl<EP: ActiveEndpoint, STATE: EpState> AsyncRxEp for RxContextImpl<EP, STATE> {
    fn retrieve_rx_cq(&self) -> &MyRc<impl AsyncReadCq + ?Sized> {
        self.cq.get().unwrap()
    }
}

impl<'a, EP: ActiveEndpoint + AsyncRxEp, STATE: EpState> RxContextBuilder<'a, EP, STATE> {
    pub fn build_async(self) -> Result<RxContext<EP, STATE>, crate::error::Error> {
        RxContext::new(self.ep.inner.clone(), self.index, self.rx_attr, self.ctx)
    }
}

impl<'a, EP: ActiveEndpoint + AsyncTxEp, STATE: EpState>
    TxContextBuilder<'a, EP, STATE>
{
    pub fn build_async(self) -> Result<TxContext<EP, STATE>, crate::error::Error> {
        TxContext::new(self.ep.inner.clone(), self.index, self.tx_attr, self.ctx)
    }
}

pub struct RxIncompleteBindCq<'a, EP: ActiveEndpoint, STATE: EpState> {
    pub(crate) ep: &'a RxContextImpl<EP, STATE>,
    pub(crate) flags: u64,
}

impl<'a, EP: ActiveEndpoint, STATE: EpState> RxIncompleteBindCq<'a, EP, STATE> {
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

pub struct RxIncompleteBindCntr<'a, EP: ActiveEndpoint, STATE: EpState> {
    pub(crate) ep: &'a RxContextImpl<EP, STATE>,
    pub(crate) flags: u64,
}

impl<'a, EP: ActiveEndpoint, STATE: EpState> RxIncompleteBindCntr<'a, EP, STATE> {
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
