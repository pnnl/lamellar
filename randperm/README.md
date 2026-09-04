# Randperm benchmark

This package runs a single unified `randperm` benchmark. Choose algorithms
with `--variants` rather than using separate binaries per algorithm.

```sh
cargo build --profile release-dev -p randperm

cargo run --profile release-dev -p randperm -- \
  --variants cas-dart atomic-array
```

Use `release-dev` for development and validation. For performance collection,
replace `--profile release-dev` with `--release` in the run command above.

Run `cargo run --profile release-dev -p randperm -- --help` for benchmark
options.

This backport targets lamellar 0.7.1, which has no `#[lamellar::main]`
launcher, so there is no second `--`-delimited launcher-argument block.
Runtime configuration (backend, threads-per-PE, etc.) is set via
`LAMELLAR_*` environment variables instead. Multi-PE/multi-node execution is
not yet re-verified on this branch; treat it as unconfirmed until validated
at runtime.

PE 0 writes JSONL result/provenance records
to `benchmarking/` by default when the benchmark's `benchmark-record` feature
is enabled. Set `BENCHMARK_RECORD_OUTPUT_DIR` to use a different output
directory.

