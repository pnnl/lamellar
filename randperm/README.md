# Randperm benchmark

This package runs the Dev-era unified `randperm` benchmark. Choose algorithms
with `--variants` rather than using the former standalone source layout.

```sh
cargo build --profile release-dev -p randperm

cargo run --profile release-dev -p randperm -- \
  --variants cas-dart atomic-array \
  -- \
  --pes 1 --threads-per-pe 1 --lamellae local
```

Use `release-dev` for development and validation. For performance collection,
replace `--profile release-dev` with `--release` in the run command above.

The options before `--` configure Randperm; those after it configure the
`lamellar_main` launch. Run `cargo run --profile release-dev -p randperm --
--help` for benchmark options or `cargo run --profile release-dev -p randperm
-- -- --help` for launcher options. PE 0 writes JSONL result/provenance records
to `benchmarking/`.

Historical implementations remain under `src/deprecated/` and are not Cargo
targets.
