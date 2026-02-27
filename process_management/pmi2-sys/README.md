# pmi2-sys

`pmi2-sys` mirrors the PMI-2 interface: it generates bindings via `bindgen`, links to `-lpmi2`, and can either reuse `PMI_LIB_DIR`/`PMI_INCLUDE_DIR` from the environment or build the bundled MPICH sources when `vendored` is active.

## Feature flags

- `vendored` instructs the build script to drive `pmi-mpich-src` instead of requiring an existing `libpmi2`. Use this from `pmi` via `with-pmi2-vendored`.
