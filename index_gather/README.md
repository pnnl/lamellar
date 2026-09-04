# Index Gather benchmark

This package benchmarks index-gather operations with a single unified
`index_gather` executable. Implementations are chosen at runtime, rather than
being exposed as separate binaries.

## Build and run

From the workspace root:

```sh
cargo build --profile release-dev -p index_gather

cargo run --profile release-dev -p index_gather -- \
  --variants safe-am read-only-array --local-size 1000 --local-updates 1000 \
  -- \
  --pes 1 --threads-per-pe 1 --lamellae local
```

Use `release-dev` for development and validation. For performance collection,
replace `--profile release-dev` with `--release` in the run command above.

Use `--variants`, `--am-index-size`, and `--array-distribution` to select the
implementations. `lamellar_main` receives the arguments after `--`; use
`cargo run --profile release-dev -p index_gather -- --help` for benchmark help
and `cargo run --profile release-dev -p index_gather -- -- --help` for launcher
help.

For a distributed launch, use launcher options such as
`--nodes 2 --pes-per-node 1 --threads-per-pe 8 --lamellae ucx` after the
separator.

PE 0 writes one JSONL record per variant/iteration to `benchmarking/` by
default when the benchmark's `benchmark-record` feature is enabled. Set
`BENCHMARK_RECORD_OUTPUT_DIR` to use a different output directory.

