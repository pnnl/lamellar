# rofi-sys #

This system crate provides Rust language bindings (via the use of Bindgen) to the Rust-OFI library.

## Dependencies

**rofi-sys** has the following dependencies:

* gcc 4.8.5
* openmpi 2.1.1
* clang 5.0.1
* rust-ofi 0.4


The **OFI_DIR** environment variable must be specified with the location of the OFI installation.

The **ROFI_DIR** environment variable must be specified with the location of the Rust-OFI installation.

## Provenance of `rofi/`

Upstream [pnnl/rofi](https://github.com/pnnl/rofi) (now also native in this repository as `communication_frameworks/rofi`) has never published an official "make dist" release tarball (checked every tag — none carry a dist asset). So `rofi/` is not an official release artifact: it's a tarball generated from a fresh checkout of upstream commit [`d112544`](https://github.com/pnnl/rofi/commit/d112544) (which already includes an `AM_MAINTAINER_MODE` fix and a `make dist` completeness fix, both pushed upstream) by running `autoreconf -ivf && ./configure && make dist`, then vendoring the resulting `rofi-0.1.tar.gz` verbatim (generated `configure`/`Makefile.in`/etc. included). The only changes since are `src/` C fixes copied byte-for-byte from [`communication_frameworks/rofi`](../rofi) (currently in sync with `f32adbc` of the upstream history, which is imported alongside this crate); `rofi/` here should be refreshed from there rather than edited independently. This removes the need for `autotools::Config::reconf` at build time — `rofi/` already ships a working `configure` script.

The tarball's `uthash/` subdirectory (a single-header hash table, vendored upstream as plain files rather than a submodule) has its `include -> src` symlink dereferenced into a real directory copy by `make dist`; contents are identical either way and no build-flag changes were needed.

The nested `libfabric` submodule has been removed entirely — `build.rs` never reads from a local `libfabric/` path; it gets libfabric only through the `lamellar-libfabric-sys` Cargo dependency (`DEP_OFI_ROOT`).

## PMIx Support

Building with the `pmix` feature vendors PMIx through Lamellar's `pmix-sys` crate, which in turn builds the bundled `openpmix-src` tree and passes the resulting install root into ROFI's `configure --with-pmix` path.

Example:

```bash
cargo build --features pmix
```

If `ROFI_DIR` is unset, `rofi-sys` also builds the bundled ROFI source tree and configures `pmi-simple` to use the vendored PMIx backend.

## Debug Support

Building with the `debug` feature passes `--enable-debug` to the bundled ROFI `configure` step, enabling ROFI runtime debug logging and debug-friendly compile flags.

Example:

```bash
cargo build --features debug
```

## Authors

Current Team Members

* **Mark Raugas**, PNNL
* **Ryan Friese**, PNNL

Past Team Members

* **Roberto Gioiosa**

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.
