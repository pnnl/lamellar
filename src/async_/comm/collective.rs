use std::rc::Rc;

use crate::{ep::{Address, EndpointBase, EndpointImplBase}, enums::{JoinOptions, CollectiveOptions}, comm::collective::{MulticastGroupCollective, MulticastGroupCollectiveImpl, CollectiveEpImpl}, fid::AsRawFid, eq::Event, mr::DataDescriptor, async_::{eq::AsyncReadEq, cq::AsyncReadCq, AsyncCtx}, cq::{SingleCompletion, ReadCq}, error::Error, MappedAddress, infocapsoptions::CollCap};

impl MulticastGroupCollectiveImpl {
    #[inline]
    pub(crate) async fn join_async_impl(&self, ep: &Rc<EndpointImplBase<impl CollCap + 'static, impl AsyncReadEq + 'static, impl ReadCq + 'static>>, addr: &Address, options: JoinOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<Event<usize>, Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.join_impl(ep, addr, options, Some(&mut async_ctx))?;
        let eq = ep.eq.get().expect("Endpoint not bound to an Event Queue");
        eq.async_event_wait(libfabric_sys::FI_JOIN_COMPLETE, self.as_raw_fid(),  &mut async_ctx as *mut AsyncCtx as usize).await
    }

    #[inline]
    pub(crate) async fn join_collective_async_impl(&self, ep: &Rc<EndpointImplBase<impl CollCap + 'static, impl AsyncReadEq + 'static, impl ReadCq + 'static>>, mapped_addr: &MappedAddress, set: &crate::av::AddressVectorSet, options: JoinOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<Event<usize>, Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.join_collective_impl(ep, mapped_addr, set, options, Some(&mut async_ctx))?;
        let eq = ep.eq.get().expect("Endpoint not bound to an Event Queue");
        eq.async_event_wait(libfabric_sys::FI_JOIN_COMPLETE, self.as_raw_fid(),  &mut async_ctx as *mut AsyncCtx as usize).await
    }
}

impl MulticastGroupCollective {
    pub async fn join_async(&self, ep: &EndpointBase<impl CollCap + 'static, impl AsyncReadEq + 'static, impl ReadCq + 'static>, addr: &Address, options: JoinOptions) -> Result<Event<usize>, Error> {
        self.inner.join_async_impl(&ep.inner, addr, options, None).await
    }

    pub async fn join_async_with_context<T>(&self, ep: &EndpointBase<impl CollCap + 'static, impl AsyncReadEq + 'static, impl ReadCq + 'static>, addr: &Address, options: JoinOptions, context: &mut T) -> Result<Event<usize>, Error> {
        self.inner.join_async_impl(&ep.inner, addr, options, Some((context as *mut T) as *mut std::ffi::c_void)).await
    }

    pub async fn join_collective_async(&self, ep: &EndpointBase<impl CollCap + 'static, impl AsyncReadEq + 'static, impl ReadCq + 'static>, mapped_addr: &MappedAddress, set: &crate::av::AddressVectorSet, options: JoinOptions) -> Result<Event<usize>, Error> {
        self.inner.join_collective_async_impl(&ep.inner, mapped_addr, set, options, None).await
    }

    pub async fn join_collective_async_with_context<T>(&self, ep: &EndpointBase<impl CollCap + 'static, impl AsyncReadEq + 'static, impl ReadCq + 'static>, mapped_addr: &MappedAddress, set: &crate::av::AddressVectorSet, options: JoinOptions, context: &mut T) -> Result<Event<usize>, Error> {
        self.inner.join_collective_async_impl(&ep.inner, mapped_addr, set, options, Some((context as *mut T) as *mut std::ffi::c_void)).await
    }
}

