//! Staging blocks carved from a backend's own registered runtime heap, for the libfabric backends.
//!
//! The pool itself is [`super::staging`]; this supplies its memory. A libfabric backend already owns large registered
//! pools (`runtime_allocs`) that the provider recognises as registered, so a staging block is a sub-range of one of
//! them and needs no registration of its own: the provider finds the covering registration when the operation is
//! posted (`local_desc_for`), so a block carries no token.
//!
//! Memory comes in slabs, each one runtime-heap allocation, kept for the life of the backend: the shared pool recycles
//! blocks by size class instead of giving them back. Allocations above the pool's limits (or over-aligned ones) are
//! still staged, with an exact-size runtime-heap block that goes back to the heap when the operation is done: a
//! libfabric provider that registers such memory on the fly can serve a stale registration whatever the size, so
//! unlike the UCX backends nothing is left to the provider.

use parking_lot::Mutex;
use std::sync::Arc;

use super::staging::{RawBlock, Staging, StagingBacking, StagingPool, Try, BLOCK_ALIGN};

/// Slabs are this big (or one block, if bigger) until the pool's capacity is reached.
const SLAB_BYTES: usize = 4 << 20;
/// A slab added past the capacity for a starved size class is no bigger than this (or one block).
const FORCED_SLAB_BYTES: usize = 4 << 20;
/// Pooled blocks go up to this size unless `LAMELLAR_RDMA_STAGING_MAX` says otherwise.
const DEFAULT_MAX_BYTES: usize = 16 << 20;

/// Allocates `len` bytes of the backend's runtime heap: the start address and the handle that keeps it allocated.
pub(crate) type RtGrow<H> = dyn Fn(usize, usize) -> Option<(usize, H)> + Send + Sync;

struct Slabs<H> {
    held: Vec<Arc<H>>,
    /// Index in `held` of the newest slab: the token of the blocks cut from it.
    /// Next free byte of the newest slab (always [`BLOCK_ALIGN`]-aligned) and the end of it.
    cur: usize,
    next: usize,
    end: usize,
    total: usize,
    released: bool,
}

/// The pool's backing: slabs from the runtime heap.
pub(crate) struct RtSlabBacking<H: Send + Sync + 'static> {
    cap: usize,
    grow: Box<RtGrow<H>>,
    inner: Mutex<Slabs<H>>,
}

impl<H: Send + Sync + 'static> RtSlabBacking<H> {
    /// The slab a block came from, from its token.
    fn slab(&self, token: usize) -> Option<Arc<H>> {
        self.inner.lock().held.get(token).cloned()
    }

    /// Unmap every slab. Must run before the backend drops its runtime pools (a slab holds a reference to the backend,
    /// which would keep it alive forever); running it twice is harmless. No block may be in use when it runs.
    pub(crate) fn release_all(&self) {
        let held = {
            let mut s = self.inner.lock();
            s.released = true;
            s.total = 0;
            s.next = 0;
            s.end = 0;
            std::mem::take(&mut s.held)
        };
        drop(held);
    }
}

impl<H: Send + Sync + 'static> StagingBacking for RtSlabBacking<H> {
    fn alloc_block(&self, len: usize, force: bool) -> Option<RawBlock> {
        debug_assert!(len.is_power_of_two() && len >= BLOCK_ALIGN);
        let mut s = self.inner.lock();
        if s.released {
            return None;
        }
        // Bump allocation from the newest slab. Blocks are multiples of BLOCK_ALIGN, so offsets stay aligned.
        if s.next + len <= s.end {
            let ptr = s.next as *mut u8;
            s.next += len;
            return Some(RawBlock { ptr, len, token: s.cur });
        }
        // The newest slab is full: add another, as big as the capacity left allows. A forced block (its size class has
        // nothing cached or checked out) goes past the capacity, with a modest slab, so that no size class can be
        // starved by room held as cached blocks of other sizes.
        let mut slab_len = SLAB_BYTES.max(len);
        if s.total + slab_len > self.cap {
            slab_len = len;
            if force {
                slab_len = (len * 64).clamp(len, FORCED_SLAB_BYTES.max(len));
            } else if s.total + slab_len > self.cap {
                return None;
            }
        }
        // Room to align the start of the slab.
        let request = slab_len + BLOCK_ALIGN;
        let (start, handle) = (self.grow)(request, BLOCK_ALIGN)?;
        let base = (start + BLOCK_ALIGN - 1) & !(BLOCK_ALIGN - 1);
        s.next = base + len;
        s.end = start + request;
        s.total += request;
        s.held.push(Arc::new(handle));
        s.cur = s.held.len() - 1;
        Some(RawBlock { ptr: base as *mut u8, len, token: s.cur })
    }
}

type Pool<H> = StagingPool<RtSlabBacking<H>>;

/// A backend's staging memory: the shared pool plus the exact-size fallback for what the pool does not take.
pub(crate) struct RtStagingPool<H: Send + Sync + 'static> {
    enabled: bool,
    max_bytes: usize,
    pool: Arc<Pool<H>>,
    /// Exact-size runtime-heap allocation (for what the pool does not take): same arguments as [`RtGrow`].
    direct: Box<RtGrow<H>>,
}

