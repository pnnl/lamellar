use crate::{ep::Address, infocapsoptions::CollCap, enums::{JoinOptions, CollectiveOptions}, comm::collective::{MulticastGroupCollectiveBase, MulticastGroupCollectiveImplBase}, cq::SingleCompletionFormat, fid::AsRawFid, eq::Event, mr::DataDescriptor, async_::{eq::{EventQueueFut, AsyncEventQueueImpl}, ep::Endpoint, cq::AsyncCompletionQueueImpl, AsyncCtx}};

pub type AsyncMulticastGroupCollective = MulticastGroupCollectiveBase<AsyncEventQueueImpl, AsyncCompletionQueueImpl>;
pub(crate) type AsyncMulticastGroupCollectiveImpl = MulticastGroupCollectiveImplBase<AsyncEventQueueImpl, AsyncCompletionQueueImpl>;

// use AsyncMulticastGroupCollective as MulticastGroupCollective;

impl<E: CollCap> Endpoint<E> {

    async fn join_impl_async(&self, addr: &Address, options: JoinOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<(Event<usize>,AsyncMulticastGroupCollective), crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let mc = self.join_with_context(addr, options, &mut async_ctx)?;
        let eq = self.inner.eq.get().expect("Endpoint not bound to an Event Queue").clone();
        let res = EventQueueFut::<{libfabric_sys::FI_JOIN_COMPLETE}>{eq, req_fid: mc.as_raw_fid(), ctx: &mut async_ctx as *mut AsyncCtx as usize}.await?;
        
        Ok((res, mc))
    }

    pub async fn join_async(&self, addr: &Address, options: JoinOptions) -> Result<(Event<usize>,AsyncMulticastGroupCollective), crate::error::Error> { // [TODO]
        self.join_impl_async(addr, options, None).await
    }

    pub async fn join_with_context_async<T>(&self, addr: &Address, options: JoinOptions, context: &mut T) -> Result<(Event<usize>,AsyncMulticastGroupCollective), crate::error::Error> {
        self.join_impl_async(addr, options, Some((context as *mut T).cast())).await
    }

    async fn join_collective_impl_async(&self, coll_mapped_addr: &crate::MappedAddress, set: &crate::async_::av::AsyncAddressVectorSet, options: JoinOptions, user_ctx : Option<*mut std::ffi::c_void>) -> Result<(Event<usize>,AsyncMulticastGroupCollective), crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        let mc = self.join_collective_with_context(coll_mapped_addr, set, options, &mut async_ctx)?;
        let eq = self.inner.eq.get().expect("Endpoint not bound to an Event Queue").clone();
        let res = EventQueueFut::<{libfabric_sys::FI_JOIN_COMPLETE}>{eq, req_fid: mc.as_raw_fid(), ctx: &mut async_ctx as *mut AsyncCtx as usize}.await?;
        
        Ok((res,mc))
    }

    pub async fn join_collective_async(&self, coll_mapped_addr: &crate::MappedAddress, set: &crate::async_::av::AsyncAddressVectorSet, options: JoinOptions) -> Result<(Event<usize>,AsyncMulticastGroupCollective), crate::error::Error> {
        self.join_collective_impl_async(coll_mapped_addr, set, options, None).await
    }

    pub async fn join_collective_with_context_async<T>(&self, coll_mapped_addr: &crate::MappedAddress, set: &crate::async_::av::AsyncAddressVectorSet, options: JoinOptions, context : &mut T) -> Result<(Event<usize>,AsyncMulticastGroupCollective), crate::error::Error> {
        self.join_collective_impl_async(coll_mapped_addr, set, options, Some((context as *mut T).cast())).await
    }
}


impl AsyncMulticastGroupCollective {

