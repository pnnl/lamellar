use std::os::raw::{c_int, c_void};

use crate::heap::{data_off, offset_of};
use crate::state::{put_local_complete, state, DATA_FLAG};

fn fail_unregistered(what: &str, p: *const u8) -> ! {
    let why = if state().data.is_some() {
        "it is neither in the symmetric heap nor in the program's data segment"
    } else {
        "it is not in the symmetric heap, and static/global data is not available as a remote \
         target on this backend (set SHMEM_WARN_DATA=1 for the registration error)"
    };
    eprintln!("shmem: {what}: address {p:p}: {why}");
    std::process::abort();
}

/// Symmetric offset of `p`: a heap offset, or `DATA_FLAG | offset` within the data segment.
pub(crate) fn sym_off(what: &str, p: *const u8) -> usize {
    if let Some(o) = offset_of(p) {
        return o;
    }
    if let Some(o) = data_off(p) {
        return DATA_FLAG | o;
    }
    fail_unregistered(what, p)
}

/// Local address of a symmetric offset produced by `sym_off` (or a raw heap offset).
pub(crate) fn local_ptr(off: usize) -> *mut u8 {
    let s = state();
    if off & DATA_FLAG != 0 {
        (s.data_base + (off & !DATA_FLAG)) as *mut u8
    } else {
        (s.base + off) as *mut u8
    }
}

/// Blocking put of `n` bytes from arbitrary local memory `src` to region offset `doff` on `pe`.
/// The local PE is a plain memcpy; remote PEs go straight from `src` (no staging copy).
pub(crate) unsafe fn put_off(doff: usize, src: *const u8, n: usize, pe: usize) {
    if n == 0 {
        return;
    }
    let s = state();
    if pe == s.my_pe {
        std::ptr::copy(src, local_ptr(doff), n);
    } else if put_local_complete() {
        // ucx: the runtime's unmanaged put returns once `src` is reusable; remote completion waits for quiet
        if doff & DATA_FLAG != 0 {
            s.data.as_ref().unwrap().put_nbi(pe, doff & !DATA_FLAG, src, n);
        } else {
            s.heap.put_nbi(pe, doff, src, n);
        }
    } else if doff & DATA_FLAG != 0 {
        s.data.as_ref().unwrap().put(pe, doff & !DATA_FLAG, src, n);
    } else {
        s.heap.put(pe, doff, src, n);
    }
}

/// Blocking get of `n` bytes from region offset `soff` on `pe` into arbitrary local memory `dest`.
pub(crate) unsafe fn get_off(soff: usize, dest: *mut u8, n: usize, pe: usize) {
    if n == 0 {
        return;
    }
    let s = state();
    if pe == s.my_pe {
        std::ptr::copy(local_ptr(soff) as *const u8, dest, n);
    } else if soff & DATA_FLAG != 0 {
        s.data.as_ref().unwrap().get(pe, soff & !DATA_FLAG, dest, n);
    } else {
        s.heap.get(pe, soff, dest, n);
    }
}

/// Non-blocking put: returns once the transfer is initiated. `src` must stay valid and unmodified until the
/// next `shmem_quiet` / `shmem_fence` / barrier (the spec's contract for `*_nbi`). The local PE is a memcpy.
pub(crate) unsafe fn put_nbi_off(doff: usize, src: *const u8, n: usize, pe: usize) {
    if n == 0 {
        return;
    }
    let s = state();
    if pe == s.my_pe {
        std::ptr::copy(src, local_ptr(doff), n);
    } else if doff & DATA_FLAG != 0 {
        s.data.as_ref().unwrap().put_nbi(pe, doff & !DATA_FLAG, src, n);
    } else {
        s.heap.put_nbi(pe, doff, src, n);
    }
}

/// Non-blocking get: `dest` is valid to read only after the next `shmem_quiet` / barrier.
pub(crate) unsafe fn get_nbi_off(soff: usize, dest: *mut u8, n: usize, pe: usize) {
    if n == 0 {
        return;
    }
    let s = state();
    if pe == s.my_pe {
        std::ptr::copy(local_ptr(soff) as *const u8, dest, n);
    } else if soff & DATA_FLAG != 0 {
        s.data.as_ref().unwrap().get_nbi(pe, soff & !DATA_FLAG, dest, n);
    } else {
        s.heap.get_nbi(pe, soff, dest, n);
    }
}

pub(crate) unsafe fn put_nbi_bytes(dest: *mut u8, src: *const u8, n: usize, pe: usize) {
    put_nbi_off(sym_off("put_nbi", dest), src, n, pe)
}

pub(crate) unsafe fn get_nbi_bytes(dest: *mut u8, src: *const u8, n: usize, pe: usize) {
    get_nbi_off(sym_off("get_nbi", src), dest, n, pe)
}

/// Put to a symmetric *address* (translated to a heap offset).
pub(crate) unsafe fn put_bytes(dest: *mut u8, src: *const u8, n: usize, pe: usize) {
    put_off(sym_off("put", dest), src, n, pe)
}

/// Get from a symmetric *address* (translated to a heap offset).
pub(crate) unsafe fn get_bytes(dest: *mut u8, src: *const u8, n: usize, pe: usize) {
    get_off(sym_off("get", src), dest, n, pe)
}

