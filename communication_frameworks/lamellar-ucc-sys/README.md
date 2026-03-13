# lamellar-ucc-sys

This system crate exposes the [UCC](https://github.com/openucx/ucc) C APIs to Rust through `bindgen` so that the Lamellar runtime can utilize a UCC-backed, UCX Lamellae (e.g., for IB fabrics or high-performance networking).

## Overview

- Provides the `lamellar_ucc_sys` `#![no_std]` bindings that the Lamellar runtime and other crates can consume to commiunicate via UCC.
- Builds UCC (release 1.7.0 by default) through the `ucc/` submodule when no `UCC_DIR` is supplied, but it also honors an existing installation when the environment variable is set.
- Links against `ucx`, plus its dependencies, and forwards `rustc` arguments so downstream crates automatically look in the right directories.
- Currently UCC is built as a shared library. It may be necessary to either set LD_LIBRARY_PATH to include the build directory, typically located at ./target/<release,debug>/build/lamellar-ucc=sys*/out/lib. Alternatively if your crate utilizes a build.rs file you can identify the appropriate library path via the `DEP_UCC_ROOT` and then have cargo set the rpath via something like:
```rust 
if let Ok(lamellar_ucc_lib_dir) = env::var("DEP_UCC_ROOT") {
    let lib_path = PathBuf::from(lamellar_ucc_lib_dir).join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display()););
}
```

## Building

1. `cargo build` (the build script automatically:
   - clones/checks out the `ucc` submodule and runs Autotools to configure UCC with shared libraries, and optimizations enabled)
   - runs `bindgen` on `wrapper.h` to regenerate the Rust bindings whenever the UCC sources or `UCC_DIR` change.
2. Environment overrides:
   - Set `UCC_DIR` to point at a pre-existing UCC installation.
   - If `UCC_DIR` contains shared libraries, the build script prefers them; otherwise it falls back to the submodule copy.

## Usage

Add `lamellar-ucc-sys` as a dependency when you want Lamellar to target UCX collectives explicitly:

```toml
lamellar-ucc-sys = 0.1.0
```

Downstream crates consume `lamellar_ucc_sys` for low-level collective calls on and rely on the `build.rs` output to know where the headers and libraries live.

## Environment variables

- `UCC_DIR`: optional path to an existing UCC installation; the build script uses it when the required symbols are present.

If you unset `UCC_DIR`, the crate automatically builds UCC from source using the vendorized copy so that Lamellar can still compile on systems without UCC preinstalled.

STATUS
------
lamellar-ucc-sys has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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
