use std::mem::ManuallyDrop;
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use lamellar::active_messaging::prelude::*;
use lamellar::memregion::prelude::*;
use lamellar::{LamellarWorld, LamellarWorldBuilder};
use parking_lot::Mutex;

use crate::heap::HeapAlloc;
use crate::sym::Sym;

/// Default symmetric heap size (user-visible part), overridable with
/// `SHMEM_SYMMETRIC_SIZE` (bytes).
const DEFAULT_SYMMETRIC_SIZE: usize = 512 << 20;
/// Lamellar's documented default for LAMELLAR_HEAP_SIZE.
const DEFAULT_LAMELLAR_HEAP_SIZE: usize = 4 << 30;
/// Slack in the pool for runtime-internal allocations (barrier/command-queue state, etc.).
const RUNTIME_HEADROOM: usize = 256 << 20;

/// Per-peer counters for the counter-based dissemination barrier (see `sync.rs`).
pub(crate) struct CollState {
    pub sent: Vec<u64>,
    pub recvd: Vec<u64>,
}

/// Set in a symmetric offset to mean "relative to the registered data segment" instead of the heap.
pub(crate) const DATA_FLAG: usize = 1 << 62;

pub(crate) struct State {
    pub my_pe: usize,
    pub num_pes: usize,
    /// Symmetric region: `[0, user_len)` is the user heap, followed by the internal metadata area.
    /// ManuallyDrop: the world's teardown waits for all distributed references to be
    /// released, so finalize must drop these *before* the world (see `shmem_finalize`).
    pub region: ManuallyDrop<SharedMemoryRegion<u8>>,
    /// Pointer RDMA/atomics over `region` (holds its own references; also dropped in finalize).
    pub heap: ManuallyDrop<Sym>,
    pub base: usize,
    pub user_len: usize,
    pub alloc: Mutex<HeapAlloc>,
    /// Region offset of the internal metadata area: `num_pes` u64 sync slots (slot `i` is
    /// written by PE `i`), followed by one u64 scratch word used by `collect`.
    /// The main program's `.data`/`.bss`, registered so globals/statics can be remote targets
    /// (libfabric only; `None` elsewhere, in which case non-heap addresses are rejected).
    pub data: Option<ManuallyDrop<Sym>>,
    pub data_base: usize,
    pub data_len: usize,
    pub meta_off: usize,
    /// Serializes collectives and holds the per-peer message counters they synchronize with.
    pub coll: Mutex<CollState>,
    /// Optional background progress thread (`SHMEM_PROGRESS_THREAD=1`), stopped in `shmem_finalize`.
    progress_thread: Mutex<Option<ProgressThread>>,
    pub team: ManuallyDrop<std::sync::Arc<LamellarTeam>>,
    /// Kept so finalize can drop it (drop performs the closing barriers).
    world: Mutex<Option<LamellarWorld>>,
}

// SAFETY: the region/team handles are reference counted and internally
// synchronized; raw addresses are stored as integers.
unsafe impl Sync for State {}
unsafe impl Send for State {}

static STATE: OnceLock<State> = OnceLock::new();
static PROVIDED_LEVEL: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(THREAD_MULTIPLE);

/// Let the runtime make progress: serve incoming requests and poll completions. The default
/// `single_thread` executor runs nothing in the background (its task queue is only drained inside
/// `block_on`/`exec_task`), so every spin loop in the shim must call this or peers that need this
/// PE's progress (software-emulated atomics, lock hand-offs) deadlock against it.
pub(crate) fn progress() {
    state().team.block_on(YieldOnce(false));
}