trait AsyncCollectiveEpImpl: CollectiveEpImpl {
    async fn barrier_impl_async(&self, mc_group: &MulticastGroupCollective, user_ctx: Option<*mut std::ffi::c_void>, options: Option<CollectiveOptions>) -> Result<SingleCompletion, crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    async fn broadcast_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, 
        root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    async fn alltoall_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, 
        options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    async fn allreduce_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, 
        op: crate::enums::Op,  options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    async fn allgather_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, 
        options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;    
    #[allow(clippy::too_many_arguments)]
    async fn reduce_scatter_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, 
        op: crate::enums::Op,  options: CollectiveOptions,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    async fn reduce_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, 
        root_mapped_addr: Option<&crate::MappedAddress>,op: crate::enums::Op,  options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    async fn scatter_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, 
        root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    async fn gather_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, 
        root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> ;
} 

pub trait AsyncCollectiveEp {
    fn barrier_async(&self, mc_group: &MulticastGroupCollective) ->  impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn barrier_with_context_async<T>(&self, mc_group: &MulticastGroupCollective, context: &mut T) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn barrier_with_options_async(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn barrier_with_context_with_options_async<T>(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context : &mut T) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    fn broadcast_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn allgather_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;    
    #[allow(clippy::too_many_arguments)]
    fn reduce_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn scatter_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn scatter_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn gather_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
    #[allow(clippy::too_many_arguments)]
    fn gather_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> ;
}


// impl<E, EQ: ?Sized + AsyncReadEq,  CQ: ?Sized + AsyncReadCq> EndpointBase<E, EQ, CQ> {

impl<EP: CollCap, EQ: ?Sized + AsyncReadEq,  CQ: ?Sized + AsyncReadCq> AsyncCollectiveEpImpl for EndpointImplBase<EP, EQ, CQ> {

    #[inline]
    async fn barrier_impl_async(&self, mc_group: &MulticastGroupCollective, user_ctx: Option<*mut std::ffi::c_void>, options: Option<CollectiveOptions>) -> Result<SingleCompletion, crate::error::Error> { 
        let mut async_ctx = AsyncCtx{user_ctx};
        self.barrier_impl(mc_group, Some(&mut async_ctx), options)?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    } 

