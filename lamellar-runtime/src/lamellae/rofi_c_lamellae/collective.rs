//! Native collectives of the `rofi_c` lamellae: libfabric's collective support, reached through ROFI
//! (`rofi_collective_post` / `rofi_collective_progress`, see `communication_frameworks/rofi/src/include/rofi.h`).
//!
//! What the provider implements is asked of ROFI per operation, element type and reduction by
//! [`RofiCComm::collective_avail`](super::comm::RofiCComm); region and array collectives call the native path only where
//! that says yes and otherwise run the put/get algorithms, so nothing here has to handle an unsupported combination.
//!
//! A collective is started when its handle is blocked on, spawned or first polled, from the calling thread: collectives
//! match across PEs by the order they are started, so starting them later (in a spawned task) could reorder them on some
//! PEs. ROFI does not wait for it: the future polls the request, and between polls other threads keep the transport lock
//! they need to put, get and flush, which a collective can depend on to complete.
//!
//! Collectives run over the allocation's PEs (the team of a sub-team allocation, every PE otherwise), and the `root_pe` of
//! the rooted ones is a rank in that team.

use crate::{
    active_messaging::AMCounters,
    lamellae::{
        collective::{
            BroadcastInput, CollectiveAllGatherIntoBufferOpFuture,
            CollectiveAllGatherIntoBufferOpHandle, CollectiveAllGatherOpFuture,
            CollectiveAllGatherOpHandle, CollectiveAllReduceInPlaceOpFuture,
            CollectiveAllReduceInPlaceOpHandle, CollectiveAllReduceIntoBufferOpFuture,
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
            CollectiveScatterOpFuture, CollectiveScatterOpHandle, CommAllocCollectiveAllGather,
            CommAllocCollectiveAllReduce, CommAllocCollectiveAllToAll,
            CommAllocCollectiveBroadcast, CommAllocCollectiveGather, CommAllocCollectiveReduce,
            CommAllocCollectiveReduceScatter, CommAllocCollectiveScatter, ReduceOp,
            RootOrLamellarBuffer, RootSrcOrLamellarBuffer, ScatterInput,
        },
        CollectiveOpKind,
    },
    scheduler::Scheduler,
    warnings::RuntimeWarning,
    AsLamellarBuffer, LamellarBuffer, LamellarTask, Remote,
};

use super::{fabric::RofiCAlloc, rofi::RofiCCollectiveReq, rofi::rofi_c_collective_post};

use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// One native collective: how to start it, and how to turn the buffers it filled into the handle's result. The op owns
/// every buffer ROFI reads or writes, so they stay put until the collective has completed.
pub(crate) trait RofiCCollectiveOp<R>: Send {
    /// Hands the collective to ROFI.
    fn post(&mut self) -> RofiCCollectiveReq;
    /// The result, once the collective has completed.
    fn finish(self: Box<Self>) -> R;
}

/// The `RofiC` variant of every `Collective*OpFuture`.
pub(crate) struct RofiCCollectiveFuture<R: Send + 'static> {
    op: Option<Box<dyn RofiCCollectiveOp<R>>>,
    // `Some` from when the collective is started until it has completed
    req: Option<RofiCCollectiveReq>,
    started: bool,
    scheduler: Arc<Scheduler>,
    counters: Option<Arc<[Arc<AMCounters>]>>,
}

impl<R: Send + 'static> RofiCCollectiveFuture<R> {
    fn new(
        op: impl RofiCCollectiveOp<R> + 'static,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
    ) -> Self {
        Self {
            op: Some(Box::new(op)),
            req: None,
            started: false,
            scheduler: scheduler.clone(),
            counters,
        }
    }

    fn start(&mut self) {
        if !self.started {
            self.req = Some(self.op.as_mut().expect("collective already finished").post());
            self.started = true;
        }
    }

    // makes progress; true once the collective has completed
    fn progress(&mut self) -> bool {
        match &self.req {
            Some(req) => {
                if req.progress() {
                    self.req = None;
                    true
                } else {
                    false
                }
            }
            None => true,
        }
    }

    fn result(&mut self) -> R {
        self.op.take().expect("collective already finished").finish()
    }

    pub(crate) fn block(mut self) -> R {
        self.start();
        let scheduler = self.scheduler.clone();
        scheduler.block_in_place(move || {
            while !self.progress() {
                std::thread::yield_now();
            }
            self.result()
        })
    }

    pub(crate) fn spawn(mut self) -> LamellarTask<R> {
        self.start();
        let counters = self.counters.clone();
        self.scheduler.clone().spawn_task(self, counters)
    }
}

