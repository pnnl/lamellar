# openpmix-src

This crate vendorizes the [OpenPMIx](https://github.com/openpmix/openpmix) source tree, copies it into the Cargo build directory, and runs its Autotools build (via `autogen.pl` / `configure`).

The build helper requires paths to `libevent` and `hwloc`. When integrated via `pmix-sys`'s `vendored-hwloc`/`vendored-libevent` features, those paths are provided through the bundled `hwlocality-sys`/`libevent-sys` build metadata (`DEP_HWLOC_ROOT`/`DEP_EVENT_ROOT`). To link against a system install instead, set `HWLOC_DIR`/`LIBEVENT_DIR` directly — this takes priority over the `DEP_*` metadata, so it also works when the `hwlocality-sys`/`libevent-sys` deps are disabled entirely (via the `vendored-hwloc`/`vendored-libevent` features on this crate).

The `Artifacts` struct exposes the resulting `include`, `lib`, and `bin` directories so downstream bindings know where to find headers, libs, and executables.


STATUS
------
openpmix-src has been developed as part of the Lamellar project and is still under development, thus not all intended features are yet
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
