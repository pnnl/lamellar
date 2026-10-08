use std::collections::BTreeMap;
use std::os::raw::c_void;

use crate::state::state;

/// Deterministic first-fit allocator over symmetric-heap *offsets*. shmem_malloc is
/// collective and called in the same order on every PE, so identical state on every
/// PE yields identical offsets with no communication.
pub(crate) struct HeapAlloc {
    free: BTreeMap<usize, usize>, // offset -> len
    used: BTreeMap<usize, usize>, // offset -> len
}

impl HeapAlloc {
    pub fn new(len: usize) -> Self {
        let mut free = BTreeMap::new();
        free.insert(0, len);
        Self {
            free,
            used: BTreeMap::new(),
        }
    }

    pub fn alloc(&mut self, size: usize, align: usize) -> Option<usize> {
        let align = align.max(16).next_power_of_two();
        let size = size.max(1).checked_add(align - 1)? & !(align - 1);
        let (&off, &len) = self.free.iter().find(|(&o, &l)| {
            let a = (o + align - 1) & !(align - 1);
            a - o + size <= l
        })?;
        let a = (off + align - 1) & !(align - 1);
        self.free.remove(&off);
        if a > off {
            self.free.insert(off, a - off);
        }
        let end = a + size;
        if off + len > end {
            self.free.insert(end, off + len - end);
        }
        self.used.insert(a, size);
        Some(a)
    }

    pub fn free(&mut self, off: usize) -> Option<usize> {
        let len = self.used.remove(&off)?;
        self.free.insert(off, len);
        // coalesce with next
        if let Some(&nl) = self.free.get(&(off + len)) {
            self.free.remove(&(off + len));
            *self.free.get_mut(&off).unwrap() += nl;
        }
        // coalesce with previous
        if let Some((&po, &pl)) = self.free.range(..off).next_back() {
            if po + pl == off {
                let l = self.free.remove(&off).unwrap();
                *self.free.get_mut(&po).unwrap() += l;
            }
        }
        Some(len)
    }

    pub fn size_of(&self, off: usize) -> Option<usize> {
        self.used.get(&off).copied()
    }
}

/// Offset of `p` within the user symmetric heap, if it lies inside it.
pub(crate) fn offset_of(p: *const u8) -> Option<usize> {
    let s = state();
    let a = p as usize;
    (a >= s.base && a < s.base + s.user_len).then(|| a - s.base)
}

/// Offset of `p` within the registered data segment (`.data`/`.bss` of the main program).
pub(crate) fn data_off(p: *const u8) -> Option<usize> {
    let s = state();
    let a = p as usize;
    (s.data.is_some() && a >= s.data_base && a < s.data_base + s.data_len).then(|| a - s.data_base)
}

pub(crate) fn ptr_at(off: usize) -> *mut u8 {
    (state().base + off) as *mut u8
}

#[no_mangle]
pub extern "C" fn shmem_malloc(size: usize) -> *mut c_void {
    shmem_align(16, size)
}

#[no_mangle]
pub extern "C" fn shmem_align(alignment: usize, size: usize) -> *mut c_void {
    // Collective: sync so PEs agree the call happened (spec requires a barrier on exit).
    let r = state()
        .alloc
        .lock()
        .alloc(size, alignment)
        .map_or(std::ptr::null_mut(), |o| ptr_at(o) as *mut c_void);
    state().team.barrier();
    r
}

#[no_mangle]
pub unsafe extern "C" fn shmem_calloc(count: usize, size: usize) -> *mut c_void {
    let Some(n) = count.checked_mul(size) else {
        return std::ptr::null_mut();
    };
    let p = shmem_malloc(n);
    if !p.is_null() {
        std::ptr::write_bytes(p as *mut u8, 0, n);
        state().team.barrier();
    }
    p
}

#[no_mangle]
pub unsafe extern "C" fn shmem_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    if let Some(off) = offset_of(ptr as *const u8) {
        state().alloc.lock().free(off);
    }
    state().team.barrier();
}

#[no_mangle]
pub unsafe extern "C" fn shmem_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return shmem_malloc(size);
    }
    if size == 0 {
        shmem_free(ptr);
        return std::ptr::null_mut();
    }
    let old = offset_of(ptr as *const u8)
        .and_then(|o| state().alloc.lock().size_of(o))
        .unwrap_or(0);
    let new = shmem_malloc(size);
    if !new.is_null() {
        std::ptr::copy_nonoverlapping(ptr as *const u8, new as *mut u8, old.min(size));
        shmem_free(ptr);
    }
    new
}

#[no_mangle]
pub extern "C" fn shmalloc(size: usize) -> *mut c_void {
    shmem_malloc(size)
}
#[no_mangle]
pub unsafe extern "C" fn shfree(ptr: *mut c_void) {
    shmem_free(ptr)
}
#[no_mangle]
pub unsafe extern "C" fn shrealloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    shmem_realloc(ptr, size)
}
#[no_mangle]
pub extern "C" fn shmemalign(alignment: usize, size: usize) -> *mut c_void {
    shmem_align(alignment, size)
}

/// shmem_ptr: direct load/store address for a remote symmetric object. Only the
/// calling PE itself is directly addressable in Phase A.
#[no_mangle]
pub extern "C" fn shmem_ptr(dest: *const c_void, pe: std::os::raw::c_int) -> *mut c_void {
    if pe as usize == state().my_pe && offset_of(dest as *const u8).is_some() {
        dest as *mut c_void
    } else {
        std::ptr::null_mut()
    }
}
