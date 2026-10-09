//! Atomic memory operations and locks.
//!
//! The local PE applies a `std::sync::atomic` operation directly; remote PEs use the
//! backend's atomics via `the runtime's public region API` (NIC atomics where the fabric has them).

use std::os::raw::{c_int, c_long, c_void};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::SeqCst};

use crate::rma::sym_off;
use crate::state::{progress, shmem_quiet, state, DATA_FLAG};
use crate::sym::AtomicKind;

const OP_FETCH: u8 = 0;
const OP_SET: u8 = 1;
const OP_SWAP: u8 = 2;
const OP_CSWAP: u8 = 3;
const OP_ADD: u8 = 4;
const OP_AND: u8 = 5;
const OP_OR: u8 = 6;
const OP_XOR: u8 = 7;

/// Apply `op` atomically to the `size`-byte integer at `ptr`; returns the previous value
/// (zero-extended). `a` is the operand (or the compare value), `b` the swap-in for CSWAP.
unsafe fn apply(ptr: *mut u8, size: u8, op: u8, a: u64, b: u64) -> u64 {
    macro_rules! go {
        ($A:ty, $T:ty) => {{
            let at = <$A>::from_ptr(ptr as *mut $T);
            let (a, b) = (a as $T, b as $T);
            (match op {
                OP_FETCH => at.load(SeqCst),
                OP_SET => {
                    at.store(a, SeqCst);
                    0
                }
                OP_SWAP => at.swap(a, SeqCst),
                OP_CSWAP => match at.compare_exchange(a, b, SeqCst, SeqCst) {
                    Ok(v) | Err(v) => v,
                },
                OP_ADD => at.fetch_add(a, SeqCst),
                OP_AND => at.fetch_and(a, SeqCst),
                OP_OR => at.fetch_or(a, SeqCst),
                OP_XOR => at.fetch_xor(a, SeqCst),
                _ => unreachable!(),
            }) as u64
        }};
    }
    if size == 4 {
        go!(AtomicU32, u32)
    } else {
        go!(AtomicU64, u64)
    }
}

/// Whether the owner's own AMOs go through the network path (see `amo`). `SHMEM_LOCAL_AMO=net|cpu`
/// forces it; otherwise it follows the backend: UCX uses NIC atomics (not atomic against CPU atomics
/// on ATOMIC_HCA devices), libfabric's `rxm` layer executes atomics on the target CPU, so a CPU atomic
/// on the owner is in the same domain and cheaper.
fn local_amo_via_net() -> bool {
    static NET: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *NET.get_or_init(|| match std::env::var("SHMEM_LOCAL_AMO").as_deref() {
        Ok("net") => true,
        Ok("cpu") => false,
        _ => match std::env::var("LAMELLAR_BACKEND").ok().as_deref().map(str::to_ascii_lowercase).as_deref() {
            Some("ucx") | Some("ucx-opt") | Some("ucx_opt") => true,
            Some(_) => false,
            // backend chosen by the build: a UCX-only build runs UCX
            None => (cfg!(feature = "ucx") || cfg!(feature = "ucx-opt")) && !cfg!(feature = "libfabric"),
        },
    })
}

