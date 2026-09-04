# Triangle Count benchmark

The `triangle_count` executable loads a graph and runs the selected
active-message implementation (`single`, `buffered`, or `am-group`).

```sh
cargo build --profile release-dev -p triangle_count

cargo run --profile release-dev -p triangle_count -- \
  --graph-file input_graphs/graph500-scale18-ef16_adj.tsv \
  --variants single buffered --buffer-size 10000
```

Use `release-dev` for development and validation. For performance collection,
replace `--profile release-dev` with `--release` in the run command above.

Use `--iterations` and `--launch-threads` to control repeated experiments and
local launch concurrency. Use
`cargo run --profile release-dev -p triangle_count -- --help` for benchmark
help.

This backport targets lamellar 0.7.1, which has no `#[lamellar::main]`
launcher, so there is no second `--`-delimited launcher-argument block.
Runtime configuration (backend, threads-per-PE, etc.) is set via
`LAMELLAR_*` environment variables instead. Multi-PE/multi-node execution is
not yet re-verified on this branch; treat it as unconfirmed until validated
at runtime.

PE 0 writes a JSONL result/provenance record for every variant, buffer size,
and iteration to `benchmarking/` by default when the benchmark's
`benchmark-record` feature is enabled. Set `BENCHMARK_RECORD_OUTPUT_DIR` to use
a different output directory.

