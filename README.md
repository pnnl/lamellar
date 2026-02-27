Lamellar - Rust HPC runtime
=================================================

Lamellar is an asynchronous tasking runtime for HPC systems developed in RUST  
(Crates.io: https://crates.io/crates/lamellar)  
(Main runtime repository: https://github.com/pnnl/lamellar-runtime)  

SUMMARY
-------

Lamellar is an investigation of the applicability of the Rust systems programming language for HPC as an alternative to C and C++, with a focus on PGAS approaches.

Lamellar provides several different communication patterns to distributed applications. 
First, Lamellar allows for sending and executing active messages on remote nodes in a distributed environments. 
The runtime supports two forms of active messages:
The first method works with Stable rust and requires the user the register the active message by implementing a runtime exported trait (LamellarAM) and calling a procedural macro (\#[lamellar::am]) on the implementation.
The second method only works on nightly, but allows users to write serializable closures that are transfered and exectued by the runtime without registration 
It also exposes the concept of remote memory regions, i.e. allocations of memory that can read/written into by remote nodes.

This repository is a staging area for various repositories and crates we have developed for use by Lamellar.

SUBMODULES
----------
Runtime
-------
The `lamellar-runtime/` crate (see [lamellar-runtime/README.md](lamellar-runtime/README.md)) implements the core asynchronous tasking runtime, the distributed arrays and Darcs interfaces, and the `local`, `shmem`, and optional `rofi` Lamellae backends. Its README documents the API, executors, and how to configure backends via features or runtime builders.

Communication frameworks
------------------------
Lamellae transport providers live under `communication_frameworks/`. Each subcrate wraps either UCX or OFI in a way that the runtime can consume without exposing C details directly.
#### UCX transport
- `lamellar-ucx-sys/` ([README](communication_frameworks/lamellar-ucx-sys/README.md)) builds the UCX submodule (v1.19.0) or reuses an `UCX_DIR`, generates bindings via `bindgen`, and provides helpers for contiguous datatypes and UCS pointer handling.

#### OFI transports
- `libfabric/` ([README](communication_frameworks/libfabric/README.md)) offers ergonomic Rust types, async executor bindings, and threading helpers over the low-level OFI interface, while gating how the underlying `libfabric-sys` stack is built.
- `libfabric-sys/` ([README](communication_frameworks/libfabric-sys/README.md)) invokes `libfabric-src`, wraps inline functions, and emits `bindgen` bindings for every OFI symbol so downstream crates can link against `libfabric` transparently.
- `libfabric-src/` ([README](communication_frameworks/libfabric-src/README.md)) copies the vendored `libfabric` tree into `OUT_DIR`, runs Autotools to build shared or static artifacts (honoring `OFI_DIR` if set), and exposes the include/lib directories.

#### ROFI transport
- `rofi/` ([README](communication_frameworks/rofi/README.md)) is the Rust OFI transport layer (libfabric-based) with autotools-driven configure scripts, RDMA PUT/GET APIs, and provider selection for verbs/tcp.
- `rofi-sys/` ([README](communication_frameworks/rofi-sys/README.md)) binds the ROFI C library via `bindgen`, honors `OFI_DIR`/`ROFI_DIR`, and exposes the necessary symbols for the runtime’s `rofi` Lamellae.

Benchmarks & utilities
----------------------
`lamellar-benchmarks/` ([README](lamellar-benchmarks/README.md)) aggregates multiple benchmark suites and helper crates that exercise Lamellar’s distributed and Lamellae-specific features. Each subdirectory has its own README describing the benchmark, build requirements, and how to run it on multi-node systems.

- `benchmark_record/` ([README](lamellar-benchmarks/benchmark_record/README.md)) records runtime/build metadata, writes JSON lines, and prints structured summaries so benchmarks can capture context-sensitive outputs.
- `histo/` ([README](lamellar-benchmarks/histo/README.md)) ports the Histo benchmark (DMA, buffered, safe/unsafe variants) to Lamellar, stressing remote memory updates and active messages.
- `index_gather/` ([README](lamellar-benchmarks/index_gather/README.md)) implements the Index Gather benchmark with atomic arrays, buffered variants, and read-only modes derived from Bale.
- `randperm/` ([README](lamellar-benchmarks/randperm/README.md)) runs the Randperm benchmark to exercise asynchronous initialization and permutation kernels on Lamellae.
- `triangle_count/` ([README](lamellar-benchmarks/triangle_count/README.md)) runs buffered and unbuffered triangle-counting workloads on Graph500 inputs with active-message and ROFI window variants.

Process management
------------------
`process_management/` ([README](process_management/README.md)) gathers the PMI/PMIx/PRRTE bindings that Lamellar uses to integrate with MPI-style launchers.

- `pmi/` ([README](process_management/pmi/README.md)) chooses between PMI-1, PMI-2, and PMIx back ends via Cargo features and optionally builds the vendorized sources when system libraries are absent.
- `pmi-sys/` ([README](process_management/pmi-sys/README.md)), `pmi2-sys/` ([README](process_management/pmi2-sys/README.md)), and `pmix-sys/` ([README](process_management/pmix-sys/README.md)) contain the low-level `bindgen` bindings and describe their vendored/system build modes.
- `pmi-mpich-src/` ([README](process_management/pmi-mpich-src/README.md)) is the patched MPICH source tree used when the PMI crates build in vendored mode.
- `openpmix-src/` ([README](process_management/openpmix-src/README.md)) copies the OpenPMIx/Autotools sources into `OUT_DIR`, runs `autogen.pl`, and exposes the resulting include/lib/bin artifacts to `pmix-sys`.
- `prrte-sys/` ([README](process_management/prrte-sys/README.md)) and `prrte-src/` ([README](process_management/prrte-src/README.md)) bind PRRTE and optionally vendor PRRTE/libevent/hwloc/PMIx so the runtime can interoperate with PRRTE-based launches.

STATUS
------
Lamellar is still under development, thus not all intended features are yet
implemented.

CONTACTS
--------
Ryan Friese     - ryan.friese@pnnl.gov  
Roberto Gioiosa - roberto.gioiosa@pnnl.gov  
Mark Raugas     - mark.raugas@pnnl.gov  

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.
