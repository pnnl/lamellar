# lamellar-openshmem

An OpenSHMEM 1.4 C API on top of the [Lamellar](../lamellar-runtime) runtime. Unmodified C/C++ OpenSHMEM programs
build against a generated `shmem.h` and run on any Lamellar backend (libfabric, UCX, shared memory), including
programs that use global/static data as remote targets.

The design rule for this crate is that **the shim stays thin**: it maps SHMEM calls onto runtime operations
(RMA, atomics, collectives, progress) and holds as little logic of its own as possible. Where the runtime is slow
or missing something, the fix belongs in the runtime (see [Runtime changes](#runtime-changes)).

## Contents

1. [Quick start](#quick-start)
2. [Building and installing](#building-and-installing)
3. [Usage](#usage)
4. [Backends](#backends)
5. [Configuration](#configuration)
6. [Supported features](#supported-features)
7. [Current limitations](#current-limitations)
8. [Performance](#performance)
9. [Testing and benchmarking](#testing-and-benchmarking)
10. [Runtime changes](#runtime-changes)
11. [Future plans](#future-plans)
12. [Repository layout](#repository-layout)

## Quick start

```bash
cargo install cargo-c                                              # once; provides `cargo cinstall`
cargo cinstall --release --prefix $HOME/shmem --features ucx-opt   # libshmem.so/.a, shmem.h, shmem.pc, CMake package
cargo install --path wrappers --root $HOME/shmem                   # shmemcc, oshcc, shmemcxx, oshcxx, oshrun, shmemrun
export PATH=$HOME/shmem/bin:$PATH

shmemcc hello.c -o hello
LAMELLAR_BACKEND=ucx-opt oshrun -N 2 --pes-per-node 4 ./hello      # inside a Slurm allocation
```

Pick the backend with the Cargo features at install time and `LAMELLAR_BACKEND` at run time (see [Backends](#backends)).

## Building and installing

Prerequisites: a Rust toolchain, a C compiler, Python 3 (it generates `include/shmem.h`, which is also checked in), and for
the network backends the vendored libfabric/UCX/PMIx build dependencies already used by the Lamellar workspace (see
`../NATIVE_DEPENDENCIES.md`). For installing and for the tests you also need [cargo-c](https://github.com/lu-zero/cargo-c)
and CMake 3.20 or newer. cargo-c needs a recent Rust and, to build, OpenSSL 3 headers (or
`cargo install cargo-c --features vendored-openssl`).

Installation is done with cargo:

```bash
cargo cinstall --release --prefix <prefix> [--features <backend features>]   # the library
cargo install --path wrappers --root <prefix>                                # the compiler wrappers and launcher
```

`cargo cinstall` installs `lib/libshmem.so` (with its soname links), `lib/libshmem.a`, `include/shmem.h`,
`lib/pkgconfig/shmem.pc` and the CMake package `share/cmake/LamellarShmem/LamellarShmemConfig.cmake`. The wrappers go to
`<prefix>/bin`.

Backends are Cargo features:

| Run-time backend | Cargo features |
|---|---|
| `shmem` | none (default) |
| `shmem-opt` | `shmem-opt` |
| `ucx` | `ucx` |
| `ucx-opt` | `ucx-opt` |
| `libfabric` | `libfabric-shared` |
| `libfabric-sys-opt` | `libfabric-sys-opt-shared` |

Notes:

* The libfabric backends build libfabric as `libfabric.so`, and `libshmem.so` finds it through its `$ORIGIN` runpath, so it has
  to be copied next to `libshmem.so`: `tools/copy-libfabric.sh <target-dir>/<target-triple> <prefix>/lib` after installing.
  (`ucx-shared`, which builds UCX as shared libraries, has the equivalent `tools/copy-ucx.sh`.) The other backends are
  self-contained.
* `make` has a few conveniences (`make wrappers`, `make header`, `make check`); installing is not one of them.
* Multi-node runs of the network backends need either Slurm with PMIx support or `prterun`; `make prrte` builds the vendored
  PRRTE once into `target/fab/prrte` for clusters without it.

## Usage

### Compiling

`shmemcc` / `oshcc` wrap the C compiler and `shmemcxx` / `oshcxx` the C++ compiler:

```bash
shmemcc prog.c -o prog                 # dynamic link against libshmem.so
shmemcc --shmem-static prog.c -o prog  # link libshmem.a (see limitations)
shmemcc --showme:compile               # print the flags instead of compiling (also :link, :command, :version)
```

`OSHCC_CC`, `OSHCC_CXX`, `OSHCC_CFLAGS` and `OSHCC_LDFLAGS` adjust the driver. The wrappers find the header and library next
to themselves (`<prefix>/bin/../{include,lib}`), or through `SHMEM_PREFIX` / `SHMEM_INCDIR` / `SHMEM_LIBDIR`, or `pkg-config
shmem`. Without the wrappers, use the installed files directly:

```bash
cc prog.c $(pkg-config --cflags --libs shmem) -Wl,-rpath,<prefix>/lib -o prog
# CMake: find_package(LamellarShmem REQUIRED); target_link_libraries(prog PRIVATE Lamellar::shmem)
```

Cargo does not allow `+` in a binary name, so the `shmemc++` / `oshc++` names are not provided; symlink them to `shmemcxx` if
you want them. There are no Fortran wrappers.

### Running

```bash
oshrun [-np N] [-N|--nodes M] [--pes-per-node P] prog [args...]
```

* **Inside a Slurm allocation** the processes are placed on the allocated nodes with `srun`: the single-node backends
  (`shmem`, `shmem-opt`) directly, the network backends with `srun --mpi=pmix` where Slurm has PMIx, otherwise `prterun`.
* **Outside an allocation** the single-node backends start N local processes; other backends use `prterun` if it is on
  `PATH`. Set `SHMEM_REQUIRE_ALLOCATION=1` to make `oshrun` refuse to run outside an allocation instead.
* Each PE gets its own cores with `prterun` (`OSHRUN_CORES_PER_PE`, default 1, or 2 when the progress thread is on).
* `-np N` is `nodes * pes-per-node` when both are given; an inconsistent combination is rejected.

### Global and static data

Static/global variables work as remote targets on the backends that can register the program's `.data`/`.bss`
(see [Backends](#backends)). The shim registers the writable, non-RELRO part of the main program's data segment at
`shmem_init`. Registration failure is not fatal: the program still runs and only global targets are unavailable
(`SHMEM_WARN_DATA=1` prints the reason). PIE and ASLR layouts are supported because offsets are taken relative to each
PE's own address.

## Backends

| Backend (`LAMELLAR_BACKEND`) | Scope | Globals as remote targets | Notes |
|---|---|---|---|
| `libfabric-sys-opt` | multi-node | yes | Raw libfabric calls. Collectives come from libfabric's collective provider and are slow (see limitations). |
| `ucx-opt` | multi-node | yes | UCX with ticketed completion and UCC collectives. Best collective performance of the network backends. |
| `shmem-opt` | single node | yes | POSIX shared memory. Remaps the program's `.data/.bss` onto a shared file; see caveats below. |
| `libfabric` | multi-node | yes | Original libfabric lamellae. |
| `ucx` | multi-node | yes | Original UCX lamellae. |
| `shmem` | single node | **no** | Original shared-memory lamellae. |
| `rofi` | multi-node | not tested | The feature exists; it is not exercised by the tests. |

The optimized lamellae (`*-opt`) are the ones the tests and tuning use. The original lamellae still build but are treated as
legacy.

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `LAMELLAR_BACKEND` / `SHMEM_BACKEND` | build default | Which lamellae to use at run time. |
| `SHMEM_SYMMETRIC_SIZE` | 512 MiB | Size of the symmetric heap per PE, as a plain number of bytes (no unit suffixes). |
| `LAMELLAR_HEAP_SIZE` | runtime default (4 GiB) | The RDMA-able pool the heap is carved from. Must exceed the symmetric heap plus headroom. |
| `LAMELLAR_THREADS` | runtime default | Lamellar worker threads. |
| `LAMELLAR_EXECUTOR` | `single_thread` (set by the shim) | Override the executor; the work-stealing one is `lamellar`. |
| `SHMEM_PROGRESS_THREAD` | off | `1` starts a background progress thread (costs a core per PE). |
| `SHMEM_PROGRESS_INTERVAL_US` | 0 | Sleep between progress passes. |
| `SHMEM_RUNTIME_REDUCE` | on | `0` forces the shim's pull-based fold for whole-team `*_to_all` reductions. |
| `SHMEM_RUNTIME_BARRIER` | on | `0` forces the shim's own dissemination barrier for whole-team `shmem_barrier`/`shmem_sync`. |
| `SHMEM_LOCAL_AMO` | backend dependent | `net` / `cpu`: whether a PE's atomics on its own memory go through the network path. |
| `SHMEM_WARN_DATA` | off | Print why the data segment could not be registered. |
| `SHMEM_PUT_LOCAL_COMPLETE`, `SHMEM_QUIET=region` | off | Experimental UCX switches. |
| `LAMELLAR_SHMEM_OPT_EXTERNAL` | set to `1` by the shim for `shmem-opt` | Enables the runtime's `.data/.bss` remap on `shmem-opt`. |
| `SHMEM_REQUIRE_ALLOCATION` | off | `oshrun` refuses to run outside a Slurm allocation. |
| `OSHRUN_LAUNCHER`, `OSHRUN_CORES_PER_PE`, `OSHRUN_MAP_BY`, `OSHRUN_BIND`, `OSHRUN_CPUS` | | Launcher controls for the `srun-pmix` / `prterun` paths (see `wrappers/src/launch.rs`). |
| `SHMEM_PREFIX`, `SHMEM_INCDIR`, `SHMEM_LIBDIR`, `OSHCC_*` | | Where the compiler wrappers look for the library, and which compiler they drive. |

`SHMEM_RUNTIME_REDUCE` and `SHMEM_RUNTIME_BARRIER` are read once per process.

## Supported features

The built `libshmem.so` exports 596 `shmem_*` symbols, and `tests/check_header.sh` verifies that every function declared
in `include/shmem.h` is exported. By area:

* **Setup and queries:** `shmem_init`, `shmem_init_thread` / `shmem_query_thread`, `shmem_finalize`,
  `shmem_global_exit`, `shmem_my_pe`, `shmem_n_pes`, `shmem_pe_accessible`, `shmem_addr_accessible`,
  `shmem_info_get_version` / `shmem_info_get_name`.
* **Symmetric heap:** `shmem_malloc`, `shmem_calloc`, `shmem_align`, `shmem_realloc`, `shmem_free`, `shmem_ptr`.
* **RMA:** typed and sized `put`/`get`, single-element `p`/`g`, strided `iput`/`iget`, and the non-blocking `*_nbi`
  forms, plus `shmem_putmem`/`getmem` (and `_nbi`). Remote targets may be the symmetric heap or, on supporting
  backends, global/static data.
* **Atomics:** the 1.4 typed atomic family (fetch, set, swap, compare-swap, add, increment, and, or, xor and their
  fetch and non-blocking variants). `swap` uses the backends' native swap.
* **Ordering:** `shmem_fence`, `shmem_quiet`.
* **Synchronization:** `shmem_barrier_all`, `shmem_sync_all`, `shmem_barrier`, `shmem_sync` (active sets),
  point-to-point `shmem_wait_until` and `shmem_test` families, and distributed locks
  (`shmem_set_lock`, `shmem_clear_lock`, `shmem_test_lock`).
* **Collectives (legacy active-set API):** `shmem_broadcast32/64`, `shmem_collect32/64`, `shmem_fcollect32/64`, and the
  `*_to_all` reductions (`and`, `or`, `xor`, `max`, `min`, `sum`, `prod`) for the integer and `float`/`double` types.
* **Threading:** `SHMEM_THREAD_*` levels are accepted. The default executor is Lamellar's `single_thread`, so no
  background worker threads run next to the application.

### How collectives are served

* Whole-team `broadcast` and `fcollect` with a symmetric-heap source go to the runtime's collectives.
* Whole-team `*_to_all` reductions with a symmetric-heap source go to the runtime's allreduce (on by default;
  `SHMEM_RUNTIME_REDUCE=0` restores the old fold).
* Whole-team `shmem_barrier`/`shmem_sync` go to the runtime barrier (on by default; `SHMEM_RUNTIME_BARRIER=0` restores
  the shim's dissemination barrier). `shmem_barrier_all` always uses the runtime barrier.
* Everything else (active-set subsets, static/global sources, `collect`) falls back to the shim's pull-based
  implementation: an entry barrier, then each member reads what it needs from the others.

## Current limitations

**API coverage**

* Not implemented: `shmem_alltoall*` / `shmem_alltoalls*` (part of 1.4), complex and `long double` reductions,
  `long double` types generally, and everything added in OpenSHMEM 1.5: teams (`shmem_team_*`), contexts
  (`shmem_ctx_*`), signalling operations, `test_all`/`wait_until_all` style multi-variable waits, `*_with_hints`
  allocation, and the `*mem` collective forms. The header says version 1.4.
* No Fortran wrappers.
* Subset collectives (active sets that are not the whole team) never use runtime collectives; they use the slower
  pull-based fallback, because the shim has no runtime team for an arbitrary `(start, stride, size)`.

**Backends**

* The original `shmem` backend cannot register globals, and with the runtime reduce on (the default) a float reduction
  (`float`/`double` `*_to_all`) panics there: its runtime allreduce is built on integer-only atomics
  (`lamellar-runtime/src/lamellae/comm/atomic.rs`). Set `SHMEM_RUNTIME_REDUCE=0` for that backend. The original
  `libfabric` and `ucx` backends have not been checked with the runtime reduce.
* `shmem-opt` global-data caveats: the program's `.data/.bss` is copied to a per-PE file in `/dev/shm` and remapped
  `MAP_SHARED` over the original range, so (a) transparent huge pages are lost for that range, (b) a `fork`ed child
  shares the range with its parent, (c) writes to a global between the copy and the remap can be lost, (d) a failed
  remap destroys the mapping and aborts, (e) statically linking `libshmem.a` puts runtime statics inside the range,
  (f) the registered start must be page aligned, and (g) `/dev/shm` usage grows by the segment size per PE.
* `libfabric-sys-opt` collectives depend on libfabric 1.22's collective provider, which has two known bugs (below).
* **Static linking** (`shmemcc --shmem-static`) works for `shmem` and `shmem-opt`. For the UCX and libfabric backends the
  vendored static archives (UCX, UCC, libfabric) that `libshmem.a` needs are not installed by `cargo cinstall`, so a static
  application link from an installed prefix does not work. The shared `libshmem.so` needs `libfabric.so` next to it for the
  libfabric backends, because libfabric's static objects carry versioned symbols that cannot be linked into a shared library.

**Performance** (relative to Open MPI's OSHMEM on the same InfiniBand cluster; see [Performance](#performance))

* The libfabric collectives scale linearly with PE count. The runtime barrier is slower than the shim's own dissemination
  barrier. Both are runtime items.
* Non-fetching atomics (`add`, `inc`) wait for completion on the network backends, unlike OSHMEM, which returns at once.
  Strided `iput`/`iget` are far slower than OSHMEM.
* On `libfabric-sys-opt`, put and get between ordinary memory and the heap are staged through the runtime's registered
  pool from 512 B up (the transfer size at which libfabric stops injecting). This protects against libfabric's
  registration cache going stale when a heap range is recycled, which can land a get in old physical pages. Staging
  blocks are recycled, and `*_nbi` operations stay asynchronous (the block is held until the next `shmem_quiet`). The
  copy is within about 5% of letting the provider register the buffer directly up to 4 KiB; at 64 KiB a put is about
  13% and a get about 32% slower, and from 1 MiB a put is 20% to 40% slower and a get about 1.9x slower (1 MiB get 169 us
  versus 90 us, 4 MiB get 659 us versus 345 us; two nodes). `LAMELLAR_RDMA_STAGING=0` restores the faster path without
  the protection. `ucx-opt` is not affected.

**Other**

* The symmetric heap size is fixed at `shmem_init`.
* Heap-size settings interact: `LAMELLAR_HEAP_SIZE` must exceed `SHMEM_SYMMETRIC_SIZE` plus headroom.

## Performance

Compared with Open MPI's OSHMEM (5.0.2 on UCX) on one two-node InfiniBand cluster, medians of repeated runs; these are
indicative, since fabric state swings results by several times between jobs (compare variants inside one allocation):

* **Point-to-point, inter-node:** 8 B put and get latency is about 1.3 to 1.5x OSHMEM's on `ucx-opt` and `libfabric-sys-opt`;
  large transfers (64 KiB and up) are on par, and `libfabric-sys-opt` reaches OSHMEM's bandwidth at 64 KiB. 8 B put message
  rate is about 0.6x.
* **Atomics, inter-node:** fetching atomics are about 1.5x OSHMEM's latency. `swap` was about 3x before it moved to the native swap, which halved its
  latency (5.8 to 2.8 us on `ucx-opt`, 8.4 to 4.6 us on `libfabric-sys-opt`); it has not been re-measured against OSHMEM.
* **Collectives, 64 PEs on 2 nodes:** `ucx-opt` broadcast and fcollect are within 1.5 to 2.5x of OSHMEM, `libfabric-sys-opt` is
  about 50x to 80x slower, and the barrier and `sum_to_all` are slower on both. Routing `sum_to_all` through the runtime
  was 4 to 20x faster than the shim's own fold.
* **Single node:** `shmem-opt` is far ahead of OSHMEM on blocking atomics and contended fetch-add, behind it on small
  put/get latency and 8 B put message rate, and on par for large transfers.

## Testing and benchmarking

Tests are CMake/CTest programs built against an installed copy of the library, so they exercise the same install and
wrappers users get. Stage a backend and run its tests (the run needs a Slurm allocation, or a machine where `oshrun` can
start the processes):

```bash
tests/stage.sh ucx-opt                                   # install into target/stage/ucx-opt and build the tests
ctest --test-dir target/tests-build/ucx-opt -L single-node --output-on-failure
ctest --test-dir target/tests-build/ucx-opt -L multinode   --output-on-failure   # >= 2 nodes, network backends
ctest --test-dir target/tests-build/ucx-opt -L aslr        --output-on-failure   # PIE / RELRO data-segment layouts
```

`tests/stage.sh` takes `shmem`, `shmem-opt`, `ucx`, `ucx-opt`, `ucx-shared`, `libfabric` or `libfabric-sys-opt`. The tests
are the programs in `tests/c` and `tests/cxx` (ring, full API coverage, globals, bulk transfers, threads, non-blocking,
atomicity, progress, C++, and a reduction-correctness check); a test passes when the launcher exits 0, the success marker
appears once per PE, and no failure text appears (`tests/cmake/expect.cmake`).

Other checks: `tests/check_header.sh` (every declared function is exported), `tests/toolchain.sh` (installs into a
temporary prefix and builds and runs programs through the wrappers, static link, pkg-config and CMake;
`FEATURES=<backend features>` selects the backend), and `make check` runs both.

`tests/bench/shmem_bench.c` is a micro-benchmark that uses only the public API, so the same source builds against other
OpenSHMEM implementations (for example Open MPI's OSHMEM) for comparison:

```bash
oshrun -N 2 --pes-per-node 1 ./shmem_bench pt2pt amo        # sections: pt2pt amo contend barrier coll ops multi
```

PE 0 prints `CSV,bench,size_bytes,np,mem,p50_us,p99_us,mean_us,min_us,bw_MBps,rate_Mops` lines;
`BENCH_QUICK=1` shortens a run and `BENCH_NO_GLOBAL=1` skips targets in static memory. `tests/bench/summarize_bench.py`
summarizes the CSV output.

## Runtime changes

The shim depends on the following changes in `lamellar-runtime` (branch `openshmem-ffi-dev`, based on `origin/dev`).

**Already made**

* The public `SharedMemoryRegion` API the shim is built on (all in `memregion/shared.rs`):
  * `atomic_{add,sub,mul,bit_and,bit_or,bit_xor}` and their `atomic_fetch_*` forms, `atomic_compare_exchange`, and
    `_blocking` / `_unmanaged` variants of every atomic. Atomics run on the NIC where the backend has them.
  * `try_cast` / `cast`: view a region as another element type, checking that the byte offset and length are multiples
    of the new element size and that the allocation is aligned for it. The shim allocates its heap as `u64` words and
    views it as bytes and as `u32` / `u64`.
  * `put_ptr_blocking`, `put_ptr_unmanaged`, `get_into_ptr_blocking`, `get_into_ptr_unmanaged`: RMA to and from any local
    pointer. The local side is passed as an *unregistered* input (`MemregionRdmaInputInner::Unregistered`,
    `UnregisteredSlice`), so backends that need a registered source stage it; `CommSlice` is documented as
    registered-memory-only.
  * `try_register_external_shared_mem_region` (on `RemoteMemoryRegion`): collectively register caller-owned memory such
    as `.data/.bss` and get an ordinary `SharedMemoryRegion` back.
* `MemoryRegion`'s put, get, atomic and compare-exchange methods are generic over the element type, so
  `SharedMemoryRegion` and `OneSidedMemoryRegion` no longer clone the region with `as_base` on every call (about 20-25 ns
  per operation, measured on `shmem-opt`).
* `register_external` (register caller-owned memory such as `.data/.bss` for remote access): a default in `CommMem`
  that returns `Unsupported`, with implementations on `libfabric`, `ucx`, `ucx-opt`, `libfabric-sys-opt` and `shmem-opt`.
  On `shmem-opt` it remaps the range onto a shared file (`shmem_opt_lamellae/external.rs`).
* `shmem_opt_lamellae/rdma.rs`: a 4 KiB-aliasing-aware copy (`copy_bytes`) for remote reads. Without it a 64 KiB+ heap
  get from a `malloc`'d buffer ran at 3.5 GB/s instead of about 50 GB/s whenever the destination was a few bytes past a
  page boundary relative to the page-aligned source.
* `libfabric_sys_opt_lamellae/fabric.rs::broadcast_inner`: broadcasts whose element count is not a multiple of the group
  size go through a zero-padded bounce buffer, because libfabric's collective provider writes `ceil(count/ranks)*ranks`
  elements into the caller's buffer.

**In the vendored libfabric** (`communication_frameworks/libfabric-src/libfabric`)

* `prov/rxm/src/rxm_cq.c`: RxM only completed collective-provider (`FI_PEER_TRANSFER`) transfers through the provider's
  callback on the eager path, so any collective message above the 16 KiB eager limit hung. The patch routes SAR and
  rendezvous sends and receives through the callback. It carries a prototype switch, `RXM_COLL_PEER_PATCH=0`, that
  restores the original behaviour; remove it before anything permanent. The same two gaps are present in upstream main
  (checked 2026-10-01), so this is also an upstream report.

**Needed or proposed (not done)**

1. **Collectives implemented in the runtime on one-sided primitives**, with algorithm choice by rank count, node layout
   and message size, taken from UCC and Open MPI's tuned/han components: dissemination or hierarchical barrier,
   k-nomial broadcast, recursive-doubling/Bruck allgather, k-nomial and SRA allreduce, with libfabric's own collectives
   kept only for large messages. Motivation: libfabric's provider scales linearly in PE count and has the two bugs above.
2. **A faster runtime barrier**: node-leader hierarchy with a shared-memory node barrier.
3. **A true all-to-all collective.** The runtime's `alltoall` is an all-gather on at least `ucx_opt` and `shmem_opt`
   (every PE contributes the same `len` elements) and the libfabric version was not verified (documented on
   `CommAllocCollectiveAllToAll` in `comm/collective.rs`). `shmem_alltoall*` / `shmem_alltoalls*` stay unimplemented in the
   shim until the runtime has a personalized exchange (source `len * num_pes`, block `j` to PE `j`) that behaves the same on
   every lamellae.
4. **Float reductions on the original `shmem` lamellae**, or retire that lamellae: `typed_atomic_fetch_op` in
   `comm/atomic.rs` has no `f32`/`f64`; `shmem-opt` solves it with a host fold in `local_reduce`.
5. **Non-blocking-completion semantics for `add`/`inc`** on the network lamellae, to close the AMO gap to OSHMEM.
6. **Runtime teams for arbitrary PE subsets** (cached per PE list) so OpenSHMEM active sets and, later, 1.5 teams can use
   runtime collectives.
7. **Error reporting for collective failures** instead of silent spins: the libfabric-sys-opt wait loops should surface
   a failed collective with a message and a timeout.
8. **Strided RMA (`iput`/`iget`) as a native runtime operation** instead of per-element transfers.
9. Upstream the two libfabric fixes.
10. **Feature-gate the LamellarArray code.** The shim uses only memory regions, but the runtime always
    compiles the array module (about 85k lines), which adds build time and code the shim never reaches. Gating it behind a
    Cargo feature (off for the shim) would shrink the build. The main coupling is that the memory-region code imports a few
    array traits from `src/array.rs`, which would have to move out first. Arrays such as `UnsafeArray` were evaluated as a
    foundation for the shim and offer nothing it needs: they sit on top of memory regions, do arithmetic and reductions
    through active messages, and expose no raw pointer.
11. **Static libfabric inside `libshmem.so`.** Building libfabric without symbol versioning would let its static objects link
    into the shared library, removing the need to ship `libfabric.so` next to it. Installing the vendored static archives
    would also make `--shmem-static` work for the UCX and libfabric backends.

## Future plans

* Fill the API gaps: `shmem_alltoall*`, complex and `long double` reductions (Rust has no `long double`; an opaque 16-byte value
  with the arithmetic in a small C helper would do), then the 1.5 features (teams, contexts, signalling ops,
  multi-variable waits, `*_with_hints`, `*mem` collectives).
* Route active-set subsets and teams through runtime collectives once the runtime has subset teams.
* Land the runtime collectives layer above and re-measure against OSHMEM: target within 2x at 64 PEs for 8 B barrier,
  broadcast, fcollect and `sum_to_all`.
* Fix the AMO gaps and strided RMA, then re-measure the four optimized lamellae.
* Check the original `libfabric`/`ucx` backends with the runtime reduce and barrier defaults, or retire them.
* A single-node PIE/ASLR test for `shmem-opt`, and a measurement of the huge-page cost of the `.data/.bss` remap on
  large static arrays.

## Repository layout

```
src/        the shim: state.rs (init, heap, data segment, progress), rma.rs, amo.rs, coll.rs, sync.rs, heap.rs
include/    gen_shmem_h.py generates shmem.h (checked in)
assets/     shmem.h as cargo-c installs it
cmake/      the CMake package installed by cargo cinstall
wrappers/   a separate small crate: shmemcc, oshcc, shmemcxx, oshcxx (compiler wrappers); oshrun, shmemrun (launcher)
tests/      CMakeLists.txt and c/, cxx/ programs; stage.sh (install a backend and build the tests); bench/ micro-benchmark
tools/      helpers that copy libfabric.so / UCX libraries next to libshmem.so, and build the vendored PRRTE
Makefile    wrappers, header, prrte and check targets
```
