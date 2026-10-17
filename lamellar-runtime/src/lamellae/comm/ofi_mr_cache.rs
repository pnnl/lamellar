//! Keeps libfabric's memory-registration cache from serving stale registrations.
//!
//! The libfabric providers register memory for the NIC on demand and cache the registrations, keyed by address
//! range. A cached entry is only valid while the pages behind it are unchanged, so the cache relies on a "memory
//! monitor" to learn about `munmap` / `mmap`. libfabric's default monitor (memhooks) works by patching the libc entry
//! points. When UCX is linked into the same program (any build with `enable-ucx` or `enable-ucx-opt`) its `ucm`
//! library installs its own hooks on the same functions when the program starts, even if the UCX backend is not the
//! one in use, and with both in place memhooks no longer sees the unmaps: a large buffer that is freed and allocated
//! again at the same address then hits the old entry and the NIC reads or writes the old physical pages.
//!
//! This showed up as an all-reduce that finished but returned only the calling PE's own contribution. The
//! collective provider allocates a fresh temporary buffer for every operation, and rxm registers it on the fly for
//! any message that goes through the rendezvous protocol (256 KiB here). Eager and segmented (SAR) messages copy
//! through bounce buffers and are not affected.
//! The results were correct with UCX's memory hooks switched off (`UCX_MEM_EVENTS=no`) and with the cache disabled.
//! The userfaultfd monitor, whose notifications come from the kernel, also fixed that case, but it is not a safe
//! choice: with it, libfabric runs failed intermittently (a runtime allocation reporting out of memory, or a
//! segfault) in tests that create many arrays, with or without UCX's hooks.
//!
//! So, in a build that links UCX and while UCX's hooks are active, turn the registration cache off and warn: slower
//! for registrations that would have hit the cache, but correct. Nothing is changed in a build without UCX, or when
//! the hooks are switched off (`UCX_MEM_EVENTS=no` or `UCX_MEM_MMAP_HOOK_MODE=none`, which has to be in the
//! environment when the process starts, because UCX installs its hooks from a load-time constructor, e.g. exported
//! by the launcher), and anything the user has set (`FI_MR_CACHE_MONITOR` or `FI_MR_CACHE_MAX_COUNT`) is left alone.

use std::env;

/// Disable libfabric's registration cache before the fabric is opened if UCX's memory hooks would break it. `my_pe`
/// only decides who prints the warning.
pub(crate) fn configure_mr_cache_monitor(my_pe: usize) {
    if env::var_os("FI_MR_CACHE_MONITOR").is_some() || env::var_os("FI_MR_CACHE_MAX_COUNT").is_some() {
        return;
    }
    if !ucx_memory_hooks_active() {
        return;
    }
    env::set_var("FI_MR_CACHE_MAX_COUNT", "0");
    if my_pe == 0 {
        eprintln!(
            "[LAMELLAR WARNING] Lamellar was compiled with both ucx and libfabric support. UCX's memory hooks break \
             libfabric's memory registration cache, so the cache has been disabled (FI_MR_CACHE_MAX_COUNT=0) and some \
             operations will register memory every time. To keep the cache, either set UCX_MEM_EVENTS=no in the \
             environment before the program starts, annotate your main with #[lamellar::main] (its launcher sets it \
             for libfabric runs), or recompile without ucx support."
        );
    }
}

/// Whether UCX's memory hooks (ucm) are installed in this process: UCX is linked in (the `enable-ucx` and
/// `enable-ucx-opt` features, whichever backend is selected) and the user has not turned them off.
fn ucx_memory_hooks_active() -> bool {
    if !cfg!(any(feature = "enable-ucx", feature = "enable-ucx-opt")) {
        return false;
    }
    let is = |name: &str, values: &[&str]| {
        env::var(name).map_or(false, |v| values.contains(&v.trim().to_ascii_lowercase().as_str()))
    };
    // UCX boolean spellings for "off", and the mmap hook mode that installs no hook
    !is("UCX_MEM_EVENTS", &["n", "no", "off", "0", "f", "false"]) && !is("UCX_MEM_MMAP_HOOK_MODE", &["none"])
}
