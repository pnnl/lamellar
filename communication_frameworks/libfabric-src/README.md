# libfabric-src

This crate vendors the upstream libfabric sources, copies them into `OUT_DIR`, and invokes Autotools to build the chosen configuration (shared or static) while honoring an optional `OFI_DIR` override. The build helper also bundles wrapper generation for inline functions so the Rust bindings in `libfabric-sys` can link to them.

## Usage

`libfabric-sys` and other downstream crates depend on this helper indirectly through the build script. No direct dependency is necessary: cargo sets `libfabric-src` as a build dependency, and it prints the `include`/`lib` paths for upstream binding generation.

## License

BSD (see [`LICENSE`](LICENSE)).
