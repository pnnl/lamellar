# prrte-sys

`prrte-sys` exposes the PRRTE API via `bindgen`. It links against `libprrte` and optionally brings along vendored versions of libevent, hwloc, and PMIx.

## Features

- `vendored` builds the entire stack from the `prrte-src` tree plus the vendorized libevent/hwloc/pmix dependencies, ensuring the downstream code can run without a system PRRTE installation.
- `vendored-libevent`, `vendored-hwloc`, and `vendored-pmix` allow vendoring one dependency at a time when a system version of the others already exists.

The build script emits the include/lib directories so downstream crates know where to find PRRTE’s headers and libraries.
