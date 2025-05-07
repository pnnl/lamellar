use crate::async_::ep::AsyncTxEp;
use crate::comm::rma::{ConnectedWriteEp, ReadEpImpl, WriteEp, WriteEpImpl};
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::ep::{Connected, Connectionless, EndpointImplBase};
use crate::infocapsoptions::RmaCap;
use crate::mr::MemoryRegionDesc;
use crate::msg::{MsgRma, MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut};
use crate::utils::Either;
use crate::Context;
use crate::{
    async_::{cq::AsyncReadCq, eq::AsyncReadEq},
    cq::SingleCompletion,
    enums::{ReadMsgOptions, WriteMsgOptions},
    ep::EndpointBase,
    infocapsoptions::{ReadMod, WriteMod},
    mr::MappedMemoryRegionKey,
    MappedAddress,
};

use super::while_try_again;

pub(crate) trait AsyncReadEpImpl: AsyncTxEp + ReadEpImpl {
    async unsafe fn read_async_impl<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.read_impl(
                buf,
                desc,
                src_mapped_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async unsafe fn readv_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.readv_impl(
                iov,
                desc,
                src_mapped_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async unsafe fn readmsg_async_impl<'a>(
        &self,
        mut msg: Either<&mut crate::msg::MsgRmaMut<'a>, &mut crate::msg::MsgRmaConnectedMut<'a>>,
        options: ReadMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match msg {
            Either::Left(ref mut msg) => Either::<&MsgRmaMut, &MsgRmaConnectedMut>::Left(msg),
            Either::Right(ref mut msg) => Either::<&MsgRmaMut, &MsgRmaConnectedMut>::Right(msg),
        };
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.readmsg_impl(imm_msg.to_owned(), options)
        })
        .await?;

        let ctx = match msg {
            Either::Left(ref mut msg) => msg.context(),
            Either::Right(ref mut msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncReadEp {
    /// Async version of [crate::comm::rma::ReadEp::read_from]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::read_from]
    unsafe fn read_from_async<T0>(
        &self,
        buf: &mut [T0],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::ReadEp::readv_from]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::readv_from]
    unsafe fn readv_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn readmsg_from_async(
        &self,
        msg: &mut crate::msg::MsgRmaMut,
        options: ReadMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncReadEp {
    /// Async version of [crate::comm::rma::ReadEp::read]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::read]
    unsafe fn read_async<T0>(
        &self,
        buf: &mut [T0],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::ReadEp::readv]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::readv]
    unsafe fn readv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::ReadEp::readmsg]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::readmsg]
    unsafe fn readmsg_async(
        &self,
        msg: &mut crate::msg::MsgRmaConnectedMut,
        options: ReadMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

// impl<E: ReadMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointBase<E> {
impl<EP: RmaCap + ReadMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized> AsyncReadEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<E: AsyncReadEpImpl> AsyncReadEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncReadEpImpl> AsyncReadEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AsyncReadEpImpl> AsyncReadEp for EP {
    async unsafe fn read_from_async<T0>(
        &self,
        buf: &mut [T0],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.read_async_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn readv_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.readv_async_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn readmsg_from_async<'a>(
        &self,
        msg: &mut crate::msg::MsgRmaMut<'a>,
        options: ReadMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.readmsg_async_impl(Either::Left(msg), options).await
    }
}

impl<EP: AsyncReadEpImpl> ConnectedAsyncReadEp for EP {
    async unsafe fn read_async<T0>(
        &self,
        buf: &mut [T0],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.read_async_impl(buf, desc, None, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn readv_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.readv_async_impl(iov, desc, None, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn readmsg_async<'a>(
        &self,
        msg: &mut crate::msg::MsgRmaConnectedMut<'a>,
        options: ReadMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.readmsg_async_impl(Either::Right(msg), options).await
    }
}

pub(crate) trait AsyncWriteEpImpl: AsyncTxEp + WriteEpImpl {
    async unsafe fn write_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();

        while_try_again(cq.as_ref(), || {
            self.write_impl(
                buf,
                desc,
                dest_mapped_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async unsafe fn inject_write_async_impl<T>(
        &self,
        buf: &[T],
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();

        while_try_again(cq.as_ref(), || {
            self.inject_write_impl(buf, dest_mapped_addr, mem_addr, mapped_key)
        })
        .await
    }

    async unsafe fn writev_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.writev_impl(
                iov,
                desc,
                dest_mapped_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async unsafe fn writedata_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.writedata_impl(
                buf,
                desc,
                data,
                dest_mapped_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async unsafe fn inject_writedata_async_impl<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.inject_writedata_impl(
                buf,
                data,
                dest_mapped_addr,
                mem_addr,
                mapped_key,
            )
        })
        .await
    }

    async unsafe fn writemsg_async_impl<'a>(
        &self,
        mut msg: Either<&mut crate::msg::MsgRma<'a>, &mut crate::msg::MsgRmaConnected<'a>>,
        options: WriteMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match msg {
            Either::Left(ref mut msg) => Either::<&MsgRma, &MsgRmaConnected>::Left(msg),
            Either::Right(ref mut msg) => Either::<&MsgRma, &MsgRmaConnected>::Right(msg),
        };
        let cq = self.retrieve_tx_cq();

        while_try_again(cq.as_ref(), || {
            self.writemsg_impl(imm_msg.to_owned(), options)
        })
        .await?;

        let ctx = match msg {
            Either::Left(ref mut msg) => msg.context(),
            Either::Right(ref mut msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncWriteEp: WriteEp {
    /// Async version of [crate::comm::rma::WriteEp::write_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::write_to]
    unsafe fn write_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::inject_write_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_write_to]
    unsafe fn inject_write_to_async<T>(
        &self,
        buf: &[T],
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::writev_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writev_to]
    unsafe fn writev_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::writemsg_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writemsg_to]
    unsafe fn writemsg_to_async(
        &self,
        msg: &mut crate::msg::MsgRma,
        options: WriteMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::writedata_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writedata_to]
    unsafe fn writedata_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::inject_writedata_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_writedata_to]
    unsafe fn inject_writedata_to_async<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;
}

pub trait ConnectedAsyncWriteEp: ConnectedWriteEp {
    /// Async version of [crate::comm::rma::WriteEp::write]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::write]
    unsafe fn write_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::inject_write]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_write]
    unsafe fn inject_write_async<T>(
        &self,
        buf: &[T],
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::writev]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writev]
    unsafe fn writev_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::writemsg]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writemsg]
    unsafe fn writemsg_async(
        &self,
        msg: &mut crate::msg::MsgRmaConnected,
        options: WriteMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::writedata]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writedata]
    unsafe fn writedata_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::inject_writedata]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_writedata]
    unsafe fn inject_writedata_async<T>(
        &self,
        buf: &[T],
        data: u64,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;
}

