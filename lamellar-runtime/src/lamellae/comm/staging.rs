//! Registered staging blocks for RDMA operations whose local buffer is not registered memory.
//!
//! When the local side of a get (or put) is ordinary heap memory (a `Vec`, a stack value), the communication
//! library has to make it reachable by the NIC. Registering it on the fly costs hundreds of microseconds per
//! operation, which dwarfs the transfer for anything from about a kilobyte up to a few megabytes, and a registration
//! cache can also go stale when the heap range is recycled (a transfer then lands in the old physical pages). A
//! staging block avoids both: the transfer targets a block of memory that was registered once, and the caller's
//! buffer is filled by a copy once the transfer has completed (for a put: filled before it starts).
//!
//! The copy costs memory bandwidth, so staging only pays off in a size range. Below `min_bytes` the library copies
//! internally and cheaply anyway; above `max_bytes` the copy outweighs the one registration the library would do for
//! the transfer. Both limits are the caller's to choose (see `LAMELLAR_RDMA_STAGING_MIN` / `_MAX`).
//!
//! This module only manages the blocks: power-of-two size classes, recycled through per-class free lists. A backend
//! supplies the registered memory by implementing [`StagingBacking`] and keeps its own record of how to address a
//! block to its library (a memory handle, a descriptor, ...), found from [`Staging::token`]. The completion side (when
//! it is safe to copy out and release the block) stays in the backend's futures.
//!
//! # When the pool is empty
//!
//! The number of operations in flight is not bounded by anything here (a program may issue hundreds of thousands of
//! asynchronous gets), so the pool will run out. An operation that can wait (`poll_acquire`) then queues for a block
//! of its size class instead of dropping to the slow unregistered path: a block that is released goes **directly** to
//! the oldest waiter of its class (first in, first out), so there is no wake-up to lose and no thundering herd, and a
//! waiter that is cancelled hands any block it was given on to the next one. Memory in flight stays bounded by the
//! pool's capacity. An operation only waits if some block of its class is currently checked out (so one will come
//! back); otherwise it falls back to the caller's own buffer, with a one-time warning.

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// Smallest block class: 64 bytes.
const MIN_CLASS_LOG2: u32 = 6;
/// Largest block class: 2^37 bytes, far above any limit worth setting.
const NUM_CLASSES: usize = 32;
/// Every block is aligned to at least this.
pub(crate) const BLOCK_ALIGN: usize = 64;

/// A block of registered memory handed out by a [`StagingBacking`].
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub(crate) struct RawBlock {
    pub(crate) ptr: *mut u8,
    pub(crate) len: usize,
    /// Whatever the backend needs to find the registration that covers this block (for example a memory handle).
    pub(crate) token: usize,
}

unsafe impl Send for RawBlock {}
unsafe impl Sync for RawBlock {}

/// Where a backend's staging memory comes from.
pub(crate) trait StagingBacking: Send + Sync + 'static {
    /// Registered memory for one block: `len` bytes (a power of two, at least 64), aligned to [`BLOCK_ALIGN`].
    /// `None` when the backing cannot provide more. Blocks are never given back to the backing: the pool recycles
    /// them, so a backing only has to be able to grow.
    ///
    /// `force` is set when the size class has no block cached and none checked out, so it could never be served by
    /// waiting: the backing should provide the memory even if that goes past its usual capacity (by a bounded amount:
    /// the pool asks at most about once per size class), and return `None` only if it truly cannot.
    fn alloc_block(&self, len: usize, force: bool) -> Option<RawBlock>;
}

/// What a queued operation is waiting for.
enum SlotState {
    /// Queued; wake this task when a block is handed over.
    Waiting(Waker),
    /// A block has been handed over and is waiting to be claimed.
    Granted(RawBlock),
    /// Nothing will ever come back for this class: use the caller's buffer.
    Fallback,
    /// No longer waiting (claimed, or abandoned). Skipped when a block is released.
    Cancelled,
}

struct Slot {
    state: Mutex<SlotState>,
}

