# Index Gather benchmark

This package benchmarks index-gather operations with the unified Dev-era
`index_gather` executable. Implementations are chosen at runtime, rather than
being exposed as separate binaries.

## Build and run

From the workspace root:

```sh
cargo build --profile release-dev -p index_gather

LAMELLAR_BACKEND=local LAMELLAR_PES=1 \
  cargo run --profile release-dev -p index_gather -- \
  --variants safe-am read-only-array --local-size 1000 --local-updates 1000
```

Use `--variants`, `--am-index-size`, and `--array-distribution` to select the
Dev implementations. Backend-specific setup is provided by Lamellar.

PE 0 writes one JSONL record per variant/iteration to `benchmarking/`.

## Legacy implementations

The old standalone implementations live in `src/deprecated/` as archival
source only. They are not Cargo targets.
