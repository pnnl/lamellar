# openpmix-src

This crate vendorizes the [OpenPMIx](https://github.com/openpmix/openpmix) source tree, copies it into the Cargo build directory, and runs its Autotools build (via `autogen.pl` / `configure`).

The build helper requires paths to `libevent` and `hwloc` (`DEP_EVENT_ROOT`, `DEP_HWLOC_ROOT`). When integrated via `pmix-sys`, those dependencies are provided through the bundled `libevent-sys`/`hwlocality-sys` features.

The `Artifacts` struct exposes the resulting `include`, `lib`, and `bin` directories so downstream bindings know where to find headers, libs, and executables.
