Benchmarks for the Lamellar Runtime
=================================================

Lamellar is an asynchronous tasking runtime for HPC systems developed in RUST

SUMMARY
-------

A collection of benchmarks to test the functionality and performance of the Lamellar runtime (https://github.com/pnnl/lamellar-runtime)

NEWS
----

Sept 2020: Update for Lamellar 0.2.1 release
July 2020: Second alpha release
Feb 2020: First alpha release

BUILD REQUIREMENTS
------------------

These benchmarks requires the following dependencies:

* [Lamellar](https://github.com/pnnl/lamellar-runtime) - now on [crates.io](https://crates.io/crates/lamellar)
At the time of release, Lamellar has been tested with the following external packages:

| **GCC** | **CLANG** | **ROFI**  | **OFI**   | **IB VERBS**  | **MPI**       | **SLURM** | **LAMELLAR** |
|--------:|----------:|----------:|----------:|--------------:|--------------:|----------:|-------------:|
| 7.1.0   | 8.0.1     | 0.1.0     | 1.9.0     | 1.13          | mvapich2/2.3a | 17.02.7   | 0.2.1        |

The OFI_DIR environment variable must be specified with the location of the OFI installation.
The ROFI_DIR environment variable must be specified with the location of the ROFI installation.

BUILDING PACKAGE
----------------

In the following, assume a root directory ${ROOT}

1. download Benchmarks to ${ROOT}/Benchmarks 
    `cd ${ROOT} && git clone https://github.com/pnnl/lamellar-benchmarks`
2. download Lamellar to ${ROOT}/lamellar-runtime  -- or update Cargo.toml to point to the proper location
    `cd ${ROOT} && git clone https://github.com/pnnl/lamellar-runtime`

3. Choose a build profile before running a benchmark:

   - Use `release-dev` during development and validation. It keeps optimized
     code while compiling faster.
   - Use `release` when collecting performance results. It enables the
     workspace's full LTO/low-codegen-unit configuration.

   For example, a development build is:

   `cargo build --profile release-dev -p histo`

   Replace `--profile release-dev` with `--release` for a performance build.

4. Pass benchmark options directly; no launcher-argument block. For example:

   ```sh
   cargo run --profile release-dev -p histo -- \
     --variants safe-am --local-size 1000 --local-updates 1000
   ```

   Use `cargo run --profile release-dev -p <benchmark> -- --help` for
   benchmark-specific options. See each package README for benchmark-specific
   examples.

OPTIONS
-------

**This backport branch targets lamellar 0.7.1, which has no
`#[lamellar::main]` launcher.** The `lamellar_main`-provided second
`--`-delimited launcher-argument block (`--nodes`, `--pes-per-node`,
`--threads-per-pe`, `--lamellae`, `--salloc-opts`, etc.) does not exist here;
those flags are gone, not just undocumented. Runtime configuration instead
goes through `LAMELLAR_*` environment variables read directly by the
lamellar 0.7.1 crate (`LAMELLAR_BACKEND`, `LAMELLAR_THREADS`,
`LAMELLAR_HEAP_SIZE`, etc. — see that crate's `env_var` module docs for the
full list). There is no `LAMELLAR_NODES`/`LAMELLAR_PES` equivalent in 0.7.1.

### Running distributed

Multi-PE/multi-node execution has not been re-verified on this branch.
Without `lamellar_main`'s built-in launcher, starting multiple PEs likely
requires launching multiple OS processes directly (e.g. via `mpirun`/`srun`)
with the `enable-rofi` feature enabled, each process becoming one PE — but
this is unconfirmed until validated at runtime.


  
NOTES
-----

STATUS
------

Working on additional benchmarks

CONTACTS
--------

Ryan Friese     - ryan.friese@pnnl.gov  
Mark Raugas     - mark.raugas@pnnl.gov
Joseph Cottam   - joseph.cottam@pnnl.gov

License
-------

This project is licensed under the BSD License - see the [LICENSE.md](LICENSE.md) file for details.

Acknowledgments
---------------

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.
