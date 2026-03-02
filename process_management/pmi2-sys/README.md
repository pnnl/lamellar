# pmi2-sys

`pmi2-sys` mirrors the PMI-2 interface: it generates bindings via `bindgen`, links to `-lpmi2`, and can either reuse `PMI_LIB_DIR`/`PMI_INCLUDE_DIR` from the environment or build the bundled MPICH sources when `vendored` is active.

## Feature flags

- `vendored` instructs the build script to drive `pmi-mpich-src` instead of requiring an existing `libpmi2`. Use this from `pmi` via `with-pmi2-vendored`.


STATUS
------
pmi2-sys has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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
