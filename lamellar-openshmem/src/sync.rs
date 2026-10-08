use std::os::raw::{c_int, c_long};
use std::sync::atomic::{fence, AtomicU64, Ordering};

use crate::rma::put_off;
use crate::state::{progress, state, wait_for, CollState};

/// A legacy OpenSHMEM active set: PEs `start + i * 2^logstride` for `i in 0..size`.
#[derive(Clone, Copy)]
pub(crate) struct ActiveSet {
    start: usize,
    stride: usize,
    pub size: usize,
}

impl ActiveSet {
    pub fn new(start: c_int, logstride: c_int, size: c_int) -> Self {
        let s = state();
        if start < 0 || logstride < 0 || size < 1 || logstride > 30 {
            eprintln!("shmem: invalid active set ({start}, {logstride}, {size})");
            std::process::abort();
        }
        let set = Self {
            start: start as usize,
            stride: 1usize << logstride,
            size: size as usize,
        };
        if set.pe(set.size - 1) >= s.num_pes {
            eprintln!("shmem: active set ({start}, {logstride}, {size}) exceeds {} PEs", s.num_pes);
            std::process::abort();
        }
        set
    }

    pub fn pe(&self, idx: usize) -> usize {
        self.start + idx * self.stride
    }

    /// Index of the calling PE within the set.
    pub fn my_index(&self) -> usize {
        let me = state().my_pe;
        if me >= self.start && (me - self.start) % self.stride == 0 && (me - self.start) / self.stride < self.size {
            (me - self.start) / self.stride
        } else {
            eprintln!("shmem: PE {me} called a collective on an active set it is not part of");
            std::process::abort();
        }
    }
}

fn slot_ptr(sender: usize) -> *const AtomicU64 {
    let s = state();
    (s.base + s.meta_off + 8 * sender) as *const AtomicU64
}

/// Dissemination barrier over `set`, synchronizing with per-sender monotonic counters rather
/// than the user's `pSync`: every message is "I have now sent you N messages", the receiver
/// waits for N >= what it expects. Monotonic values make interleaved barriers on overlapping
/// active sets safe, and the user's pSync may be any memory (including globals).
pub(crate) unsafe fn barrier_locked(st: &mut CollState, set: &ActiveSet) {
    let s = state();
    let me = s.my_pe;
    let i = set.my_index();
    let n = set.size;
    let mut d = 1;
    while d < n {
        let dest = set.pe((i + d) % n);
        let src = set.pe((i + n - d) % n);
        st.sent[dest] += 1;
        let v = st.sent[dest];
        put_off(s.meta_off + 8 * me, &v as *const u64 as *const u8, 8, dest);
        st.recvd[src] += 1;
        let want = st.recvd[src];
        let slot = &*slot_ptr(src);
        wait_for(|| slot.load(Ordering::Acquire) >= want);
        d <<= 1;
    }
}

/// Run whole-team `shmem_barrier` / `shmem_sync` on the runtime's barrier instead of the shim's dissemination barrier
/// below, on every backend. Measured 1.06-1.43x slower than that dissemination barrier on libfabric-sys-opt and
/// ucx-opt (a runtime barrier improvement to make, not a reason to keep a shim copy). `SHMEM_RUNTIME_BARRIER=0` forces
/// the shim barrier.
fn runtime_barrier_for(set: &ActiveSet) -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let on = *ON.get_or_init(|| std::env::var("SHMEM_RUNTIME_BARRIER").as_deref() != Ok("0"));
    on && set.size == state().num_pes && set.pe(0) == 0 && (set.size == 1 || set.pe(1) == 1)
}

/// Barrier including a quiet, as `shmem_barrier` requires.
#[no_mangle]
pub unsafe extern "C" fn shmem_barrier(start: c_int, logstride: c_int, size: c_int, _psync: *mut c_long) {
    let set = ActiveSet::new(start, logstride, size);
    crate::state::shmem_quiet();
    if runtime_barrier_for(&set) {
        state().team.barrier();
        return;
    }
    let mut st = state().coll.lock();
    barrier_locked(&mut st, &set);
}

/// Sync without the completion guarantee of a barrier (orders only prior local loads/stores).
#[no_mangle]
pub unsafe extern "C" fn shmem_sync(start: c_int, logstride: c_int, size: c_int, _psync: *mut c_long) {
    let set = ActiveSet::new(start, logstride, size);
    if runtime_barrier_for(&set) {
        state().team.barrier();
        return;
    }
    let mut st = state().coll.lock();
    barrier_locked(&mut st, &set);
}

// --- point-to-point synchronization -----------------------------------------

pub const CMP_EQ: c_int = 0;
pub const CMP_NE: c_int = 1;
pub const CMP_GT: c_int = 2;
pub const CMP_LE: c_int = 3;
pub const CMP_LT: c_int = 4;
pub const CMP_GE: c_int = 5;

fn cmp_ok<T: PartialOrd>(a: T, cmp: c_int, b: T) -> bool {
    match cmp {
        CMP_EQ => a == b,
        CMP_NE => a != b,
        CMP_GT => a > b,
        CMP_LE => a <= b,
        CMP_LT => a < b,
        CMP_GE => a >= b,
        _ => {
            eprintln!("shmem: invalid comparison operator {cmp}");
            std::process::abort();
        }
    }
}

unsafe fn wait_cond<T: PartialOrd + Copy>(ivar: *const T, cmp: c_int, v: T) {
    wait_for(|| cmp_ok(std::ptr::read_volatile(ivar), cmp, v));
    fence(Ordering::Acquire);
}

macro_rules! wait_types {
    ($( $name:ident : $t:ty ),* $(,)?) => { $( paste::paste! {
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _wait_until>](ivar: *mut $t, cmp: c_int, v: $t) {
            wait_cond(ivar, cmp, v)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _test>](ivar: *mut $t, cmp: c_int, v: $t) -> c_int {
            let cur = std::ptr::read_volatile(ivar);
            let r = cmp_ok(cur, cmp, v);
            if r { fence(Ordering::Acquire); } else { progress(); } // callers poll this in a loop
            r as c_int
        }
    } )* };
}

wait_types! {
    short: i16, int: i32, long: std::os::raw::c_long, longlong: i64,
    ushort: u16, uint: u32, ulong: std::os::raw::c_ulong, ulonglong: u64,
    int32: i32, int64: i64, uint32: u32, uint64: u64, size: usize, ptrdiff: isize,
}

/// Legacy `shmem_TYPE_wait(ivar, v)`: wait until `*ivar != v`.
macro_rules! wait_legacy {
    ($( $name:ident : $t:ty ),* $(,)?) => { $( paste::paste! {
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _wait>](ivar: *mut $t, v: $t) {
            wait_cond(ivar, CMP_NE, v)
        }
    } )* };
}
wait_legacy! { short: i16, int: i32, long: std::os::raw::c_long, longlong: i64 }
