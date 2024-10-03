use crate::{
    async_::{
        cq::AsyncReadCq,
        ep::{AsyncCmEp, AsyncTxEp},
        eq::AsyncReadEq,
        AsyncCtx,
    },
    comm::collective::{
        CollectiveEp, CollectiveEpImpl, MulticastGroupCollective, MulticastGroupCollectiveImpl,
    },
    cq::SingleCompletion,
    enums::{CollectiveOptions, JoinOptions},
    ep::{Connected, Connectionless, EndpointBase, EndpointImplBase, EpState},
    eq::Event,
    error::Error,
    fid::{AsRawFid, AsRawTypedFid, EpRawFid, Fid},
    infocapsoptions::CollCap,
    mr::DataDescriptor,
    AsFiType, MyRc,
};

impl MulticastGroupCollectiveImpl {
    // pub(crate) async fn join_async_impl(&self, ep: &MyRc<impl AsyncCmEp + AsyncCollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static>, addr: &Address, options: JoinOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<Event, Error> {
    //     let mut async_ctx = AsyncCtx{user_ctx};
    //     self.join_impl(ep, options, Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void))?;
    //     let eq = ep.retrieve_eq();
    //     eq.async_event_wait(libfabric_sys::FI_JOIN_COMPLETE, self.as_raw_fid(),  &mut async_ctx as *mut AsyncCtx as usize).await
    // }

    pub(crate) async fn join_collective_async_impl(
        &self,
        ep: &MyRc<impl AsyncCollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static>,
        options: JoinOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<Event, Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.join_collective_impl(
            ep,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let eq = ep.retrieve_eq();
        eq.async_event_wait(
            libfabric_sys::FI_JOIN_COMPLETE,
            Fid(self.as_raw_fid()),
            &mut async_ctx as *mut AsyncCtx as usize,
        )
        .await
    }
}

impl MulticastGroupCollective {
    // pub async fn join_async(&self, ep: &EndpointBase<impl AsyncCollectiveEp + 'static + AsRawTypedFid<Output = EpRawFid>>, addr: &Address, options: JoinOptions) -> Result<Event, Error> {
    //     self.inner.join_async_impl(&ep.inner, addr, options, None).await
    // }

    // pub async fn join_async_with_context<T>(&self, ep: &EndpointBase<impl AsyncCollectiveEp + 'static + AsRawTypedFid<Output = EpRawFid>>, addr: &Address, options: JoinOptions, context: &mut T) -> Result<Event, Error> {
    //     self.inner.join_async_impl(&ep.inner, addr, options, Some((context as *mut T) as *mut std::ffi::c_void)).await
    // }

    pub async fn join_collective_async<
        E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static + AsyncCollectiveEp,
        STATE: EpState,
    >(
        &self,
        ep: &EndpointBase<E, STATE>,
        options: JoinOptions,
    ) -> Result<Event, Error> {
        self.inner
            .join_collective_async_impl(&ep.inner, options, None)
            .await
    }

