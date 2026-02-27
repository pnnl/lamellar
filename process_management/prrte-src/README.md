# prrte-src

This crate vendors the PRRTE source tree, copies it into `OUT_DIR`, and runs the Autotools build configured with the provided `libevent`, `hwloc`, and PMIx directories.

The build helper exposes the produced `include`, `lib`, and `bin` directories via the exported `Artifacts` struct so the `prrte-sys` build script can link against the freshly built libraries.