/// Perform an AMO on symmetric address `target` on `pe`. The local PE applies a CPU atomic
/// directly; remote PEs use the backend's atomics through the heap's region views.
unsafe fn amo(pe: usize, target: *mut u8, size: u8, op: u8, a: u64, b: u64) -> u64 {
    let s = state();
    let off = sym_off("atomic", target);
    // Atomics on the owner's own memory: a CPU atomic is only atomic against other CPU accesses. Peers'
    // atomics go through the NIC, and on ATOMIC_HCA devices those are not atomic against the CPU, so
    // mixing the two loses updates (seen as broken locks / lost compare-and-swaps). SHMEM_LOCAL_AMO=net
    // sends the owner's AMOs through the same network path as everyone else's.
    if pe == s.my_pe && !local_amo_via_net() {
        return apply(target, size, op, a, b);
    }
    let rop = match op {
        OP_FETCH => AtomicKind::Fetch,
        OP_SET => AtomicKind::Set,
        OP_SWAP => AtomicKind::Swap,
        OP_CSWAP => AtomicKind::CompareSwap,
        OP_ADD => AtomicKind::Add,
        OP_AND => AtomicKind::And,
        OP_OR => AtomicKind::Or,
        OP_XOR => AtomicKind::Xor,
        _ => unreachable!(),
    };
    if off & DATA_FLAG != 0 {
        let d = s.data.as_ref().unwrap();
        let o = off & !DATA_FLAG;
        return if size == 4 {
            d.atomic_u32(pe, o, rop, a as u32, b as u32) as u64
        } else {
            d.atomic_u64(pe, o, rop, a, b)
        };
    }
    if size == 4 {
        s.heap.atomic_u32(pe, off, rop, a as u32, b as u32) as u64
    } else {
        s.heap.atomic_u64(pe, off, rop, a, b)
    }
}

macro_rules! amo_types {
    ($( $name:ident : $t:ty ),* $(,)?) => { $( paste::paste! {
        const [<SZ_ $name:upper>]: u8 = std::mem::size_of::<$t>() as u8;

        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch>](src: *const $t, pe: c_int) -> $t {
            amo(pe as usize, src as *mut u8, [<SZ_ $name:upper>], OP_FETCH, 0, 0) as $t
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_set>](dst: *mut $t, v: $t, pe: c_int) {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_SET, v as u64, 0);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_swap>](dst: *mut $t, v: $t, pe: c_int) -> $t {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_SWAP, v as u64, 0) as $t
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_compare_swap>](dst: *mut $t, cond: $t, v: $t, pe: c_int) -> $t {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_CSWAP, cond as u64, v as u64) as $t
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_add>](dst: *mut $t, v: $t, pe: c_int) -> $t {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_ADD, v as u64, 0) as $t
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_add>](dst: *mut $t, v: $t, pe: c_int) {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_ADD, v as u64, 0);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_inc>](dst: *mut $t, pe: c_int) -> $t {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_ADD, 1, 0) as $t
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_inc>](dst: *mut $t, pe: c_int) {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_ADD, 1, 0);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_and>](dst: *mut $t, v: $t, pe: c_int) -> $t {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_AND, v as u64, 0) as $t
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_and>](dst: *mut $t, v: $t, pe: c_int) {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_AND, v as u64, 0);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_or>](dst: *mut $t, v: $t, pe: c_int) -> $t {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_OR, v as u64, 0) as $t
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_or>](dst: *mut $t, v: $t, pe: c_int) {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_OR, v as u64, 0);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_xor>](dst: *mut $t, v: $t, pe: c_int) -> $t {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_XOR, v as u64, 0) as $t
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_xor>](dst: *mut $t, v: $t, pe: c_int) {
            amo(pe as usize, dst as *mut u8, [<SZ_ $name:upper>], OP_XOR, v as u64, 0);
        }

        // Non-blocking fetching forms complete eagerly, which is conforming (they must
        // complete by the next quiet).
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_nbi>](out: *mut $t, src: *const $t, pe: c_int) {
            *out = [<shmem_ $name _atomic_fetch>](src, pe);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_swap_nbi>](out: *mut $t, dst: *mut $t, v: $t, pe: c_int) {
            *out = [<shmem_ $name _atomic_swap>](dst, v, pe);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_compare_swap_nbi>](out: *mut $t, dst: *mut $t, cond: $t, v: $t, pe: c_int) {
            *out = [<shmem_ $name _atomic_compare_swap>](dst, cond, v, pe);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_add_nbi>](out: *mut $t, dst: *mut $t, v: $t, pe: c_int) {
            *out = [<shmem_ $name _atomic_fetch_add>](dst, v, pe);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_inc_nbi>](out: *mut $t, dst: *mut $t, pe: c_int) {
            *out = [<shmem_ $name _atomic_fetch_inc>](dst, pe);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_and_nbi>](out: *mut $t, dst: *mut $t, v: $t, pe: c_int) {
            *out = [<shmem_ $name _atomic_fetch_and>](dst, v, pe);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_or_nbi>](out: *mut $t, dst: *mut $t, v: $t, pe: c_int) {
            *out = [<shmem_ $name _atomic_fetch_or>](dst, v, pe);
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _atomic_fetch_xor_nbi>](out: *mut $t, dst: *mut $t, v: $t, pe: c_int) {
            *out = [<shmem_ $name _atomic_fetch_xor>](dst, v, pe);
        }
    } )* };
}

