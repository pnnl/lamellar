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
- lamellar-runtime/ ([lamellar-runtime/README.md](lamellar-runtime/README.md)) — The primary crate that implements Lamellar’s asynchronous tasking runtime, distributed arrays, Darcs, and the `local`, `shmem`, and optional `rofi` Lamellae backends.
- communication_frameworks/ — Transport adapters and bindings:
  - lamellar-ucx-sys/ ([communication_frameworks/lamellar-ucx-sys/README.md](communication_frameworks/lamellar-ucx-sys/README.md)) — Generated UCX FFI plus helpers for contiguous datatypes and pointer-status handling so Lamellar can ride UCX fabrics; the `ucx` submodule sources the upstream UCX repository.
  - rofi/ ([communication_frameworks/rofi/README.md](communication_frameworks/rofi/README.md)) — Rust OFI transport layer with autotools-based build scripts, RDMA PUT/GET APIs, and provider selection for verbs/tcp.
  - rofi-sys/ ([communication_frameworks/rofi-sys/README.md](communication_frameworks/rofi-sys/README.md)) — System crate that wraps the ROFI C library via `bindgen`, honoring `OFI_DIR`/`ROFI_DIR` so downstream crates can enable the `rofi` Lamellae backend.
- lamellar-benchmarks/ ([lamellar-benchmarks/README.md](lamellar-benchmarks/README.md)) — Collection of ramps and utilities that exercise Lamellar’s distributed features:
  - benchmark_record/ ([lamellar-benchmarks/benchmark_record/README.md](lamellar-benchmarks/benchmark_record/README.md)) — Library for capturing runtime and build-time metadata and emitting JSON lines output.
  - histo/ ([lamellar-benchmarks/histo/README.md](lamellar-benchmarks/histo/README.md)) — Histo benchmark variations (DMA, buffered, safe/unsafe) targeting Lamellar’s remote memory and active-message APIs.
  - index_gather/ ([lamellar-benchmarks/index_gather/README.md](lamellar-benchmarks/index_gather/README.md)) — Index gather benchmark demonstrating atomic-array, buffered, and read-only semantics derived from Bale.
  - randperm/ ([lamellar-benchmarks/randperm/README.md](lamellar-benchmarks/randperm/README.md)) — Randperm benchmark exercising asynchronous initialization and permutation kernels.
  - triangle_count/ ([lamellar-benchmarks/triangle_count/README.md](lamellar-benchmarks/triangle_count/README.md)) — Triangle counting benchmarks (buffered/unbuffered) with Graph500 inputs and AM/ROFI-backed variants.

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