impl<H: Send + Sync + 'static> RtStagingPool<H> {
    /// `alloc` allocates from the backend's runtime heap (`len`, `align`). It must not keep the backend alive by
    /// itself (capture a `Weak`): the slabs it returns do, and are released by [`release_all`](Self::release_all).
    pub(crate) fn new(
        name: &'static str,
        alloc: impl Fn(usize, usize) -> Option<(usize, H)> + Send + Sync + Clone + 'static,
    ) -> Self {
        let cfg = crate::config();
        let enabled = cfg.rdma_staging.unwrap_or(true);
        let max_bytes = cfg.rdma_staging_max.unwrap_or(DEFAULT_MAX_BYTES);
        let backing = RtSlabBacking {
            // The slabs come out of the runtime heap, and these backends used to stage out of all of it: cap at half.
            cap: cfg.rdma_staging_pool.unwrap_or_else(|| cfg.heap_size.unwrap_or(4 << 30) / 2),
            grow: Box::new(alloc.clone()),
            inner: Mutex::new(Slabs {
                held: Vec::new(),
                cur: 0,
                next: 0,
                end: 0,
                total: 0,
                released: false,
            }),
        };
        RtStagingPool {
            enabled,
            max_bytes,
            // Every size is staged by default: the provider must never register the caller's buffer itself.
            pool: StagingPool::new(name, backing, enabled, cfg.rdma_staging_min.unwrap_or(1), max_bytes),
            direct: Box::new(alloc),
        }
    }

    /// A registered block for a buffer of `bytes` bytes, aligned for `align`. `None`: use the caller's own buffer
    /// (staging off, nothing to transfer, or no memory even after `progress` had let operations in flight finish).
    pub(crate) fn alloc(&self, bytes: usize, align: usize, mut progress: impl FnMut()) -> Option<RtStaging<H>> {
        if !self.enabled || bytes == 0 {
            return None;
        }
        if bytes > self.max_bytes || align > BLOCK_ALIGN {
            for attempt in 0..3 {
                if let Some((start, handle)) = (self.direct)(bytes, align.max(BLOCK_ALIGN)) {
                    return Some(RtStaging::direct(start, handle, bytes));
                }
                if attempt < 2 {
                    progress();
                }
            }
            return None;
        }
        for attempt in 0..3 {
            match self.pool.try_acquire(bytes) {
                Try::Got(block) => return self.pooled(block, bytes),
                Try::NotWanted => return None,
                Try::Exhausted if attempt < 2 => progress(),
                Try::Exhausted => break,
            }
        }
        // Notes the fallback (and warns once); a block may have come back meanwhile.
        self.pool.acquire(bytes).and_then(|block| self.pooled(block, bytes))
    }

    fn pooled(&self, block: Staging<RtSlabBacking<H>>, bytes: usize) -> Option<RtStaging<H>> {
        // None only after `release_all`, when there is no staging any more.
        let slab = self.pool.backing().slab(block.token())?;
        Some(RtStaging { kind: Kind::Pooled { block, slab }, bytes })
    }

    /// How many operations fell back to the caller's buffer because no block could be had.
    pub(crate) fn fallbacks(&self) -> usize {
        self.pool.fallbacks()
    }

    /// Give the slabs back to the runtime heap. Call during teardown, before the runtime pools are dropped.
    pub(crate) fn release_all(&self) {
        self.pool.backing().release_all();
    }
}

enum Kind<H: Send + Sync + 'static> {
    /// A block of a slab; `slab` is the runtime-heap allocation it lies in.
    Pooled { block: Staging<RtSlabBacking<H>>, slab: Arc<H> },
    Direct { start: usize, handle: H },
}

/// A registered scratch buffer of `bytes` bytes for one RDMA operation. The memory goes back to the pool (or, for a
/// block that was not pooled, to the runtime heap) when this is dropped.
pub(crate) struct RtStaging<H: Send + Sync + 'static> {
    kind: Kind<H>,
    bytes: usize,
}

impl<H: Send + Sync + 'static> RtStaging<H> {
    fn direct(start: usize, handle: H, bytes: usize) -> Self {
        RtStaging { kind: Kind::Direct { start, handle }, bytes }
    }

    /// The runtime-heap allocation this buffer lies in (its slab, for a pooled block). For backends whose operations
    /// name the registration of the local buffer (the descriptor of the allocation).
    pub(crate) fn handle(&self) -> &H {
        match &self.kind {
            Kind::Pooled { slab, .. } => slab,
            Kind::Direct { handle, .. } => handle,
        }
    }

    pub(crate) fn start(&self) -> usize {
        match &self.kind {
            Kind::Pooled { block, .. } => block.ptr() as usize,
            Kind::Direct { start, .. } => *start,
        }
    }

    /// The `bytes` requested, as a slice of `T`.
    pub(crate) unsafe fn as_slice<T>(&self) -> &[T] {
        std::slice::from_raw_parts(self.start() as *const T, self.bytes / std::mem::size_of::<T>())
    }

    pub(crate) unsafe fn as_mut_slice<T>(&self) -> &mut [T] {
        std::slice::from_raw_parts_mut(self.start() as *mut T, self.bytes / std::mem::size_of::<T>())
    }
}
