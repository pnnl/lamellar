use crate::async_::ep::{AsyncRxEp, AsyncTxEp};
use crate::async_::xcontext::{RxContext, RxContextImpl, TxContext, TxContextImpl};
// use crate::async_::xcontext::{RxContext, RxContextImpl, TxContext, TxContextImpl};
use crate::comm::message::{
    ConnectedRecvEp, ConnectedSendEp, RecvEp, RecvEpImpl, SendEp, SendEpImpl,
};
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::ep::{Connected, Connectionless, EndpointImplBase, EpState};
use crate::infocapsoptions::{MsgCap, RecvMod, SendMod};
use crate::mr::{MemoryRegionDesc, MemoryRegionSlice, MemoryRegionSliceMut};
use crate::utils::Either;
use crate::Context;
use crate::{
    async_::{cq::AsyncReadCq, eq::AsyncReadEq},
    cq::SingleCompletion,
    enums::{RecvMsgOptions, SendMsgOptions},
    ep::EndpointBase,
    MappedAddress,
};

use super::while_try_again;

pub(crate) trait AsyncRecvEpImpl: AsyncRxEp + RecvEpImpl {
    async fn recv_async_imp<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_rx_cq();
        while_try_again(cq.as_ref(), || {
            self.recv_impl(buf, desc, mapped_addr, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[inline]
    async fn recvv_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_rx_cq();
        while_try_again(cq.as_ref(), || {
            self.recvv_impl(iov, desc, mapped_addr, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn recvmsg_async_impl<'a>(
        &self,
        mut msg: Either<&mut crate::msg::MsgMut<'a>, &mut crate::msg::MsgConnectedMut<'a>>,
        options: RecvMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let cq = self.retrieve_rx_cq();
        while_try_again(cq.as_ref(), || {
            self.recvmsg_impl(imm_msg.to_owned(), options)
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncRecvEp: RecvEp {
    fn recv_from_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recv_from_any_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recvv_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recvmsg_from_async(
        &self,
        msg: &mut crate::msg::MsgMut,
        options: RecvMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait AsyncRecvEpMrSlice: AsyncRecvEp {
    fn recv_mr_slice_from_async<T>(
        &self,
        mr_slice: &mut MemoryRegionSliceMut,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        let desc = mr_slice.desc();
        self.recv_from_async(mr_slice.as_mut_slice(), Some(desc), mapped_addr, ctx)
    }
    
    fn recv_mr_slice_from_any_async<T>(
        &self,
        mr_slice: &mut MemoryRegionSliceMut,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        let desc = mr_slice.desc();
        self.recv_from_any_async(mr_slice.as_mut_slice(), Some(desc), ctx)
    }
}

impl<EP: AsyncRecvEp> AsyncRecvEpMrSlice for EP {}

pub trait ConnectedAsyncRecvEp: ConnectedRecvEp {
    fn recv_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recvv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn recvmsg_async(
        &self,
        msg: &mut crate::msg::MsgConnectedMut,
        options: RecvMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncRecvEpMrSlice: ConnectedAsyncRecvEp {
    fn recv_mr_slice_async<T>(
        &self,
        mr_slice: &mut MemoryRegionSliceMut,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        let desc = mr_slice.desc();
        self.recv_async( mr_slice.as_mut_slice() , Some(desc), ctx)
    }
}

impl<EP: ConnectedAsyncRecvEp> ConnectedAsyncRecvEpMrSlice for EP {}

impl<E: AsyncRecvEpImpl> AsyncRecvEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncRecvEpImpl> AsyncRecvEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: MsgCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized> AsyncRecvEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: MsgCap + RecvMod, STATE: EpState> AsyncRecvEpImpl for RxContextImpl<I, STATE> {}

impl<I: MsgCap + RecvMod, STATE: EpState> AsyncRecvEpImpl for RxContext<I, STATE> {}

impl<EP: AsyncRecvEpImpl + ConnlessEp> AsyncRecvEp for EP {
    fn recv_from_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recv_async_imp(buf, desc, Some(mapped_addr), ctx)
    }

    fn recv_from_any_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recv_async_imp(buf, desc, None, ctx)
    }

    fn recvv_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, Some(mapped_addr), ctx)
    }

    fn recvmsg_from_async(
        &self,
        msg: &mut crate::msg::MsgMut,
        options: RecvMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvmsg_async_impl(Either::Left(msg), options)
    }
}

impl<EP: AsyncRecvEpImpl + ConnectedEp> ConnectedAsyncRecvEp for EP {
    fn recv_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recv_async_imp(buf, desc, None, ctx)
    }

    fn recvv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvv_async_impl(iov, desc, None, ctx)
    }

    fn recvmsg_async(
        &self,
        msg: &mut crate::msg::MsgConnectedMut,
        options: RecvMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.recvmsg_async_impl(Either::Right(msg), options)
    }
}

pub(crate) trait AsyncSendEpImpl: AsyncTxEp + SendEpImpl {
    async fn send_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.send_impl(buf, desc, mapped_addr, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn inject_async_impl<T>(
        &self,
        buf: &[T],
        mapped_addr: Option<&MappedAddress>,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || self.inject_impl(buf, mapped_addr)).await
    }

    async fn sendv_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.sendv_impl(iov, desc, mapped_addr, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn sendmsg_async_impl<'a>(
        &self,
        mut msg: Either<&mut crate::msg::Msg<'a>, &mut crate::msg::MsgConnected<'a>>,
        options: SendMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.sendmsg_impl(imm_msg.to_owned(), options)
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }

    async fn senddata_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: Option<&MappedAddress>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.senddata_impl(buf, desc, data, mapped_addr, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn injectdata_async_impl<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: Option<&MappedAddress>,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || self.injectdata_impl(buf, data, mapped_addr)).await
    }
}

pub trait AsyncSendEp: SendEp {
    fn send_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn inject_to_async<T>(
        &self,
        buf: &[T],
        mapped_addr: &MappedAddress,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    fn sendv_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn sendmsg_to_async(
        &self,
        msg: &mut crate::msg::Msg,
        options: SendMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn senddata_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn injectdata_to_async<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: &MappedAddress,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;
}

pub trait ConnectedAsyncSendEp: ConnectedSendEp {
    fn send_async<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn inject_async<T>(
        &self,
        buf: &[T],
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    fn sendv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn sendmsg_async(
        &self,
        msg: &mut crate::msg::MsgConnected,
        options: SendMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn senddata_async<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn injectdata_async<T>(
        &self,
        buf: &[T],
        data: u64,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;
}

pub trait AsyncSendEpMrSlice: AsyncSendEp {
    fn send_mr_slice_to_async<T>(
        &self,
        mr_slice: &MemoryRegionSlice,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.send_to_async( mr_slice.as_slice() , Some(mr_slice.desc()), mapped_addr, ctx)
    }

    fn inject_mr_slice_to_async<T>(
        &self,
        mr_slice: &MemoryRegionSlice,
        mapped_addr: &MappedAddress,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.inject_to_async( mr_slice.as_slice() , mapped_addr)
    }

    fn senddata_mr_slice_to_async<T>(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: u64,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.senddata_to_async( mr_slice.as_slice() , Some(mr_slice.desc()), data, mapped_addr, ctx)
    }

    fn injectdata_mr_slice_to_async<T>(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: u64,
        mapped_addr: &MappedAddress,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.injectdata_to_async( mr_slice.as_slice() , data, mapped_addr)
    }
}

impl<EP: AsyncSendEp> AsyncSendEpMrSlice for EP {}

pub trait ConnectedAsyncSendEpMrSlice: ConnectedAsyncSendEp {
    fn send_mr_slice_async<T>(
        &self,
        mr_slice: &MemoryRegionSlice,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.send_async( mr_slice.as_slice() , Some(mr_slice.desc()), ctx)
    }

    fn inject_mr_slice_async<T>(
        &self,
        mr_slice: &MemoryRegionSlice,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.inject_async( mr_slice.as_slice() )
    }

    fn senddata_mr_slice_async<T>(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.senddata_async( mr_slice.as_slice() , Some(mr_slice.desc()), data, ctx)
    }

    fn injectdata_mr_slice_async<T>(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: u64,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.injectdata_async( mr_slice.as_slice() , data)
    }
}

impl<EP: ConnectedAsyncSendEp> ConnectedAsyncSendEpMrSlice for EP {}

// impl<E, EQ: ?Sized +  AsyncReadEq,  CQ: AsyncReadCq + ? Sized> EndpointBase<E> {

impl<EP: AsyncSendEpImpl + ConnlessEp> AsyncSendEp for EP {
    async fn send_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.send_async_impl(buf, desc, Some(mapped_addr), ctx)
            .await
    }

    async fn inject_to_async<T>(
        &self,
        buf: &[T],
        mapped_addr: &MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.inject_async_impl(buf, Some(mapped_addr)).await
    }

    async fn sendv_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendv_async_impl(iov, desc, Some(mapped_addr), ctx)
            .await
    }

    async fn sendmsg_to_async<'a>(
        &self,
        msg: &mut crate::msg::Msg<'a>,
        options: SendMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendmsg_async_impl(Either::Left(msg), options).await
    }

    async fn senddata_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &MappedAddress,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, Some(mapped_addr), ctx)
            .await
    }

    async fn injectdata_to_async<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: &MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.injectdata_async_impl(buf, data, Some(mapped_addr))
            .await
    }
}

impl<EP: AsyncSendEpImpl + ConnectedEp> ConnectedAsyncSendEp for EP {
    async fn send_async<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.send_async_impl(buf, desc, None, ctx).await
    }

    async fn inject_async<T>(&self, buf: &[T]) -> Result<(), crate::error::Error> {
        self.inject_async_impl(buf, None).await
    }

    async fn sendv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendv_async_impl(iov, desc, None, ctx).await
    }

    async fn sendmsg_async<'a>(
        &self,
        msg: &mut crate::msg::MsgConnected<'a>,
        options: SendMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.sendmsg_async_impl(Either::Right(msg), options).await
    }

    async fn senddata_async<T>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.senddata_async_impl(buf, desc, data, None, ctx).await
    }

    async fn injectdata_async<T>(&self, buf: &[T], data: u64) -> Result<(), crate::error::Error> {
        self.injectdata_async_impl(buf, data, None).await
    }
}

impl<EP: MsgCap + SendMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized> AsyncSendEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: MsgCap + SendMod, STATE: EpState> AsyncSendEpImpl for TxContextImpl<I, STATE> {}

impl<I: MsgCap + SendMod, STATE: EpState> AsyncSendEpImpl for TxContext<I, STATE> {}

impl<E: AsyncSendEpImpl> AsyncSendEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncSendEpImpl> AsyncSendEpImpl for EndpointBase<E, Connectionless> {}
