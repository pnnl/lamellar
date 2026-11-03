//! Collectives on a [`SharedMemoryRegion`], choosing between the backend's native implementation and the put/get
//! ("Manual") algorithms of `array/collective/algorithm.rs`.
//!
//! A backend reports what it can do natively through `CommInfo::collective_avail`. When it cannot do an operation for
//! the element type (`rofi_c` has no collectives at all), the operation runs as the same put/get algorithms the array
//! collectives fall back to. They use the region's sync allocation (`coll_sync_alloc`) as scratch and flag space and the
//! ticket gate (`coll_ticket` / `coll_now_serving`) to keep pipelined collectives in order, so, as for arrays, every PE
//! must issue collectives on a region in the same order.
//!
//! Memory without a sync allocation (external memory, local regions) cannot run the algorithms, so it keeps calling
//! the native implementation and gets its panic message if the backend has none.

use std::{future::Future, pin::Pin, sync::atomic::Ordering, sync::Arc};

use crate::{
    array::collective::algorithm::{
        do_all_gather, do_all_gather_in_buffer, do_all_reduce, do_all_reduce_bitwise,
        do_all_reduce_bitwise_in_buffer, do_all_reduce_bitwise_in_place,
        do_all_reduce_comparison, do_all_reduce_comparison_in_buffer,
        do_all_reduce_comparison_in_place, do_all_reduce_in_buffer, do_all_reduce_in_place,
        do_all_to_all,
        do_all_to_all_in_buffer, do_broadcast, do_broadcast_in_buffer, do_gather,
        do_gather_in_buffer, do_reduce, do_reduce_bitwise, do_reduce_bitwise_in_buffer,
        do_reduce_comparison, do_reduce_comparison_in_buffer, do_reduce_in_buffer,
        do_reduce_scatter, do_reduce_scatter_bitwise, do_reduce_scatter_bitwise_in_buffer,
        do_reduce_scatter_comparison, do_reduce_scatter_comparison_in_buffer,
        do_reduce_scatter_in_buffer, do_scatter, do_scatter_in_buffer, ManualCollectiveSource,
        ManualCollectiveUpdate, ManualCollectiveUpdateBitwise, ManualCollectiveUpdateComparison,
    },
    lamellae::{
        collective::{
            local_reduce, BroadcastInput, CollectiveAllGatherIntoBufferOpFuture,
            CollectiveAllGatherIntoBufferOpHandle, CollectiveAllGatherOpFuture,
            CollectiveAllGatherOpHandle, CollectiveAllReduceIntoBufferOpFuture,
            CollectiveAllReduceInPlaceOpFuture, CollectiveAllReduceInPlaceOpHandle,
            CollectiveAllReduceIntoBufferOpHandle, CollectiveAllReduceOpFuture,
            CollectiveAllReduceOpHandle, CollectiveAllToAllIntoBufferOpFuture,
            CollectiveAllToAllIntoBufferOpHandle, CollectiveAllToAllOpFuture,
            CollectiveAllToAllOpHandle, CollectiveBroadcastIntoBufferOpFuture,
            CollectiveBroadcastIntoBufferOpHandle, CollectiveBroadcastOpFuture,
            CollectiveBroadcastOpHandle, CollectiveGatherIntoBufferOpFuture,
            CollectiveGatherIntoBufferOpHandle, CollectiveGatherOpFuture, CollectiveGatherOpHandle,
            CollectiveReduceIntoBufferOpFuture, CollectiveReduceIntoBufferOpHandle,
            CollectiveReduceOpFuture, CollectiveReduceOpHandle,
            CollectiveReduceScatterIntoBufferOpFuture, CollectiveReduceScatterIntoBufferOpHandle,
            CollectiveReduceScatterOpFuture, CollectiveReduceScatterOpHandle,
            CollectiveScatterIntoBufferOpFuture, CollectiveScatterIntoBufferOpHandle,
            CollectiveScatterOpFuture, CollectiveScatterOpHandle, ManualCollectiveFuture,
            ReduceOp, RootOrLamellarBuffer, RootSrcOrLamellarBuffer, ScatterInput,
        },
        CollectiveOpKind, CommAlloc, CommInfo, Remote,
    },
    memregion::{CollTicketGate, MemoryRegion, SharedMemoryRegion},
    AsLamellarBuffer, LamellarBuffer,
};