/// A pool of staging blocks over a backend's registered memory.
pub(crate) struct StagingPool<B: StagingBacking> {
    backing: B,
    name: &'static str,
    enabled: bool,
    min_bytes: usize,
    max_bytes: usize,
    /// Free blocks, one list per size class.
    free: Vec<Mutex<Vec<RawBlock>>>,
    /// Blocks of each class that are checked out (including ones handed to a waiter and not yet claimed).
    holders: Vec<AtomicUsize>,
    /// Operations waiting for a block, per class, oldest first.
    waiters: Vec<Mutex<VecDeque<Arc<Slot>>>>,
    fallbacks: AtomicUsize,
    warned: AtomicBool,
}

impl<B: StagingBacking> std::fmt::Debug for StagingPool<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StagingPool")
            .field("name", &self.name)
            .field("enabled", &self.enabled)
            .field("min_bytes", &self.min_bytes)
            .field("max_bytes", &self.max_bytes)
            .field("fallbacks", &self.fallbacks.load(Ordering::Relaxed))
            .finish()
    }
}

/// The result of trying to take a block without waiting.
pub(crate) enum Try<B: StagingBacking> {
    /// Staging does not apply to this buffer (disabled, or outside the size range).
    NotWanted,
    Got(Staging<B>),
    /// Staging applies but there is no block to be had right now.
    Exhausted,
}

/// A queued request for a block; kept by the operation that is waiting (see [`StagingPool::poll_acquire`]).
pub(crate) struct StagingWait<B: StagingBacking> {
    pool: Arc<StagingPool<B>>,
    class: usize,
    bytes: usize,
    slot: Arc<Slot>,
}

impl<B: StagingBacking> Drop for StagingWait<B> {
    fn drop(&mut self) {
        // Abandoned while waiting: if a block was already handed over, pass it on.
        let state = std::mem::replace(&mut *self.slot.state.lock(), SlotState::Cancelled);
        if let SlotState::Granted(block) = state {
            self.pool.give_back(self.class, block);
        }
    }
}

fn class_of(bytes: usize) -> Option<(usize, usize)> {
    let class_len = bytes.next_power_of_two().max(1 << MIN_CLASS_LOG2);
    let class = (class_len.trailing_zeros() - MIN_CLASS_LOG2) as usize;
    (class < NUM_CLASSES).then_some((class, class_len))
}

impl<B: StagingBacking> StagingPool<B> {
    /// `name` appears in the exhaustion warning. Buffers of `min_bytes..=max_bytes` are staged when `enabled`.
    pub(crate) fn new(
        name: &'static str,
        backing: B,
        enabled: bool,
        min_bytes: usize,
        max_bytes: usize,
    ) -> Arc<Self> {
        Arc::new(StagingPool {
            backing,
            name,
            enabled,
            min_bytes,
            max_bytes,
            free: (0..NUM_CLASSES).map(|_| Mutex::new(Vec::new())).collect(),
            holders: (0..NUM_CLASSES).map(|_| AtomicUsize::new(0)).collect(),
            waiters: (0..NUM_CLASSES).map(|_| Mutex::new(VecDeque::new())).collect(),
            fallbacks: AtomicUsize::new(0),
            warned: AtomicBool::new(false),
        })
    }

    pub(crate) fn backing(&self) -> &B {
        &self.backing
    }

    /// Whether a buffer of `bytes` bytes is in the range this pool stages.
    #[inline]
    pub(crate) fn wants(&self, bytes: usize) -> bool {
        self.enabled && bytes >= self.min_bytes && bytes <= self.max_bytes && bytes != 0
    }

    /// Take a block if one is available now. Neither waits nor counts a fallback.
    pub(crate) fn try_acquire(self: &Arc<Self>, bytes: usize) -> Try<B> {
        if !self.wants(bytes) {
            return Try::NotWanted;
        }
        let Some((class, class_len)) = class_of(bytes) else {
            return Try::NotWanted;
        };
        let recycled = self.free[class].lock().pop();
        let block = match recycled {
            Some(block) => block,
            None => match self.backing.alloc_block(class_len, false) {
                Some(block) => block,
                // Out of room. Waiting only helps if a block of this class is checked out and will come back; if there
                // is none (the room is all held as cached blocks of other sizes), a block has to be forced.
                None if self.holders[class].load(Ordering::Acquire) == 0 => {
                    match self.backing.alloc_block(class_len, true) {
                        Some(block) => block,
                        None => return Try::Exhausted,
                    }
                }
                None => return Try::Exhausted,
            },
        };
        self.holders[class].fetch_add(1, Ordering::AcqRel);
        Try::Got(Staging {
            pool: self.clone(),
            block: Some(block),
            bytes,
            class,
        })
    }

