# prrte-src

This crate vendors the PRRTE source tree, copies it into `OUT_DIR`, and runs the Autotools build configured with the provided `libevent`, `hwloc`, and PMIx directories.

The build helper exposes the produced `include`, `lib`, and `bin` directories via the exported `Artifacts` struct so the `prrte-sys` build script can link against the freshly built libraries.


STATUS
------
prrte-src has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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
