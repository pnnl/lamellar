# libfabric-sys

`libfabric-sys` is the low-level FFI bridge to OFI. It uses `bindgen` during the build process to expose all the types, functions, constants, and structures that the `libfabric` crate (and other consumers) rely on.

## Build mechanics

- The crate depends on `libfabric-src` to provision OFI headers and libraries. The build script compiles the bundled libfabric tree via Autotools and then generates a header named `fabric_sys.h` that includes every OFI header so `bindgen` only has a single input.
- Inline functions in the OFI headers are manually wrapped (the wrappers live in `inlined.c`) so that their symbols are callable from Rust.
- `bindgen` generates Rust bindings for each OFI symbol, which are then written to `OUT_DIR/bindings.rs` and included by `lib.rs`.

## Features

| Feature | Description |
|---------|-------------|
| `shared` | Enables `libfabric-src/shared`, causing the OFI build to prefer shared libraries over static ones and propagates the `-fPIC`/`-shared` options through the Autotools configuration. |

## Environment variables

- `OFI_DIR`: If present, the build script reuses the existing installation under this path instead of rebuilding the bundled sources. `libfabric-sys` will rerun the build whenever files under `OFI_DIR/lib` or `OFI_DIR/include` change.

STATUS
------
libfabric-sys has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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

