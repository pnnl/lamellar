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

4. The benchmark executables use `lamellar_main`. Pass benchmark options
   first, followed by `--`, followed by launcher options. For example:

   ```sh
   cargo run --profile release-dev -p histo -- \
     --variants safe-am --local-size 1000 --local-updates 1000 \
     -- \
     --pes 1 --threads-per-pe 1 --lamellae local
   ```

   Use `cargo run --profile release-dev -p <benchmark> -- --help` for
   benchmark-specific options and `cargo run --profile release-dev -p
   <benchmark> -- -- --help` for `lamellar_main` launcher options. See each
   package README for benchmark-specific examples.

OPTIONS
-------

Everything after the second `--` in a `cargo run` invocation is a launch
option, handled by `lamellar_main` rather than the benchmark itself. Each can
also be set as an environment variable, which is useful in job scripts.

| flag                  | env var                 | purpose                                                            |
|-----------------------|--------------------------|---------------------------------------------------------------------|
| `--nodes <N>`         | `LAMELLAR_NODES`        | request `N` nodes via `salloc`; not needed for a single-node run     |
| `--pes <N>`           | `LAMELLAR_PES`          | total PEs across all nodes                                          |
| `--pes-per-node <N>`  | `LAMELLAR_PES_PER_NODE` | PEs per node                                                         |
| `--threads-per-pe <N>`| `LAMELLAR_THREADS`      | worker threads per PE                                                |
| `--lamellae <name>`   | `LAMELLAR_BACKEND`      | communication backend: `local`, `shmem`, `ucx`, `libfabric`, `rofi_c`|
| `--cmd-queue <name>`  | `LAMELLAR_CMD_QUEUE`    | command queue protocol (default `get`)                              |
| `--batcher <name>`    | `LAMELLAR_BATCHER`      | active-message batcher (default `simple`)                           |
| `--output-dir <dir>`  | -                        | redirect each PE's stdout/stderr into `<dir>`                       |
| `--salloc-opts ...`   | -                        | extra args forwarded to `salloc`, terminated by `--`                |

Any flag not listed above is passed through verbatim to the underlying
launcher (`prterun`/`srun`). The full list is always available from the
binary itself: `cargo run --profile release-dev -p <benchmark> -- -- --help`.

Note that these benchmarks are built with `ucx`, `libfabric`, and `rofi_c` all
enabled, so if `--lamellae`/`LAMELLAR_BACKEND` is left unset the runtime picks
`rofi_c` rather than `local`. Always pass `--lamellae local` explicitly for a
single-node development run.

### Running distributed

To run across multiple nodes, pick the backend that matches the interconnect
and either let `--nodes` drive a `salloc` allocation or launch from inside an
existing one:

```sh
cargo run --release -p histo -- \
  --variants atomic-array \
  -- \
  --nodes 2 --pes-per-node 1 --threads-per-pe 16 --lamellae ucx
```

For a multi-process run confined to one node with no network fabric involved,
use `--lamellae shmem` (or `local` for a single PE) instead of requesting
nodes.


  
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
