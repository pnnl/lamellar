use crate::async_::cq::AsyncReadCq;
use crate::async_::eq::AsyncReadEq;
use crate::async_::xcontext::{TxContext, TxContextImpl};
// use crate::async_::xcontext::{TxContext, TxContextImpl};
use crate::comm::atomic::{AtomicCASImpl, AtomicFetchEpImpl};
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::enums::{AtomicFetchMsgOptions, AtomicMsgOptions};
use crate::ep::{Connected, Connectionless, EndpointBase, EndpointImplBase, EpState};
use crate::infocapsoptions::{AtomicCap, ReadMod, WriteMod};
use crate::mr::MemoryRegionDesc;
use crate::utils::Either;
use crate::{
    async_::ep::AsyncTxEp, comm::atomic::AtomicWriteEpImpl, cq::SingleCompletion,
    mr::MappedMemoryRegionKey, AsFiType, Context,
};
use crate::{RemoteMemAddrSlice, RemoteMemAddrSliceMut, RemoteMemoryAddress};

use super::while_try_again;

pub(crate) trait AsyncAtomicWriteEpImpl: AtomicWriteEpImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async fn atomic_async_impl<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.atomic_impl(
                buf,
                desc,
                dest_addr,
                mem_addr,
                mapped_key,
                op,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn inject_atomic_async_impl<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.inject_atomic_impl(buf, dest_addr, mem_addr, mapped_key, op)
        })
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn atomicv_async_impl<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.atomicv_impl(
                ioc,
                desc,
                dest_addr,
                mem_addr,
                mapped_key,
                op,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn atomicmsg_async_impl<T: AsFiType>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgAtomic<'_, T>,
            &mut crate::msg::MsgAtomicConnected<'_, T>,
        >,
        options: AtomicMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.atomicmsg_impl(imm_msg.to_owned(), options)
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncAtomicWriteEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn inject_atomic_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncAtomicWriteEp {
    unsafe fn atomic_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn inject_atomic_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>>;

    unsafe fn atomicv_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

impl<E: AsyncAtomicWriteEpImpl> AsyncAtomicWriteEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicWriteEpImpl> AsyncAtomicWriteEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + WriteMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicWriteEpImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + WriteMod, STATE: EpState> AsyncAtomicWriteEpImpl for TxContextImpl<I, STATE> {}

impl<I: AtomicCap + WriteMod, STATE: EpState> AsyncAtomicWriteEpImpl for TxContext<I, STATE> {}

impl<EP: AsyncAtomicWriteEpImpl + ConnlessEp> AsyncAtomicWriteEp for EP {
    #[inline]
    unsafe fn atomic_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomic_async_impl(
            buf,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[inline]
    unsafe fn inject_atomic_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.inject_atomic_async_impl(buf, Some(dest_addr), mem_addr, mapped_key, op)
    }

    #[inline]
    unsafe fn atomicv_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicv_async_impl(
            ioc,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[inline]
    unsafe fn atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_async_impl(Either::Left(msg), options)
    }
}

impl<EP: AsyncAtomicWriteEpImpl + ConnectedEp> ConnectedAsyncAtomicWriteEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomic_async_impl(buf, desc, None, mem_addr, mapped_key, op, context)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn inject_atomic_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        self.inject_atomic_async_impl(buf, None, mem_addr, mapped_key, op)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicv_async_impl(ioc, desc, None, mem_addr, mapped_key, op, context)
    }

    unsafe fn atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_async_impl(Either::Right(msg), options)
    }
}

pub trait AsyncAtomicWriteRemoteMemAddrSliceEp: AsyncAtomicWriteEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_slice_to_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(dst_slice.mem_size() == std::mem::size_of_val(buf));
        self.atomic_to_async(
            buf,
            desc,
            dest_addr,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn inject_atomic_slice_to_async<T: AsFiType>(
        &self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::AtomicOp,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        assert!(dst_slice.mem_size() == std::mem::size_of_val(buf));
        self.inject_atomic_to_async(
            buf,
            dest_addr,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_slice_to_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(dst_slice.mem_len() == crate::iovec::Ioc::total_len(ioc));
        self.atomicv_to_async(
            ioc,
            desc,
            dest_addr,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
            context,
        )
    }

    unsafe fn atomicmsg_slice_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_to_async(msg, options)
    }
}

