# prrte-sys

`prrte-sys` exposes the PRRTE API via `bindgen`. It links against `libprrte` and optionally brings along vendored versions of libevent, hwloc, and PMIx.

## Features

- `vendored` builds the entire stack from the `prrte-src` tree plus the vendorized libevent/hwloc/pmix dependencies, ensuring the downstream code can run without a system PRRTE installation.
- `vendored-libevent`, `vendored-hwloc`, and `vendored-pmix` allow vendoring one dependency at a time when a system version of the others already exists.

The build script emits the include/lib directories so downstream crates know where to find PRRTE’s headers and libraries.

STATUS
------
prrte-sys has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
implemented.

CONTACTS
--------

Current Team Members

Ryan Friese           - ryan.friese@pnnl.gov 
Polykarpos Thomadakis - polykarpos.thomadakis@pnnl.gov

## License

This project is licensed under the BSD License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.