    #[inline]
    async fn broadcast_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.broadcast_impl(buf, desc, mc_group, root_mapped_addr, options, Some(&mut async_ctx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn alltoall_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.alltoall_impl(buf, desc, result, result_desc, mc_group, options, Some(&mut async_ctx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }


    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn allreduce_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.allreduce_impl(buf, desc, result, result_desc, mc_group, op, options, Some(&mut async_ctx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn allgather_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.allgather_impl(buf, desc, result, result_desc, mc_group, options, Some(&mut async_ctx))?;
        let tx_cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // while tx_cq.trywait().is_err() {tx_cq.read(0);}
        // while rx_cq.trywait().is_err() {rx_cq.read(0);}
        // while cq.read(1).is_err() {}
        // while rxcq.read(1).is_err() {}
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        tx_cq.wait_for_ctx_async(&mut async_ctx).await
    }
   
    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn reduce_scatter_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.reduce_scatter_impl(buf, desc, result, result_desc, mc_group, op, options, Some(&mut async_ctx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
        
    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn reduce_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>,op: crate::enums::Op,  options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.reduce_impl(buf, desc, result, result_desc, mc_group, root_mapped_addr, op, options, Some(&mut async_ctx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn scatter_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.scatter_impl(buf, desc, result, result_desc, mc_group, root_mapped_addr, options, Some(&mut async_ctx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
        
    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn gather_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.gather_impl(buf, desc, result, result_desc, mc_group, root_mapped_addr, options, Some(&mut async_ctx))?;
        let cq = self.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }
}

impl<E: CollCap, EQ: ?Sized + AsyncReadEq,  CQ: ?Sized + AsyncReadCq> AsyncCollectiveEpImpl for EndpointBase<E, EQ, CQ> {

    #[inline]
    async fn barrier_impl_async(&self, mc_group: &MulticastGroupCollective, user_ctx: Option<*mut std::ffi::c_void>, options: Option<CollectiveOptions>) -> Result<SingleCompletion, crate::error::Error> { 
        self.inner.barrier_impl_async(mc_group, user_ctx, options).await
    } 

    #[inline]
    async fn broadcast_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx : Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.broadcast_impl_async(buf, desc, mc_group, root_mapped_addr, options, user_ctx).await
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn alltoall_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.alltoall_impl_async(buf, desc, result, result_desc, mc_group, options, user_ctx).await
    }


    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn allreduce_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.allreduce_impl_async(buf, desc, result, result_desc, mc_group, op, options, user_ctx).await
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn allgather_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.allgather_impl_async(buf, desc, result, result_desc, mc_group, options, user_ctx).await
    }
   
    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn reduce_scatter_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions,  user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.reduce_scatter_impl_async(buf, desc, result, result_desc, mc_group, op, options, user_ctx).await
    }
        
    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn reduce_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>,op: crate::enums::Op,  options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        let mut async_ctx = AsyncCtx{user_ctx};
        self.reduce_impl(buf, desc, result, result_desc, mc_group, root_mapped_addr, op, options, Some(&mut async_ctx))?;
        let cq = self.inner.tx_cq.get().expect("Endpoint not bound to a Completion Queue").clone(); 
        // crate::async_::cq::AsyncTransferCq::new(cq, &mut async_ctx as *mut AsyncCtx as usize).await
        cq.wait_for_ctx_async(&mut async_ctx).await
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn scatter_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.scatter_impl_async(buf, desc, result, result_desc, mc_group, root_mapped_addr, options, user_ctx).await
    }
        
    #[inline]
    #[allow(clippy::too_many_arguments)]
    async fn gather_impl_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, user_ctx: Option<*mut std::ffi::c_void>) -> Result<SingleCompletion, crate::error::Error> {
        self.inner.gather_impl_async(buf, desc, result, result_desc, mc_group, root_mapped_addr, options, user_ctx).await
    }
}


impl<EP: AsyncCollectiveEpImpl> AsyncCollectiveEp for EP {
    #[inline]
    fn barrier_async(&self, mc_group: &MulticastGroupCollective) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, None, None)
    }

    fn barrier_with_context_async<T>(&self, mc_group: &MulticastGroupCollective, context: &mut T) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, Some((context as *mut T).cast()), None)
    }

    fn barrier_with_options_async(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, None, Some(options))
    }

    fn barrier_with_context_with_options_async<T>(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context : &mut T) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.barrier_impl_async(mc_group, Some((context as *mut T).cast()), Some(options))
    }

    fn broadcast_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.broadcast_impl_async(buf, desc, mc_group, Some(root_mapped_addr), options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context : &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.broadcast_impl_async(buf, desc, mc_group, Some(root_mapped_addr), options, Some((context as *mut T0).cast()))
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.alltoall_impl_async(buf, desc, result, result_desc, mc_group, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.alltoall_impl_async(buf, desc, result, result_desc, mc_group, options, Some((context as *mut T0).cast()))
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allreduce_impl_async(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allreduce_impl_async(buf, desc, result, result_desc, mc_group, op, options, Some((context as *mut T0).cast()))
    }

    fn allgather_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allgather_impl_async(buf, desc, result, result_desc, mc_group, options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.allgather_impl_async(buf, desc, result, result_desc, mc_group, options, Some((context as *mut T0).cast()))
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.reduce_scatter_impl_async(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.reduce_scatter_impl_async(buf, desc, result, result_desc, mc_group, op, options, Some((context as *mut T0).cast()))
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.reduce_impl_async(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), op, options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn reduce_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.reduce_impl_async(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), op, options, Some((context as *mut T0).cast()))
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.scatter_impl_async(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn scatter_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.scatter_impl_async(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), options, Some((context as *mut T0).cast()))
    }

    #[allow(clippy::too_many_arguments)]
    fn gather_async<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.gather_impl_async(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn gather_with_context_async<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.gather_impl_async(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), options, Some((context as *mut T0).cast()))
    }
}