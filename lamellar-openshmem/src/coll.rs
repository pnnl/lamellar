//! Legacy (1.4) active-set collectives. All are pull based: after an entry barrier every PE
//! reads what it needs from the other members' symmetric source buffers, so no PE ever
//! writes into memory another PE might still be reading. `pSync`/`pWrk` are accepted and
//! unused (synchronization uses internal counters, see `sync.rs`).
//!
//! Where the runtime has the collective (broadcast and fcollect over the whole team, data in the
//! symmetric heap) the call is handed to it (UCC on the ucx backend); the pull-based versions
//! below are the fallback for sub-sets, static/global sources and `collect`.

use std::os::raw::{c_int, c_long, c_void};

use lamellar::ffi::RawReduceOp;
use lamellar::BroadcastInput;

use crate::heap::offset_of;
use crate::rma::{get_bytes, get_off};
use crate::state::state;
use crate::sync::{barrier_locked, ActiveSet};

/// Heap index of `source` if this call can be handed to the runtime's collectives: the active set is the
/// whole team in PE order and the data is in the symmetric heap. Every PE must reach the same answer; that
/// holds for conforming programs, where `source` is a symmetric object (same location class on all PEs).
fn runtime_path(set: &ActiveSet, source: *const c_void) -> Option<usize> {
    let world = set.size == state().num_pes && set.pe(0) == 0 && (set.size == 1 || set.pe(1) == 1);
    if world {
        offset_of(source as *const u8)
    } else {
        None
    }
}

#[no_mangle]
pub unsafe extern "C" fn shmem_broadcast64(
    target: *mut c_void, source: *const c_void, nlong: usize,
    root: c_int, start: c_int, logstride: c_int, size: c_int, _psync: *mut c_long,
) {
    bcast(target, source, nlong * 8, root, start, logstride, size)
}

#[no_mangle]
pub unsafe extern "C" fn shmem_broadcast32(
    target: *mut c_void, source: *const c_void, nlong: usize,
    root: c_int, start: c_int, logstride: c_int, size: c_int, _psync: *mut c_long,
) {
    bcast(target, source, nlong * 4, root, start, logstride, size)
}

unsafe fn bcast(target: *mut c_void, source: *const c_void, nbytes: usize, root: c_int, start: c_int, lg: c_int, size: c_int) {
    let set = ActiveSet::new(start, lg, size);
    if root < 0 || root as usize >= set.size {
        eprintln!("shmem: broadcast root {root} is outside the active set");
        std::process::abort();
    }
    let me = set.my_index();
    let mut st = state().coll.lock();
    if let Some(idx) = runtime_path(&set, source).filter(|_| nbytes > 0) {
        let input = if me == root as usize {
            BroadcastInput::Root(idx)
        } else {
            BroadcastInput::NotRoot(root as usize)
        };
        // the root's own target is not updated (per spec): only non-roots get data back
        if let Some(v) = (*state().region).broadcast_from_pe(input, nbytes).block() {
            std::ptr::copy_nonoverlapping(v.as_ptr(), target as *mut u8, nbytes);
        }
        return;
    }
    barrier_locked(&mut st, &set);
    // The root's own target is not updated (per spec).
    if me != root as usize {
        get_bytes(target as *mut u8, source as *const u8, nbytes, set.pe(root as usize));
    }
    barrier_locked(&mut st, &set);
}

#[no_mangle]
pub unsafe extern "C" fn shmem_fcollect64(
    target: *mut c_void, source: *const c_void, nlong: usize,
    start: c_int, logstride: c_int, size: c_int, _psync: *mut c_long,
) {
    fcollect(target, source, nlong * 8, start, logstride, size)
}

#[no_mangle]
pub unsafe extern "C" fn shmem_fcollect32(
    target: *mut c_void, source: *const c_void, nlong: usize,
    start: c_int, logstride: c_int, size: c_int, _psync: *mut c_long,
) {
    fcollect(target, source, nlong * 4, start, logstride, size)
}

unsafe fn fcollect(target: *mut c_void, source: *const c_void, nbytes: usize, start: c_int, lg: c_int, size: c_int) {
    let set = ActiveSet::new(start, lg, size);
    let mut st = state().coll.lock();
    if let Some(idx) = runtime_path(&set, source).filter(|_| nbytes > 0) {
        let v = (*state().region).gather_all(idx, nbytes).block();
        std::ptr::copy_nonoverlapping(v.as_ptr(), target as *mut u8, set.size * nbytes);
        return;
    }
    barrier_locked(&mut st, &set);
    for j in 0..set.size {
        get_bytes((target as *mut u8).add(j * nbytes), source as *const u8, nbytes, set.pe(j));
    }
    barrier_locked(&mut st, &set);
}

#[no_mangle]
pub unsafe extern "C" fn shmem_collect64(
    target: *mut c_void, source: *const c_void, nlong: usize,
    start: c_int, logstride: c_int, size: c_int, _psync: *mut c_long,
) {
    collect(target, source, nlong * 8, start, logstride, size)
}