/// Block until `cond()` holds, driving the runtime for the whole wait: a single `block_on` whose
/// future re-wakes itself until the condition is true, so the executor's own loop runs the other
/// queued tasks between polls (no task is allocated per spin). Use for every wait on memory that a
/// peer updates; `progress()` is the single-step form for calls that must not block.
pub(crate) fn wait_for(mut cond: impl FnMut() -> bool) {
    if cond() {
        return;
    }
    state().team.block_on(std::future::poll_fn(|cx| {
        if cond() {
            std::task::Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }));
}

/// Background progress thread. With the `single_thread` executor nothing runs while the application
/// computes, so operations that need the target's participation (libfabric `rxm` atomics) stall until the
/// target re-enters SHMEM. This thread keeps the runtime's `recv_data` task (which calls the lamellae's
/// progress function) running: one long `block_on` whose future re-wakes itself until asked to stop.
/// It is safe next to the application thread(s) because several threads may enter `block_on`.
/// Costs a core per PE. Off by default; `SHMEM_PROGRESS_THREAD=1` enables it, and
/// `SHMEM_PROGRESS_INTERVAL_US` (default 0 = just yield) sleeps between passes to trade latency for CPU.
struct ProgressThread {
    stop: std::sync::Arc<AtomicBool>,
    handle: std::thread::JoinHandle<()>,
}

fn start_progress_thread(team: &std::sync::Arc<LamellarTeam>) -> Option<ProgressThread> {
    match std::env::var("SHMEM_PROGRESS_THREAD").as_deref() {
        Ok("") | Ok("0") | Err(_) => return None,
        _ => {}
    }
    let interval = std::env::var("SHMEM_PROGRESS_INTERVAL_US").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let stop = std::sync::Arc::new(AtomicBool::new(false));
    let (team, flag) = (team.clone(), stop.clone());
    let handle = std::thread::Builder::new()
        .name("shmem-progress".into())
        .spawn(move || {
            team.block_on(std::future::poll_fn(move |cx| {
                if flag.load(Ordering::Acquire) {
                    return std::task::Poll::Ready(());
                }
                if interval > 0 {
                    std::thread::sleep(std::time::Duration::from_micros(interval));
                } else {
                    std::thread::yield_now();
                }
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }));
        })
        .ok()?;
    Some(ProgressThread { stop, handle })
}

/// A future that is pending exactly once, so `block_on` runs the other queued tasks before it returns.
struct YieldOnce(bool);
impl std::future::Future for YieldOnce {
    type Output = ();
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        if self.0 {
            std::task::Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }
}

pub(crate) fn state() -> &'static State {
    match STATE.get() {
        Some(s) => s,
        None => {
            eprintln!("shmem: library used before shmem_init()");
            std::process::abort();
        }
    }
}

