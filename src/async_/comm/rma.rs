use crate::async_::ep::AsyncTxEp;
use crate::async_::xcontext::{TxContext, TxContextImpl};
// use crate::async_::xcontext::{RxContext, RxContextImpl, TxContext, TxContextImpl};
use crate::comm::rma::{ConnectedWriteEp, ReadEpImpl, WriteEp, WriteEpImpl};
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::ep::{Connected, Connectionless, EndpointImplBase, EpState};
use crate::infocapsoptions::RmaCap;
use crate::mr::{MemoryRegionDesc, MemoryRegionSlice, MemoryRegionSliceMut};
use crate::msg::{MsgRma, MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut};
use crate::utils::Either;
use crate::{
    async_::{cq::AsyncCq, eq::AsyncReadEq},
    cq::SingleCompletion,
    enums::{ReadMsgOptions, WriteMsgOptions},
    ep::EndpointBase,
    infocapsoptions::{ReadMod, WriteMod},
    mr::MappedMemoryRegionKey,
    MappedAddress,
};
use crate::{Context, RemoteMemAddrSlice, RemoteMemAddrSliceMut, RemoteMemoryAddress};

use super::while_try_again;

pub(crate) trait AsyncReadEpImpl: AsyncTxEp + ReadEpImpl {
    async unsafe fn read_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        src_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            // println!("READ: while_try_again");

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
        mem_addr: RemoteMemoryAddress,
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
    unsafe fn read_from_async<T: Copy, RT: Copy>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        src_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn readmsg_from_async(
        &self,
        msg: &mut crate::msg::MsgRmaMut,
        options: ReadMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait AsyncReadEpMrSlice: AsyncReadEp {
    /// Async version of [crate::comm::rma::ReadEp::read_from]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::read_from]
    unsafe fn read_mr_slice_from_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &mut MemoryRegionSliceMut,
        src_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        let desc = mr_slice.desc();
        self.read_from_async(mr_slice.as_mut_slice(), Some(desc), src_addr, mem_addr, mapped_key, ctx)
    }
}

impl<EP: AsyncReadEp> AsyncReadEpMrSlice for EP {}

pub trait AsyncReadRemoteMemAddrSliceEp: AsyncReadEp {
    /// Async version of [crate::comm::rma::ReadEp::read_from]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::read_from]
    unsafe fn read_slice_from_async<T: Copy>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        src_addr: &MappedAddress,
        rma_slice: &RemoteMemAddrSlice<T>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.read_from_async(
            buf,
            desc,
            src_addr,
            rma_slice.mem_address(),
            &rma_slice.key(),
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::ReadEp::readv_from]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::readv_from]
    unsafe fn readv_slice_from_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &MappedAddress,
        rma_slice: &RemoteMemAddrSlice<u8>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(crate::iovec::IoVecMut::total_len(iov) == rma_slice.mem_len(), "Source and destination slice sizes do not match");
        self.readv_from_async(
            iov,
            desc,
            src_addr,
            rma_slice.mem_address(),
            &rma_slice.key(),
            ctx,
        )
    }

    unsafe fn readmsg_slice_from_async(
        &self,
        msg: &mut crate::msg::MsgRmaMut,
        options: ReadMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.readmsg_from_async(msg, options)
    }
}

pub trait ConnectedAsyncReadEp {
    /// Async version of [crate::comm::rma::ReadEp::read]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::read]
    unsafe fn read_async<T: Copy, RT: Copy>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
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

pub trait ConnectedAsyncReadEpMrSlice: ConnectedAsyncReadEp {
    /// Async version of [crate::comm::rma::ReadEp::read]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::read]
    unsafe fn read_mr_slice_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &mut MemoryRegionSliceMut,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        let desc = mr_slice.desc();
        self.read_async(
            mr_slice.as_mut_slice(),
            Some(desc),
            mem_addr,
            mapped_key,
            ctx,
        )
    }
}

impl<EP: ConnectedAsyncReadEp> ConnectedAsyncReadEpMrSlice for EP {}

