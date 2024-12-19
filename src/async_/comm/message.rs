use crate::async_::ep::{AsyncRxEp, AsyncTxEp};
use crate::async_::xcontext::{
    ReceiveContext, ReceiveContextImpl, TransmitContext, TransmitContextImpl,
};
use crate::comm::message::{
    ConnectedRecvEp, ConnectedSendEp, RecvEp, RecvEpImpl, SendEp, SendEpImpl,
};
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::ep::{Connected, Connectionless, EndpointImplBase};
use crate::infocapsoptions::{MsgCap, RecvMod, SendMod};
use crate::utils::Either;
use crate::Context;
use crate::{
    async_::{cq::AsyncReadCq, eq::AsyncReadEq},
    cq::SingleCompletion,
    enums::{RecvMsgOptions, SendMsgOptions},
    ep::EndpointBase,
    mr::DataDescriptor,
    MappedAddress,
};

pub(crate) trait AsyncRecvEpImpl: AsyncRxEp + RecvEpImpl {
    async fn recv_async_imp<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.recv_impl(buf, desc, mapped_addr, Some(ctx.inner_mut()))?;
        let cq = self.retrieve_rx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    #[inline]
    async fn recvv_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: &mut [impl DataDescriptor],
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.recvv_impl(iov, desc, mapped_addr, Some(ctx.inner_mut()))?;
        let cq = self.retrieve_rx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    async fn recvmsg_async_impl<'a>(
        &self,
        mut msg: Either<&mut crate::msg::MsgMut<'a>, &mut crate::msg::MsgConnectedMut<'a>>,
        options: RecvMsgOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let c_msg = match &mut msg {
            Either::Left(msg) => &mut msg.c_msg,
            Either::Right(msg) => &mut msg.c_msg,
        };

        c_msg.context = ctx.inner_mut();

        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        self.recvmsg_impl(imm_msg, options)?;

        let cq = self.retrieve_rx_cq();
        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncRecvEp: RecvEp {
    fn recv_from_async<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recvv_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: &mut [impl DataDescriptor],
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recvmsg_from_async(
        &self,
        msg: &mut crate::msg::MsgMut,
        options: RecvMsgOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncRecvEp: ConnectedRecvEp {
    fn recv_async<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recvv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: &mut [impl DataDescriptor],
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recvmsg_async(
        &self,
        msg: &mut crate::msg::MsgConnectedMut,
        options: RecvMsgOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

impl<E: AsyncRecvEpImpl> AsyncRecvEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncRecvEpImpl> AsyncRecvEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: MsgCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized> AsyncRecvEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<EP: AsyncRecvEpImpl + ConnlessEp> AsyncRecvEp for EP {
    fn recv_from_async<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recv_async_imp(buf, desc, Some(mapped_addr), ctx)
    }

    fn recvv_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: &mut [impl DataDescriptor],
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, Some(mapped_addr), ctx)
    }

    fn recvmsg_from_async(
        &self,
        msg: &mut crate::msg::MsgMut,
        options: RecvMsgOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvmsg_async_impl(Either::Left(msg), options, ctx)
    }
}

impl<EP: AsyncRecvEpImpl + ConnectedEp> ConnectedAsyncRecvEp for EP {
    fn recv_async<T>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recv_async_imp(buf, desc, None, ctx)
    }

    fn recvv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: &mut [impl DataDescriptor],
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, None, ctx)
    }

    fn recvmsg_async(
        &self,
        msg: &mut crate::msg::MsgConnectedMut,
        options: RecvMsgOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvmsg_async_impl(Either::Right(msg), options, ctx)
    }
}

pub(crate) trait AsyncSendEpImpl: AsyncTxEp + SendEpImpl {
    async fn sendv_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: &mut [impl DataDescriptor],
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendv_impl(iov, desc, mapped_addr, Some(ctx.inner_mut()))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    async fn send_async_impl<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.send_impl(buf, desc, mapped_addr, Some(ctx.inner_mut()))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    async fn sendmsg_async_impl<'a>(
        &self,
        mut msg: Either<&mut crate::msg::Msg<'a>, &mut crate::msg::MsgConnected<'a>>,
        options: SendMsgOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let c_msg = match &mut msg {
            Either::Left(msg) => &mut msg.c_msg,
            Either::Right(msg) => &mut msg.c_msg,
        };

        c_msg.context = ctx.inner_mut();

        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        self.sendmsg_impl(imm_msg, options)?;

        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    async fn senddata_async_impl<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_impl(buf, desc, data, mapped_addr, Some(ctx.inner_mut()))?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncSendEp: SendEp {
    fn sendv_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: &mut [impl DataDescriptor],
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn send_to_async<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn sendmsg_to_async(
        &self,
        msg: &mut crate::msg::Msg,
        options: SendMsgOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn senddata_to_async<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncSendEp: ConnectedSendEp {
    fn sendv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: &mut [impl DataDescriptor],
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn send_async<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn sendmsg_async(
        &self,
        msg: &mut crate::msg::MsgConnected,
        options: SendMsgOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn senddata_async<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

// impl<E, EQ: ?Sized +  AsyncReadEq,  CQ: AsyncReadCq + ? Sized> EndpointBase<E> {

impl<EP: AsyncSendEpImpl + ConnlessEp> AsyncSendEp for EP {
    async fn sendv_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: &mut [impl DataDescriptor],
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendv_async_impl(iov, desc, Some(mapped_addr), ctx)
            .await
    }

    async fn send_to_async<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.send_async_impl(buf, desc, Some(mapped_addr), ctx)
            .await
    }

    async fn sendmsg_to_async<'a>(
        &self,
        msg: &mut crate::msg::Msg<'a>,
        options: SendMsgOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendmsg_async_impl(Either::Left(msg), options, ctx)
            .await
    }

    async fn senddata_to_async<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, Some(mapped_addr), ctx)
            .await
    }
}

impl<EP: AsyncSendEpImpl + ConnectedEp> ConnectedAsyncSendEp for EP {
    async fn sendv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: &mut [impl DataDescriptor],
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendv_async_impl(iov, desc, None, ctx).await
    }

    async fn send_async<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.send_async_impl(buf, desc, None, ctx).await
    }

    async fn sendmsg_async<'a>(
        &self,
        msg: &mut crate::msg::MsgConnected<'a>,
        options: SendMsgOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendmsg_async_impl(Either::Right(msg), options, ctx)
            .await
    }

    async fn senddata_async<T>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        data: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, None, ctx).await
    }
}

impl<EP: MsgCap + SendMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized> AsyncSendEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}
impl<E: AsyncSendEpImpl> AsyncSendEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncSendEpImpl> AsyncSendEpImpl for EndpointBase<E, Connectionless> {}

impl AsyncSendEpImpl for TransmitContext {}
impl AsyncSendEpImpl for TransmitContextImpl {}
impl AsyncRecvEpImpl for ReceiveContext {}
impl AsyncRecvEpImpl for ReceiveContextImpl {}
