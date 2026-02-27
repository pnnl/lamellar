# process_management

This directory gathers MPI process-management bindings and vendored sources that Lamellar can use to discover the scheduling information required by the runtime and related benchmarks.

| Subdirectory | Purpose |
|--------------|---------|
| `pmi/` | High-level Rust crate that conditionally enables PMI-1, PMI-2, or PMIx support via Cargo features and toggles vendored builds. |
| `pmi-sys/`, `pmi2-sys/`, `pmix-sys/` | Low-level `bindgen` crates that expose the native PMI/PMIx APIs and support using system installations or the bundled sources. |
| `pmi-mpich-src/` | Vendorized MPICH PMI/PMI2 sources plus the patch that exposes the headers/libraries for the bindings. |
| `openpmix-src/` | Vendorized OpenPMIx source tree along with an Autotools build helper that respects `libevent` and `hwloc`. |
| `prrte-sys/` | FFI crate that binds PRRTE plus the optional vendored libevent/hwloc/PMIx stack. |
| `prrte-src/` | Vendorized PRRTE Autotools tree with the glue script that wires in the other vendored dependencies when requested. |
