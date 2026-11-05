# lamellar-ucx-sys

This system crate exposes the [UCX](https://github.com/openucx/ucx) C APIs to Rust through `bindgen` so that the Lamellar runtime can utilize a UCX-backed Lamellae (e.g., for IB fabrics or high-performance networking).

## Overview

- Provides the `lamellar_ucx_sys` `#![no_std]` bindings that the Lamellar runtime and other crates can consume to commiunicate via UCX.
- Vendors the official [UCX](https://github.com/openucx/ucx) release tarball (v1.20.0 — the pre-generated "make dist" tarball from GitHub Releases, not a git checkout) under `ucx/`, and builds it via Autotools when no `UCX_DIR` is supplied. Because the vendored tree came from the tarball, `configure` is already generated, so no `autogen.sh`/`autoreconf` step is run and no Flex/Autoconf/Automake/Libtool toolchain is required on the build machine. It also honors an existing installation when the `UCX_DIR` environment variable is set.
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
   - copies the vendored `ucx/` tree and runs Autotools to configure UCX with shared libraries, threading, and optimizations enabled, while disabling CUDA/ROCm/Go/Java.)
   - runs `bindgen` on `wrapper.h` to regenerate the Rust bindings whenever the UCX sources change (the build script reruns whenever any `.c` or `.h` under `ucx/` changes).
2. Environment overrides:
   - Set `UCX_DIR` to point at a pre-existing UCX installation (the script checks for `libucm`, `libucp`, `libucs`, and `libuct` symbols before accepting it).
   - If `UCX_DIR` contains shared libraries, the build script prefers them; otherwise it falls back to the vendored copy.

## Usage

Add `lamellar-ucx-sys` as a dependency when you want Lamellar to target UCX transports explicitly:

```toml
lamellar-ucx-sys = 0.1.1
```

Downstream crates consume `lamellar_ucx_sys` for low-level fabric configuration (PE initialization, tag matching, etc.) and rely on the `build.rs` output to know where the headers and libraries live.

## Tested library versions

The bindings are generated from the headers of the UCX being used, and `bindgen` names anonymous C unions and structs by their position
(`ucp_request_param_t__bindgen_ty_1`). A UCX release other than those below can renumber them, and the failure is then a compile error
in the crate that uses the bindings, not in this one.

| UCX | Status |
|-----|--------|
| 1.20.0 (UCP API 1.20) | vendored, the default |
| 1.17.0 | checked, through `UCX_DIR` |
| anything else | untested |

The build script reads `UCP_API_MAJOR`/`UCP_API_MINOR` from the header and prints a `cargo:warning` when the version is outside the range
above. Cargo shows build-script warnings only for path and workspace dependencies, so a crate that depends on the published
`lamellar-ucx-sys` will not see it; check this table before pointing `UCX_DIR` at another release.

## Environment variables

- `UCX_DIR`: optional path to an existing UCX installation; the build script uses it when the required symbols are present.

If you unset `UCX_DIR`, the crate automatically builds UCX from the vendored release tarball so that Lamellar can still compile on systems without UCX preinstalled.

STATUS
------
lamellar-ucx-sys has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
implemented.

CONTACTS
--------

Current Team Members

Ryan Friese           - ryan.friese@pnnl.gov 

Past Team Members

Polykarpos Thomadakis

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.

