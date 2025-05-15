use crate::async_::ep::{AsyncRxEp, AsyncTxEp};
use crate::async_::xcontext::{RxContext, RxContextImpl, TxContext, TxContextImpl};
// use crate::async_::xcontext::{RxContext, RxContextImpl, TxContext, TxContextImpl};
use crate::comm::tagged::{ConnectedTagSendEp, TagRecvEpImpl, TagSendEp, TagSendEpImpl};
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::ep::{Connected, Connectionless, EndpointImplBase, EpState};
use crate::infocapsoptions::{RecvMod, SendMod, TagCap};
use crate::mr::MemoryRegionDesc;
use crate::msg::{MsgTagged, MsgTaggedConnected, MsgTaggedConnectedMut, MsgTaggedMut};
use crate::utils::Either;
use crate::Context;
use crate::{
    async_::{cq::AsyncReadCq, eq::AsyncReadEq},
    cq::SingleCompletion,
    enums::{TaggedRecvMsgOptions, TaggedSendMsgOptions},
    ep::EndpointBase,
    MappedAddress,
};

use super::while_try_again;

pub(crate) trait AsyncTagRecvEpImpl: AsyncRxEp + TagRecvEpImpl {
    async fn trecv_async_impl<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_rx_cq();
        while_try_again(cq.as_ref(), || {
            self.trecv_impl(buf, desc, mapped_addr, tag, ignore, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn trecvv_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_rx_cq();
        while_try_again(cq.as_ref(), || {
            self.trecvv_impl(
                iov,
                desc,
                src_mapped_addr,
                tag,
                ignore,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn trecvmsg_async_impl<'a>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgTaggedMut<'a>,
            &mut crate::msg::MsgTaggedConnectedMut<'a>,
        >,
        options: TaggedRecvMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::<&MsgTaggedMut, &MsgTaggedConnectedMut>::Left(msg),
            Either::Right(msg) => Either::<&MsgTaggedMut, &MsgTaggedConnectedMut>::Right(msg),
        };

        let cq = self.retrieve_rx_cq();
        while_try_again(cq.as_ref(), || {
            self.trecvmsg_impl(imm_msg.to_owned(), options)
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncTagRecvEp {
    fn trecv_from_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn trecvv_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn trecvmsg_from_async<'a>(
        &self,
        msg: &mut crate::msg::MsgTaggedMut<'a>,
        options: TaggedRecvMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncTagRecvEp {
    fn trecv_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn trecvv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn trecvmsg_async<'a>(
        &self,
        msg: &mut crate::msg::MsgTaggedConnectedMut<'a>,
        options: TaggedRecvMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

// impl<E: TagCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> EndpointBase<E> {
// impl<E:, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> EndpointBase<E> {
impl<EP: TagCap + RecvMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized> AsyncTagRecvEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: TagCap + RecvMod, STATE: EpState> AsyncTagRecvEpImpl
    for RxContextImpl<I, STATE>
{
}

impl<I: TagCap + RecvMod, STATE: EpState> AsyncTagRecvEpImpl
    for RxContext<I, STATE>
{
}

impl<E: AsyncTagRecvEpImpl> AsyncTagRecvEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncTagRecvEpImpl> AsyncTagRecvEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AsyncTagRecvEpImpl> AsyncTagRecvEp for EP {
    #[inline]
    async fn trecv_from_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, Some(mapped_addr), tag, ignore, ctx)
            .await
    }

    #[inline]
    async fn trecvv_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: &MappedAddress,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, Some(src_mapped_addr), tag, ignore, ctx)
            .await
    }

    #[inline]
    async fn trecvmsg_from_async<'a>(
        &self,
        msg: &mut crate::msg::MsgTaggedMut<'a>,
        options: TaggedRecvMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvmsg_async_impl(Either::Left(msg), options).await
    }
}

impl<EP: AsyncTagRecvEpImpl + ConnectedEp> ConnectedAsyncTagRecvEp for EP {
    #[inline]
    async fn trecv_async<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.trecv_async_impl(buf, desc, None, tag, ignore, ctx)
            .await
    }

    #[inline]
    async fn trecvv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ignore: Option<u64>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvv_async_impl(iov, desc, None, tag, ignore, ctx)
            .await
    }

    #[inline]
    async fn trecvmsg_async<'a>(
        &self,
        msg: &mut crate::msg::MsgTaggedConnectedMut<'a>,
        options: TaggedRecvMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.trecvmsg_async_impl(Either::Right(msg), options).await
    }
}

pub(crate) trait AsyncTagSendEpImpl: AsyncTxEp + TagSendEpImpl {
    async fn tsend_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.tsend_impl(buf, desc, mapped_addr, tag, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn tinject_async_impl<T>(
        &self,
        buf: &[T],
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || self.tinject_impl(buf, mapped_addr, tag)).await
    }

    async fn tsendv_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.tsendv_impl(iov, desc, dest_mapped_addr, tag, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn tsendmsg_async_impl<'a>(
        &self,
        mut msg: Either<&mut crate::msg::MsgTagged<'a>, &mut crate::msg::MsgTaggedConnected<'a>>,
        options: TaggedSendMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(ref msg) => Either::<&MsgTagged, &MsgTaggedConnected>::Left(msg),
            Either::Right(ref msg) => Either::<&MsgTagged, &MsgTaggedConnected>::Right(msg),
        };
        let cq = self.retrieve_tx_cq();

        while_try_again(cq.as_ref(), || {
            self.tsendmsg_impl(imm_msg.to_owned(), options)
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }

    async fn tsenddata_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.tsenddata_impl(buf, desc, data, mapped_addr, tag, Some(ctx.inner_mut()))
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn tinjectdata_async_impl<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: Option<&MappedAddress>,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.tinjectdata_impl(buf, data, mapped_addr, tag)
        })
        .await
    }
}

