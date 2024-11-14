use std::marker::PhantomData;

use crate::av::AddressVectorSet;
use crate::av::AddressVectorSetImpl;
use crate::cq::ReadCq;
use crate::enums;
use crate::enums::CollectiveOptions;
use crate::enums::JoinOptions;
use crate::ep::Connected;
use crate::ep::Connectionless;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::ep::EpState;
use crate::eq::ReadEq;
use crate::error::Error;
use crate::fid::AsTypedFid;
use crate::fid::BorrowedTypedFid;
use crate::trigger::TriggeredContext;
use crate::AsFiType;
use crate::Context;
use crate::MyOnceCell;
use crate::MyRc;
use crate::MyRefCell;
use crate::RawMappedAddress;
use crate::SyncSend;

use super::message::extract_raw_addr_and_ctx;
use super::message::extract_raw_ctx;
use crate::fid::EpRawFid;
use crate::fid::McRawFid;
use crate::fid::OwnedMcFid;
use crate::fid::AsRawTypedFid;
use crate::infocapsoptions::CollCap;
use crate::mr::DataDescriptor;
use crate::utils::check_error;

pub struct MulticastGroupCollective {
    pub(crate) inner: MyRc<MulticastGroupCollectiveImpl>,
}

// impl std::fmt::Debug for MulticastGroupCollective {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "")
//     }
// }

pub struct MulticastGroupCollectiveImpl {
    c_mc: MyOnceCell<OwnedMcFid>,
    eps: MyRefCell<Vec<MyRc<dyn CollectiveValidEp>>>,
    addr: MyOnceCell<RawMappedAddress>,
    avset: MyRc<AddressVectorSetImpl>,
}

pub(crate) trait CollectiveValidEp: SyncSend {}
impl<EP: CollectiveEp + SyncSend> CollectiveValidEp for EP {}

impl MulticastGroupCollectiveImpl {
    pub(crate) fn new(avset: &MyRc<AddressVectorSetImpl>) -> Self {
        Self {
            c_mc: MyOnceCell::new(),
            addr: MyOnceCell::new(),
            eps: MyRefCell::new(Vec::new()),
            avset: avset.clone(),
        }
    }

    // pub(crate) fn join_impl<T, EP: CollectiveValidEp + AsRawTypedFid<Output = EpRawFid> + 'static>(&self, ep: &MyRc<EP>, addr: &Address, options: JoinOptions, context: Option<&mut T>) -> Result<(), Error> {
    //     let mut c_mc: McRawFid = std::ptr::null_mut();
    //     let err =
    //         if let Some(ctx) = context {
    //             unsafe { libfabric_sys::inlined_fi_join(ep.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), options.as_raw(), &mut c_mc, (ctx as *mut T).cast()) }
    //         }
    //         else {
    //             unsafe { libfabric_sys::inlined_fi_join(ep.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), options.as_raw(), &mut c_mc, std::ptr::null_mut()) }
    //         };

    //     if err != 0 {
    //         Err(Error::from_err_code((-err).try_into().unwrap()))
    //     }
    //     else {
    //         if let Err(old_mc)  = self.c_mc.set(OwnedMcFid::from(c_mc)) {
    //             assert!(old_mc.as_raw_typed_fid() == c_mc);
    //         }
    //         else {
    //             self.addr.set(unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc)}).unwrap()
    //         }
    //         self.eps.write().push(ep.clone());
    //         Ok(())
    //     }
    // }

    pub(crate) fn join_collective_impl<
        EP: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
    >(
        &self,
        ep: &MyRc<EP>,
        options: JoinOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), Error> {
        let mut c_mc: McRawFid = std::ptr::null_mut();
        let ctx = extract_raw_ctx(context);
        let addr = self.addr.get();
        let raw_addr = if let Some(addr) = addr {
            addr.clone()
        } else {
            self.avset.get_addr()?
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_join_collective(
                ep.as_typed_fid().as_raw_typed_fid(),
                raw_addr.get(),
                self.avset.as_typed_fid().as_raw_typed_fid(),
                options.as_raw(),
                &mut c_mc,
                ctx,
            )
        };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        } else {
            if let Err(old_mc) = self.c_mc.set(OwnedMcFid::from(c_mc)) {
                assert!(old_mc.as_typed_fid().as_raw_typed_fid() == c_mc);
            } else {
                self.addr
                    .set(RawMappedAddress::from_raw(
                        self.avset._av_rc.type_(),
                        unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc) },
                    ))
                    .unwrap()
            }
            #[cfg(feature = "thread-safe")]
            self.eps.write().push(ep.clone());
            #[cfg(not(feature = "thread-safe"))]
            self.eps.borrow_mut().push(ep.clone());
            Ok(())
        }
    }
}

impl MulticastGroupCollective {
    #[allow(dead_code)]
    pub(crate) fn from_impl(mc_impl: &MyRc<MulticastGroupCollectiveImpl>) -> Self {
        Self {
            inner: mc_impl.clone(),
        }
    }