pub trait ConnectedAsyncReadRemoteMemAddrSliceEp: ConnectedAsyncReadEp {
    /// Async version of [crate::comm::rma::ReadEp::read]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::read]
    unsafe fn read_slice_async<T: Copy>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        rma_slice: &RemoteMemAddrSlice<T>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.read_async(buf, desc, rma_slice.mem_address(), &rma_slice.key(), ctx)
    }

    /// Async version of [crate::comm::rma::ReadEp::readv]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::readv]
    unsafe fn readv_slice_async<'a>(
        &self,
        iov: &[crate::iovec::IoVecMut<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        rma_slice: &RemoteMemAddrSlice<u8>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(crate::iovec::IoVecMut::total_len(iov) == rma_slice.mem_len(), "Source and destination slice sizes do not match");
        self.readv_async(iov, desc, rma_slice.mem_address(), &rma_slice.key(), ctx)
    }

    /// Async version of [crate::comm::rma::ReadEp::readmsg]
    /// # Safety
    /// See [crate::comm::rma::ReadEp::readmsg]
    unsafe fn readmsg_slice_async(
        &self,
        msg: &mut crate::msg::MsgRmaConnectedMut,
        options: ReadMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.readmsg_async(msg, options)
    }
}

// impl<E: ReadMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointBase<E> {
impl<EP: RmaCap + ReadMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncCq + ?Sized> AsyncReadEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: RmaCap + ReadMod, STATE: EpState> AsyncReadEpImpl for TxContextImpl<I, STATE> {}

impl<I: RmaCap + ReadMod, STATE: EpState> AsyncReadEpImpl for TxContext<I, STATE> {}

impl<E: AsyncReadEpImpl> AsyncReadEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncReadEpImpl> AsyncReadEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AsyncReadEpImpl> AsyncReadEp for EP {
    async unsafe fn read_from_async<T: Copy, RT: Copy>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        src_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
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
    async unsafe fn read_async<T: Copy, RT: Copy>(
        &self,
        buf: &mut [T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
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
    async unsafe fn write_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            // println!("WRITE: while_try_again");
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

    async unsafe fn inject_write_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();

        let res = while_try_again(cq.as_ref(), || {
            self.inject_write_impl(buf, dest_mapped_addr, mem_addr, mapped_key)
        })
        .await;

        res
    }
    async unsafe fn writev_async_impl<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress,
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
    async unsafe fn writedata_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
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
    async unsafe fn inject_writedata_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.inject_writedata_impl(buf, data, dest_mapped_addr, mem_addr, mapped_key)
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

pub trait AsyncWriteEp {
    /// Async version of [crate::comm::rma::WriteEp::write_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::write_to]
    unsafe fn write_to_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::inject_write_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_write_to]
    unsafe fn inject_write_to_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
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
    unsafe fn writedata_to_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::inject_writedata_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_writedata_to]
    unsafe fn inject_writedata_to_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;
}

pub trait AsyncWriteRemoteMemAddrSliceEp: AsyncWriteEp {
    /// Async version of [crate::comm::rma::WriteEp::write_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::write_to]
    unsafe fn write_slice_to_async<T: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &MappedAddress,
        rma_slice: &RemoteMemAddrSliceMut<T>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.write_to_async(
            buf,
            desc,
            dest_addr,
            rma_slice.mem_address(),
            &rma_slice.key(),
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::WriteEp::inject_write_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_write_to]
    unsafe fn inject_write_slice_to_async<T: Copy>(
        &self,
        buf: &[T],
        dest_addr: &MappedAddress,
        rma_slice: &RemoteMemAddrSliceMut<T>,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.inject_write_to_async(buf, dest_addr, rma_slice.mem_address(), &rma_slice.key())
    }

    /// Async version of [crate::comm::rma::WriteEp::writev_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writev_to]
    unsafe fn writev_slice_to_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &MappedAddress,
        rma_slice: &RemoteMemAddrSliceMut<u8>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(crate::iovec::IoVec::total_len(iov) == rma_slice.mem_len(), "Source and destination slice sizes do not match");
        self.writev_to_async(
            iov,
            desc,
            dest_addr,
            rma_slice.mem_address(),
            &rma_slice.key(),
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::WriteEp::writemsg_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writemsg_to]
    unsafe fn writemsg_slice_to_async(
        &self,
        msg: &mut crate::msg::MsgRma,
        options: WriteMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.writemsg_to_async(msg, options)
    }

    /// Async version of [crate::comm::rma::WriteEp::writedata_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writedata_to]
    unsafe fn writedata_slice_to_async<T: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &MappedAddress,
        rma_slice: &RemoteMemAddrSliceMut<T>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.writedata_to_async(
            buf,
            desc,
            data,
            dest_addr,
            rma_slice.mem_address(),
            &rma_slice.key(),
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::WriteEp::inject_writedata_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_writedata_to]
    unsafe fn inject_writedata_slice_to_async<T: Copy>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: &MappedAddress,
        rma_slice: &RemoteMemAddrSliceMut<T>,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.inject_writedata_to_async(
            buf,
            data,
            dest_addr,
            rma_slice.mem_address(),
            &rma_slice.key(),
        )
    }
}