impl<R: Send + 'static> Future for RofiCCollectiveFuture<R> {
    type Output = R;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<R> {
        let this = self.get_mut();
        this.start();
        if this.progress() {
            Poll::Ready(this.result())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl<R: Send + 'static> Drop for RofiCCollectiveFuture<R> {
    fn drop(&mut self) {
        if !self.started {
            RuntimeWarning::DroppedHandle("a RofiCCollectiveFuture").print();
        }
        // ROFI is still reading and writing the op's buffers until the collective completes
        while !self.progress() {
            std::thread::yield_now();
        }
    }
}

/// A buffer a collective writes its result to.
pub(crate) trait OutBuf<T>: Send + 'static {
    fn slice_mut(&mut self) -> &mut [T];
}

impl<T: Remote> OutBuf<T> for Vec<T> {
    fn slice_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Remote, B: AsLamellarBuffer<T>> OutBuf<T> for LamellarBuffer<T, B> {
    fn slice_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

/// The destination of a rooted collective: the caller's buffer where this PE is the root, scratch space elsewhere (ROFI
/// is given a buffer on every PE, but only the root's holds a result).
pub(crate) enum RootBuf<T: Remote, B: AsLamellarBuffer<T>> {
    Buf(LamellarBuffer<T, B>),
    Scratch(Vec<T>),
}

impl<T: Remote, B: AsLamellarBuffer<T>> OutBuf<T> for RootBuf<T, B> {
    fn slice_mut(&mut self) -> &mut [T] {
        match self {
            RootBuf::Buf(buf) => buf.as_mut_slice(),
            RootBuf::Scratch(vec) => vec.as_mut_slice(),
        }
    }
}

// Where a collective reads its input from.
enum Source {
    // not at all (non-root of a scatter)
    None,
    // `index` elements into the allocation
    Alloc(usize),
    // the destination buffer, in place
    InPlace,
}

/// The one shape every native collective has: an input in the allocation (or none), a destination buffer, and a closure
/// that turns the filled destination into the handle's result.
struct CollectiveOp<T: Remote, D: OutBuf<T>, R: Send + 'static> {
    alloc: RofiCAlloc,
    kind: CollectiveOpKind,
    // rank of the root in the allocation's team (0 where there is none)
    root: usize,
    // `count` of the libfabric call, in elements of `T`
    count: usize,
    src: Source,
    // the root of a broadcast: copy this many elements from the allocation (starting at the index) into the destination
    // first, so ROFI, which writes the buffer it broadcasts from, never writes the allocation
    fill_dst_from: Option<usize>,
    // the destination must hold at least this many elements
    dst_min: usize,
    dst: D,
    finish: Box<dyn FnOnce(D) -> R + Send>,
    _t: PhantomData<T>,
}

impl<T: Remote, D: OutBuf<T>, R: Send + 'static> RofiCCollectiveOp<R> for CollectiveOp<T, D, R> {
    fn post(&mut self) -> RofiCCollectiveReq {
        let alloc_ptr = self.alloc.sub_data as *mut T;
        let dst_len = self.dst.slice_mut().len();
        assert!(
            dst_len >= self.dst_min,
            "collective destination holds {dst_len} elements but the collective needs {}",
            self.dst_min
        );
        let dst_ptr = self.dst.slice_mut().as_mut_ptr();
        if let Some(index) = self.fill_dst_from {
            unsafe { std::ptr::copy_nonoverlapping(alloc_ptr.add(index), dst_ptr, self.count) };
        }
        let src_ptr = match self.src {
            Source::None | Source::InPlace => dst_ptr as *const T,
            Source::Alloc(index) => unsafe { alloc_ptr.add(index) as *const T },
        };
        unsafe {
            rofi_c_collective_post::<T>(
                &self.kind,
                self.alloc.team_pes(),
                self.root,
                src_ptr,
                dst_ptr,
                self.count,
            )
        }
    }

