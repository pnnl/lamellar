use std::marker::PhantomData;

use super::message::extract_raw_addr_and_ctx;
use super::message::extract_raw_ctx;
use crate::cq::ReadCq;
use crate::enums;
use crate::enums::CollectiveOptions;
use crate::ep::Connected;
use crate::ep::Connectionless;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::eq::ReadEq;
use crate::fid::AsRawTypedFid;
use crate::fid::AsTypedFid;
use crate::fid::EpRawFid;
use crate::infocapsoptions::CollCap;
use crate::mcast::MultiCastGroup;
use crate::mr::MemoryRegionDesc;
use crate::trigger::TriggeredContext;
use crate::utils::check_error;
use crate::AsFiType;
use crate::Context;

pub(crate) trait CollectiveEpImpl: AsTypedFid<EpRawFid> {
    fn barrier_impl(
        &self,
        mc_group: &MultiCastGroup,
        context: Option<*mut std::ffi::c_void>,
        options: Option<CollectiveOptions>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);

        let err = if let Some(opt) = options {
            unsafe {
                libfabric_sys::inlined_fi_barrier2(
                    self.as_typed_fid_mut().as_raw_typed_fid(),
                    mc_group.raw_addr().get(),
                    opt.as_raw(),
                    ctx,
                )
            }
        } else {
            unsafe {
                libfabric_sys::inlined_fi_barrier(
                    self.as_typed_fid_mut().as_raw_typed_fid(),
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
        desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_broadcast(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe {
            libfabric_sys::inlined_fi_alltoall(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                result.as_mut_ptr().cast(),
                result_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe {
            libfabric_sys::inlined_fi_allgather(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                result.as_mut_ptr().cast(),
                result_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe {
            libfabric_sys::inlined_fi_allreduce(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                result.as_mut_ptr().cast(),
                result_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe {
            libfabric_sys::inlined_fi_reduce_scatter(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                result.as_mut_ptr().cast(),
                result_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: Option<&crate::MappedAddress>,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_reduce(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                result.as_mut_ptr().cast(),
                result_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_scatter(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                result.as_mut_ptr().cast(),
                result_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: Option<&crate::MappedAddress>,
        options: CollectiveOptions,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_gather(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                buf.len(),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                result.as_mut_ptr().cast(),
                result_desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
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
    fn barrier(&self, mc_group: &MultiCastGroup) -> Result<(), crate::error::Error>;
    fn barrier_with_options(
        &self,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    fn barrier_with_context(
        &self,
        mc_group: &MultiCastGroup,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn barrier_with_context_with_options(
        &self,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    fn barrier_triggered(
        &self,
        mc_group: &MultiCastGroup,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn barrier_triggered_with_options(
        &self,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn broadcast<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn broadcast_triggered<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn alltoall<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allreduce<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    fn allgather<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn allgather_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn reduce_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn scatter<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn scatter_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn scatter_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn gather<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn gather_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    #[allow(clippy::too_many_arguments)]
    fn gather_triggered<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
}

impl<EP: CollectiveEpImpl> CollectiveEp for EP {
    fn barrier(&self, mc_group: &MultiCastGroup) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, None, None)
    }

    fn barrier_with_options(
        &self,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, None, Some(options))
    }

    fn barrier_with_context(
        &self,
        mc_group: &MultiCastGroup,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context.inner_mut()), None)
    }

    fn barrier_triggered(
        &self,
        mc_group: &MultiCastGroup,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context.inner_mut()), None)
    }

    fn barrier_with_context_with_options(
        &self,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context.inner_mut()), Some(options))
    }

    fn barrier_triggered_with_options(
        &self,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context.inner_mut()), Some(options))
    }

    fn broadcast<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.broadcast_impl(buf, desc, mc_group, Some(root_mapped_addr), options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context<T: AsFiType>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.alltoall_impl(buf, desc, result, result_desc, mc_group, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.allreduce_impl(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.allgather_impl(buf, desc, result, result_desc, mc_group, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
        options: CollectiveOptions,
    ) -> Result<(), crate::error::Error> {
        self.reduce_scatter_impl(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context<T: AsFiType>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        op: crate::enums::ReduceOp,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::ReduceOp,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::ReduceOp,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
        root_mapped_addr: &crate::MappedAddress,
        op: crate::enums::ReduceOp,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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
        buf: &[T],
        desc: Option<&MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<&MemoryRegionDesc>,
        mc_group: &MultiCastGroup,
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

    pub fn op(mut self, op: &enums::ReduceOp) -> Self {
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