pub trait ConnectedAsyncWriteEp: ConnectedWriteEp {
    /// Async version of [crate::comm::rma::WriteEp::write]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::write]
    unsafe fn write_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::inject_write]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_write]
    unsafe fn inject_write_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::writev]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writev]
    unsafe fn writev_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
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
    unsafe fn writedata_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    /// Async version of [crate::comm::rma::WriteEp::inject_writedata]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_writedata]
    unsafe fn inject_writedata_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        data: u64,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;
}

pub trait ConnectedAsyncWriteRemoteMemAddrSliceEp: ConnectedAsyncWriteEp {
    /// Async version of [crate::comm::rma::WriteEp::write]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::write]
    unsafe fn write_slice_async<T: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        rma_slice: &RemoteMemAddrSliceMut<T>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.write_async(buf, desc, rma_slice.mem_address(), &rma_slice.key(), ctx)
    }

    /// Async version of [crate::comm::rma::WriteEp::inject_write]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_write]
    unsafe fn inject_write_slice_async<T: Copy>(
        &self,
        buf: &[T],
        rma_slice: &RemoteMemAddrSliceMut<T>,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.inject_write_async(buf, rma_slice.mem_address(), &rma_slice.key())
    }

    /// Async version of [crate::comm::rma::WriteEp::writev]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writev]
    unsafe fn writev_slice_async<'a>(
        &self,
        iov: &[crate::iovec::IoVec<'a>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        rma_slice: &RemoteMemAddrSliceMut<u8>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(crate::iovec::IoVec::total_len(iov) == rma_slice.mem_len(), "Source and destination slice sizes do not match");
        self.writev_async(iov, desc, rma_slice.mem_address(), &rma_slice.key(), ctx)
    }

    /// Async version of [crate::comm::rma::WriteEp::writemsg]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writemsg]
    unsafe fn writemsg_slice_async(
        &self,
        msg: &mut crate::msg::MsgRmaConnected,
        options: WriteMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.writemsg_async(msg, options)
    }

    /// Async version of [crate::comm::rma::WriteEp::writedata]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writedata]
    unsafe fn writedata_slice_async<T: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        rma_slice: &RemoteMemAddrSliceMut<T>,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.writedata_async(
            buf,
            desc,
            data,
            rma_slice.mem_address(),
            &rma_slice.key(),
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::WriteEp::inject_writedata]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_writedata]
    unsafe fn inject_writedata_slice_async<T: Copy>(
        &self,
        buf: &[T],
        data: u64,
        rma_slice: &RemoteMemAddrSliceMut<T>,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        assert!(
            std::mem::size_of_val(buf) == rma_slice.mem_size(),
            "Source and destination slice sizes do not match"
        );
        self.inject_writedata_async(buf, data, rma_slice.mem_address(), &rma_slice.key())
    }
}

pub trait AsyncWriteEpMrSlice: AsyncWriteEp {
    /// Async version of [crate::comm::rma::WriteEp::write_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::write_to]
    unsafe fn write_mr_slice_to_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &MemoryRegionSlice,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.write_to_async(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            dest_addr,
            mem_addr,
            mapped_key,
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::WriteEp::inject_write_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_write_to]
    unsafe fn inject_write_mr_slice_to_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &MemoryRegionSlice,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.inject_write_to_async(
            mr_slice.as_slice(),
            dest_addr,
            mem_addr,
            mapped_key,
        )
    }
    /// Async version of [crate::comm::rma::WriteEp::writedata_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writedata_to]
    unsafe fn writedata_mr_slice_to_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.writedata_to_async(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            data,
            dest_addr,
            mem_addr,
            mapped_key,
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::WriteEp::inject_writedata_to]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_writedata_to]
    unsafe fn inject_writedata_mr_slice_to_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.inject_writedata_to_async(
            mr_slice.as_slice(),
            data,
            dest_addr,
            mem_addr,
            mapped_key,
        )
    }
}

impl<EP: AsyncWriteEp> AsyncWriteEpMrSlice for EP {}

