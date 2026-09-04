# Histo benchmark

This package benchmarks histogram updates using the Lamellar runtime.  It uses
a single unified `histo` executable and selects implementations with CLI
options rather than maintaining one binary per variant.

## Build

From the workspace root:

```sh
cargo build --profile release-dev -p histo
```

Use `release-dev` for development and validation. For performance collection,
replace `--profile release-dev` with `--release` in the run command below.

Lamellar backend setup depends on the target system.  See the Lamellar runtime
documentation for the required environment for local, shared-memory, OFI, UCX,
or ROFI-C execution.

## Run

This backport targets lamellar 0.7.1, which has no `#[lamellar::main]`
launcher and therefore no second `--`-delimited launcher-argument block.
All arguments go to the benchmark's own CLI; runtime configuration (backend,
threads-per-PE, etc.) is set via `LAMELLAR_*` environment variables instead
(see the `lamellar` crate's `env_var` docs).

For a small local run:

```sh
cargo run --profile release-dev -p histo -- \
  --variants safe-am atomic-array --local-size 1000 --local-updates 1000
```

Use `--variants`, `--am-index-size`, and `--array-distribution` to narrow the
implementations under test. Benchmark help is available with
`cargo run --profile release-dev -p histo -- --help`.

Multi-PE/multi-node execution is not yet re-verified on this branch since it
depends on how PEs are launched without `lamellar_main`'s built-in launcher
(likely an external `mpirun`/`srun` invocation with the `enable-rofi`
feature); treat this as unconfirmed until validated at runtime.

With the benchmark's `benchmark-record` feature enabled, each PE 0 run writes
JSONL provenance/results to `benchmarking/` by default. Set
`BENCHMARK_RECORD_OUTPUT_DIR` to use a different output directory.