// A region is a valid source for the Manual algorithms: they only need the position in the team, a barrier, a remote
// read and the local window, all of which the region has. The region handed to them is the whole allocation (see
// `SharedMemoryRegion::collective_ops`), so the indices they use are the absolute ones the native path also takes.
impl<T: Remote> ManualCollectiveSource<T> for SharedMemoryRegion<T> {
    // These use the runtime team directly (`rt_team`) and not `self.mr.team()`: that builds a user-facing team handle,
    // and dropping the handle of a sub-team tears the team down (a barrier and its removal from the parent), so
    // calling it from inside a collective hangs.
    fn my_pe(&self) -> usize {
        self.mr
            .inner()
            .rt_team()
            .team_pe_id()
            .expect("the PE is in the region's team")
    }
    fn num_pes(&self) -> usize {
        self.mr.inner().rt_team().num_pes()
    }
    fn world_pe(&self, team_pe: usize) -> usize {
        self.mr
            .inner()
            .rt_team()
            .arch
            .world_pe(team_pe)
            .expect("the PE is in the region's team")
    }
    async fn async_barrier(&self) {
        self.mr.inner().rt_team().async_barrier().await
    }
    fn copy_local_data(&self, index: usize, count: usize, buffer: &mut [T]) {
        let window = unsafe { self.as_slice() };
        buffer.copy_from_slice(&window[index..index + count]);
    }
    fn store_to_local_data(&self, index: usize, count: usize, data: &[T]) {
        let window = unsafe { self.as_mut_slice() };
        window[index..index + count].copy_from_slice(data);
    }
    async fn batch_load(&self, pe: usize, _num_pes: usize, start: usize, count: usize) -> Vec<T> {
        // `pe` is a rank in the region's team and the region's get addresses a world PE
        unsafe { self.get_buffer(self.world_pe(pe), start, count).await }
    }
}

// The reduction itself is the one the native shared-memory backends use, which handles every integer and float type.
impl<T: Remote> ManualCollectiveUpdate<T> for SharedMemoryRegion<T> {
    fn update_local_data(&self, index: usize, count: usize, data: &[T], op: ReduceOp) {
        let window = unsafe { self.as_mut_slice() };
        local_reduce(&op, &mut window[index..index + count], data);
    }
}
impl<T: Remote> ManualCollectiveUpdateBitwise<T> for SharedMemoryRegion<T> {
    fn update_local_data_bitwise(&self, index: usize, count: usize, data: &[T], op: ReduceOp) {
        self.update_local_data(index, count, data, op)
    }
}
impl<T: Remote> ManualCollectiveUpdateComparison<T> for SharedMemoryRegion<T> {
    fn update_local_data_comparison(&self, index: usize, count: usize, data: &[T], op: ReduceOp) {
        self.update_local_data(index, count, data, op)
    }
}

// The algorithms are split by the kind of reduction, as the array ones are.
enum Family {
    Arithmetic,
    Bitwise,
    Comparison,
}

fn family(op: &ReduceOp) -> Family {
    match op {
        ReduceOp::Sum | ReduceOp::Prod => Family::Arithmetic,
        ReduceOp::BitOr | ReduceOp::BitXor | ReduceOp::BitAnd => Family::Bitwise,
        ReduceOp::Min | ReduceOp::Max => Family::Comparison,
    }
}

fn boxed<R, F: Future<Output = R> + Send + 'static>(
    future: F,
) -> Pin<Box<dyn Future<Output = R> + Send>> {
    Box::pin(future)
}

// What a Manual collective needs besides its arguments; taking it draws this call's ticket.
struct ManualCtx {
    sync_alloc: Arc<CommAlloc>,
    ticket: usize,
    now_serving: Arc<CollTicketGate>,
}

// Calls a `do_*` algorithm on this region with the common leading arguments.
macro_rules! run {
    ($s:ident, $c:ident, $algorithm:ident, ($($arg:expr),*)) => {
        $algorithm(
            $s.region.clone(),
            $s.base.scheduler.clone(),
            $c.sync_alloc,
            $c.ticket,
            $c.now_serving,
            $($arg),*
        )
    };
}

// Picks the algorithm for the reduction family of `$op`.
macro_rules! run_family {
    ($s:ident, $c:ident, $op:expr, ($arith:ident, $bitwise:ident, $comparison:ident), ($($arg:expr),*)) => {
        match family(&$op) {
            Family::Arithmetic => boxed(run!($s, $c, $arith, ($($arg),*))),
            Family::Bitwise => boxed(run!($s, $c, $bitwise, ($($arg),*))),
            Family::Comparison => boxed(run!($s, $c, $comparison, ($($arg),*))),
        }
    };
}

