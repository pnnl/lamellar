//! The UCX side of the shared staging pool (`super::staging`), used by the `ucx` and `ucx-opt` backends.
//!
//! The pool hands out blocks carved from slabs of registered memory. Each slab is one `ucp_mem_map` allocation, so
//! its `ucp_mem_h` covers every block inside it; that handle is carried in the block's token and passed in the
//! request (`UCP_OP_ATTR_FIELD_MEMH`), which lets UCX use the memory zero-copy without registering anything or copying
//! through its own bounce buffers.
//!
//! The slabs belong to the UCP context they were mapped on and must be unmapped before it is cleaned up, so each
//! backend's `Context` owns its pool and releases it in its `Drop`.

use std::mem::MaybeUninit;

use lamellar_ucx_sys::*;
use parking_lot::Mutex;

use super::staging::{
    RawBlock, Staging, StagingBacking, StagingPool, StagingWait, BLOCK_ALIGN,
};

/// Slabs are mapped this big (or bigger, for a block that needs it) until the pool's capacity is reached.
const SLAB_BYTES: usize = 16 << 20;
/// A slab mapped past the capacity for a starved size class is no bigger than this (or one block).
const FORCED_SLAB_BYTES: usize = 4 << 20;

struct Slab {
    memh: ucp_mem_h,
    base: usize,
    len: usize,
    used: usize,
}

struct SlabSet {
    slabs: Vec<Slab>,
    /// Total bytes mapped.
    total: usize,
    released: bool,
}

/// Registered memory for the staging pool: slabs mapped on one UCP context.
pub(crate) struct UcxSlabBacking {
    context: ucp_context_h,
    cap: usize,
    inner: Mutex<SlabSet>,
}

// `ucp_context_h` and `ucp_mem_h` are opaque handles that UCP documents as thread safe.
unsafe impl Send for UcxSlabBacking {}
unsafe impl Sync for UcxSlabBacking {}

impl UcxSlabBacking {
    /// `cap` is the most memory, in bytes, the slabs may add up to.
    pub(crate) fn new(context: ucp_context_h, cap: usize) -> Self {
        UcxSlabBacking {
            context,
            cap,
            inner: Mutex::new(SlabSet {
                slabs: Vec::new(),
                total: 0,
                released: false,
            }),
        }
    }

    /// The memory handle covering a block, from its token.
    #[inline]
    pub(crate) fn memh(token: usize) -> ucp_mem_h {
        token as ucp_mem_h
    }

    /// Unmap every slab. Must run before the context is cleaned up; running it twice is harmless. No block may be in
    /// use by an operation when it runs.
    pub(crate) fn release_all(&self) {
        let mut set = self.inner.lock();
        set.released = true;
        for slab in set.slabs.drain(..) {
            unsafe { ucp_mem_unmap(self.context, slab.memh) };
        }
        set.total = 0;
    }

    /// Map a new slab of `len` bytes.
    fn map_slab(&self, len: usize) -> Option<Slab> {
        let params = ucp_mem_map_params_t {
            field_mask: (ucp_mem_map_params_field::UCP_MEM_MAP_PARAM_FIELD_LENGTH
                | ucp_mem_map_params_field::UCP_MEM_MAP_PARAM_FIELD_FLAGS)
                .0 as u64,
            address: std::ptr::null_mut(),
            length: len as _,
            flags: UCP_MEM_MAP_ALLOCATE as _,
            prot: 0,
            memory_type: ucs_memory_type::UCS_MEMORY_TYPE_HOST,
            exported_memh_buffer: std::ptr::null_mut(),
        };
        let mut handle = MaybeUninit::uninit();
        let status = unsafe { ucp_mem_map(self.context, &params, handle.as_mut_ptr()) };
        if status != ucs_status_t::UCS_OK {
            return None;
        }
        let memh = unsafe { handle.assume_init() };
        let mut attr = ucp_mem_attr_t {
            field_mask: (ucp_mem_attr_field::UCP_MEM_ATTR_FIELD_ADDRESS
                | ucp_mem_attr_field::UCP_MEM_ATTR_FIELD_LENGTH)
                .0 as u64,
            length: len as _,
            address: std::ptr::null_mut(),
            mem_type: ucs_memory_type::UCS_MEMORY_TYPE_HOST,
        };
        let status = unsafe { ucp_mem_query(memh, &mut attr) };
        if status != ucs_status_t::UCS_OK {
            unsafe { ucp_mem_unmap(self.context, memh) };
            return None;
        }
        Some(Slab {
            memh,
            base: attr.address as usize,
            len: attr.length as usize,
            used: 0,
        })
    }
}