#[no_mangle]
pub unsafe extern "C" fn shmem_collect32(
    target: *mut c_void, source: *const c_void, nlong: usize,
    start: c_int, logstride: c_int, size: c_int, _psync: *mut c_long,
) {
    collect(target, source, nlong * 4, start, logstride, size)
}

/// Each PE contributes a different number of elements; publish the byte count in the
/// metadata scratch word, read everyone's, then place data at the running offset.
unsafe fn collect(target: *mut c_void, source: *const c_void, nbytes: usize, start: c_int, lg: c_int, size: c_int) {
    let s = state();
    let set = ActiveSet::new(start, lg, size);
    let scratch = s.meta_off + 8 * s.num_pes;
    let mut st = s.coll.lock();
    *((s.base + scratch) as *mut u64) = nbytes as u64;
    barrier_locked(&mut st, &set);
    let mut off = 0usize;
    for j in 0..set.size {
        let mut cnt = 0u64;
        get_off(scratch, &mut cnt as *mut u64 as *mut u8, 8, set.pe(j));
        get_bytes((target as *mut u8).add(off), source as *const u8, cnt as usize, set.pe(j));
        off += cnt as usize;
    }
    barrier_locked(&mut st, &set);
}

/// Reduction over the active set. Values are folded in member order so every PE computes a
/// bit-identical result (matters for floating point). The second barrier precedes writing
/// `dest`, so `dest` may alias `source`.
unsafe fn reduce<T: Copy + lamellar::Remote>(
    dest: *mut T, source: *const T, n: usize, set: &ActiveSet, rop: RawReduceOp, f: impl Fn(T, T) -> T,
) {
    // Whole team, symmetric-heap source: hand it to the runtime's allreduce (SHMEM_RUNTIME_REDUCE=1).
    if runtime_reduce_enabled() {
        if let Some(idx) = runtime_path(set, source as *const c_void).filter(|_| n > 0) {
            if let Some(v) = state().raw.allreduce::<T>(idx, n, rop) {
                std::ptr::copy_nonoverlapping(v.as_ptr(), dest, n);
                return;
            }
        }
    }
    let mut st = state().coll.lock();
    barrier_locked(&mut st, set);
    let bytes = n * std::mem::size_of::<T>();
    let mut acc: Vec<T> = Vec::with_capacity(n);
    let mut tmp: Vec<T> = Vec::with_capacity(n);
    for j in 0..set.size {
        get_bytes(tmp.as_mut_ptr() as *mut u8, source as *const u8, bytes, set.pe(j));
        tmp.set_len(n);
        if j == 0 {
            acc.extend_from_slice(&tmp);
        } else {
            for k in 0..n {
                acc[k] = f(acc[k], tmp[k]);
            }
        }
    }
    barrier_locked(&mut st, set);
    std::ptr::copy_nonoverlapping(acc.as_ptr(), dest, n);
}

/// Route whole-team `*_to_all` reductions through the runtime's allreduce instead of the pull-based fold below, on
/// every backend. Measured on libfabric-sys-opt (4-9x faster) and ucx-opt (5-21x), with every reduction op checked
/// against a local result at 4/16/64 PEs. `SHMEM_RUNTIME_REDUCE=0` forces the fold.
fn runtime_reduce_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("SHMEM_RUNTIME_REDUCE").as_deref() != Ok("0"))
}

macro_rules! rop {
    (and) => { RawReduceOp::And };
    (or) => { RawReduceOp::Or };
    (xor) => { RawReduceOp::Xor };
    (max) => { RawReduceOp::Max };
    (min) => { RawReduceOp::Min };
    (sum) => { RawReduceOp::Sum };
    (prod) => { RawReduceOp::Prod };
}

macro_rules! to_all {
    ($name:ident $t:ty : $op:ident $f:expr) => { paste::paste! {
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _ $op _to_all>](
            dest: *mut $t, source: *const $t, nreduce: c_int,
            start: c_int, logstride: c_int, size: c_int, _pwrk: *mut $t, _psync: *mut c_long,
        ) {
            if nreduce <= 0 { return; }
            reduce(dest, source, nreduce as usize, &ActiveSet::new(start, logstride, size), rop!($op), $f)
        }
    } };
}

macro_rules! to_all_ints {
    ($( $n:ident : $t:ty ),* $(,)?) => { $(
        to_all!($n $t: and |a, b| a & b);
        to_all!($n $t: or |a, b| a | b);
        to_all!($n $t: xor |a, b| a ^ b);
        to_all!($n $t: max |a, b| if b > a { b } else { a });
        to_all!($n $t: min |a, b| if b < a { b } else { a });
        to_all!($n $t: sum |a, b| a.wrapping_add(b));
        to_all!($n $t: prod |a, b| a.wrapping_mul(b));
    )* };
}
to_all_ints! { short: i16, int: i32, long: std::os::raw::c_long, longlong: i64 }

macro_rules! to_all_floats {
    ($( $n:ident : $t:ty ),* $(,)?) => { $(
        to_all!($n $t: max |a, b| if b > a { b } else { a });
        to_all!($n $t: min |a, b| if b < a { b } else { a });
        to_all!($n $t: sum |a, b| a + b);
        to_all!($n $t: prod |a, b| a * b);
    )* };
}
to_all_floats! { float: f32, double: f64 }