// Wraps a Manual future in the handle the caller expects.
macro_rules! handle {
    ($s:ident, $handle:ident, $future:ident, $fut:expr) => {
        $handle {
            future: $future::Manual($s.manual($fut)),
        }
    };
}

/// The collectives of one shared memory region. Each method has the signature of the matching [`MemoryRegion`]
/// collective and calls it when the backend supports the operation natively.
pub(crate) struct RegionCollective<T: Remote> {
    // the region in the element type of the collective, for the native calls
    base: MemoryRegion<T>,
    // the whole allocation, with no sub-region offset
    region: SharedMemoryRegion<T>,
}

impl<T: Remote> RegionCollective<T> {
    pub(crate) fn new(base: MemoryRegion<T>, region: SharedMemoryRegion<T>) -> Self {
        Self { base, region }
    }

    // `Some` when this operation should run as the put/get algorithms: the backend has no native implementation of
    // it for `T`, and the region has the sync allocation they need.
    fn manual_ctx(&self, kind: CollectiveOpKind) -> Option<ManualCtx> {
        if self.base.rdma.comm().collective_avail::<T>(kind) {
            return None;
        }
        let sync_alloc = self.base.get_collective_sync_alloc()?;
        let (ticket_ctr, now_serving) = self.base.get_collective_ticket_state();
        Some(ManualCtx {
            sync_alloc,
            ticket: ticket_ctr.fetch_add(1, Ordering::SeqCst),
            now_serving,
        })
    }

