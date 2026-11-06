# randperf

Benchmarks for RDMA-style `put`/`get`/`swap` operations, comparing the
[Lamellar](https://github.com/pnnl/lamellar-runtime) runtime against direct
OpenSHMEM, over either **libfabric** or **UCX** as the network transport.

## Use it

```bash
./randperf build                 # build all binaries (release profile; static except Open MPI; --ucx / --libfabric build only that network's OpenSHMEM backends)
./randperf run --fabric libfabric-sys-opt --osh sos --nodes 4     # submit the sweep with sbatch; results go to results_dd_mm_yy/
./randperf plots                 # redraw the charts of the newest results directory
```

`./randperf run` submits itself as a Slurm job (`--time MIN`, default 240 minutes) and prints the job id; inside an allocation it runs directly.
`./randperf run --help` lists the options. `./randperf` only calls `tools/build-binaries.sh`, `hpc_driver.sh` and `tools/stage_runtime.py`.

The measurements of the run of 2026-10-26 to 2026-10-29 (1 to 32 nodes) are in `junction_results_26_10_26/`; its `README.md` lists the versions,
settings and layout, `tables.md` has the numbers, and `python3 verify_results.py junction_results_26_10_26 --iters 25` re-checks them. `./randperf plots junction_results_26_10_26` redraws
its charts (`./randperf plots` without an argument looks for the newest `results_dd_mm_yy` directory, which is how new runs are named).


## 1. Overview

Workspace layout:

- **`randperf-core`** — shared library: `TestWorld`/`TestArray`/`OperationTest`
  traits and the `run_optest` harness. Each timed iteration prints a line of
  the form `FINAL TIME: <seconds> seconds ...` to stdout — this is the line
  every tool in this repo parses to collect timing samples.
- **`randperf-lamellar`** — benchmark binary built on the Lamellar runtime
  (`#[lamellar::main]`). Cargo features compile Lamellar lamellae backends in (`ucx`, `ucx-opt`, `libfabric`, `libfabric-sys`,
  `libfabric-sys-opt`, `rofi-c`, `shmem-opt`; the single-node `shmem` lamellae is always there); several can be compiled into one binary
  and the one to use is chosen at *run time* with `LAMELLAR_BACKEND` (`rofi-c` is run as `rofi_c`).
- **`randperf-shmem`** — benchmark binary built directly on OpenSHMEM
  (`#[hpc_launch::main]`, which launches the PEs itself through `prterun`).
  Selects its OpenSHMEM implementation + transport at build time via a
  Cargo feature on `randperf-shmem` (which forwards to `shmem-shim`).
- **`shmem-shim`** — C shim + `bindgen` wrapper exposing OpenSHMEM to Rust.
  Depending on the feature selected, it either builds against a
  system-installed OpenSHMEM or vendors and builds one from source:
  - **`sos-src`** — builds Sandia OpenSHMEM (SOS) from the vendored `SOS/` tree.
  - **`ompi-src`** — builds OpenMPI's OSHMEM from the vendored `ompi/` tree.

  These three crates live in `openshmem-rs/` (see the self-containment section below).

## Self-containment (`openshmem-rs/`)

Everything OpenSHMEM-specific is vendored in the workspace:

- `openshmem-rs/shmem-shim`: the bindgen shim exposing the SHMEM API (`sos-ofi`, `sos-ucx`, `ompi-ucx`, `hpcx-ucx` and
  `lamellar-openshmem` features). Its `build.rs` adds `SHMEM_CMP_*` aliases when the header declares them as `enum shmem_wait_ops` (Open MPI).
- `openshmem-rs/sos-src`: the vendored Sandia OpenSHMEM (SOS), including the UCX completion fix. Over libfabric it links libfabric statically.
- `openshmem-rs/ompi-src`: vendored Open MPI/OSHMEM, used only by `ompi-ucx`. Its `ucx` feature builds UCX as shared
  libraries and passes the UCX link libraries to Open MPI's configure through `LIBS`.

Lamellar and its helper crates are path dependencies on the sibling crates in this repository: the root `Cargo.toml` names `lamellar`
(`lamellar-runtime/`) and `hpc_launch` there, `shmem-shim` uses `lamellar-openshmem/`, and `[patch.crates-io]` points the PMIx, PRRTE, UCX,
UCC, libfabric and ROFI `-sys` / `-src` crates at `process_management/` and `communication_frameworks/`. The results in
`junction_results_26_10_26/` were produced with Lamellar commit `6f56669ce582deab03920ec8d206e9f6a1486bb6`; check that commit out to rebuild them.
The Lamellar commit a binary was built from is recorded in its `builds/bin/<binary>.provenance`.

## lamellar-openshmem backend (`lamellar-openshmem` feature)

`randperf-shmem` can also run on **lamellar-openshmem**, an OpenSHMEM 1.4 implementation on the Lamellar runtime. It is a feature of
`shmem-shim` (forwarded by `randperf-shmem/lamellar-openshmem`), not a separate crate: the real `lamellar-openshmem` crate from
the Lamellar git repository is an ordinary dependency (lamellae `ucx`, `ucx-opt`, `shmem-opt`, `libfabric-sys-opt` and `rofi_c` compiled in), its `shmem_*`
symbols are linked in with `extern crate shmem as _;`, and the bindings are generated from a vendored copy of its `shmem.h`
(`openshmem-rs/shmem-shim/c-src/lamellar-openshmem/shmem.h`; refresh it from `lamellar-openshmem/include/shmem.h` of the Lamellar repository if that
changes). There is no prefix to build and no `cargo-c`. The lamellae are chosen at run time with `LAMELLAR_BACKEND`; the symmetric heap
size comes from `SHMEM_SYMMETRIC_SIZE` (plain bytes). In `hpc_driver.sh` this is `--osh lamellar-openshmem`, supported for `--fabric ucx|ucx-opt|libfabric|libfabric-sys|libfabric-sys-opt|rofi_c|shmem|shmem-opt` (shmem and shmem-opt on one node only).

## 2. Prerequisites

- A Rust toolchain (stable) and `cargo`, network access for the first build (git dependencies), `patchelf` (used by
  `tools/stage_runtime.py` for Open MPI).
- A Slurm cluster with `sbatch` (the driver submits itself) and PMIx set up (PMIx is a hard dependency of the vendored SOS/OMPI builds).
- InfiniBand/verbs hardware. UCX, libfabric, PMIx, PRRTE, SOS and ROFI are all built from the vendored or git sources and linked
  statically; the usual autotools toolchain (`autoconf`, `automake`, `libtool`) plus a working C compiler must be on `PATH` for that.

### Static by default, system libraries on request

A default build is self-contained: UCX, libfabric, PMIx, PRRTE, ROFI and SOS are compiled from source and linked statically, so a binary
needs no fabric library at run time (`ldd` shows only the system libc and friends). To use an installation you already have, set one of
these variables for `cargo build` (they are read by the build scripts of the dependencies; a changed variable rebuilds that part):

| Variable | Uses the system version of | Read by |
|---|---|---|
| `UCX_DIR=<prefix>` | UCX (needs `lib/libucp`, `libucs`, `libuct`, `libucm`; static `.a` is preferred, a shared install is linked dynamically) | `lamellar-ucx-sys` |
| `OFI_DIR=<prefix>` | libfabric (`lib/`, `include/`) | `lamellar-libfabric-src` |
| `ROFI_DIR=<prefix>` | ROFI (C), the library behind the `rofi_c` lamellae | `rofisys` |
| `SOS_DIR=<prefix>` | an installed Sandia OpenSHMEM instead of building `openshmem-rs/sos-src` | `sos-src` |
| `PMIX_NO_VENDORED=1` `PMIX_LIB_DIR=… PMIX_INCLUDE_DIR=…` | PMIx | `pmix-sys` |
| `PRRTE_NO_VENDORED=1` `PRRTE_LIB_DIR=… PRRTE_INCLUDE_DIR=… PRRTE_BIN_DIR=…` | PRRTE (the launcher) | `prrte-sys` |
| `LIBEVENT_DIR`, `HWLOC_DIR` | libevent / hwloc for the vendored PMIx, PRRTE and SOS builds | `pmix-sys`, `openpmix-src`, `prrte-src`, `sos-src` |
| `OMPI_DIR=<prefix>` with feature `ompi-system` | an installed Open MPI OSHMEM (`bin/`, `lib/liboshmem`, `include/`); nothing is compiled, `liboshmem` is linked from there | `shmem-shim` |
| `HPCX_UCX_DIR`, `HPCX_OSHMEM_DIR` with feature `hpcx-ucx` (needs `PMIX_NO_VENDORED=1`) | the UCX and OSHMEM of an HPC-X installation (**experimental**) | `shmem-shim` |

`ompi-system` is opt-in: `OMPI_DIR=<prefix> cargo build --release -p randperf-shmem --features ompi-system` (the default `ompi-ucx` feature
builds the vendored Open MPI). A system library that is only available as a shared object makes that binary dynamic, so keep its prefix
visible on the compute nodes, and `tools/build-binaries.sh`'s static check does not apply to it. The system-library switches come from the
dependencies and have not been exercised for these benchmarks; the default static build is the one the results are produced with.

## 3. Supported backend/fabric matrix

| Backend | Fabric | `randperf-lamellar` feature | `randperf-shmem`/`shmem-shim` feature |
|---|---|---|---|
| Lamellar | libfabric | `libfabric` | — |
| Lamellar | libfabric-sys | `libfabric-sys` | — |
| Lamellar | libfabric-sys-opt | `libfabric-sys-opt` | — |
| Lamellar | UCX | `ucx` | — |
| Lamellar | UCX (optimised lamellae) | `ucx-opt` | — |
| Lamellar | rofi_c (ROFI over libfabric, PMIx) | `rofi-c` (= `lamellar/enable-rofi-c-pmix`) | — |
| Lamellar | single node | `shmem-opt` (`shmem` is always there) | — |
| lamellar-openshmem | ucx, ucx-opt, shmem-opt | — | `lamellar-openshmem-ucx` |
| lamellar-openshmem | libfabric, libfabric-sys, libfabric-sys-opt, rofi_c, shmem-opt | — | `lamellar-openshmem-libfabric` |
| Shmem (Sandia OpenSHMEM) | libfabric / libfabric-sys / libfabric-sys-opt | *(built alongside, see note below)* | `sos-ofi` |
| Shmem (Sandia OpenSHMEM) | UCX | *(built alongside, see note below)* | `sos-ucx` |
| Shmem (OpenMPI OSHMEM) | UCX | *(built alongside, see note below)* | `ompi-ucx` |
| Shmem (OpenMPI OSHMEM) | libfabric / libfabric-sys / libfabric-sys-opt | **not supported** | — |

`rofi_c` runs with `--osh none` (native Lamellar) and `--osh lamellar-openshmem`; `sos` and `ompi` have no rofi_c variant, so with them only
the Lamellar leg runs. Its results are stored with the libfabric lamellae (`run_libfabric_<n>.json`) and charted with them.

Open MPI OSHMEM has no atomic-array `put` row: its `shmem_atomic_set` issues a UCX swap without a reply buffer, which UCX 1.20 cannot execute
on this hardware (it aborts with "'abort' is not implemented for protocol amo64/post"). The driver skips that row; the plain `put` and the
atomic `get` and `swap` are measured. SOS implements `atomic_set` as a fetching swap and lamellar-openshmem is not affected.

OMPI+libfabric has no wired feature anywhere in the workspace (see
`openshmem-rs/shmem-shim/Cargo.toml`) — don't try to combine `--osh ompi` with
`--fabric libfabric`/`libfabric-sys`/`libfabric-sys-opt`.

The three libfabric variants are alternate Lamellar lamellae backends (the `LAMELLAR_BACKEND` values `libfabric`,
`libfabric-sys`, `libfabric-sys-opt`), not different network fabrics — the SOS
shmem leg's OFI transport is the same regardless of which one is selected, so
they all map to the single `sos-ofi` feature above. Because they're
cfg-additive in the runtime (not mutually exclusive), all three can be
compiled into the same binary and picked between at *run time* via
`LAMELLAR_BACKEND` — that's what `hpc_driver.sh` does (see below), so a single
build serves all three and switching between them never forces a rebuild.

## 4. Quick start: build once, then run

```bash
tools/build-binaries.sh            # every variant (or name some: lamellar sos-ucx sos-ofi ompi-ucx lamellar-openshmem)
tools/build-binaries.sh --ucx      # only the UCX family: sos-ucx, ompi-ucx, lamellar-openshmem with ucx + ucx-opt
tools/build-binaries.sh --libfabric  # only the libfabric family: sos-ofi, lamellar-openshmem with libfabric, libfabric-sys, libfabric-sys-opt, rofi_c
# --ucx and --libfabric add up (both if neither is given); both shmem lamellae are always built
RANDPERF_PROFILE=release-dev tools/build-binaries.sh lamellar   # a faster (thin LTO) smoke build; the default is release
./hpc_driver.sh --fabric ucx --osh sos --nodes 1 --save results_dd_mm_yy
```

Build on the login node; the driver never builds on a compute node. `tools/build-binaries.sh` builds each variant into the workspace's
single target directory with the cargo profile `RANDPERF_PROFILE` (default **`release`**: fat LTO, the profile every measurement uses;
`release-dev` has thin LTO and builds faster, use it for smoke builds only), copies the binary (found from cargo's own output, so wherever
cargo put it) to `builds/bin/` with a `.sha256` and a `.provenance` file (date, **profile**, Lamellar commit, features), and checks with `ldd`
that it needs no libfabric, UCX, PMIx or PRRTE shared library. Both profiles write the same `builds/bin` names, so the profile in the
`.provenance` file is how a binary is told apart; `hpc_driver.sh` warns about a binary that is not a release build. Everything is static
except Open MPI (`ompi-ucx`: shared UCX and Open MPI libraries, which are staged next to the binaries, see below). The binaries run with
`LD_LIBRARY_PATH` unset. One `randperf-lamellar` holds every lamellae backend (`ucx`, `ucx-opt`, `libfabric`, `libfabric-sys`,
`libfabric-sys-opt`, `rofi_c`, `shmem-opt`), chosen at run time with `LAMELLAR_BACKEND`; the `randperf-shmem-*` binaries are separate because
their OpenSHMEM implementations are mutually exclusive (`randperf-shmem-lamellar-openshmem` is lamellar-openshmem).

Build directory: the intermediates (the C builds of SOS, UCX, libfabric, PMIx, ROFI, Open MPI) are large. By default cargo builds in `target/`. To
put them on node-local disk, uncomment the `build-dir` line in `.cargo/config.toml` or run
`RANDPERF_BUILD_DIR=/scratch/you/randperf_build tools/build-binaries.sh` (or set `CARGO_BUILD_BUILD_DIR`). Final binaries are copied to `builds/bin`
on shared disk. If the build directory is node-local, compute nodes cannot see it, so `tools/stage_runtime.py` (run by `tools/build-binaries.sh`;
repeatable) copies what the runs need out of the build directory into `builds/bin/runtime/`:

- `runtime/prrte/`: the PRRTE launcher with its PMIx/PRRTE data files. `hpc_driver.sh` starts the binaries with `LAMELLAR_PRTERUN`
  pointing there; set `LAMELLAR_PRTERUN` (full path) or `PRRTE_BIN_DIR` yourself to use another launcher.
- `runtime/ompi/`: the vendored Open MPI install tree (`lib`, `share`, `etc`), the UCX it was built against and its `build/config.log`.
  The RPATHs are rewritten with `patchelf` (`$ORIGIN` for the libraries, `$ORIGIN/..` for the plugin directories, and
  `$ORIGIN/runtime/ompi/lib` for `randperf-shmem-ompi-ucx`, which sits in `builds/bin`), so the tree is relocatable. For `--osh ompi`,
  `hpc_driver.sh` sets `OPAL_PREFIX=builds/bin/runtime/ompi` (forwarded to the ranks) and uses Open MPI's own `prterun` if its build
  directory is visible on the node, otherwise the staged PRRTE. A system Open MPI (`ompi-system`) is left where it is; running the
  staging again changes nothing.

`hpc_driver.sh` never builds unless asked (`--build`): it stops with a message if a binary or the staged launcher is missing. Use
`RANDPERF_ITERS` (the driver's `--iters`, default 25) for shorter smoke tests.

### Smoke tests

Before a long sweep, run each (fabric, osh) you care about as a short job: 2 nodes, a few PEs-per-node values, 3 iterations, a short time limit,
results into a throwaway directory:

```bash
./randperf run --fabric rofi_c --osh none --nodes 2 --ns "1 2" --iters 3 --time 30 --save results_smoke
```

and check that every row produced timing samples (`results_smoke/nodes_2/run_*.json`, no `outputs/.../failed_*.log`).

## 4b. Build and run directly with cargo

The driver is optional: the binaries can be built and started by hand. The workspace has three profiles (root `Cargo.toml`):

| Profile | Use |
|---|---|
| `release` | **measurements**: opt-level 3, fat LTO, one codegen unit (slow to link, fastest binaries) |
| `release-dev` | smoke builds: thin LTO, 16 codegen units |
| `release-debug` | `release-dev` plus debug info, for profiling and debugging |

```bash
# randperf-lamellar with every lamellae backend compiled in -- pick one at run time with LAMELLAR_BACKEND
cargo build --profile release -p randperf-lamellar --features ucx,ucx-opt,libfabric,libfabric-sys,libfabric-sys-opt,rofi-c,shmem-opt
# randperf-shmem: Sandia OpenSHMEM over UCX / over libfabric, Open MPI OSHMEM over UCX, lamellar-openshmem (one at a time)
cargo build --profile release -p randperf-shmem --features sos-ucx
cargo build --profile release -p randperf-shmem --features sos-ofi
cargo build --profile release -p randperf-shmem --features ompi-ucx
cargo build --profile release -p randperf-shmem --features lamellar-openshmem   # ucx, ucx-opt, shmem-opt, libfabric-sys-opt, rofi_c, ...
```

The `randperf-shmem` builds all write the same `randperf-shmem` file, so copy it away after each (this is what
`tools/build-binaries.sh` does). Run `python3 tools/stage_runtime.py` afterwards to stage the launcher and Open MPI for compute nodes.
Inside an allocation a build is started by hand like this (the launch options after `--` are described in section 5):

```bash
LAMELLAR_BACKEND=rofi_c UCX_MEM_EVENTS=no LAMELLAR_EXECUTOR=single_thread LAMELLAR_HEAP_SIZE=1073741824 LAMELLAR_HEAP_MODE=static \
  LAMELLAR_PRTERUN=builds/bin/runtime/prrte/bin/prterun \
  target/release/randperf-lamellar --op put --array-type lamellar -- --nodes 2 --pes 4 --threads-per-pe 4
```

## 5. Running manually

Both binaries share the same benchmark CLI:

```bash
<binary> --op <put|get|swap> --array-type <array-type> [--scale-nodes N] -- --nodes N --pes N --threads-per-pe N
```

- `--op` / `--array-type` are consumed by `randperf-core`'s CLI (see
  `randperf-lamellar/src/main.rs` / `randperf-shmem/src/main.rs` for the
  exact `ArrayType` variants each binary accepts, e.g. `lamellar`,
  `lamellar-atomic`, `shmem`, `shmem-atomic`).
- `--scale-nodes N` sets the array size: every PE holds `100000 * N` elements. N is the number of **nodes**, not the number of PEs, and it
  must be the same on every PE; it is part of the command line, which the launcher hands to all ranks unchanged. `hpc_driver.sh` passes it.
  Without it the benchmark falls back to `$SLURM_NNODES` and prints a warning, which is unsafe when the launcher itself runs inside a batch
  job (the rank on the launcher's node and the other ranks can see different values).
- Everything after the `--` is consumed by `hpc_launch`, which starts the PEs through `prterun`
  (and asks for an `salloc` allocation itself if one isn't already active and `--nodes` is given).

Required environment variables:

Always:

```bash
LAMELLAR_ROFI_PROVIDER=verbs
SMA_OFI_PROVIDER=verbs
OMPI_MCA_btl='^sm'
```

`randperf-lamellar` only (the Lamellar runtime picks its transport at
*runtime*, not build time, via `LAMELLAR_BACKEND`):

```bash
LAMELLAR_BACKEND=libfabric-sys-opt   # or: ucx, ucx-opt, libfabric, libfabric-sys, rofi_c, shmem-opt, shmem
LAMELLAR_PRTERUN=<builds/bin/runtime/prrte/bin/prterun>   # the staged launcher (hpc_driver.sh sets it)
LAMELLAR_EXECUTOR=single_thread
LAMELLAR_HEAP_SIZE=1073741824
LAMELLAR_HEAP_MODE=static
```

`randperf-shmem` needs none of the `LAMELLAR_*` vars except `LAMELLAR_PRTERUN` (and, for lamellar-openshmem,
`LAMELLAR_BACKEND` and `SHMEM_SYMMETRIC_SIZE` in plain bytes) — its OpenSHMEM implementation and transport are both fixed at
build time by the Cargo feature you built it with. SOS over libfabric needs `SMA_OFI_PROVIDER=verbs`; libfabric and rofi_c rows with UCX
also compiled in need `UCX_MEM_EVENTS=no` (the driver sets it for `libfabric*` and `rofi_c`, native and lamellar-openshmem, and passes it
to the ranks with `prterun -x`). The Open MPI binary needs `OPAL_PREFIX=builds/bin/runtime/ompi` (also set and forwarded by the driver).
Do not set `UCX_TLS` or `LD_LIBRARY_PATH`. OpenSHMEM binaries want 2 cores per PE.

## 6. Using `hpc_driver.sh`

`hpc_driver.sh` automates the full sweep: it submits itself as a Slurm job
(if you're not already inside one), runs the prebuilt binaries (section 4) for every
`(n, op, array-type)` combination across increasing PE counts
(1, 2, 4, 8, 16, 32), and optionally saves results to JSON and renders
charts as it goes. It requires only `bash`, `awk`, and `cargo` — no
`rust-script`, `jq`, or extra one-off crates.

```
Usage: hpc_driver.sh --fabric <ucx|ucx-opt|libfabric|libfabric-sys|libfabric-sys-opt|rofi_c|shmem|shmem-opt>
                     --osh <sos|ompi|lamellar-openshmem|none> --nodes N [options]

Required:
  --fabric F               Lamellar lamellae backend / transport (shmem and shmem-opt: single node only;
                            rofi_c: --osh none or lamellar-openshmem only)
  --osh O                  OpenSHMEM implementation for the randperf-shmem leg (none = Lamellar leg only)
  --nodes N                Number of nodes of the job

Options:
  --pes-per-node N         Override auto-picked threads-per-pe
  --ns "1 2 4 ..."         PEs-per-node values to sweep (default: 1 2 4 8 16 32)
  --iters N                RANDPERF_ITERS for each run (default 25; use fewer to smoke test)
  --bin-dir DIR            Prebuilt binaries (default: builds/bin)
  --ops "put get swap"     Operations to run (default all three)
  --arrays "plain atomic"  Array types to run (default both); with --ops this selects single rows
  --run-timeout MIN        Kill a single run that takes longer than MIN minutes (default 20, 0 = no limit) and record it as failed
  --init-retries N         Retry a run at most N times (default 1) if it died in OpenSHMEM start-up ("SHMEM failed
                            to initialize") before any timing line; each retry is recorded in DIR/retries.tsv. 0 disables.
  --build                  Build any missing binary first (tools/build-binaries.sh), before submitting the job
  --time MIN               Slurm time limit of the job in minutes (default 240; only used when the script submits itself)
  --save DIR               Save results as DIR/nodes_N/run_GROUP_<n>.json (+ run_GROUP.checks.tsv with PE 0's
                            Accum/Check per sample), plot as you go, and append this invocation to
                            DIR/sweep_submissions.tsv (nodes, job id, purpose, minutes, exit code).
                            GROUP is FABRIC, except that ucx/ucx-opt, the three libfabric lamellae and
                            shmem/shmem-opt each share one group and file (rofi_c is part of the libfabric group).
  --plot FILE              Plot a single results JSON file and exit
  --plot-scaling DIR       Plot scaling charts for DIR/nodes_N/run_GROUP and exit
  --plot-dir DIR           Plot per-n charts + scaling charts for DIR/nodes_N/run_GROUP and exit
  --cancel                 scancel all your jobs and exit
  --sbatch-opts ...        Remaining args are passed through to sbatch (for example: --partition foo)
  -h, --help               Show this help
```

Examples:

```bash
# Run the full sweep, save JSON + charts under results/sos/
./hpc_driver.sh --fabric ucx --osh sos --nodes 4 --save results/sos
./hpc_driver.sh --fabric libfabric --osh sos --nodes 4 --save results/sos
./hpc_driver.sh --fabric libfabric-sys --osh sos --nodes 4 --save results/sos
./hpc_driver.sh --fabric libfabric-sys-opt --osh sos --nodes 4 --save results/sos
./hpc_driver.sh --fabric ucx --osh ompi --nodes 4 --save results/ompi
./hpc_driver.sh --fabric rofi_c --osh none --nodes 4 --save results/rofi
./hpc_driver.sh --fabric rofi_c --osh lamellar-openshmem --nodes 4 --save results/rofi

# Re-render charts from previously-saved JSON, without running anything
./hpc_driver.sh --fabric ucx --osh sos --nodes 4 --plot-dir results/sos

# Cancel your own SLURM jobs
./hpc_driver.sh --fabric ucx --osh sos --nodes 4 --cancel
```

If you're not already inside a Slurm job, the script checks its arguments and binaries, then submits itself with

```
sbatch --parsable -N <nodes> --exclusive --no-requeue [--exclude=$RANDPERF_EXCLUDE_NODES] -J bench_<fabric> -t <--time minutes> -o <DIR>/nodes_<N>/slurm/run_%j.slurm.out <this script> <the same arguments>
```

prints the job id and exits; the job runs the same command line inside the allocation. Set `RANDPERF_EXCLUDE_NODES` to a Slurm node list to keep
the jobs off nodes you do not trust; it is empty by default. The environment of the shell that submitted
(loaded modules, conda, `PATH`, …) is exported to the job (`--export=ALL`). The time limit is `--time` (default **240 minutes**; a sweep of
one fabric and one OpenSHMEM library at 1–2 nodes took about 10–30 minutes, larger node counts take longer, so give more time or split the
sweep with `--ns`/`--ops`). The job name has the fabric only, and Slurm options of your own go after `--sbatch-opts`. Without `--save`
the job log goes to `./slurm/`. If you're already inside an allocation (`SLURM_JOB_ID` is set, e.g. under `salloc`), the script uses it
directly and submits nothing.

`--save`/`--plot-scaling`/`--plot-dir` all resolve to
`DIR/nodes_<N>/run_<fabric>` internally, so results from different node
counts and fabrics never collide in the same directory.

If a run produces zero timing samples for a given combination (a crash, a
hang that got killed, etc.), its full command line + captured stdout/stderr
is written to `outputs/nodes_<N>/failed_<total_pes>_<runtime>_<osh>_<fabric>_<op>_<array>.log`
instead of silently being dropped.

## 7. Plotting dependencies

`hpc_driver.sh` (and `plot_results.py`, which it calls) prefer **Python 3
with `matplotlib`** for full-fidelity charts. The interpreter is detected by
checking `python3` then `python` on `PATH`, and each candidate is verified by
running `--version` (must report Python ≥ 3.6) and `import matplotlib` —
the binary's name alone isn't trusted, since `python` may be Python 2 and
`python3` may be a wrapper stub on some systems. If no candidate passes, it
falls back to **`gnuplot`** for reduced-fidelity charts (same
underlying data, output filenames, and axes — the boxplot is a plain
single-color chart; the scaling chart uses the same color scheme as the
Python path). If neither is available, the `--plot*` modes exit with an
install hint; a `--save` sweep warns once and keeps running, saving JSON you
can plot later with `--plot-dir`. During a sweep, a plotting failure is
never fatal — it prints a warning and moves on to the next experiments.

Charts use a colorblind-safe palette with one hue per implementation family: green = native Lamellar,
orange = lamellar-openshmem, blue = SOS, purple = Open MPI, pink = Lamellar over rofi_c (lamellar-openshmem over rofi_c: olive). The
optimised `*-opt` lamellae are the same hue darker (and `libfabric-sys` lighter, to keep the three libfabric lamellae apart). Atomic
arrays are distinguished by hatching (bar charts) or dashed lines with square markers (gnuplot scaling charts), so no distinction relies
on color alone. The legend names are `Lamellar <backend>`, `lamellar-openshmem <backend>`, `SOS` and `Open MPI`.

```bash
# preferred
pip install --user matplotlib

# fallback
sudo yum install gnuplot      # RHEL/CentOS/Rocky
sudo apt-get install gnuplot  # Debian/Ubuntu
```

The results-JSON bookkeeping in `hpc_driver.sh` (saving/loading/checking
for already-run combinations) is hand-rolled in bash — no `jq` or any other
extra tool is required for that part, on either plotting path. The results
JSON files use a fixed, simple layout that only this script's own
`load_results`/`save_results` functions are meant to read and write; avoid
hand-editing them or reformatting them with an external JSON tool.

## 8. Note on prior results

This repository ships with no results or charts. Running `hpc_driver.sh`
with `--save`/`--plot-scaling`/`--plot-dir` will create and populate
whatever directory you point it at.