    fn note_fallback(&self, bytes: usize) {
        self.fallbacks.fetch_add(1, Ordering::Relaxed);
        if !self.warned.swap(true, Ordering::Relaxed) {
            eprintln!(
                "[LAMELLAR WARNING] {} registered staging pool exhausted (needed {} bytes); RDMA operations \
                 fall back to unregistered buffers, which is slower. Raise LAMELLAR_RDMA_STAGING_POOL to avoid this.",
                self.name, bytes
            );
        }
    }

    /// A staging block for a buffer of `bytes` bytes without waiting, or `None` when staging does not apply (disabled,
    /// outside the range) or the pool is exhausted, in which case the caller uses its own buffer. For callers that
    /// cannot yield (blocking calls).
    pub(crate) fn acquire(self: &Arc<Self>, bytes: usize) -> Option<Staging<B>> {
        match self.try_acquire(bytes) {
            Try::Got(staging) => Some(staging),
            Try::NotWanted => None,
            Try::Exhausted => {
                self.note_fallback(bytes);
                None
            }
        }
    }

    /// A staging block for a buffer of `bytes` bytes, queueing for one if the pool is empty.
    ///
    /// `Ready(None)` means staging does not apply (or nothing will come back for this size class), and the caller
    /// uses its own buffer. `Pending` means the task has been queued and will be woken once a block is handed to it;
    /// poll again with the same `wait`. Dropping `wait` abandons the request.
    pub(crate) fn poll_acquire(
        self: &Arc<Self>,
        cx: &mut Context<'_>,
        bytes: usize,
        wait: &mut Option<StagingWait<B>>,
    ) -> Poll<Option<Staging<B>>> {
        // Already queued: see whether a block has been handed over.
        if let Some(w) = wait.take() {
            let mut state = w.slot.state.lock();
            match &mut *state {
                SlotState::Waiting(waker) => {
                    if !waker.will_wake(cx.waker()) {
                        *waker = cx.waker().clone();
                    }
                    drop(state);
                    *wait = Some(w);
                    return Poll::Pending;
                }
                SlotState::Granted(_) => {
                    let SlotState::Granted(block) = std::mem::replace(&mut *state, SlotState::Cancelled) else {
                        unreachable!()
                    };
                    drop(state);
                    return Poll::Ready(Some(Staging {
                        pool: self.clone(),
                        block: Some(block),
                        bytes: w.bytes,
                        class: w.class,
                    }));
                }
                SlotState::Fallback | SlotState::Cancelled => {
                    drop(state);
                    self.note_fallback(w.bytes);
                    return Poll::Ready(None);
                }
            }
        }

        match self.try_acquire(bytes) {
            Try::NotWanted => return Poll::Ready(None),
            Try::Got(staging) => return Poll::Ready(Some(staging)),
            Try::Exhausted => {}
        }
        let Some((class, _)) = class_of(bytes) else {
            return Poll::Ready(None);
        };
        // Only wait if a block of this class is out there to come back.
        if self.holders[class].load(Ordering::Acquire) == 0 {
            self.note_fallback(bytes);
            return Poll::Ready(None);
        }
        let slot = Arc::new(Slot {
            state: Mutex::new(SlotState::Waiting(cx.waker().clone())),
        });
        self.waiters[class].lock().push_back(slot.clone());
        // A block may have been released between the failed attempt and joining the queue; look once more.
        match self.try_acquire(bytes) {
            Try::Got(staging) => {
                *slot.state.lock() = SlotState::Cancelled; // leaves a stale queue entry, skipped on release
                return Poll::Ready(Some(staging));
            }
            Try::NotWanted | Try::Exhausted => {}
        }
        if self.holders[class].load(Ordering::Acquire) == 0 {
            *slot.state.lock() = SlotState::Cancelled;
            self.note_fallback(bytes);
            return Poll::Ready(None);
        }
        *wait = Some(StagingWait {
            pool: self.clone(),
            class,
            bytes,
            slot,
        });
        Poll::Pending
    }