fn env_bytes(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

/// Thread levels from the spec.
const THREAD_MULTIPLE: c_int = 3;

/// Two different kinds of threading meet here, and they are independent:
///  * SHMEM thread levels (`shmem_init_thread`): how many *application* threads may call SHMEM.
///  * Lamellar's executor: how many *runtime* threads run its async tasks.
/// SHMEM programs are overwhelmingly single threaded, so the default is Lamellar's `single_thread`
/// executor: no worker threads spinning next to the application, and none of the core pinning
/// the work-stealing executor applies without knowing about other PEs on the node (it made
/// co-located PEs fight over the same cores: 60-90 ms barriers at 8 PEs/node vs ~20 us here).
/// `LAMELLAR_EXECUTOR` (e.g. `lamellar`, the work-stealing executor) overrides this. The level
/// reported to the application is capped at MULTIPLE: `tests/c/threads.c` (concurrent puts, gets,
/// atomics and bulk transfers from 8 and 16 application threads per PE) passes under `single_thread`.
fn init_inner(requested: c_int) -> c_int {
    if STATE.get().is_some() {
        return PROVIDED_LEVEL.load(Ordering::SeqCst);
    }
    if std::env::var_os("LAMELLAR_EXECUTOR").is_none() {
        std::env::set_var("LAMELLAR_EXECUTOR", "single_thread");
    }
    // Every SHMEM call blocks by design. Lamellar warns about blocking calls from any thread other
    // than the one that started it (it cannot tell application threads from async workers), which
    // would fire for every extra thread under SHMEM_THREAD_MULTIPLE.
    if std::env::var_os("LAMELLAR_BLOCKING_CALL_WARNING").is_none() {
        std::env::set_var("LAMELLAR_BLOCKING_CALL_WARNING", "0");
    }
    PROVIDED_LEVEL.store(requested.clamp(0, THREAD_MULTIPLE), Ordering::SeqCst);
    if std::env::var_os("LAMELLAR_BACKEND").is_none() {
        if let Some(b) = std::env::var_os("SHMEM_BACKEND") {
            std::env::set_var("LAMELLAR_BACKEND", b);
        }
    }
    // The shmem-opt lamellae can make the program's data segment remotely accessible by turning it into shared memory
    // (see the runtime's shmem_opt_lamellae/external.rs). That is invasive, so the runtime keeps it off by default;
    // the shim exists to give C programs working globals, so it asks for it unless the user said otherwise.
    // The original `shmem` lamellae register external memory through the same code (it needs the shmem-opt feature compiled
    // in), but only when SHMEM_SYMMETRIC_VA asks for the canonical heap: that heap is registered as external memory, and
    // with the switch on a program's globals become remote targets as well.
    let backend = std::env::var("LAMELLAR_BACKEND").unwrap_or_default();
    if (backend == "shmem-opt" || (backend == "shmem" && canonical_heap_address().is_some()))
        && std::env::var_os("LAMELLAR_SHMEM_OPT_EXTERNAL").is_none()
    {
        std::env::set_var("LAMELLAR_SHMEM_OPT_EXTERNAL", "1");
    }
    let user_len = env_bytes("SHMEM_SYMMETRIC_SIZE", DEFAULT_SYMMETRIC_SIZE);
    // The symmetric region is carved from Lamellar's RDMAable pool, so the pool
    // (LAMELLAR_HEAP_SIZE, default 4GB) must be larger than the region plus headroom
    // for the runtime's own allocations. Must be set before the world is built.
    // With SHMEM_SYMMETRIC_VA the symmetric heap is mapped and registered by the shim itself (see canonical_region), so the
    // pool only has to hold the runtime's own allocations.
    let canonical = canonical_heap_address().is_some();
    let needed = if canonical { RUNTIME_HEADROOM } else { user_len + RUNTIME_HEADROOM + (1 << 20) };
    if env_bytes("LAMELLAR_HEAP_SIZE", DEFAULT_LAMELLAR_HEAP_SIZE) < needed {
        std::env::set_var("LAMELLAR_HEAP_SIZE", needed.to_string());
    }
    // The canonical range is claimed before the runtime picks its own mappings: the single-node lamellae place a window of
    // all PEs' regions at an address the PEs agree on, and at 32 PEs that window can cover the canonical address.
    if let Some(va) = canonical_heap_address() {
        unsafe { canonical_premap(va, user_len + CANONICAL_META_RESERVE) };
    }
    let world = LamellarWorldBuilder::new().build();
    // Collective: every PE requests the identical size.
    let num_pes = world.num_pes();
    let meta_off = user_len;
    let meta_len = 8 * num_pes + 8;
    // Allocated as u64 words so that the whole region is aligned for the 4- and 8-byte atomics, then viewed as bytes.
    let region: SharedMemoryRegion<u8> = match canonical_heap_address() {
        Some(va) => unsafe { canonical_region(&world.team(), va, meta_off + meta_len) },
        None => {
            let words = world
                .alloc_shared_mem_region::<u64>((meta_off + meta_len).div_ceil(8))
                .block();
            let region: SharedMemoryRegion<u8> = unsafe { words.cast() };
            drop(words);
            region
        }
    };
    let base = unsafe { region.as_mut_ptr().expect("local symmetric region") } as usize;
    // Peers only write into this after the barrier below, so zeroing it here is race free.
    unsafe { std::ptr::write_bytes((base + meta_off) as *mut u8, 0, meta_len) };
    let team = world.team();
    let progress_thread = start_progress_thread(&team);
    let (data_base, data_len, data) = unsafe { register_data_segment(&team) };
    let st = State {
        my_pe: world.my_pe(),
        num_pes,
        heap: ManuallyDrop::new(unsafe { Sym::new(region.clone()) }.expect("symmetric heap is not aligned for atomics")),
        region: ManuallyDrop::new(region),
        base,
        user_len,
        alloc: Mutex::new(HeapAlloc::new(user_len)),
        data: data.map(ManuallyDrop::new),
        data_base,
        data_len,
        meta_off,
        progress_thread: Mutex::new(progress_thread),
        coll: Mutex::new(CollState {
            sent: vec![0; num_pes],
            recvd: vec![0; num_pes],
        }),
        team: ManuallyDrop::new(team),
        world: Mutex::new(Some(world)),
    };
    let _ = STATE.set(st);
    state().team.barrier();
    PROVIDED_LEVEL.load(Ordering::SeqCst)
}

#[no_mangle]
pub extern "C" fn shmem_init() {
    init_inner(0);
}

/// Thread levels from the spec: SINGLE=0, FUNNELED=1, SERIALIZED=2, MULTIPLE=3.
#[no_mangle]
pub unsafe extern "C" fn shmem_init_thread(requested: c_int, provided: *mut c_int) -> c_int {
    let level = init_inner(requested);
    if !provided.is_null() {
        *provided = level;
    }
    0
}

#[no_mangle]
pub extern "C" fn start_pes(_npes: c_int) {
    init_inner(0);
}

#[no_mangle]
pub extern "C" fn shmem_query_thread(provided: *mut c_int) {
    if !provided.is_null() {
        unsafe { *provided = PROVIDED_LEVEL.load(Ordering::SeqCst) };
    }
}

#[no_mangle]
pub extern "C" fn shmem_finalize() {
    static FINALIZED: AtomicBool = AtomicBool::new(false);
    if let Some(s) = STATE.get() {
        if FINALIZED.swap(true, Ordering::SeqCst) {
            return;
        }
        s.team.wait_all();
        s.team.barrier();
        // the progress thread holds its own reference to the team: stop and join it first
        if let Some(pt) = s.progress_thread.lock().take() {
            pt.stop.store(true, Ordering::Release);
            let _ = pt.handle.join();
        }
        // SAFETY: finalize is terminal; no shmem call may follow. Taking the handles out
        // of the ManuallyDrops releases our references so the world can shut down.
        unsafe {
            if let Some(d) = &s.data {
                drop(std::ptr::read(&**d));
            }
            drop(std::ptr::read(&*s.heap));
            drop(std::ptr::read(&*s.region));
            drop(std::ptr::read(&*s.team));
        }
        // Dropping the world runs the runtime's shutdown (barriers, PMI finalize).
        drop(s.world.lock().take());
    }
}

#[no_mangle]
pub extern "C" fn shmem_global_exit(status: c_int) {
    std::process::exit(status);
}

#[no_mangle]
pub extern "C" fn shmem_my_pe() -> c_int {
    state().my_pe as c_int
}
#[no_mangle]
pub extern "C" fn shmem_n_pes() -> c_int {
    state().num_pes as c_int
}
#[no_mangle]
pub extern "C" fn _my_pe() -> c_int {
    shmem_my_pe()
}
#[no_mangle]
pub extern "C" fn _num_pes() -> c_int {
    shmem_n_pes()
}

#[no_mangle]
pub extern "C" fn shmem_pe_accessible(pe: c_int) -> c_int {
    (pe >= 0 && (pe as usize) < state().num_pes) as c_int
}

#[no_mangle]
pub extern "C" fn shmem_addr_accessible(addr: *const u8, pe: c_int) -> c_int {
    (shmem_pe_accessible(pe) != 0
        && (crate::heap::offset_of(addr).is_some() || crate::heap::data_off(addr).is_some())) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn shmem_info_get_version(major: *mut c_int, minor: *mut c_int) {
    *major = 1;
    *minor = 4;
}

#[no_mangle]
pub unsafe extern "C" fn shmem_info_get_name(name: *mut c_char) {
    const N: &[u8] = b"Lamellar OpenSHMEM\0";
    std::ptr::copy_nonoverlapping(N.as_ptr() as *const c_char, name, N.len());
}

// --- ordering / completion ---------------------------------------------------

/// Experimental switches, both ucx-only and off by default (the other backends' runtime calls do not have the
/// semantics they rely on). `SHMEM_PUT_LOCAL_COMPLETE=1`: a blocking put returns after local completion (source
/// reusable) instead of after a worker-wide flush; remote completion is left to `quiet`, as SHMEM allows.
/// `SHMEM_QUIET=region`: `quiet` waits on the symmetric region (worker flush) instead of the whole team, which
/// also loops over active-message counters and polls the scheduler.
fn ucx_switch(var: &str, want: &str, cell: &'static std::sync::OnceLock<bool>) -> bool {
    *cell.get_or_init(|| {
        let on = std::env::var(var).map(|v| v == want).unwrap_or(false);
        let ucx = std::env::var("LAMELLAR_BACKEND").map(|b| b == "ucx").unwrap_or(false);
        if on && !ucx {
            eprintln!("shmem: {var}={want} is only supported on the ucx backend; ignored");
        }
        on && ucx
    })
}

pub(crate) fn put_local_complete() -> bool {
    static C: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    ucx_switch("SHMEM_PUT_LOCAL_COMPLETE", "1", &C)
}

fn quiet_region() -> bool {
    static C: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    ucx_switch("SHMEM_QUIET", "region", &C)
}

/// All prior puts/AMOs issued by this PE are complete and visible.
#[no_mangle]
pub extern "C" fn shmem_quiet() {
    if quiet_region() {
        state().heap.quiet();
    } else {
        state().team.wait_all();
    }
}

/// Phase A: fence is implemented as quiet (stronger than required).
#[no_mangle]
pub extern "C" fn shmem_fence() {
    shmem_quiet();
}

#[no_mangle]
pub extern "C" fn shmem_barrier_all() {
    shmem_quiet();
    state().team.barrier();
}

#[no_mangle]
pub extern "C" fn shmem_sync_all() {
    state().team.barrier();
}

// --- data segment registration -----------------------------------------------------------------

/// Writable, non-RELRO part of the main program's data segment, as `(start, len)`.
///
/// The linker symbols `__data_start`/`_end` cannot be used from a shared library (they would
/// resolve to the library's own segment), so ask the loader for the main program's program
/// headers. RELRO pages are made read-only at startup and cannot be registered for write, so the
/// range starts after them; it starts on the page containing the segment start unless another
/// segment shares that page.
fn data_segment_range() -> Option<(usize, usize)> {
    struct Out(Option<(usize, usize)>);
    unsafe extern "C" fn cb(info: *mut libc::dl_phdr_info, _: libc::size_t, data: *mut libc::c_void) -> c_int {
        let info = &*info;
        let base = info.dlpi_addr as usize;
        let page = libc::sysconf(libc::_SC_PAGESIZE) as usize;
        let up = |x: usize| (x + page - 1) & !(page - 1);
        let mut relro_end = 0usize;
        let mut best: Option<(usize, usize)> = None;
        let mut loads: Vec<(usize, usize)> = Vec::new();
        let mut writable: Vec<(usize, usize)> = Vec::new();
        for ph in std::slice::from_raw_parts(info.dlpi_phdr, info.dlpi_phnum as usize) {
            let (s, e) = (base + ph.p_vaddr as usize, base + ph.p_vaddr as usize + ph.p_memsz as usize);
            match ph.p_type {
                libc::PT_GNU_RELRO => relro_end = up(e),
                libc::PT_LOAD => {
                    loads.push((s, e));
                    if ph.p_flags & libc::PF_W != 0 {
                        writable.push((s, e));
                    }
                }
                _ => {}
            }
        }
        // Pick the writable segment with the most registrable bytes left once its RELRO pages are
        // dropped (with `-z relro` the RELRO segment can be larger than the real .data/.bss).
        let mut best_left = 0usize;
        for &(s, e) in &writable {
            let left = e - if relro_end > s { relro_end.min(e) } else { s };
            if left > best_left {
                best_left = left;
                best = Some((s, e));
            }
        }
        if let Some((s, e)) = best {
            // Start at the page containing the segment start unless another segment reaches into
            // that page (then it may be read-only and cannot be registered for write).
            let down = s & !(page - 1);
            let shares_page = loads.iter().any(|&(ls, le)| (ls, le) != (s, e) && le > down && ls < s);
            let mut start = if shares_page { up(s) } else { down };
            // RELRO pages are made read-only after startup.
            if relro_end > start && relro_end < e {
                start = relro_end;
            }
            if e > start {
                (*(data as *mut Out)).0 = Some((start, e - start));
            }
        }
        1 // the first object reported is the main program: stop
    }
    let mut out = Out(None);
    unsafe { libc::dl_iterate_phdr(Some(cb), &mut out as *mut Out as *mut libc::c_void) };
    out.0
}

/// Default virtual address of the canonical symmetric heap (48 TiB: far above where the kernel places a program, its
/// libraries and its mmaps, and far below the stack).
const DEFAULT_CANONICAL_HEAP_ADDRESS: usize = 0x3000_0000_0000;

/// `SHMEM_SYMMETRIC_VA` asks for the symmetric heap to live at one virtual address that is the same on every PE:
/// `1` (or `on`) uses the default address, a hex number (`0x...`) names one. Off by default.
///
/// The runtime's own pool is mapped wherever the kernel puts it, per PE, and the shim translates every pointer to an offset
/// from the local base, so a program never needs equal addresses to be correct. Some programs still compare or ship raw
/// pointer values (the original C HAMR checks that `shmem_malloc` returned the same pointer on every PE); this makes that
/// true, with ASLR and PIE as they are.
fn canonical_heap_address() -> Option<usize> {
    let v = std::env::var("SHMEM_SYMMETRIC_VA").ok()?;
    let v = v.trim();
    match v {
        "" | "0" | "off" | "no" => None,
        "1" | "on" | "yes" => Some(DEFAULT_CANONICAL_HEAP_ADDRESS),
        _ => usize::from_str_radix(v.trim_start_matches("0x"), 16).ok(),
    }
}

/// Room reserved after the user's part of the canonical heap for the shim's per-PE metadata (`8 * num_pes + 8` bytes, known
/// only once the world exists).
const CANONICAL_META_RESERVE: usize = 1 << 20;

/// Whether `canonical_premap` already mapped the range, and where/how long.
static CANONICAL_MAP: std::sync::Mutex<Option<(usize, usize)>> = std::sync::Mutex::new(None);

/// Map `len` bytes at `va` (anonymous, lazily committed). Aborts if the address is taken on this PE, because a PE that fell
/// back to the runtime's pool would give a different pointer and would also leave the collective registration half done.
unsafe fn canonical_premap(va: usize, len: usize) {
    let len = len.next_multiple_of(4096);
    let p = libc::mmap(
        va as *mut std::os::raw::c_void,
        len,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE | libc::MAP_FIXED_NOREPLACE,
        -1,
        0,
    );
    if p == libc::MAP_FAILED || p as usize != va {
        eprintln!(
            "shmem: cannot map the symmetric heap ({len} bytes) at {va:#x}: {}. Unset SHMEM_SYMMETRIC_VA or choose another address.",
            std::io::Error::last_os_error()
        );
        std::process::abort();
    }
    *CANONICAL_MAP.lock().unwrap() = Some((va, len));
}

/// Register the premapped canonical range (its first `len` bytes) with the runtime as the symmetric region (the same
/// external registration the data segment uses). Collective.
unsafe fn canonical_region(team: &std::sync::Arc<LamellarTeam>, va: usize, len: usize) -> SharedMemoryRegion<u8> {
    let len = len.next_multiple_of(4096);
    match *CANONICAL_MAP.lock().unwrap() {
        Some((mva, mlen)) if mva == va && len <= mlen => {}
        _ => {
            eprintln!("shmem: the canonical heap at {va:#x} is not mapped or is too small for {len} bytes");
            std::process::abort();
        }
    }
    let p = va as *mut u8;
    match team.try_register_external_shared_mem_region::<u8>(p, len).block() {
        Ok(region) => region,
        Err(e) => {
            eprintln!("shmem: the symmetric heap at {va:#x} could not be registered with the runtime: {e}");
            std::process::abort();
        }
    }
}

/// Collectively register the data segment (every PE runs the same binary, so all PEs attempt the
/// same thing). Failure is not fatal: globals simply stay unsupported, with a clear message when
/// one is used.
unsafe fn register_data_segment(team: &std::sync::Arc<LamellarTeam>) -> (usize, usize, Option<Sym>) {
    let Some((start, len)) = data_segment_range() else {
        return (0, 0, None);
    };
    let registered = team
        .try_register_external_shared_mem_region::<u8>(start as *mut u8, len)
        .block()
        .map_err(|e| e.to_string())
        .and_then(|region| Sym::new(region));
    match registered {
        Ok(sym) => (start, len, Some(sym)),
        Err(e) => {
            if std::env::var_os("SHMEM_DEBUG").is_some() || std::env::var_os("SHMEM_WARN_DATA").is_some() {
                eprintln!("shmem: data segment ({len} bytes) not registered: {e}; globals/statics unsupported");
            }
            (0, 0, None)
        }
    }
}