    pub fn new(avset: &AddressVectorSet) -> Self {
        Self {
            inner: MyRc::new(MulticastGroupCollectiveImpl::new(&avset.inner)),
        }
    }

    pub(crate) fn raw_addr(&self) -> &RawMappedAddress {
        self.inner.addr.get().unwrap()
    }

    // pub fn join<E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static>(&self, ep: &EndpointBase<E>, addr: &Address, options: JoinOptions) -> Result<(), Error> {
    //     self.inner.join_impl::<(), E>(&ep.inner, addr, options, None)
    // }

    // pub fn join_with_context<E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static,T>(&self, ep: &EndpointBase<E>, addr: &Address, options: JoinOptions, context: &mut Context) -> Result<(), Error> {
    //     self.inner.join_impl(&ep.inner, addr, options, Some(context.inner_mut()))
    // }

    pub fn join_collective_with_context<
        E: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
        STATE: EpState,
    >(
        &self,
        ep: &EndpointBase<E, STATE>,
        options: JoinOptions,
        context: &mut Context,
    ) -> Result<(), Error> {
        self.inner
            .join_collective_impl(&ep.inner, options, Some(context.inner_mut()))
    }
    pub fn join_collective<
        E: CollectiveEp + AsTypedFid<EpRawFid> + 'static + SyncSend,
        STATE: EpState,
        const INIT: bool,
    >(
        &self,
        ep: &EndpointBase<E, STATE>,
        options: JoinOptions,
    ) -> Result<(), Error> {
        self.inner.join_collective_impl(&ep.inner, options, None)
    }
}

pub(crate) trait CollectiveEpImpl: AsTypedFid<EpRawFid> {
    fn barrier_impl(
        &self,
        mc_group: &MulticastGroupCollective,
        context: Option<*mut std::ffi::c_void>,
        options: Option<CollectiveOptions>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);

        let err = if let Some(opt) = options {
            unsafe {
                libfabric_sys::inlined_fi_barrier2(
                    self.as_typed_fid().as_raw_typed_fid(),
                    mc_group.raw_addr().get(),
                    opt.as_raw(),
                    ctx,
                )
            }
        } else {
            unsafe {
                libfabric_sys::inlined_fi_barrier(
                    self.as_typed_fid().as_raw_typed_fid(),
                    mc_group.raw_addr().get(),
                    ctx,
                )
            }
        };

