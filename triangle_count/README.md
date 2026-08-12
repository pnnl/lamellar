# Triangle Count benchmark

The Dev-era `triangle_count` executable loads a graph and runs the selected
active-message implementation (`single`, `buffered`, or `am-group`).

```sh
cargo build --profile release-dev -p triangle_count

LAMELLAR_BACKEND=local LAMELLAR_PES=1 \
  cargo run --profile release-dev -p triangle_count -- \
  --graph-file input_graphs/graph500-scale18-ef16_adj.tsv \
  --variants single buffered --buffer-size 10000
```

Use `--iterations` and `--launch-threads` to control repeated experiments and
local launch concurrency. Backend setup is supplied by Lamellar. PE 0 writes a
JSONL result/provenance record for every variant, buffer size, and iteration to
`benchmarking/`.

The former `triangle_count` and `triangle_count_buffered` standalone sources
are archived in `src/deprecated/`; they are not Cargo targets.
