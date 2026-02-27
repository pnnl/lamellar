# PMI crate

`pmi` is a thin Rust wrapper that lets Lamellar (and other crates) consume PMI-1, PMI-2, or PMIx depending on the features you enable.

## Features

- `with-pmi1`, `with-pmi2`, `with-pmix` enable the corresponding backend. Each pulls in the matching `*-sys` crate and the associated native libraries.
- `vendored` attempts to build the bundled implementations instead of requiring a system provider; it combines with the `with-*` flags (`with-pmi1-vendored`, etc.) to force a vendor build for the requested protocol layers.

## Usage

Add `pmi` as a dependency and choose the backend that matches the process manager on your cluster. Use the vendored features when you do not have a system PMI/PMIx stack available yet still need to run Lamellar examples with job-level coordination.
