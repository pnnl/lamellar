# Randperm benchmark

This package runs the Dev-era unified `randperm` benchmark. Choose algorithms
with `--variants` rather than using the former standalone source layout.

```sh
cargo build --profile release-dev -p randperm

LAMELLAR_BACKEND=local LAMELLAR_PES=1 \
  cargo run --profile release-dev -p randperm -- \
  --variants cas-dart atomic-array
```

Run `cargo run -p randperm -- --help` for the size and iteration options. PE 0
writes JSONL result/provenance records to `benchmarking/`.

Historical implementations remain under `src/deprecated/` and are not Cargo
targets.
