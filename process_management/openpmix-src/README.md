# openpmix-src

This crate vendorizes the [OpenPMIx](https://github.com/openpmix/openpmix) source tree, copies it into the Cargo build directory, and runs its Autotools build (via `autogen.pl` / `configure`).

The build helper requires paths to `libevent` and `hwloc` (`DEP_EVENT_ROOT`, `DEP_HWLOC_ROOT`). When integrated via `pmix-sys`, those dependencies are provided through the bundled `libevent-sys`/`hwlocality-sys` features.

The `Artifacts` struct exposes the resulting `include`, `lib`, and `bin` directories so downstream bindings know where to find headers, libs, and executables.


STATUS
------
openpmix-src has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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