    fn finish(self: Box<Self>) -> R {
        (self.finish)(self.dst)
    }
}

fn zeroed_vec<T: Remote>(len: usize) -> Vec<T> {
    (0..len).map(|_| unsafe { std::mem::zeroed() }).collect()
}

// Builds the future of a `CollectiveOp`.
#[allow(clippy::too_many_arguments)]
fn collective<T: Remote, D: OutBuf<T>, R: Send + 'static>(
    alloc: &RofiCAlloc,
    scheduler: &Arc<Scheduler>,
    counters: Option<Arc<[Arc<AMCounters>]>>,
    kind: CollectiveOpKind,
    root: usize,
    count: usize,
    src: Source,
    fill_dst_from: Option<usize>,
    dst_min: usize,
    dst: D,
    finish: impl FnOnce(D) -> R + Send + 'static,
) -> RofiCCollectiveFuture<R> {
    RofiCCollectiveFuture::new(
        CollectiveOp {
            alloc: alloc.clone(),
            kind,
            root,
            count,
            src,
            fill_dst_from,
            dst_min,
            dst,
            finish: Box::new(finish),
            _t: PhantomData,
        },
        scheduler,
        counters,
    )
}

impl CommAllocCollectiveAllReduce for RofiCAlloc {
    fn reduce_all<T: Remote>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        index: usize,
        len: usize,
        op: ReduceOp,
    ) -> CollectiveAllReduceOpHandle<T> {
        CollectiveAllReduceOpHandle {
            future: CollectiveAllReduceOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::AllReduce(op),
                0,
                len,
                Source::Alloc(index),
                None,
                len,
                zeroed_vec::<T>(len),
                |dst| dst,
            )),
        }
    }

    fn reduce_all_into_buffer<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        index: usize,
        len: usize,
        op: ReduceOp,
        dst: LamellarBuffer<T, B>,
    ) -> CollectiveAllReduceIntoBufferOpHandle<T, B> {
        CollectiveAllReduceIntoBufferOpHandle {
            future: CollectiveAllReduceIntoBufferOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::AllReduce(op),
                0,
                len,
                Source::Alloc(index),
                None,
                len,
                dst,
                |_| (),
            )),
        }
    }

    fn reduce_all_in_place<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        mut src_and_dst: LamellarBuffer<T, B>,
        op: ReduceOp,
    ) -> CollectiveAllReduceInPlaceOpHandle<T, B> {
        let len = src_and_dst.as_mut_slice().len();
        CollectiveAllReduceInPlaceOpHandle {
            future: CollectiveAllReduceInPlaceOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::AllReduce(op),
                0,
                len,
                Source::InPlace,
                None,
                len,
                src_and_dst,
                |_| (),
            )),
        }
    }
}

impl CommAllocCollectiveReduce for RofiCAlloc {
    fn reduce<T: Remote>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        op: ReduceOp,
        index: usize,
        len: usize,
        root_pe: usize,
    ) -> CollectiveReduceOpHandle<T> {
        let is_root = root_pe == self.team_pe();
        CollectiveReduceOpHandle {
            future: CollectiveReduceOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::Reduce(op),
                root_pe,
                len,
                Source::Alloc(index),
                None,
                len,
                zeroed_vec::<T>(len),
                move |dst| if is_root { Some(dst) } else { None },
            )),
        }
    }

    fn reduce_into_buffer<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        op: ReduceOp,
        index: usize,
        len: usize,
        root_or_buffer: RootOrLamellarBuffer<T, B>,
    ) -> CollectiveReduceIntoBufferOpHandle<T, B> {
        let (dst, root) = match root_or_buffer {
            RootOrLamellarBuffer::Root(buffer) => (RootBuf::Buf(buffer), self.team_pe()),
            RootOrLamellarBuffer::NotRoot(root_pe) => {
                (RootBuf::Scratch(zeroed_vec::<T>(len)), root_pe)
            }
        };
        CollectiveReduceIntoBufferOpHandle {
            future: CollectiveReduceIntoBufferOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::Reduce(op),
                root,
                len,
                Source::Alloc(index),
                None,
                len,
                dst,
                |_| (),
            )),
        }
    }
}