    /// How many times an operation had to fall back to the caller's buffer because the pool was exhausted.
    #[allow(dead_code)]
    pub(crate) fn fallbacks(&self) -> usize {
        self.fallbacks.load(Ordering::Relaxed)
    }

    /// Return a block: to the oldest waiter of its class if there is one (the block stays checked out, now by the
    /// waiter), otherwise to the free list.
    fn give_back(&self, class: usize, block: RawBlock) {
        loop {
            let next = self.waiters[class].lock().pop_front();
            let Some(slot) = next else {
                self.free[class].lock().push(block);
                self.holders[class].fetch_sub(1, Ordering::AcqRel);
                return;
            };
            let mut state = slot.state.lock();
            if let SlotState::Waiting(_) = &*state {
                let SlotState::Waiting(waker) = std::mem::replace(&mut *state, SlotState::Granted(block)) else {
                    unreachable!()
                };
                drop(state);
                waker.wake();
                return;
            }
            // Abandoned or already settled: try the next one.
        }
    }

    /// A block that was checked out will never come back (its operation was dropped while the NIC might still be
    /// writing it). If that was the last one of its class, nothing will ever be released for the waiters, so they
    /// are told to use their own buffers.
    fn leak(&self, class: usize) {
        if self.holders[class].fetch_sub(1, Ordering::AcqRel) != 1 {
            return;
        }
        let stranded: Vec<Arc<Slot>> = self.waiters[class].lock().drain(..).collect();
        for slot in stranded {
            let mut state = slot.state.lock();
            if let SlotState::Waiting(_) = &*state {
                let SlotState::Waiting(waker) = std::mem::replace(&mut *state, SlotState::Fallback) else {
                    unreachable!()
                };
                drop(state);
                waker.wake();
            }
        }
    }
}

/// A staging block lent to one RDMA operation; it returns to the pool when dropped.
///
/// Dropping it while the operation that targets it is still in flight would let another operation reuse the block
/// under the NIC. Whoever holds it must keep it until the operation has completed (see `forget_in_flight`).
pub(crate) struct Staging<B: StagingBacking> {
    pool: Arc<StagingPool<B>>,
    block: Option<RawBlock>,
    bytes: usize,
    class: usize,
}

impl<B: StagingBacking> Staging<B> {
    /// Start of the block.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut u8 {
        self.block.as_ref().expect("staging block").ptr
    }

    /// The backend's token for the registration covering this block.
    #[inline]
    pub(crate) fn token(&self) -> usize {
        self.block.as_ref().expect("staging block").token
    }

    /// The pool this block came from (for its backing).
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn pool(&self) -> &Arc<StagingPool<B>> {
        &self.pool
    }

    /// The `bytes` requested, viewed as `T`s.
    #[allow(dead_code)]
    #[inline]
    pub(crate) unsafe fn as_slice<T>(&self) -> &[T] {
        std::slice::from_raw_parts(self.ptr() as *const T, self.bytes / std::mem::size_of::<T>())
    }

    /// The `bytes` requested, viewed as mutable `T`s.
    #[allow(dead_code)]
    #[inline]
    pub(crate) unsafe fn as_mut_slice<T>(&self) -> &mut [T] {
        std::slice::from_raw_parts_mut(self.ptr() as *mut T, self.bytes / std::mem::size_of::<T>())
    }

    /// Give up the block without returning it to the pool. For an operation that is dropped while still in flight:
    /// the NIC may still be writing, so the block must never be handed to another operation. The block is lost to
    /// the pool (the pool's capacity is bounded, so this is bounded too).
    pub(crate) fn forget_in_flight(mut self) {
        if self.block.take().is_some() {
            self.pool.leak(self.class);
        }
    }
}