    fn manual<R: Send + 'static>(
        &self,
        future: Pin<Box<dyn Future<Output = R> + Send>>,
    ) -> ManualCollectiveFuture<R> {
        ManualCollectiveFuture::new(future, &self.base.scheduler, self.base.counters.clone())
    }

    pub(crate) fn reduce_all(
        &self,
        index: usize,
        len: usize,
        op: ReduceOp,
    ) -> CollectiveAllReduceOpHandle<T> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::AllReduce(op.clone())) else {
            return self.base.reduce_all(index, len, op);
        };
        let fut = run_family!(
            self,
            c,
            op,
            (do_all_reduce, do_all_reduce_bitwise, do_all_reduce_comparison),
            (index, len, op.clone())
        );
        handle!(self, CollectiveAllReduceOpHandle, CollectiveAllReduceOpFuture, fut)
    }

    pub(crate) fn reduce_all_into_buffer<B: AsLamellarBuffer<T>>(
        &self,
        index: usize,
        len: usize,
        op: ReduceOp,
        buffer: LamellarBuffer<T, B>,
    ) -> CollectiveAllReduceIntoBufferOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::AllReduce(op.clone())) else {
            return self.base.reduce_all_into_buffer(index, len, op, buffer);
        };
        let fut = run_family!(
            self,
            c,
            op,
            (
                do_all_reduce_in_buffer,
                do_all_reduce_bitwise_in_buffer,
                do_all_reduce_comparison_in_buffer
            ),
            (index, len, op.clone(), buffer)
        );
        handle!(
            self,
            CollectiveAllReduceIntoBufferOpHandle,
            CollectiveAllReduceIntoBufferOpFuture,
            fut
        )
    }

    // The caller's buffer is not memory peers can read, so the algorithms stage it in the first `len` elements of the
    // region for the duration of the call and put the region's contents back afterwards (as the native ones do).
    pub(crate) fn reduce_all_in_place<B: AsLamellarBuffer<T>>(
        &self,
        src_and_dst: LamellarBuffer<T, B>,
        op: ReduceOp,
    ) -> CollectiveAllReduceInPlaceOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::AllReduce(op.clone())) else {
            return self.base.reduce_all_in_place(src_and_dst, op);
        };
        let fut = run_family!(
            self,
            c,
            op,
            (
                do_all_reduce_in_place,
                do_all_reduce_bitwise_in_place,
                do_all_reduce_comparison_in_place
            ),
            (op.clone(), src_and_dst)
        );
        handle!(
            self,
            CollectiveAllReduceInPlaceOpHandle,
            CollectiveAllReduceInPlaceOpFuture,
            fut
        )
    }

    pub(crate) fn reduce(
        &self,
        op: ReduceOp,
        index: usize,
        len: usize,
        root_pe: usize,
    ) -> CollectiveReduceOpHandle<T> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::Reduce(op.clone())) else {
            return self.base.reduce(op, index, len, root_pe);
        };
        let fut = run_family!(
            self,
            c,
            op,
            (do_reduce, do_reduce_bitwise, do_reduce_comparison),
            (index, len, root_pe, op.clone())
        );
        handle!(self, CollectiveReduceOpHandle, CollectiveReduceOpFuture, fut)
    }

    pub(crate) fn reduce_into_buffer<B: AsLamellarBuffer<T>>(
        &self,
        op: ReduceOp,
        index: usize,
        len: usize,
        root_or_buffer: RootOrLamellarBuffer<T, B>,
    ) -> CollectiveReduceIntoBufferOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::Reduce(op.clone())) else {
            return self.base.reduce_into_buffer(op, index, len, root_or_buffer);
        };
        let fut = run_family!(
            self,
            c,
            op,
            (
                do_reduce_in_buffer,
                do_reduce_bitwise_in_buffer,
                do_reduce_comparison_in_buffer
            ),
            (index, len, op.clone(), root_or_buffer)
        );
        handle!(
            self,
            CollectiveReduceIntoBufferOpHandle,
            CollectiveReduceIntoBufferOpFuture,
            fut
        )
    }

    pub(crate) fn gather_all(&self, index: usize, len: usize) -> CollectiveAllGatherOpHandle<T> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::AllGather) else {
            return self.base.gather_all(index, len);
        };
        let fut = boxed(run!(self, c, do_all_gather, (index, len)));
        handle!(self, CollectiveAllGatherOpHandle, CollectiveAllGatherOpFuture, fut)
    }

    pub(crate) fn gather_all_into_buffer<B: AsLamellarBuffer<T>>(
        &self,
        index: usize,
        len: usize,
        buffer: LamellarBuffer<T, B>,
    ) -> CollectiveAllGatherIntoBufferOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::AllGather) else {
            return self.base.gather_all_into_buffer(index, len, buffer);
        };
        let fut = boxed(run!(self, c, do_all_gather_in_buffer, (index, len, buffer)));
        handle!(
            self,
            CollectiveAllGatherIntoBufferOpHandle,
            CollectiveAllGatherIntoBufferOpFuture,
            fut
        )
    }

    pub(crate) fn gather(
        &self,
        index: usize,
        len: usize,
        root_pe: usize,
    ) -> CollectiveGatherOpHandle<T> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::Gather) else {
            return self.base.gather(index, len, root_pe);
        };
        let fut = boxed(run!(self, c, do_gather, (index, len, root_pe)));
        handle!(self, CollectiveGatherOpHandle, CollectiveGatherOpFuture, fut)
    }

    pub(crate) fn gather_into_buffer<B: AsLamellarBuffer<T>>(
        &self,
        index: usize,
        len: usize,
        root_or_buffer: RootOrLamellarBuffer<T, B>,
    ) -> CollectiveGatherIntoBufferOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::Gather) else {
            return self.base.gather_into_buffer(index, len, root_or_buffer);
        };
        let fut = boxed(run!(self, c, do_gather_in_buffer, (index, len, root_or_buffer)));
        handle!(
            self,
            CollectiveGatherIntoBufferOpHandle,
            CollectiveGatherIntoBufferOpFuture,
            fut
        )
    }

    pub(crate) fn alltoall(&self, index: usize, len: usize) -> CollectiveAllToAllOpHandle<T> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::AllToAll) else {
            return self.base.alltoall(index, len);
        };
        let fut = boxed(run!(self, c, do_all_to_all, (index, len)));
        handle!(self, CollectiveAllToAllOpHandle, CollectiveAllToAllOpFuture, fut)
    }

    pub(crate) fn alltoall_into_buffer<B: AsLamellarBuffer<T>>(
        &self,
        index: usize,
        len: usize,
        buffer: LamellarBuffer<T, B>,
    ) -> CollectiveAllToAllIntoBufferOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::AllToAll) else {
            return self.base.alltoall_into_buffer(index, len, buffer);
        };
        let fut = boxed(run!(self, c, do_all_to_all_in_buffer, (index, len, buffer)));
        handle!(
            self,
            CollectiveAllToAllIntoBufferOpHandle,
            CollectiveAllToAllIntoBufferOpFuture,
            fut
        )
    }

    pub(crate) fn broadcast(
        &self,
        src_or_root_pe: BroadcastInput,
        len: usize,
    ) -> CollectiveBroadcastOpHandle<T> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::Broadcast) else {
            return self.base.broadcast(src_or_root_pe, len);
        };
        // the root passes the index of its data and is its own root; the others only name the root
        let (index, root) = match src_or_root_pe {
            BroadcastInput::Root(index) => (index, ManualCollectiveSource::my_pe(&self.region)),
            BroadcastInput::NotRoot(root) => (0, root),
        };
        let fut = boxed(run!(self, c, do_broadcast, (index, len, root)));
        handle!(self, CollectiveBroadcastOpHandle, CollectiveBroadcastOpFuture, fut)
    }

    pub(crate) fn broadcast_into_buffer<B: AsLamellarBuffer<T>>(
        &self,
        target: RootSrcOrLamellarBuffer<T, B>,
        len: usize,
    ) -> CollectiveBroadcastIntoBufferOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::Broadcast) else {
            return self.base.broadcast_into_buffer(target, len);
        };
        let fut = boxed(run!(self, c, do_broadcast_in_buffer, (len, target)));
        handle!(
            self,
            CollectiveBroadcastIntoBufferOpHandle,
            CollectiveBroadcastIntoBufferOpFuture,
            fut
        )
    }

    pub(crate) fn scatter(
        &self,
        src_or_pe: ScatterInput,
        len: usize,
    ) -> CollectiveScatterOpHandle<T> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::Scatter) else {
            return self.base.scatter(src_or_pe, len);
        };
        let (index, root) = match src_or_pe {
            ScatterInput::Root(index) => (index, ManualCollectiveSource::my_pe(&self.region)),
            ScatterInput::NotRoot(root) => (0, root),
        };
        let fut = boxed(run!(self, c, do_scatter, (index, len, root)));
        handle!(self, CollectiveScatterOpHandle, CollectiveScatterOpFuture, fut)
    }

    pub(crate) fn scatter_into_buffer<B: AsLamellarBuffer<T>>(
        &self,
        result: LamellarBuffer<T, B>,
        src_or_pe: ScatterInput,
        len: usize,
    ) -> CollectiveScatterIntoBufferOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::Scatter) else {
            return self.base.scatter_into_buffer(result, src_or_pe, len);
        };
        let (index, root) = match src_or_pe {
            ScatterInput::Root(index) => (index, ManualCollectiveSource::my_pe(&self.region)),
            ScatterInput::NotRoot(root) => (0, root),
        };
        let fut = boxed(run!(self, c, do_scatter_in_buffer, (index, len, root, result)));
        handle!(
            self,
            CollectiveScatterIntoBufferOpHandle,
            CollectiveScatterIntoBufferOpFuture,
            fut
        )
    }

    pub(crate) fn reduce_scatter(
        &self,
        op: ReduceOp,
        index: usize,
        len: usize,
    ) -> CollectiveReduceScatterOpHandle<T> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::ReduceScatter(op.clone())) else {
            return self.base.reduce_scatter(op, index, len);
        };
        let fut = run_family!(
            self,
            c,
            op,
            (do_reduce_scatter, do_reduce_scatter_bitwise, do_reduce_scatter_comparison),
            (index, len, op.clone())
        );
        handle!(
            self,
            CollectiveReduceScatterOpHandle,
            CollectiveReduceScatterOpFuture,
            fut
        )
    }

    pub(crate) fn reduce_scatter_into_buffer<B: AsLamellarBuffer<T>>(
        &self,
        op: ReduceOp,
        index: usize,
        len: usize,
        buffer: LamellarBuffer<T, B>,
    ) -> CollectiveReduceScatterIntoBufferOpHandle<T, B> {
        let Some(c) = self.manual_ctx(CollectiveOpKind::ReduceScatter(op.clone())) else {
            return self.base.reduce_scatter_into_buffer(op, index, len, buffer);
        };
        let fut = run_family!(
            self,
            c,
            op,
            (
                do_reduce_scatter_in_buffer,
                do_reduce_scatter_bitwise_in_buffer,
                do_reduce_scatter_comparison_in_buffer
            ),
            (index, len, op.clone(), buffer)
        );
        handle!(
            self,
            CollectiveReduceScatterIntoBufferOpHandle,
            CollectiveReduceScatterIntoBufferOpFuture,
            fut
        )
    }
}
