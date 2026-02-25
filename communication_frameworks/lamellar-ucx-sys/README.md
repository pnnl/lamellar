# lamellar-ucx-sys

This system crate exposes the [UCX](https://github.com/openucx/ucx) C APIs to Rust through `bindgen` so that the Lamellar runtime can utilize a UCX-backed Lamellae (e.g., for IB fabrics or high-performance networking).

## Overview

- Provides the `lamellar_ucx_sys` `#![no_std]` bindings that the Lamellar runtime and other crates can consume to commiunicate via UCX.
- Builds UCX (release 1.20.0 by default) through the `ucx/` submodule when no `UCX_DIR` is supplied, but it also honors an existing installation when the environment variable is set.
- Links against `ucp`, plus its dependencies (`ibverbs`, `rdmacm`), and forwards `rustc` arguments so downstream crates automatically look in the right directories.
- Currently UCX is built as a shared library. It may be necessary to either set LD_LIBRARY_PATH to include the build directory, typically located at ./target/<release,debug>/build/lamellar-ucx=sys*/out/lib. Alternatively if your crate utilizes a build.rs file you can identify the appropriate library path via the `DEP_UCX_ROOT` and then have cargo set the rpath via something like:
```rust 
if let Ok(lamellar_ucx_lib_dir) = env::var("DEP_UCX_ROOT") {
    let lib_path = PathBuf::from(lamellar_ucx_lib_dir).join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display()););
}
```

## Building

1. `cargo build` (the build script automatically:
   - clones/checks out the `ucx` submodule and runs Autotools to configure UCX with shared libraries, threading, and optimizations enabled, while disabling CUDA/ROCm/Go/Java.)
   - runs `bindgen` on `wrapper.h` to regenerate the Rust bindings whenever the UCX sources change (the build script reruns whenever any `.c` or `.h` under `ucx/` changes).
2. Environment overrides:
   - Set `UCX_DIR` to point at a pre-existing UCX installation (the script checks for `libucm`, `libucp`, `libucs`, and `libuct` symbols before accepting it).
   - If `UCX_DIR` contains shared libraries, the build script prefers them; otherwise it falls back to the submodule copy.

## Usage

Add `lamellar-ucx-sys` as a dependency when you want Lamellar to target UCX transports explicitly:

```toml
lamellar-ucx-sys = 0.1.0
```

Downstream crates consume `lamellar_ucx_sys` for low-level fabric configuration (PE initialization, tag matching, etc.) and rely on the `build.rs` output to know where the headers and libraries live.

## Environment variables

- `UCX_DIR`: optional path to an existing UCX installation; the build script uses it when the required symbols are present.

If you unset `UCX_DIR`, the crate automatically builds UCX from source using the vendorized copy so that Lamellar can still compile on systems without UCX preinstalled.

## License

This crate is licensed under the BSD License. See [`LICENSE`](LICENSE) for details.