impl StagingBacking for UcxSlabBacking {
    fn alloc_block(&self, len: usize, force: bool) -> Option<RawBlock> {
        debug_assert!(len.is_power_of_two() && len >= BLOCK_ALIGN);
        let mut set = self.inner.lock();
        if set.released {
            return None;
        }
        // Bump allocation from the newest slab. Blocks are multiples of BLOCK_ALIGN, so offsets stay aligned.
        if let Some(slab) = set.slabs.last_mut() {
            if slab.used + len <= slab.len {
                let ptr = (slab.base + slab.used) as *mut u8;
                slab.used += len;
                return Some(RawBlock {
                    ptr,
                    len,
                    token: slab.memh as usize,
                });
            }
        }
        // The newest slab is full: map another, as big as the capacity left allows. A forced block (its size class
        // has nothing cached or checked out) goes past the capacity, with a modest slab, so that no size class can be
        // starved by room held as cached blocks of other sizes.
        let mut slab_len = SLAB_BYTES.max(len);
        if set.total + slab_len > self.cap {
            slab_len = len;
            if force {
                slab_len = (len * 64).clamp(len, FORCED_SLAB_BYTES.max(len));
            } else if set.total + slab_len > self.cap {
                return None;
            }
        }
        let mut slab = self.map_slab(slab_len)?;
        let ptr = slab.base as *mut u8;
        slab.used = len;
        let token = slab.memh as usize;
        set.total += slab.len;
        set.slabs.push(slab);
        Some(RawBlock { ptr, len, token })
    }
}

impl Drop for UcxSlabBacking {
    fn drop(&mut self) {
        self.release_all();
    }
}

pub(crate) type UcxStagingPool = StagingPool<UcxSlabBacking>;
pub(crate) type UcxStaging = Staging<UcxSlabBacking>;
pub(crate) type UcxStagingWait = StagingWait<UcxSlabBacking>;

/// A get whose data is landing in a staging block and still has to be copied into the caller's buffer.
///
/// `finish` must be called once the get has completed. Dropping it before then (a cancelled task) means the NIC may
/// still be writing the block, so the block is withheld from the pool instead of being recycled.
pub(crate) struct StagedGet<T> {
    staging: Option<UcxStaging>,
    dst: *mut T,
    len: usize,
}

// The destination pointer is into a buffer the owning future keeps alive and does not touch until `finish`.
unsafe impl<T: Send> Send for StagedGet<T> {}

impl<T: Copy> StagedGet<T> {
    pub(crate) fn new(staging: UcxStaging, dst: *mut T, len: usize) -> Self {
        StagedGet {
            staging: Some(staging),
            dst,
            len,
        }
    }

    /// The get has completed: copy the fetched elements into the caller's buffer and release the block.
    pub(crate) fn finish(mut self) {
        if let Some(staging) = self.staging.take() {
            unsafe {
                std::ptr::copy_nonoverlapping(staging.ptr() as *const T, self.dst, self.len);
            }
        }
    }
}

impl<T> Drop for StagedGet<T> {
    fn drop(&mut self) {
        if let Some(staging) = self.staging.take() {
            staging.forget_in_flight();
        }
    }
}

/// What a staged get does when the staging pool has no block for the operation.
pub(crate) enum GetWait<'a, 'b> {
    /// The caller cannot wait (a blocking call): use the caller's own buffer.
    Fallback,
    /// Do not issue: the caller will try again from `poll`, where it can wait. For `spawn`.
    TryOnly,
    /// Queue for a block; the task is woken when one is handed over. `slot` keeps the place in the queue.
    Wait(&'a mut std::task::Context<'b>, &'a mut Option<UcxStagingWait>),
}
