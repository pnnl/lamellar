# Index Gather benchmark

This package benchmarks index-gather operations with a single unified
`index_gather` executable. Implementations are chosen at runtime, rather than
being exposed as separate binaries.

## Build and run

From the workspace root:

```sh
cargo build --profile release-dev -p index_gather

cargo run --profile release-dev -p index_gather -- \
  --variants safe-am read-only-array --local-size 1000 --local-updates 1000
```

Use `release-dev` for development and validation. For performance collection,
replace `--profile release-dev` with `--release` in the run command above.

Use `--variants`, `--am-index-size`, and `--array-distribution` to select the
implementations. Benchmark help is available with
`cargo run --profile release-dev -p index_gather -- --help`.

This backport targets lamellar 0.7.1, which has no `#[lamellar::main]`
launcher, so there is no second `--`-delimited launcher-argument block.
Runtime configuration (backend, threads-per-PE, etc.) is set via
`LAMELLAR_*` environment variables instead. Multi-PE/multi-node execution is
not yet re-verified on this branch; treat it as unconfirmed until validated
at runtime.

PE 0 writes one JSONL record per variant/iteration to `benchmarking/` by
default when the benchmark's `benchmark-record` feature is enabled. Set
`BENCHMARK_RECORD_OUTPUT_DIR` to use a different output directory.

