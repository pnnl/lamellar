# libfabric-src

This crate vendors the upstream libfabric sources, copies them into `OUT_DIR`, and invokes Autotools to build the chosen configuration (shared or static) while honoring an optional `OFI_DIR` override. The build helper also bundles wrapper generation for inline functions so the Rust bindings in `libfabric-sys` can link to them.

## Usage

`libfabric-sys` and other downstream crates depend on this helper indirectly through the build script. No direct dependency is necessary: cargo sets `libfabric-src` as a build dependency, and it prints the `include`/`lib` paths for upstream binding generation.


STATUS
------
Libfabric-src has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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