pub trait AsyncTagSendEp: TagSendEp {
    fn tsend_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn tinject_to_async<T>(
        &self,
        buf: &[T],
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    fn tsendv_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn tsendmsg_to_async<'a>(
        &self,
        msg: &mut crate::msg::MsgTagged<'a>,
        options: TaggedSendMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn tsenddata_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn tinjectdata_to_async<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;
}

pub trait ConnectedAsyncTagSendEp: ConnectedTagSendEp {
    fn tsend_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        tag: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn tinject_async<T>(
        &self,
        buf: &[T],
        tag: u64,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    fn tsendv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    fn tsendmsg_async<'a>(
        &self,
        msg: &mut crate::msg::MsgTaggedConnected<'a>,
        options: TaggedSendMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tsenddata_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        tag: u64,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn tinjectdata_async<T>(
        &self,
        buf: &[T],
        data: u64,
        tag: u64,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;
}

// impl<E: TagCap + SendMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ? Sized> EndpointBase<E> {
impl<EP: TagCap + SendMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized> AsyncTagSendEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}


impl<I: TagCap + SendMod, STATE: EpState> AsyncTagSendEpImpl
    for TxContextImpl<I, STATE>
{
}

impl<I: TagCap + SendMod, STATE: EpState> AsyncTagSendEpImpl
    for TxContext<I, STATE>
{
}

impl<E: AsyncTagSendEpImpl> AsyncTagSendEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncTagSendEpImpl> AsyncTagSendEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AsyncTagSendEpImpl + TagSendEpImpl + ConnlessEp> AsyncTagSendEp for EP {
    #[inline]
    async fn tsend_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mapped_addr: &MappedAddress,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, Some(mapped_addr), tag, ctx)
            .await
    }

    #[inline]
    async fn tinject_to_async<T>(
        &self,
        buf: &[T],
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinject_async_impl(buf, Some(mapped_addr), tag).await
    }

    #[inline]
    async fn tsendv_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: &MappedAddress,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.tsendv_async_impl(iov, desc, Some(dest_mapped_addr), tag, ctx)
            .await
    }

    #[inline]
    async fn tsendmsg_to_async<'a>(
        &self,
        msg: &mut crate::msg::MsgTagged<'a>,
        options: TaggedSendMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.tsendmsg_async_impl(Either::Left(msg), options).await
    }

    #[inline]
    async fn tsenddata_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, Some(mapped_addr), tag, ctx)
            .await
    }

    #[inline]
    async fn tinjectdata_to_async<T>(
        &self,
        buf: &[T],
        data: u64,
        mapped_addr: &MappedAddress,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinjectdata_async_impl(buf, data, Some(mapped_addr), tag)
            .await
    }
}

impl<EP: AsyncTagSendEpImpl + ConnectedEp> ConnectedAsyncTagSendEp for EP {
    #[inline]
    async fn tsend_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.tsend_async_impl(buf, desc, None, tag, ctx).await
    }

    #[inline]
    async fn tinject_async<T>(&self, buf: &[T], tag: u64) -> Result<(), crate::error::Error> {
        self.tinject_async_impl(buf, None, tag).await
    }

    #[inline]
    async fn tsendv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.tsendv_async_impl(iov, desc, None, tag, ctx).await
    }

    #[inline]
    async fn tsendmsg_async<'a>(
        &self,
        msg: &mut crate::msg::MsgTaggedConnected<'a>,
        options: TaggedSendMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.tsendmsg_async_impl(Either::Right(msg), options).await
    }

    #[inline]
    async fn tsenddata_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        tag: u64,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.tsenddata_async_impl(buf, desc, data, None, tag, ctx)
            .await
    }

    #[inline]
    async fn tinjectdata_async<T>(
        &self,
        buf: &[T],
        data: u64,
        tag: u64,
    ) -> Result<(), crate::error::Error> {
        self.tinjectdata_async_impl(buf, data, None, tag).await
    }
}