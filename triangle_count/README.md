# Triangle Count benchmark

The Dev-era `triangle_count` executable loads a graph and runs the selected
active-message implementation (`single`, `buffered`, or `am-group`).

```sh
cargo build --profile release-dev -p triangle_count

cargo run --profile release-dev -p triangle_count -- \
  --graph-file input_graphs/graph500-scale18-ef16_adj.tsv \
  --variants single buffered --buffer-size 10000 \
  -- \
  --pes 1 --threads-per-pe 1 --lamellae local
```

Use `release-dev` for development and validation. For performance collection,
replace `--profile release-dev` with `--release` in the run command above.

Use `--iterations` and `--launch-threads` to control repeated experiments and
local launch concurrency. Arguments after `--` are `lamellar_main` launch
options, including `--nodes`, `--pes-per-node`, `--threads-per-pe`, and
`--lamellae`. Use `cargo run --profile release-dev -p triangle_count -- --help`
for benchmark help or `cargo run --profile release-dev -p triangle_count -- --
--help` for launcher help.
PE 0 writes a JSONL result/provenance record for every variant, buffer size,
and iteration to `benchmarking/`.

The former `triangle_count` and `triangle_count_buffered` standalone sources
are archived in `src/deprecated/`; they are not Cargo targets.