impl<EP: AsyncAtomicWriteEp> AsyncAtomicWriteRemoteMemAddrSliceEp for EP {}

pub trait ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp: ConnectedAsyncAtomicWriteEp {
    unsafe fn atomic_slice_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(dst_slice.mem_size() == std::mem::size_of_val(buf));
        self.atomic_async(
            buf,
            desc,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
            context,
        )
    }

    unsafe fn inject_atomic_slice_async<T: AsFiType>(
        &self,
        buf: &[T],
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::AtomicOp,
    ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
        assert!(dst_slice.mem_size() == std::mem::size_of_val(buf));
        self.inject_atomic_async(buf, dst_slice.mem_address(), &dst_slice.key(), op)
    }

    unsafe fn atomicv_slice_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(dst_slice.mem_len() == crate::iovec::Ioc::total_len(ioc));
        self.atomicv_async(
            ioc,
            desc,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
            context,
        )
    }

    unsafe fn atomicmsg_slice_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_async(msg, options)
    }
}

impl<EP: ConnectedAsyncAtomicWriteEp> ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp for EP {}

pub(crate) trait AsyncAtomicFetchEpImpl: AtomicFetchEpImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async fn fetch_atomic_async_impl<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.fetch_atomic_impl(
                buf,
                desc,
                res,
                res_desc,
                dest_addr,
                mem_addr,
                mapped_key,
                op,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_atomicv_async_impl<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.fetch_atomicv_impl(
                ioc,
                desc,
                resultv,
                res_desc,
                dest_addr,
                mem_addr,
                mapped_key,
                op,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn fetch_atomicmsg_async_impl<T: AsFiType>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgFetchAtomic<'_, T>,
            &mut crate::msg::MsgFetchAtomicConnected<'_, T>,
        >,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.fetch_atomicmsg_impl(imm_msg.to_owned(), resultv, res_desc, options)
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn fetch_atomicmsg_from_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn fetch_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

impl<E: AsyncAtomicFetchEpImpl> AsyncAtomicFetchEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicFetchEpImpl> AsyncAtomicFetchEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + ReadMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicFetchEpImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + ReadMod, STATE: EpState> AsyncAtomicFetchEpImpl for TxContextImpl<I, STATE> {}

impl<I: AtomicCap + ReadMod, STATE: EpState> AsyncAtomicFetchEpImpl for TxContext<I, STATE> {}

impl<EP: AsyncAtomicFetchEpImpl + ConnlessEp> AsyncAtomicFetchEp for EP {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomic_async_impl(
            buf,
            desc,
            res,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicv_async_impl(
            ioc,
            desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    unsafe fn fetch_atomicmsg_from_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_async_impl(Either::Left(msg), resultv, res_desc, options)
    }
}

impl<EP: AsyncAtomicFetchEpImpl + ConnectedEp> ConnectedAsyncAtomicFetchEp for EP {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomic_async_impl(
            buf, desc, res, res_desc, None, mem_addr, mapped_key, op, context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicv_async_impl(
            ioc, desc, resultv, res_desc, None, mem_addr, mapped_key, op, context,
        )
    }

    unsafe fn fetch_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_async_impl(Either::Right(msg), resultv, res_desc, options)
    }
}

pub trait AsyncAtomicFetchRemoteMemAddrSliceEp: AsyncAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_slice_from_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(src_slice.mem_size() == std::mem::size_of_val(res));
        self.fetch_atomic_from_async(
            buf,
            desc,
            res,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            &src_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_slice_from_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(src_slice.mem_len() == crate::iovec::Ioc::total_len(resultv));
        self.fetch_atomicv_from_async(
            ioc,
            desc,
            resultv,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            &src_slice.key(),
            op,
            context,
        )
    }

    unsafe fn fetch_atomicmsg_slice_from_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_from_async(msg, resultv, res_desc, options)
    }
}

impl<EP: AsyncAtomicFetchEp> AsyncAtomicFetchRemoteMemAddrSliceEp for EP {}

