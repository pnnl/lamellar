use crate::async_::cq::AsyncReadCq;
use crate::async_::eq::AsyncReadEq;
use crate::comm::atomic::{AtomicCASImpl, AtomicFetchEpImpl};
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::enums::{AtomicFetchMsgOptions, AtomicMsgOptions};
use crate::ep::{Connected, Connectionless, EndpointBase, EndpointImplBase};
use crate::infocapsoptions::{AtomicCap, ReadMod, WriteMod};
use crate::utils::Either;
use crate::{
    async_::ep::AsyncTxEp,
    comm::atomic::AtomicWriteEpImpl,
    cq::SingleCompletion,
    mr::{DataDescriptor, MappedMemoryRegionKey},
    AsFiType, Context,
};

pub(crate) trait AsyncAtomicWriteEpImpl: AtomicWriteEpImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async fn atomic_async_impl<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.atomic_impl(
            buf,
            desc,
            dest_addr,
            mem_addr,
            mapped_key,
            op,
            Some(ctx.inner_mut()),
        )?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn atomicv_async_impl<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: &mut [impl DataDescriptor],
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        self.atomicv_impl(
            ioc,
            desc,
            dest_addr,
            mem_addr,
            mapped_key,
            op,
            Some(ctx.inner_mut()),
        )?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    async fn atomicmsg_async_impl<T: AsFiType>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgAtomic<'_, T>,
            &mut crate::msg::MsgAtomicConnected<'_, T>,
        >,
        options: AtomicMsgOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let c_msg = match &mut msg {
            Either::Left(msg) => msg.inner_mut(),
            Either::Right(msg) => msg.inner_mut(),
        };

        c_msg.context = ctx.inner_mut();

        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        self.atomicmsg_impl(imm_msg, options)?;

        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncAtomicWriteEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_to_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_to_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncAtomicWriteEp {
    unsafe fn atomic_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn atomicv_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

impl<E: AsyncAtomicWriteEpImpl> AsyncAtomicWriteEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicWriteEpImpl> AsyncAtomicWriteEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + WriteMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicWriteEpImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<EP: AsyncAtomicWriteEpImpl + ConnlessEp> AsyncAtomicWriteEp for EP {
    #[inline]
    unsafe fn atomic_to_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn atomicv_to_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_async_impl(Either::Left(msg), options, context)
    }
}

impl<EP: AsyncAtomicWriteEpImpl + ConnectedEp> ConnectedAsyncAtomicWriteEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomic_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomic_async_impl(buf, desc, None, mem_addr, mapped_key, op, context)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn atomicv_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        mem_addr: u64,
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
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_async_impl(Either::Right(msg), options, context)
    }
}

pub(crate) trait AsyncAtomicFetchEpImpl: AtomicFetchEpImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async fn fetch_atomic_async_impl<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        res: &mut [T],
        res_desc: &mut impl DataDescriptor,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
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
        )?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_atomicv_async_impl<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: &mut [impl DataDescriptor],
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
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
        )?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    async fn fetch_atomicmsg_async_impl<T: AsFiType>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgFetchAtomic<'_, T>,
            &mut crate::msg::MsgFetchAtomicConnected<'_, T>,
        >,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: &mut [impl DataDescriptor],
        options: AtomicFetchMsgOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let c_atomic_msg = match &mut msg {
            Either::Left(msg) => msg.inner_mut(),
            Either::Right(msg) => msg.inner_mut(),
        };

        c_atomic_msg.context = ctx.inner_mut();

        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        self.fetch_atomicmsg_impl(imm_msg, resultv, res_desc, options)?;

        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        res: &mut [T],
        res_desc: &mut impl DataDescriptor,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_from_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn fetch_atomicmsg_from_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        options: AtomicFetchMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncAtomicFetchEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        res: &mut [T],
        res_desc: &mut impl DataDescriptor,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    unsafe fn fetch_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        options: AtomicFetchMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

impl<E: AsyncAtomicFetchEpImpl> AsyncAtomicFetchEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicFetchEpImpl> AsyncAtomicFetchEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + ReadMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicFetchEpImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<EP: AsyncAtomicFetchEpImpl + ConnlessEp> AsyncAtomicFetchEp for EP {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_from_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        res: &mut [T],
        res_desc: &mut impl DataDescriptor,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn fetch_atomicv_from_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
        res_desc: &mut [impl DataDescriptor],
        options: AtomicFetchMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_async_impl(Either::Left(msg), resultv, res_desc, options, context)
    }
}

impl<EP: AsyncAtomicFetchEpImpl + ConnectedEp> ConnectedAsyncAtomicFetchEp for EP {
    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomic_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        res: &mut [T],
        res_desc: &mut impl DataDescriptor,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::FetchAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomic_async_impl(
            buf, desc, res, res_desc, None, mem_addr, mapped_key, op, context,
        )
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn fetch_atomicv_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        mem_addr: u64,
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
        res_desc: &mut [impl DataDescriptor],
        options: AtomicFetchMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_async_impl(Either::Right(msg), resultv, res_desc, options, context)
    }
}

pub(crate) trait AsyncAtomicCASImpl: AtomicCASImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async unsafe fn compare_atomic_async_impl<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        compare: &[T],
        compare_desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
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
        )?;
        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async unsafe fn compare_atomicv_async_impl<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: &mut [impl DataDescriptor],
        comparetv: &[crate::iovec::Ioc<'_, T>],
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: &mut [impl DataDescriptor],
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
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
        )?;
        let cq = self.retrieve_tx_cq();
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
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: &mut [impl DataDescriptor],
        options: AtomicMsgOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let c_atomic_msg = match &mut msg {
            Either::Left(msg) => msg.inner_mut(),
            Either::Right(msg) => msg.inner_mut(),
        };

        c_atomic_msg.context = ctx.inner_mut();

        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        self.compare_atomicmsg_impl(imm_msg, comparev, compare_desc, resultv, res_desc, options)?;

        let cq = self.retrieve_tx_cq();
        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        compare: &[T],
        compare_desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_to_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        options: AtomicMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncAtomicCASEp {
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        compare: &[T],
        compare_desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicv_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        mem_addr: u64,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::CompareAtomicOp,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        options: AtomicMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

impl<E: AsyncAtomicCASImpl> AsyncAtomicCASImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicCASImpl> AsyncAtomicCASImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + WriteMod + ReadMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicCASImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<EP: AsyncAtomicCASImpl + ConnlessEp> AsyncAtomicCASEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_to_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        compare: &[T],
        compare_desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
    unsafe fn compare_atomicv_to_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        dest_addr: &crate::MappedAddress,
        mem_addr: u64,
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
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        options: AtomicMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_async_impl(
            Either::Left(msg),
            comparev,
            compare_desc,
            resultv,
            res_desc,
            options,
            context,
        )
    }
}

impl<EP: AsyncAtomicCASImpl + ConnectedEp> ConnectedAsyncAtomicCASEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomic_async<T: AsFiType>(
        &self,
        buf: &[T],
        desc: &mut impl DataDescriptor,
        compare: &[T],
        compare_desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mem_addr: u64,
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
    unsafe fn compare_atomicv_async<T: AsFiType>(
        &self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: &mut [impl DataDescriptor],
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        mem_addr: u64,
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
        compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: &mut [impl DataDescriptor],
        options: AtomicMsgOptions,
        context: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_async_impl(
            Either::Right(msg),
            comparev,
            compare_desc,
            resultv,
            res_desc,
            options,
            context,
        )
    }
}