// impl<E: WriteMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointBase<E> {
impl<EP: RmaCap + WriteMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized> AsyncWriteEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<E: AsyncWriteEpImpl> AsyncWriteEpImpl for EndpointBase<E, Connected> {
    async unsafe fn write_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .write_async_impl(buf, desc, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn inject_write_async_impl<T>(
        &self,
        buf: &[T],
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inner
            .inject_write_async_impl(buf, dest_mapped_addr, mem_addr, mapped_key)
            .await
    }

    async unsafe fn writev_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .writev_async_impl(iov, desc, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn writedata_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .writedata_async_impl(buf, desc, data, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn inject_writedata_async_impl<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inner
            .inject_writedata_async_impl(buf, data, dest_mapped_addr, mem_addr, mapped_key)
            .await
    }

    async unsafe fn writemsg_async_impl<'a>(
        &self,
        msg: Either<&mut crate::msg::MsgRma<'a>, &mut crate::msg::MsgRmaConnected<'a>>,
        options: WriteMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.writemsg_async_impl(msg, options).await
    }
}

impl<E: AsyncWriteEpImpl> AsyncWriteEpImpl for EndpointBase<E, Connectionless> {
    async unsafe fn write_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .write_async_impl(buf, desc, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn inject_write_async_impl<T>(
        &self,
        buf: &[T],
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inner
            .inject_write_async_impl(buf, dest_mapped_addr, mem_addr, mapped_key)
            .await
    }

    async unsafe fn writev_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .writev_async_impl(iov, desc, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn writedata_async_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .writedata_async_impl(buf, desc, data, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn inject_writedata_async_impl<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inner
            .inject_writedata_async_impl(buf, data, dest_mapped_addr, mem_addr, mapped_key)
            .await
    }

    async unsafe fn writemsg_async_impl<'a>(
        &self,
        msg: Either<&mut crate::msg::MsgRma<'a>, &mut crate::msg::MsgRmaConnected<'a>>,
        options: WriteMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.writemsg_async_impl(msg, options).await
    }
}

impl<EP: AsyncWriteEpImpl + ConnlessEp> AsyncWriteEp for EP {
    #[inline]
    async unsafe fn write_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn inject_write_to_async<T>(
        &self,
        buf: &[T],
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_write_async_impl(buf, Some(dest_addr), mem_addr, mapped_key)
            .await
    }

    #[inline]
    async unsafe fn writev_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.writev_async_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn writemsg_to_async<'a>(
        &self,
        msg: &mut crate::msg::MsgRma<'a>,
        options: WriteMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.writemsg_async_impl(Either::Left(msg), options).await
    }

    #[inline]
    async unsafe fn writedata_to_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn inject_writedata_to_async<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_writedata_async_impl(buf, data, Some(dest_addr), mem_addr, mapped_key)
            .await
    }
}

impl<EP: AsyncWriteEpImpl + ConnectedEp> ConnectedAsyncWriteEp for EP {
    #[inline]
    async unsafe fn write_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.write_async_impl(buf, desc, None, mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn inject_write_async<T>(
        &self,
        buf: &[T],
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_write_async_impl(buf, None, mem_addr, mapped_key)
            .await
    }

    #[inline]
    async unsafe fn writev_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.writev_async_impl(iov, desc, None, mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn writemsg_async<'a>(
        &self,
        msg: &mut crate::msg::MsgRmaConnected<'a>,
        options: WriteMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.writemsg_async_impl(Either::Right(msg), options).await
    }

    #[inline]
    async unsafe fn writedata_async<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn inject_writedata_async<T>(
        &self,
        buf: &[T],
        data: u64,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_writedata_async_impl(buf, data, None, mem_addr, mapped_key)
            .await
    }
}

pub trait AsyncReadWriteEp: AsyncReadEp + AsyncWriteEp {}
impl<EP: AsyncReadEp + AsyncWriteEp> AsyncReadWriteEp for EP {}

// impl AsyncWriteEpImpl for TransmitContext {}
// impl AsyncWriteEpImpl for TransmitContextImpl {}
// impl AsyncReadEpImpl for ReceiveContext {}
// impl AsyncReadEpImpl for ReceiveContextImpl {}