impl CommAllocCollectiveAllGather for RofiCAlloc {
    fn gather_all<T: Remote>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        index: usize,
        len: usize,
    ) -> CollectiveAllGatherOpHandle<T> {
        let total = len * self.team_size();
        CollectiveAllGatherOpHandle {
            future: CollectiveAllGatherOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::AllGather,
                0,
                len,
                Source::Alloc(index),
                None,
                total,
                zeroed_vec::<T>(total),
                |dst| dst,
            )),
        }
    }

    fn gather_all_into_buffer<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        index: usize,
        len: usize,
        dst: LamellarBuffer<T, B>,
    ) -> CollectiveAllGatherIntoBufferOpHandle<T, B> {
        CollectiveAllGatherIntoBufferOpHandle {
            future: CollectiveAllGatherIntoBufferOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::AllGather,
                0,
                len,
                Source::Alloc(index),
                None,
                len * self.team_size(),
                dst,
                |_| (),
            )),
        }
    }
}

impl CommAllocCollectiveGather for RofiCAlloc {
    fn gather<T: Remote>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        index: usize,
        len: usize,
        root_pe: usize,
    ) -> CollectiveGatherOpHandle<T> {
        let is_root = root_pe == self.team_pe();
        // only the root's destination holds the result
        let total = if is_root { len * self.team_size() } else { len };
        CollectiveGatherOpHandle {
            future: CollectiveGatherOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::Gather,
                root_pe,
                len,
                Source::Alloc(index),
                None,
                total,
                zeroed_vec::<T>(total),
                move |dst| if is_root { Some(dst) } else { None },
            )),
        }
    }

    fn gather_into_buffer<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        index: usize,
        len: usize,
        root_or_buffer: RootOrLamellarBuffer<T, B>,
    ) -> CollectiveGatherIntoBufferOpHandle<T, B> {
        let (dst, root, dst_min) = match root_or_buffer {
            RootOrLamellarBuffer::Root(buffer) => {
                (RootBuf::Buf(buffer), self.team_pe(), len * self.team_size())
            }
            RootOrLamellarBuffer::NotRoot(root_pe) => {
                (RootBuf::Scratch(zeroed_vec::<T>(len)), root_pe, len)
            }
        };
        CollectiveGatherIntoBufferOpHandle {
            future: CollectiveGatherIntoBufferOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::Gather,
                root,
                len,
                Source::Alloc(index),
                None,
                dst_min,
                dst,
                |_| (),
            )),
        }
    }
}

// `alltoall` is run as ROFI's all-gather, as it is by the other lamellae that implement it natively (see
// `CommAllocCollectiveAllToAll`).
impl CommAllocCollectiveAllToAll for RofiCAlloc {
    fn alltoall<T: Remote>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        index: usize,
        len: usize,
    ) -> CollectiveAllToAllOpHandle<T> {
        let total = len * self.team_size();
        CollectiveAllToAllOpHandle {
            future: CollectiveAllToAllOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::AllToAll,
                0,
                len,
                Source::Alloc(index),
                None,
                total,
                zeroed_vec::<T>(total),
                |dst| dst,
            )),
        }
    }

    fn alltoall_into_buffer<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        index: usize,
        len: usize,
        dst: LamellarBuffer<T, B>,
    ) -> CollectiveAllToAllIntoBufferOpHandle<T, B> {
        CollectiveAllToAllIntoBufferOpHandle {
            future: CollectiveAllToAllIntoBufferOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::AllToAll,
                0,
                len,
                Source::Alloc(index),
                None,
                len * self.team_size(),
                dst,
                |_| (),
            )),
        }
    }
}