    pub async fn join_collective_async_with_context<
        T,
        E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static + AsyncCollectiveEp,
        STATE: EpState,
    >(
        &self,
        ep: &EndpointBase<E, STATE>,
        options: JoinOptions,
        context: &mut T,
    ) -> Result<Event, Error> {
        self.inner
            .join_collective_async_impl(
                &ep.inner,
                options,
                Some((context as *mut T) as *mut std::ffi::c_void),
            )
            .await
    }
}

trait AsyncCollectiveEpImpl: AsyncTxEp + CollectiveEpImpl {
    async fn barrier_impl_async(
        &self,
        mc_group: &MulticastGroupCollective,
        user_ctx: Option<*mut std::ffi::c_void>,
        options: Option<CollectiveOptions>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.barrier_impl(
            mc_group,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
            options,
        )?;
        let cq = self.retrieve_tx_cq();
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    async fn broadcast_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.broadcast_impl(
            buf,
            desc,
            mc_group,
            root_mapped_addr,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let cq = self.retrieve_tx_cq();
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn alltoall_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.alltoall_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let cq = self.retrieve_tx_cq();
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn allreduce_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.allreduce_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            op,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let cq = self.retrieve_tx_cq();
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn allgather_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.allgather_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let cq = self.retrieve_tx_cq();
        // while tx_cq.trywait().is_err() {tx_cq.read(0);}
        // while rx_cq.trywait().is_err() {rx_cq.read(0);}
        // while cq.read(1).is_err() {}
        // while rxcq.read(1).is_err() {}
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn reduce_scatter_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.reduce_scatter_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            op,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let cq = self.retrieve_tx_cq();
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn reduce_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.reduce_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            root_mapped_addr,
            op,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let cq = self.retrieve_tx_cq();
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn scatter_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.scatter_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            root_mapped_addr,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let cq = self.retrieve_tx_cq();
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn gather_impl_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        user_ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx { user_ctx };
        self.gather_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            root_mapped_addr,
            options,
            Some(&mut async_ctx as *mut AsyncCtx as *mut std::ffi::c_void),
        )?;
        let cq = self.retrieve_tx_cq();
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
}

pub trait AsyncCollectiveEp: CollectiveEp + AsyncTxEp + AsyncCmEp {
    fn barrier_async(
        &self,
        mc_group: &MulticastGroupCollective,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn barrier_with_options_async(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn barrier_with_context_async<T>(
        &self,
        mc_group: &MulticastGroupCollective,
        context: &mut T,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    fn barrier_with_context_with_options_async<T>(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut T,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // fn barrier_triggered_async<T>(&self, mc_group: &MulticastGroupCollective, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    // fn barrier_triggered_with_options_async<T>(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context : &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn broadcast_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // #[allow(clippy::too_many_arguments)]
    // fn broadcast_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context : &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // #[allow(clippy::too_many_arguments)]
    // fn alltoall_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // #[allow(clippy::too_many_arguments)]
    // fn allreduce_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::CollAtomicOp,  options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn allgather_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // #[allow(clippy::too_many_arguments)]
    // fn allgather_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // #[allow(clippy::too_many_arguments)]
    // fn reduce_scatter_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::CollAtomicOp,  options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // #[allow(clippy::too_many_arguments)]
    // fn reduce_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::CollAtomicOp,  options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn scatter_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    #[allow(clippy::too_many_arguments)]
    fn scatter_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // #[allow(clippy::too_many_arguments)]
    // fn scatter_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn gather_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    #[allow(clippy::too_many_arguments)]
    fn gather_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
    // #[allow(clippy::too_many_arguments)]
    // fn gather_triggered_async<T: AsFiType, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut TriggeredContext) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}

// impl<E, EQ: ?Sized + AsyncReadEq,  CQ: ?Sized + AsyncReadCq> EndpointBase<E, EQ, CQ> {

impl<EP: CollCap, EQ: ?Sized + AsyncReadEq, CQ: ?Sized + AsyncReadCq> AsyncCollectiveEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}
impl<E: AsyncCollectiveEpImpl> AsyncCollectiveEpImpl for EndpointBase<E, Connectionless> {}

impl<E: AsyncCollectiveEpImpl> AsyncCollectiveEpImpl for EndpointBase<E, Connected> {}

impl<EP: AsyncCollectiveEpImpl + AsyncTxEp + AsyncCmEp> AsyncCollectiveEp for EP {
    #[inline]
    fn barrier_async(
        &self,
        mc_group: &MulticastGroupCollective,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, None, None)
    }

    fn barrier_with_options_async(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, None, Some(options))
    }

    fn barrier_with_context_async<T>(
        &self,
        mc_group: &MulticastGroupCollective,
        context: &mut T,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, Some((context as *mut T).cast()), None)
    }

    fn barrier_with_context_with_options_async<T>(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut T,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, Some((context as *mut T).cast()), Some(options))
    }

    fn broadcast_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.broadcast_impl_async(buf, desc, mc_group, Some(root_mapped_addr), options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.broadcast_impl_async(
            buf,
            desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some((context as *mut T0).cast()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.alltoall_impl_async(buf, desc, result, result_desc, mc_group, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.alltoall_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            options,
            Some((context as *mut T0).cast()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allreduce_impl_async(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allreduce_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            op,
            options,
            Some((context as *mut T0).cast()),
        )
    }

    fn allgather_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allgather_impl_async(buf, desc, result, result_desc, mc_group, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allgather_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            options,
            Some((context as *mut T0).cast()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.reduce_scatter_impl_async(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.reduce_scatter_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            op,
            options,
            Some((context as *mut T0).cast()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
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
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut T0,
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
            Some((context as *mut T0).cast()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.scatter_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.scatter_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some((context as *mut T0).cast()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn gather_async<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.gather_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn gather_with_context_async<T: AsFiType, T0>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut T0,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.gather_impl_async(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some((context as *mut T0).cast()),
        )
    }
}