amo_types! {
    int: i32, long: std::os::raw::c_long, longlong: i64,
    uint: u32, ulong: std::os::raw::c_ulong, ulonglong: u64,
    int32: i32, int64: i64, uint32: u32, uint64: u64, size: usize, ptrdiff: isize,
}

/// Pre-1.4 names for int/long/longlong.
macro_rules! amo_legacy {
    ($( $name:ident : $t:ty ),* $(,)?) => { $( paste::paste! {
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _fadd>](d: *mut $t, v: $t, pe: c_int) -> $t { [<shmem_ $name _atomic_fetch_add>](d, v, pe) }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _finc>](d: *mut $t, pe: c_int) -> $t { [<shmem_ $name _atomic_fetch_inc>](d, pe) }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _add>](d: *mut $t, v: $t, pe: c_int) { [<shmem_ $name _atomic_add>](d, v, pe) }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _inc>](d: *mut $t, pe: c_int) { [<shmem_ $name _atomic_inc>](d, pe) }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _cswap>](d: *mut $t, c: $t, v: $t, pe: c_int) -> $t { [<shmem_ $name _atomic_compare_swap>](d, c, v, pe) }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _swap>](d: *mut $t, v: $t, pe: c_int) -> $t { [<shmem_ $name _atomic_swap>](d, v, pe) }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _fetch>](s: *const $t, pe: c_int) -> $t { [<shmem_ $name _atomic_fetch>](s, pe) }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _set>](d: *mut $t, v: $t, pe: c_int) { [<shmem_ $name _atomic_set>](d, v, pe) }
    } )* };
}
amo_legacy! { int: i32, long: std::os::raw::c_long, longlong: i64 }

// --- distributed locks ---------------------------------------------------------
// The lock word lives in the symmetric object; PE 0's copy is authoritative. It holds
// 0 when free, or (owner PE + 1).

const LOCK_OWNER_PE: usize = 0;

#[no_mangle]
pub unsafe extern "C" fn shmem_set_lock(lock: *mut c_long) {
    let me = state().my_pe as u64 + 1;
    let mut spins = 0u32;
    while amo(LOCK_OWNER_PE, lock as *mut u8, 8, OP_CSWAP, 0, me) != 0 {
        spins += 1;
        if spins < 8 {
            std::hint::spin_loop();
        } else {
            progress(); // the holder's release may be waiting on this PE's progress
        }
    }
}

/// Returns 0 if the lock was acquired, nonzero if it is held.
#[no_mangle]
pub unsafe extern "C" fn shmem_test_lock(lock: *mut c_long) -> c_int {
    let me = state().my_pe as u64 + 1;
    (amo(LOCK_OWNER_PE, lock as *mut u8, 8, OP_CSWAP, 0, me) != 0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn shmem_clear_lock(lock: *mut c_long) {
    // Stores from the critical section must be visible before the lock is released.
    shmem_quiet();
    amo(LOCK_OWNER_PE, lock as *mut u8, 8, OP_SET, 0, 0);
}

#[allow(dead_code)]
fn _unused(_: *mut c_void) {}
