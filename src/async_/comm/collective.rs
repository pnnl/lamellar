use crate::{
    async_::{
        cq::AsyncReadCq,
        ep::{AsyncCmEp, AsyncTxEp},
        eq::AsyncReadEq,
    },
    comm::collective::{
        CollectiveEp, CollectiveEpImpl, MulticastGroupCollective, MulticastGroupCollectiveImpl,
    },
    cq::SingleCompletion,
    enums::{CollectiveOptions, JoinOptions},
    ep::{Connected, Connectionless, EndpointBase, EndpointImplBase, EpState},
    eq::Event,
    error::Error,
    fid::{AsRawFid, AsTypedFid, EpRawFid, Fid},
    infocapsoptions::CollCap,
    mr::MemoryRegionDesc,
    AsFiType, Context, MyRc, SyncSend,
};

use super::while_try_again;

impl MulticastGroupCollectiveImpl {
    // pub(crate) async fn join_async_impl(&self, ep: &MyRc<impl AsyncCmEp + AsyncCollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static>, addr: &Address, options: JoinOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<Event, Error> {
    //     let mut async_ctx = AsyncCtx{user_ctx};
    //     self.join_impl(ep, options, Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void))?;
    //     let eq = ep.retrieve_eq();
    //     eq.async_event_wait(libfabric_sys::FI_JOIN_COMPLETE, self.as_raw_fid(),  &mut async_ctx as *mut AsyncCtx as usize).await
    // }

    pub(crate) async fn join_collective_async_impl(
        &self,
        ep: &MyRc<impl AsyncCollectiveEp + AsTypedFid<EpRawFid> + 'static>,
        options: JoinOptions,
        ctx: &mut Context,
    ) -> Result<Event, Error> {
        // let mut async_ctx = AsyncCtx::new(user_ctx );
        self.join_collective_impl(ep, options, Some(ctx.inner_mut()))?;
        let eq = ep.retrieve_eq();
        eq.async_event_wait(
            libfabric_sys::FI_JOIN_COMPLETE,
            Fid(self.as_typed_fid().as_raw_fid() as usize),
            Some(ctx),
        )
        .await
    }
}

impl MulticastGroupCollective {
    // pub async fn join_async(&self, ep: &EndpointBase<impl AsyncCollectiveEp + 'static + AsRawTypedFid<Output = EpRawFid>>, addr: &Address, options: JoinOptions) -> Result<Event, Error> {
    //     self.inner.join_async_impl(&ep.inner, addr, options, None).await
    // }

    pub async fn join_collective_async<
        E: CollectiveEp + AsTypedFid<EpRawFid> + 'static + AsyncCollectiveEp,
        STATE: EpState,
    >(
        &self,
        ep: &EndpointBase<E, STATE>,
        options: JoinOptions,
        ctx: &mut Context,
    ) -> Result<Event, Error> {
        self.inner
            .join_collective_async_impl(&ep.inner, options, ctx)
            .await
    }
}

