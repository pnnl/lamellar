# pmi-mpich-src

This crate vendors the MPICH PMI/PMI2 sources required by `pmi-sys` and `pmi2-sys`. The build script copies the `mpich` tree into `OUT_DIR`, runs Autotools, and exposes the resulting headers and static libraries.

The `pmi.patch` file lives here to tweak the upstream source so it exposes clean headers and symbols for the Rust bindings. Building this crate is automatic when you enable the `vendored` feature in one of the PMI system crates.


STATUS
------
pmi-mpich-src has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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