#[no_mangle]
pub unsafe extern "C" fn shmem_putmem(dest: *mut c_void, src: *const c_void, n: usize, pe: c_int) {
    put_bytes(dest as *mut u8, src as *const u8, n, pe as usize)
}
#[no_mangle]
pub unsafe extern "C" fn shmem_getmem(dest: *mut c_void, src: *const c_void, n: usize, pe: c_int) {
    get_bytes(dest as *mut u8, src as *const u8, n, pe as usize)
}
// Non-blocking: initiated here, complete by the next shmem_quiet / fence / barrier.
#[no_mangle]
pub unsafe extern "C" fn shmem_putmem_nbi(d: *mut c_void, s: *const c_void, n: usize, pe: c_int) {
    put_nbi_bytes(d as *mut u8, s as *const u8, n, pe as usize)
}
#[no_mangle]
pub unsafe extern "C" fn shmem_getmem_nbi(d: *mut c_void, s: *const c_void, n: usize, pe: c_int) {
    get_nbi_bytes(d as *mut u8, s as *const u8, n, pe as usize)
}

/// Typed and sized RMA families. `$sz` is bytes per element.
macro_rules! rma_types {
    ($( $name:ident : $t:ty ),* $(,)?) => { $( paste::paste! {
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _put>](d: *mut $t, s: *const $t, n: usize, pe: c_int) {
            put_bytes(d as *mut u8, s as *const u8, n * std::mem::size_of::<$t>(), pe as usize)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _get>](d: *mut $t, s: *const $t, n: usize, pe: c_int) {
            get_bytes(d as *mut u8, s as *const u8, n * std::mem::size_of::<$t>(), pe as usize)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _put_nbi>](d: *mut $t, s: *const $t, n: usize, pe: c_int) {
            put_nbi_bytes(d as *mut u8, s as *const u8, n * std::mem::size_of::<$t>(), pe as usize)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _get_nbi>](d: *mut $t, s: *const $t, n: usize, pe: c_int) {
            get_nbi_bytes(d as *mut u8, s as *const u8, n * std::mem::size_of::<$t>(), pe as usize)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _p>](d: *mut $t, v: $t, pe: c_int) {
            put_bytes(d as *mut u8, &v as *const $t as *const u8, std::mem::size_of::<$t>(), pe as usize)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _g>](s: *const $t, pe: c_int) -> $t {
            let mut v = std::mem::MaybeUninit::<$t>::uninit();
            get_bytes(v.as_mut_ptr() as *mut u8, s as *const u8, std::mem::size_of::<$t>(), pe as usize);
            v.assume_init()
        }
        /// Strided put: element i of `s` (stride `sst`) -> element i of `d` (stride `dst`).
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _iput>](
            d: *mut $t, s: *const $t, dst: isize, sst: isize, n: usize, pe: c_int,
        ) {
            for i in 0..n as isize {
                [<shmem_ $name _p>](d.offset(i * dst), *s.offset(i * sst), pe);
            }
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_ $name _iget>](
            d: *mut $t, s: *const $t, dst: isize, sst: isize, n: usize, pe: c_int,
        ) {
            for i in 0..n as isize {
                *d.offset(i * dst) = [<shmem_ $name _g>](s.offset(i * sst), pe);
            }
        }
    } )* };
}

// Standard RMA types from OpenSHMEM 1.4 (longdouble omitted: no stable Rust f128/x87 type).
rma_types! {
    float: f32, double: f64,
    char: std::os::raw::c_char, schar: i8, short: i16, int: i32, long: std::os::raw::c_long,
    longlong: i64, uchar: u8, ushort: u16, uint: u32, ulong: std::os::raw::c_ulong,
    ulonglong: u64, int8: i8, int16: i16, int32: i32, int64: i64,
    uint8: u8, uint16: u16, uint32: u32, uint64: u64, size: usize, ptrdiff: isize,
}

/// Sized (bit-width) families: shmem_put8/16/32/64/128 (counts are elements).
macro_rules! rma_sized {
    ($( $bits:literal : $bytes:literal ),*) => { $( paste::paste! {
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_put $bits>](d: *mut c_void, s: *const c_void, n: usize, pe: c_int) {
            put_bytes(d as *mut u8, s as *const u8, n * $bytes, pe as usize)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_get $bits>](d: *mut c_void, s: *const c_void, n: usize, pe: c_int) {
            get_bytes(d as *mut u8, s as *const u8, n * $bytes, pe as usize)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_put $bits _nbi>](d: *mut c_void, s: *const c_void, n: usize, pe: c_int) {
            put_nbi_bytes(d as *mut u8, s as *const u8, n * $bytes, pe as usize)
        }
        #[no_mangle]
        pub unsafe extern "C" fn [<shmem_get $bits _nbi>](d: *mut c_void, s: *const c_void, n: usize, pe: c_int) {
            get_nbi_bytes(d as *mut u8, s as *const u8, n * $bytes, pe as usize)
        }
    } )* };
}
rma_sized!(8: 1, 16: 2, 32: 4, 64: 8, 128: 16);
