# pmi-mpich-src

This crate vendors the MPICH PMI/PMI2 sources required by `pmi-sys` and `pmi2-sys`. The build script copies the `mpich` tree into `OUT_DIR`, runs Autotools, and exposes the resulting headers and static libraries.

The `pmi.patch` file lives here to tweak the upstream source so it exposes clean headers and symbols for the Rust bindings. Building this crate is automatic when you enable the `vendored` feature in one of the PMI system crates.
