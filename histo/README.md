# Histo benchmark

This package benchmarks histogram updates using the Lamellar runtime.  It uses
the unified Dev-era `histo` executable and selects implementations with CLI
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

`lamellar_main` separates benchmark arguments from launch arguments. Put the
benchmark's options first, then use `--` before Lamellar launcher options.

For a small local run:

```sh
cargo run --profile release-dev -p histo -- \
  --variants safe-am atomic-array --local-size 1000 --local-updates 1000 \
  -- \
  --pes 1 --threads-per-pe 1 --lamellae local
```

Use `--variants`, `--am-index-size`, and `--array-distribution` to narrow the
Dev implementations under test. Benchmark help is available with
`cargo run --profile release-dev -p histo -- --help`; launcher help is
available with `cargo run --profile release-dev -p histo -- -- --help`.

For a multi-node job, replace the final launcher arguments with, for example,
`--nodes 2 --pes-per-node 1 --threads-per-pe 8 --lamellae ucx`. The launcher
passes unrecognized options through to `salloc`/`srun` or the configured
underlying launcher.

Each PE 0 run writes JSONL provenance/results to `benchmarking/`.

## Legacy implementations

The former standalone sources are retained in `src/deprecated/` for historical
reference. They are not Cargo targets and must not be invoked as
`histo_safe_am`, `histo_buffered_safe_am`, or other former binary names.