trait AsyncCollectiveEpImpl: AsyncTxEp + CollectiveEpImpl {
    async fn barrier_impl_async(
        &self,
        mc_group: &MulticastGroupCollective,
        options: Option<CollectiveOptions>,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.barrier_impl(mc_group, Some(ctx.inner_mut()), options)
        })
        .await?;
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(ctx).await
    }

    async fn broadcast_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.broadcast_impl(
                buf,
                desc,
                mc_group,
                root_mapped_addr,
                options,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn alltoall_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.alltoall_impl(
                buf,
                desc,
                result,
                result_desc,
                mc_group,
                options,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn allreduce_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.allreduce_impl(
                buf,
                desc,
                result,
                result_desc,
                mc_group,
                op,
                options,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn allgather_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.allgather_impl(
                buf,
                desc,
                result,
                result_desc,
                mc_group,
                options,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn reduce_scatter_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.reduce_scatter_impl(
                buf,
                desc,
                result,
                result_desc,
                mc_group,
                op,
                options,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn reduce_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.reduce_impl(
                buf,
                desc,
                result,
                result_desc,
                mc_group,
                root_mapped_addr,
                op,
                options,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn scatter_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.scatter_impl(
                buf,
                desc,
                result,
                result_desc,
                mc_group,
                root_mapped_addr,
                options,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn gather_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.gather_impl(
                buf,
                desc,
                result,
                result_desc,
                mc_group,
                root_mapped_addr,
                options,
                Some(ctx.inner_mut()),
            )
        })
        .await?;
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncCollectiveEp: CollectiveEp + AsyncTxEp + AsyncCmEp + SyncSend {
    fn barrier_async(
        &self,
        mc_group: &MulticastGroupCollective,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn barrier_with_options_async(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // fn barrier_triggered_async<T>(&self, mc_group: &MulticastGroupCollective, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    // fn barrier_triggered_with_options_async<T>(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context : &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn broadcast_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // fn broadcast_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: Option<& impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context : &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error:BorrowedMemoryRegionDesc<'_>>;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    // #[allow(clippy::too_many_arguments)]
    // fn alltoall_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: Option<& impl DataDescriptor, result: &mut T, result_desc: Option<& impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error:BorrowedMemoryRegionDesc<'_>>;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    // #[allow(clippy::too_many_arguments)]
    // fn allreduce_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: Option<& impl DataDescriptor, result: &mut T, result_desc: Option<& impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::CollAtomicOp,  options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error:BorrowedMemoryRegionDesc<'_>>;
    #[allow(clippy::too_many_arguments)]
    fn allgather_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    // #[allow(clippy::too_many_arguments)]
    // fn allgather_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: Option<& impl DataDescriptor, result: &mut [T], result_desc: Option<& impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error:BorrowedMemoryRegionDesc<'_>>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    // fn reduce_scatter_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: Option<& impl DataDescriptor, result: &mut T, result_desc: Option<& impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::CollAtomicOp,  options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error:BorrowedMemoryRegionDesc<'_>>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    // fn reduce_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: Option<& impl DataDescriptor, result: &mut T, result_desc: Option<& impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::CollAtomicOp,  options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error:BorrowedMemoryRegionDesc<'_>>;
    #[allow(clippy::too_many_arguments)]
    fn scatter_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    // fn scatter_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: Option<& impl DataDescriptor, result: &mut T, result_desc: Option<& impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error:BorrowedMemoryRegionDesc<'_>>;
    #[allow(clippy::too_many_arguments)]
    fn gather_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;

    // fn gather_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: Option<& impl DataDescriptor, result: &mut T, result_desc: Option<& impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error:BorrowedMemoryRegionDesc<'_>>;
}

// impl<E, EQ: ?Sized + AsyncReadEq,  CQ: ?Sized + AsyncReadCq> EndpointBase<E, EQ, CQ> {

impl<EP: CollCap, EQ: ?Sized + AsyncReadEq, CQ: ?Sized + AsyncReadCq> AsyncCollectiveEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}
impl<E: AsyncCollectiveEpImpl> AsyncCollectiveEpImpl for EndpointBase<E, Connectionless> {}

impl<E: AsyncCollectiveEpImpl> AsyncCollectiveEpImpl for EndpointBase<E, Connected> {}

impl<EP: AsyncCollectiveEpImpl + AsyncTxEp + AsyncCmEp + SyncSend> AsyncCollectiveEp for EP {
    #[inline]
    fn barrier_async(
        &self,
        mc_group: &MulticastGroupCollective,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, None, ctx)
    }

    fn barrier_with_options_async(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, Some(options), ctx)
    }

    fn broadcast_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.broadcast_impl_async(buf, desc, mc_group, Some(root_mapped_addr), options, ctx)
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.alltoall_impl_async(buf, desc, result, result_desc, mc_group, options, ctx)
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allreduce_impl_async(buf, desc, result, result_desc, mc_group, op, options, ctx)
    }

    fn allgather_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allgather_impl_async(buf, desc, result, result_desc, mc_group, options, ctx)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.reduce_scatter_impl_async(buf, desc, result, result_desc, mc_group, op, options, ctx)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.reduce_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            op,
            options,
            ctx,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.scatter_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            ctx,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn gather_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        result: &mut T,
        result_desc: Option<&MemoryRegionDesc<'_>>,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        ctx: &mut Context,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.gather_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            ctx,
        )
    }
}