pub trait ConnectedAsyncWriteEpMrSlice: ConnectedAsyncWriteEp {
    /// Async version of [crate::comm::rma::WriteEp::write]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::write]
    unsafe fn write_mr_slice_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &MemoryRegionSlice,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.write_async(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            mem_addr,
            mapped_key,
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::WriteEp::inject_write]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_write]
    unsafe fn inject_write_mr_slice_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &MemoryRegionSlice,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.inject_write_async(
            mr_slice.as_slice(),
            mem_addr,
            mapped_key,
        )
    }
    /// Async version of [crate::comm::rma::WriteEp::writedata]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::writedata]
    unsafe fn writedata_mr_slice_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: u64,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.writedata_async(
            mr_slice.as_slice(),
            Some(mr_slice.desc()),
            data,
            mem_addr,
            mapped_key,
            ctx,
        )
    }

    /// Async version of [crate::comm::rma::WriteEp::inject_writedata]
    /// # Safety
    /// See [crate::comm::rma::WriteEp::inject_writedata]
    unsafe fn inject_writedata_mr_slice_async<T: Copy, RT: Copy>(
        &self,
        mr_slice: &MemoryRegionSlice,
        data: u64,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.inject_writedata_async(
            mr_slice.as_slice(),
            data,
            mem_addr,
            mapped_key,
        )
    }
}

impl<EP: ConnectedAsyncWriteEp> ConnectedAsyncWriteEpMrSlice for EP {}

// impl<E: WriteMod, EQ: ?Sized + AsyncReadEq,  CQ: AsyncReadCq  + ? Sized> EndpointBase<E> {
impl<EP: RmaCap + WriteMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncCq + ?Sized> AsyncWriteEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: RmaCap + WriteMod, STATE: EpState> AsyncWriteEpImpl for TxContextImpl<I, STATE> {}

impl<I: RmaCap + WriteMod, STATE: EpState> AsyncWriteEpImpl for TxContext<I, STATE> {}

impl<E: AsyncWriteEpImpl> AsyncWriteEpImpl for EndpointBase<E, Connected> {
    async unsafe fn write_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .write_async_impl(buf, desc, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn inject_write_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .writev_async_impl(iov, desc, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn writedata_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .writedata_async_impl(buf, desc, data, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn inject_writedata_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
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
    async unsafe fn write_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .write_async_impl(buf, desc, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn inject_write_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .writev_async_impl(iov, desc, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn writedata_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.inner
            .writedata_async_impl(buf, desc, data, dest_mapped_addr, mem_addr, mapped_key, ctx)
            .await
    }

    async unsafe fn inject_writedata_async_impl<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        data: u64,
        dest_mapped_addr: Option<&MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
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
    async unsafe fn write_to_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.write_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn inject_write_to_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
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
    async unsafe fn writedata_to_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn inject_writedata_to_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: &MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_writedata_async_impl(buf, data, Some(dest_addr), mem_addr, mapped_key)
            .await
    }
}

impl<EP: AsyncWriteEpImpl + ConnectedEp> ConnectedAsyncWriteEp for EP {
    #[inline]
    async unsafe fn write_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.write_async_impl(buf, desc, None, mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn inject_write_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress<RT>,
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
        mem_addr: RemoteMemoryAddress,
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
    async unsafe fn writedata_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.writedata_async_impl(buf, desc, data, None, mem_addr, mapped_key, ctx)
            .await
    }

    #[inline]
    async unsafe fn inject_writedata_async<T: Copy, RT: Copy>(
        &self,
        buf: &[T],
        data: u64,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_writedata_async_impl(buf, data, None, mem_addr, mapped_key)
            .await
    }
}

impl<EP: AsyncReadEp> AsyncReadRemoteMemAddrSliceEp for EP {}
impl<EP: AsyncWriteEp> AsyncWriteRemoteMemAddrSliceEp for EP {}

impl<EP: ConnectedAsyncReadEp> ConnectedAsyncReadRemoteMemAddrSliceEp for EP {}
impl<EP: ConnectedAsyncWriteEp> ConnectedAsyncWriteRemoteMemAddrSliceEp for EP {}

pub trait AsyncReadWriteEp: AsyncReadEp + AsyncWriteEp {}
impl<EP: AsyncReadEp + AsyncWriteEp> AsyncReadWriteEp for EP {}

pub trait AsyncReadWriteRemoteMemAddrSliceEp:
    AsyncReadRemoteMemAddrSliceEp + AsyncWriteRemoteMemAddrSliceEp
{
}