pub trait ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp: ConnectedAsyncAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_slice_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<&MemoryRegionDesc<'_>>,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(src_slice.mem_size() == std::mem::size_of_val(res));
        self.fetch_atomic_async(
            buf,
            desc,
            res,
            res_desc,
            src_slice.mem_address(),
            &src_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_slice_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        src_slice: &RemoteMemAddrSlice<T>,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(src_slice.mem_len() == crate::iovec::Ioc::total_len(resultv));
        self.fetch_atomicv_async(
            ioc,
            desc,
            resultv,
            res_desc,
            src_slice.mem_address(),
            &src_slice.key(),
            op,
            context,
        )
    }

    unsafe fn fetch_atomicmsg_slice_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_async(msg, resultv, res_desc, options)
    }
}

impl<EP: ConnectedAsyncAtomicFetchEp> ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp for EP {}

pub(crate) trait AsyncAtomicCASImpl: AtomicCASImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async unsafe fn compare_atomic_async_impl<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.compare_atomic_impl(
                buf,
                desc,
                compare,
                compare_desc,
                result,
                result_desc,
                dest_addr,
                mem_addr,
                mapped_key,
                op,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async unsafe fn compare_atomicv_async_impl<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<'_, T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.compare_atomicv_impl(
                ioc,
                desc,
                comparetv,
                compare_desc,
                resultv,
                res_desc,
                dest_addr,
                mem_addr,
                mapped_key,
                op,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async unsafe fn compare_atomicmsg_async_impl<T: AsFiType>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgCompareAtomic<'_, T>,
            &mut crate::msg::MsgCompareAtomicConnected<'_, T>,
        >,
        comparev: &[crate::iovec::Ioc<'_, T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.compare_atomicmsg_impl(
                imm_msg.to_owned(),
                comparev,
                compare_desc,
                resultv,
                res_desc,
                options,
            )
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

impl<E: AsyncAtomicCASImpl> AsyncAtomicCASImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicCASImpl> AsyncAtomicCASImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + WriteMod + ReadMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicCASImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + WriteMod + ReadMod, STATE: EpState> AsyncAtomicCASImpl
    for TxContextImpl<I, STATE>
{
}

impl<I: AtomicCap + WriteMod + ReadMod, STATE: EpState> AsyncAtomicCASImpl for TxContext<I, STATE> {}

impl<EP: AsyncAtomicCASImpl + ConnlessEp> AsyncAtomicCASEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomic_async_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicv_async_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_async_impl(
            Either::Left(msg),
            comparev,
            compare_desc,
            resultv,
            res_desc,
            options,
        )
    }
}

impl<EP: AsyncAtomicCASImpl + ConnectedEp> ConnectedAsyncAtomicCASEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_async<T: AsFiType, RT: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomic_async_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_async<T: AsFiType, RT: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicv_async_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            op,
            context,
        )
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_async_impl(
            Either::Right(msg),
            comparev,
            compare_desc,
            resultv,
            res_desc,
            options,
        )
    }
}

pub trait AsyncAtomicCASRemoteMemAddrSliceEp: AsyncAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_slice_to_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(dst_slice.mem_size() == std::mem::size_of_val(result));
        self.compare_atomic_to_async(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dest_addr,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_slice_to_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(dst_slice.mem_len() == crate::iovec::Ioc::total_len(resultv));
        self.compare_atomicv_to_async(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dest_addr,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_slice_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_to_async(msg, comparev, compare_desc, resultv, res_desc, options)
    }
}

impl<EP: AsyncAtomicCASEp> AsyncAtomicCASRemoteMemAddrSliceEp for EP {}

pub trait ConnectedAsyncAtomicCASRemoteMemAddrSliceEp: ConnectedAsyncAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_slice_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        assert!(dst_slice.mem_size() == std::mem::size_of_val(result));
        self.compare_atomic_async(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_slice_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        // assert!(dst_slice.mem_len() == crate::iovec::Ioc::total_len(resultv));
        self.compare_atomicv_async(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dst_slice.mem_address(),
            &dst_slice.key(),
            op,
            context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_slice_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_async(msg, comparev, compare_desc, resultv, res_desc, options)
    }
}

impl<EP: ConnectedAsyncAtomicCASEp> ConnectedAsyncAtomicCASRemoteMemAddrSliceEp for EP {}