        check_error(err)
    }

    fn broadcast_impl<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_broadcast(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.get_desc(),
                mc_group.raw_addr().get(),
                root_raw_addr,
                T::as_fi_datatype(),
                options.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_impl<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe {
            libfabric_sys::inlined_fi_alltoall(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.get_desc(),
                result as *mut T as *mut std::ffi::c_void,
                result_desc.get_desc(),
                mc_group.raw_addr().get(),
                T::as_fi_datatype(),
                options.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn allgather_impl<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe {
            libfabric_sys::inlined_fi_allgather(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.get_desc(),
                result.as_mut_ptr().cast(),
                result_desc.get_desc(),
                mc_group.raw_addr().get(),
                T::as_fi_datatype(),
                options.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_impl<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe {
            libfabric_sys::inlined_fi_allreduce(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.get_desc(),
                result as *mut T as *mut std::ffi::c_void,
                result_desc.get_desc(),
                mc_group.raw_addr().get(),
                T::as_fi_datatype(),
                op.as_raw(),
                options.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_impl<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe {
            libfabric_sys::inlined_fi_reduce_scatter(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.get_desc(),
                result as *mut T as *mut std::ffi::c_void,
                result_desc.get_desc(),
                mc_group.raw_addr().get(),
                T::as_fi_datatype(),
                op.as_raw(),
                options.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_impl<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_reduce(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.get_desc(),
                result as *mut T as *mut std::ffi::c_void,
                result_desc.get_desc(),
                mc_group.raw_addr().get(),
                root_raw_addr,
                T::as_fi_datatype(),
                op.as_raw(),
                options.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter_impl<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_scatter(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.get_desc(),
                result as *mut T as *mut std::ffi::c_void,
                result_desc.get_desc(),
                mc_group.raw_addr().get(),
                root_raw_addr,
                T::as_fi_datatype(),
                options.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn gather_impl<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_gather(
                self.as_typed_fid().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.get_desc(),
                result as *mut T as *mut std::ffi::c_void,
                result_desc.get_desc(),
                mc_group.raw_addr().get(),
                root_raw_addr,
                T::as_fi_datatype(),
                options.as_raw(),
                ctx,
            )
        };
        check_error(err)
    }
}

pub trait CollectiveEp {
    fn barrier(&self, mc_group: &MulticastGroupCollective) -> Result<(), crate::error::Error>;
    fn barrier_with_options(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    fn barrier_with_context(
        &self,
        mc_group: &MulticastGroupCollective,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn barrier_with_context_with_options(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn barrier_triggered(
        &self,
        mc_group: &MulticastGroupCollective,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn barrier_triggered_with_options(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn broadcast<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn broadcast_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn alltoall<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allreduce<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn allgather<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allgather_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn scatter<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn scatter_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn scatter_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn gather<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn gather_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn gather_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
}

impl<EP: CollectiveEpImpl> CollectiveEp for EP {
    fn barrier(&self, mc_group: &MulticastGroupCollective) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, None, None)
    }

    fn barrier_with_options(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, None, Some(options))
    }

    fn barrier_with_context(
        &self,
        mc_group: &MulticastGroupCollective,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context.inner_mut()), None)
    }

    fn barrier_triggered(
        &self,
        mc_group: &MulticastGroupCollective,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context.inner_mut()), None)
    }

    fn barrier_with_context_with_options(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context.inner_mut()), Some(options))
    }

    fn barrier_triggered_with_options(
        &self,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context.inner_mut()), Some(options))
    }

    fn broadcast<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.broadcast_impl(buf, desc, mc_group, Some(root_mapped_addr), options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.broadcast_impl(
            buf,
            desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn broadcast_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.broadcast_impl(
            buf,
            desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.alltoall_impl(buf, desc, result, result_desc, mc_group, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.alltoall_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.alltoall_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.allreduce_impl(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.allreduce_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            op,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.allreduce_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            op,
            options,
            Some(context.inner_mut()),
        )
    }

    fn allgather<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.allgather_impl(buf, desc, result, result_desc, mc_group, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.allgather_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn allgather_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut [T],
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.allgather_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.reduce_scatter_impl(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.reduce_scatter_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            op,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.reduce_scatter_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            op,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.reduce_impl(
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
    fn reduce_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.reduce_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            op,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::CollAtomicOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.reduce_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            op,
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.scatter_impl(
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
    fn scatter_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.scatter_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.scatter_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn gather<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.gather_impl(
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
    fn gather_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.gather_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some(context.inner_mut()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn gather_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: &mut impl DataDescriptor,
        result: &mut T,
        result_desc: &mut impl DataDescriptor,
        mc_group: &MulticastGroupCollective,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.gather_impl(
            buf,
            desc,
            result,
            result_desc,
            mc_group,
            Some(root_mapped_addr),
            options,
            Some(context.inner_mut()),
        )
    }
}

impl<EP: CollCap, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> CollectiveEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<E: CollectiveEpImpl> CollectiveEpImpl for EndpointBase<E, Connected> {}
impl<E: CollectiveEpImpl> CollectiveEpImpl for EndpointBase<E, Connectionless> {}

// impl AsFid for MulticastGroupCollective {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.inner.as_fid()
//     }
// }

// impl AsFid for MulticastGroupCollectiveImpl {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.c_mc.get().unwrap().as_fid()
//     }
// }

// impl AsRawFid for MulticastGroupCollective {
//     fn as_raw_fid(&self) -> RawFid {
//         self.inner.as_raw_fid()
//     }
// }

// impl AsRawFid for MulticastGroupCollectiveImpl {
//     fn as_raw_fid(&self) -> RawFid {
//         self.c_mc.get().unwrap().as_raw_fid()
//     }
// }
impl AsTypedFid<McRawFid> for MulticastGroupCollective {
    fn as_typed_fid(&self) -> BorrowedTypedFid<McRawFid> {
        self.inner.as_typed_fid()
    }
}

impl AsTypedFid<McRawFid> for MulticastGroupCollectiveImpl {
    fn as_typed_fid(&self) -> BorrowedTypedFid<McRawFid> {
        self.c_mc.get().unwrap().as_typed_fid()
    }
}

// impl AsRawTypedFid for MulticastGroupCollective {
//     type Output = McRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.inner.as_raw_typed_fid()
//     }
// }

// impl AsRawTypedFid for MulticastGroupCollectiveImpl {
//     type Output = McRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.c_mc.get().unwrap().as_raw_typed_fid()
//     }
// }

pub struct CollectiveAttr<T> {
    pub(crate) c_attr: libfabric_sys::fi_collective_attr,
    phantom: PhantomData<T>,
}

impl<T: AsFiType> CollectiveAttr<T> {
    //[TODO] CHECK INITIAL VALUES
    pub fn new() -> Self {
        Self {
            c_attr: libfabric_sys::fi_collective_attr {
                op: 0,
                datatype: T::as_fi_datatype(),
                datatype_attr: libfabric_sys::fi_atomic_attr { count: 0, size: 0 },
                max_members: 0,
                mode: 0, // [TODO] What are the valid options?
            },
            phantom: PhantomData,
        }
    }

    pub fn op(mut self, op: &enums::CollAtomicOp) -> Self {
        self.c_attr.op = op.as_raw();
        self
    }

    pub fn max_members(mut self, members: usize) -> Self {
        self.c_attr.max_members = members;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_collective_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_collective_attr {
        &mut self.c_attr
    }
}

impl<T: AsFiType> Default for CollectiveAttr<T> {
    fn default() -> Self {
        Self::new()
    }
}
