# pmi-sys

`pmi-sys` provides `bindgen`-generated bindings to the PMI-1 API, catering to both system-provided installations and the bundled MPICH sources.

## Build modes

- Without features, the build script looks for `PMI_LIB_DIR`/`PMI_INCLUDE_DIR` environment variables and links against the provided directory.
- Enabling `vendored` (or `with-pmi1-vendored` from the `pmi` crate) builds the `pmi-mpich-src` package and uses the generated headers/libraries instead.

## Outputs

The crate emits the location of the PMI headers/libraries via `cargo:root` and links with `-lpmi`, exposing the raw C symbols to downstream Rust code.

STATUS
------
pmi-sys has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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
