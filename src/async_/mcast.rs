use crate::{async_::{comm::collective::AsyncCollectiveEp, cq::AsyncReadCq, eq::AsyncReadEq}, comm::collective::CollectiveEp, enums::JoinOptions, ep::{EndpointBase, EpState}, eq::Event, error::Error, fid::{AsRawFid, AsTypedFid, EpRawFid, Fid}, mcast::{MultiCastGroup, MulticastGroupImpl, PendingMulticastGroupCollective}, Context, MyRc};

impl MulticastGroupImpl {
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
        let cq = ep.retrieve_tx_cq().get();
        eq.async_event_wait(
            libfabric_sys::FI_JOIN_COMPLETE,
            Fid(self.as_typed_fid().as_raw_fid() as usize),
            Some(ctx),
            Some(Box::new(cq)),
        )
        .await
    }
}

impl PendingMulticastGroupCollective {
    // pub async fn join_async(&self, ep: &EndpointBase<impl AsyncCollectiveEp + 'static + AsRawTypedFid<Output = EpRawFid>>, addr: &Address, options: JoinOptions) -> Result<Event, Error> {
    //     self.inner.join_async_impl(&ep.inner, addr, options, None).await
    // }

    pub async fn join_collective_async<
        E: CollectiveEp + AsTypedFid<EpRawFid> + 'static + AsyncCollectiveEp,
        STATE: EpState,
    >(
        self,
        ep: &EndpointBase<E, STATE>,
        options: JoinOptions,
        ctx: &mut Context,
    ) -> Result<(Event, MultiCastGroup), Error> {
        let event = self.inner
            .join_collective_async_impl(&ep.inner, options, ctx)
            .await?;
        Ok(
            (event, MultiCastGroup {
                inner: self.inner,
            })
        )
    }
}