    async fn barrier_impl_async(&self, user_ctx: Option<*mut std::ffi::c_void>, options: Option<CollectiveOptions>) -> Result<SingleCompletionFormat, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.barrier_impl(Some(&mut async_ctx), options)?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    } 


    pub async fn barrier_async(&self) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.barrier_impl_async(None, None).await
    }

    pub async fn barrier_with_context_async<T>(&self, context: &mut T) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.barrier_impl_async(Some((context as *mut T).cast()), None).await
    }

    pub async fn barrier_with_options_async(&self, options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.barrier_impl_async(None, Some(options)).await
    }

    pub async fn barrier_with_context_with_options_async<T>(&self, options: CollectiveOptions, context : &mut T) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.barrier_impl_async(Some((context as *mut T).cast()), Some(options)).await
    }

    async fn broadcast_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.broadcast_impl(buf, desc, root_mapped_addr, options, Some(&mut async_ctx))?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    }

    pub async fn broadcast_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.broadcast_impl_async(buf, desc, Some(root_mapped_addr), options, None).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn broadcast_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context : &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.broadcast_impl_async(buf, desc, Some(root_mapped_addr), options, Some((context as *mut T0).cast())).await
    }

    async fn alltoall_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.alltoall_impl(buf, desc, result, result_desc, options, Some(&mut async_ctx))?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn alltoall_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.alltoall_impl_async(buf, desc, result, result_desc, options, None).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn alltoall_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, options: CollectiveOptions, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.alltoall_impl_async(buf, desc, result, result_desc, options, Some((context as *mut T0).cast())).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn allreduce_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.allreduce_impl(buf, desc, result, result_desc, op, options, Some(&mut async_ctx))?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn allreduce_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.allreduce_impl_async(buf, desc, result, result_desc, op, options, None).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn allreduce_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.allreduce_impl_async(buf, desc, result, result_desc, op, options, Some((context as *mut T0).cast())).await
    }
    
    async fn allgather_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.allgather_impl(buf, desc, result, result_desc, options, Some(&mut async_ctx))?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    }

    pub async fn allgather_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.allgather_impl_async(buf, desc, result, result_desc, options, None).await
    }
    
    #[allow(clippy::too_many_arguments)]
    pub async fn allgather_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, options: CollectiveOptions, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.allgather_impl_async(buf, desc, result, result_desc, options, Some((context as *mut T0).cast())).await
    }
    
    #[allow(clippy::too_many_arguments)]
    async fn reduce_scatter_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.reduce_scatter_impl(buf, desc, result, result_desc, op, options, Some(&mut async_ctx))?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn reduce_scatter_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.reduce_scatter_impl_async(buf, desc, result, result_desc, op, options, None).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn reduce_scatter_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.reduce_scatter_impl_async(buf, desc, result, result_desc, op, options, Some((context as *mut T0).cast())).await
    }
    
    #[allow(clippy::too_many_arguments)]
    async fn reduce_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: Option<&crate::MappedAddress>,op: crate::enums::Op,  options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.reduce_impl(buf, desc, result, result_desc, root_mapped_addr, op, options, Some(&mut async_ctx))?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn reduce_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.reduce_impl_async(buf, desc, result, result_desc, Some(root_mapped_addr), op, options, None).await
    }
    
    #[allow(clippy::too_many_arguments)]
    pub async fn reduce_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.reduce_impl_async(buf, desc, result, result_desc, Some(root_mapped_addr), op, options, Some((context as *mut T0).cast())).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn scatter_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.scatter_impl(buf, desc, result, result_desc, root_mapped_addr, options, Some(&mut async_ctx))?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn scatter_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.scatter_impl_async(buf, desc, result, result_desc, Some(root_mapped_addr), options, None).await
    }
    
    #[allow(clippy::too_many_arguments)]
    pub async fn scatter_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.scatter_impl_async(buf, desc, result, result_desc, Some(root_mapped_addr), options, Some((context as *mut T0).cast())).await
    }
    
    #[allow(clippy::too_many_arguments)]
    async fn gather_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.gather_impl(buf, desc, result, result_desc, root_mapped_addr, options, Some(&mut async_ctx))?;
        let cq = self.inner.ep.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn gather_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.gather_impl_async(buf, desc, result, result_desc, Some(root_mapped_addr), options, None).await
    }
    
    #[allow(clippy::too_many_arguments)]
    pub async fn gather_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> Result<SingleCompletionFormat, crate::error::Error> {
        self.gather_impl_async(buf, desc, result, result_desc, Some(root_mapped_addr), options, Some((context as *mut T0).cast())).await
    }
}