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

`rofi/` is a build copy of ROFI, not an independently edited source tree. The source of truth is [`communication_frameworks/rofi`](../rofi). `rofi/` here mirrors its hand-written files (`src/`, `tests/`, `configure.ac`, every `Makefile.am`, the docs inputs, `uthash/` and `README.md`) and adds the generated autotools output (`configure`, `Makefile.in`, `aclocal.m4`, `ltmain.sh`, `m4/`, ...). That output is why neither `build.rs` nor the published crate needs `autoreconf` (or `autotools::Config::reconf`) at build time. Do not edit the mirrored files here.

To change ROFI, edit it in `communication_frameworks/rofi`, then run from this crate:

```
tools/sync-rofi.sh               # copy changed files into rofi/
tools/sync-rofi.sh --regenerate  # also re-run autoreconf (needed after a configure.ac / Makefile.am change)
tools/sync-rofi.sh --check       # verify nothing has drifted; exits non-zero otherwise
```

`rofi/.autotools-stamp` records a hash of `configure.ac` and every `Makefile.am` as of the last time the generated files were produced, and `--check` fails if those change without a `--regenerate`. `--regenerate` needs Autoconf, Automake and Libtool; the checked-in files were generated with Autoconf 2.69, and a different version will show up as extra churn in `git diff`.

The generated files first came from upstream [pnnl/rofi](https://github.com/pnnl/rofi) commit [`d112544`](https://github.com/pnnl/rofi/commit/d112544) (which already includes an `AM_MAINTAINER_MODE` fix and a `make dist` completeness fix) by running `autoreconf -ivf && ./configure && make dist`. Upstream never published an official "make dist" release tarball (no tag carries a dist asset), so this is not an official release artifact.

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