impl<B: StagingBacking> Drop for Staging<B> {
    fn drop(&mut self) {
        if let Some(block) = self.block.take() {
            self.pool.give_back(self.class, block);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::task::Wake;

    /// Memory from the heap: `cap` bytes normally, `hard_cap` when forced.
    struct FakeBacking {
        cap: usize,
        hard_cap: usize,
        used: AtomicUsize,
    }

    impl FakeBacking {
        fn new(cap: usize) -> Self {
            // by default forcing is no more generous than the normal cap
            Self::with_hard_cap(cap, cap)
        }
        fn with_hard_cap(cap: usize, hard_cap: usize) -> Self {
            FakeBacking {
                cap,
                hard_cap,
                used: AtomicUsize::new(0),
            }
        }
    }

    impl StagingBacking for FakeBacking {
        fn alloc_block(&self, len: usize, force: bool) -> Option<RawBlock> {
            let limit = if force { self.hard_cap } else { self.cap };
            if self.used.fetch_add(len, Ordering::SeqCst) + len > limit {
                self.used.fetch_sub(len, Ordering::SeqCst);
                return None;
            }
            let layout = std::alloc::Layout::from_size_align(len, BLOCK_ALIGN).unwrap();
            let ptr = unsafe { std::alloc::alloc(layout) };
            Some(RawBlock { ptr, len, token: 7 })
        }
    }

    struct CountWake(AtomicUsize);
    impl Wake for CountWake {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn waker() -> (Arc<CountWake>, Waker) {
        let c = Arc::new(CountWake(AtomicUsize::new(0)));
        (c.clone(), Waker::from(c))
    }

    fn pool(cap: usize) -> Arc<StagingPool<FakeBacking>> {
        StagingPool::new("test", FakeBacking::new(cap), true, 512, 1 << 20)
    }

    #[test]
    fn size_range_and_disabled() {
        let p = pool(1 << 22);
        assert!(p.acquire(511).is_none());
        assert!(p.acquire(512).is_some());
        assert!(p.acquire(1 << 20).is_some());
        assert!(p.acquire((1 << 20) + 1).is_none());
        assert!(p.acquire(0).is_none());
        let off = StagingPool::new("test", FakeBacking::new(1 << 20), false, 512, 1 << 20);
        assert!(off.acquire(1024).is_none());
    }

    #[test]
    fn blocks_are_recycled() {
        let p = pool(1 << 20);
        let a = p.acquire(1000).unwrap();
        let ptr = a.ptr();
        drop(a);
        let b = p.acquire(1000).unwrap();
        assert_eq!(b.ptr(), ptr);
    }

    #[test]
    fn exhausted_without_holders_falls_back_only_if_forcing_fails_too() {
        let p = pool(1024); // fits exactly one 1 KiB block, and forcing cannot add to it
        let a = p.acquire(1024).unwrap();
        let (_, w) = waker();
        let mut cx = Context::from_waker(&w);
        let mut wait = None;
        // a 2 KiB request: nothing of that class is checked out, so waiting would never end
        assert!(matches!(p.poll_acquire(&mut cx, 2048, &mut wait), Poll::Ready(None)));
        assert_eq!(p.fallbacks(), 1);
        drop(a);
    }

    #[test]
    fn a_class_with_nothing_cached_is_forced_a_block_when_the_room_is_held_by_other_sizes() {
        // 4 KiB of room, all of it taken by cached 512 B blocks; forcing may go to 8 KiB
        let p = StagingPool::new("test", FakeBacking::with_hard_cap(4096, 8192), true, 512, 1 << 20);
        let held: Vec<_> = (0..8).map(|_| p.acquire(512).unwrap()).collect();
        drop(held); // 8 cached 512 B blocks fill the normal cap
        // a 2 KiB request cannot be served normally and has nothing to wait for, so it is forced
        let (_, w) = waker();
        let mut wait = None;
        let Poll::Ready(Some(big)) = p.poll_acquire(&mut Context::from_waker(&w), 2048, &mut wait) else {
            panic!("a block should have been forced")
        };
        assert_eq!(p.fallbacks(), 0);
        // while it is checked out, a second 2 KiB request that cannot be forced again waits for it
        let _more = p.acquire(2048); // forced room allows 4 KiB more of 2 KiB blocks
        drop(big);
    }

    #[test]
    fn waiters_are_served_first_in_first_out() {
        let p = pool(1024);
        let a = p.acquire(1024).unwrap();
        let (cb, wb) = waker();
        let (cc, wc) = waker();
        let (mut wait_b, mut wait_c) = (None, None);
        assert!(p.poll_acquire(&mut Context::from_waker(&wb), 1024, &mut wait_b).is_pending());
        assert!(p.poll_acquire(&mut Context::from_waker(&wc), 1024, &mut wait_c).is_pending());
        let ptr = a.ptr();
        drop(a); // goes straight to b
        assert_eq!(cb.0.load(Ordering::SeqCst), 1);
        assert_eq!(cc.0.load(Ordering::SeqCst), 0);
        let Poll::Ready(Some(b)) = p.poll_acquire(&mut Context::from_waker(&wb), 1024, &mut wait_b) else {
            panic!("b should have its block")
        };
        assert_eq!(b.ptr(), ptr);
        assert!(p.poll_acquire(&mut Context::from_waker(&wc), 1024, &mut wait_c).is_pending());
        drop(b); // now c
        assert_eq!(cc.0.load(Ordering::SeqCst), 1);
        let Poll::Ready(Some(c)) = p.poll_acquire(&mut Context::from_waker(&wc), 1024, &mut wait_c) else {
            panic!("c should have its block")
        };
        drop(c);
        assert_eq!(p.holders[class_of(1024).unwrap().0].load(Ordering::SeqCst), 0);
        assert!(p.acquire(1024).is_some()); // and it went back to the free list
    }

    #[test]
    fn abandoned_waiter_passes_the_block_on() {
        let p = pool(1024);
        let a = p.acquire(1024).unwrap();
        let (_, wb) = waker();
        let (cc, wc) = waker();
        let (mut wait_b, mut wait_c) = (None, None);
        assert!(p.poll_acquire(&mut Context::from_waker(&wb), 1024, &mut wait_b).is_pending());
        assert!(p.poll_acquire(&mut Context::from_waker(&wc), 1024, &mut wait_c).is_pending());
        drop(a); // handed to b
        drop(wait_b); // b is cancelled before claiming it: the block must go to c
        assert_eq!(cc.0.load(Ordering::SeqCst), 1);
        assert!(matches!(
            p.poll_acquire(&mut Context::from_waker(&wc), 1024, &mut wait_c),
            Poll::Ready(Some(_))
        ));
    }

    #[test]
    fn abandoning_before_a_grant_is_harmless() {
        let p = pool(1024);
        let a = p.acquire(1024).unwrap();
        let (_, wb) = waker();
        let mut wait_b = None;
        assert!(p.poll_acquire(&mut Context::from_waker(&wb), 1024, &mut wait_b).is_pending());
        drop(wait_b); // gave up while still queued
        drop(a); // skips the stale entry and frees the block
        assert!(p.acquire(1024).is_some());
    }

    #[test]
    fn losing_the_last_block_sends_waiters_to_their_own_buffers() {
        let p = pool(1024);
        let a = p.acquire(1024).unwrap();
        let (cb, wb) = waker();
        let mut wait_b = None;
        assert!(p.poll_acquire(&mut Context::from_waker(&wb), 1024, &mut wait_b).is_pending());
        a.forget_in_flight(); // the operation holding the only block was dropped mid-flight
        assert_eq!(cb.0.load(Ordering::SeqCst), 1);
        assert!(matches!(
            p.poll_acquire(&mut Context::from_waker(&wb), 1024, &mut wait_b),
            Poll::Ready(None)
        ));
    }

    #[test]
    fn many_threads_contend_and_every_block_comes_back() {
        let p = pool(8 * 4096);
        let done = Arc::new(AtomicUsize::new(0));
        std::thread::scope(|s| {
            for t in 0..8 {
                let p = p.clone();
                let done = done.clone();
                s.spawn(move || {
                    for i in 0..2000 {
                        // blocking-style wait built from poll_acquire
                        let (_, w) = waker();
                        let mut cx = Context::from_waker(&w);
                        let mut wait = None;
                        let staging = loop {
                            match p.poll_acquire(&mut cx, 4096, &mut wait) {
                                Poll::Ready(Some(s)) => break s,
                                Poll::Ready(None) => panic!("thread {t} op {i} fell back"),
                                Poll::Pending => std::thread::yield_now(),
                            }
                        };
                        unsafe { std::ptr::write_bytes(staging.ptr(), t as u8, 4096) };
                        done.fetch_add(1, Ordering::SeqCst);
                        drop(staging);
                    }
                });
            }
        });
        assert_eq!(done.load(Ordering::SeqCst), 16000);
        assert_eq!(p.holders[class_of(4096).unwrap().0].load(Ordering::SeqCst), 0);
    }
}