impl CommAllocCollectiveBroadcast for RofiCAlloc {
    fn broadcast<T: Remote>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        src_or_root_pe: BroadcastInput,
        len: usize,
    ) -> CollectiveBroadcastOpHandle<T> {
        let (root, fill, is_root) = match src_or_root_pe {
            BroadcastInput::Root(index) => (self.team_pe(), Some(index), true),
            BroadcastInput::NotRoot(root_pe) => (root_pe, None, false),
        };
        CollectiveBroadcastOpHandle {
            future: CollectiveBroadcastOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::Broadcast,
                root,
                len,
                Source::None,
                fill,
                len,
                zeroed_vec::<T>(len),
                move |dst| if is_root { None } else { Some(dst) },
            )),
        }
    }

    fn broadcast_into_buffer<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        dst: RootSrcOrLamellarBuffer<T, B>,
        len: usize,
    ) -> CollectiveBroadcastIntoBufferOpHandle<T, B> {
        let (dst, root, fill) = match dst {
            RootSrcOrLamellarBuffer::Root(index) => (
                RootBuf::Scratch(zeroed_vec::<T>(len)),
                self.team_pe(),
                Some(index),
            ),
            RootSrcOrLamellarBuffer::NotRoot(buffer, root_pe) => {
                (RootBuf::Buf(buffer), root_pe, None)
            }
        };
        CollectiveBroadcastIntoBufferOpHandle {
            future: CollectiveBroadcastIntoBufferOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::Broadcast,
                root,
                len,
                Source::None,
                fill,
                len,
                dst,
                |_| (),
            )),
        }
    }
}

impl CommAllocCollectiveScatter for RofiCAlloc {
    fn scatter<T: Remote>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        src_or_root_pe: ScatterInput,
        len: usize,
    ) -> CollectiveScatterOpHandle<T> {
        let (root, src) = match src_or_root_pe {
            ScatterInput::Root(index) => (self.team_pe(), Source::Alloc(index)),
            ScatterInput::NotRoot(root_pe) => (root_pe, Source::None),
        };
        CollectiveScatterOpHandle {
            future: CollectiveScatterOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::Scatter,
                root,
                len,
                src,
                None,
                len,
                zeroed_vec::<T>(len),
                |dst| dst,
            )),
        }
    }

    fn scatter_into_buffer<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        dst: LamellarBuffer<T, B>,
        src_or_root_pe: ScatterInput,
        len: usize,
    ) -> CollectiveScatterIntoBufferOpHandle<T, B> {
        let (root, src) = match src_or_root_pe {
            ScatterInput::Root(index) => (self.team_pe(), Source::Alloc(index)),
            ScatterInput::NotRoot(root_pe) => (root_pe, Source::None),
        };
        CollectiveScatterIntoBufferOpHandle {
            future: CollectiveScatterIntoBufferOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::Scatter,
                root,
                len,
                src,
                None,
                len,
                dst,
                |_| (),
            )),
        }
    }
}

impl CommAllocCollectiveReduceScatter for RofiCAlloc {
    fn reduce_scatter<T: Remote>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        op: ReduceOp,
        index: usize,
        len: usize,
    ) -> CollectiveReduceScatterOpHandle<T> {
        let per_pe = len / self.team_size();
        CollectiveReduceScatterOpHandle {
            future: CollectiveReduceScatterOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::ReduceScatter(op),
                0,
                len,
                Source::Alloc(index),
                None,
                per_pe,
                zeroed_vec::<T>(per_pe),
                |dst| dst,
            )),
        }
    }

    fn reduce_scatter_into_buffer<T: Remote, B: AsLamellarBuffer<T>>(
        &self,
        scheduler: &Arc<Scheduler>,
        counters: Option<Arc<[Arc<AMCounters>]>>,
        op: ReduceOp,
        index: usize,
        len: usize,
        dst: LamellarBuffer<T, B>,
    ) -> CollectiveReduceScatterIntoBufferOpHandle<T, B> {
        // each PE receives `len / team_size` elements and the buffer must hold them (a longer one is written only up to that)
        assert!(
            dst.len() >= len / self.team_size(),
            "reduce_scatter into a buffer: each PE receives len / team size = {} / {} = {} elements, but the buffer holds only {}",
            len,
            self.team_size(),
            len / self.team_size(),
            dst.len()
        );
        CollectiveReduceScatterIntoBufferOpHandle {
            future: CollectiveReduceScatterIntoBufferOpFuture::RofiC(collective(
                self,
                scheduler,
                counters,
                CollectiveOpKind::ReduceScatter(op),
                0,
                len,
                Source::Alloc(index),
                None,
                len / self.team_size(),
                dst,
                |_| (),
            )),
        }
    }
}
