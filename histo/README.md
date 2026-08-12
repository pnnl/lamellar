# Histo benchmark

This package benchmarks histogram updates using the Lamellar runtime.  It uses
the unified Dev-era `histo` executable and selects implementations with CLI
options rather than maintaining one binary per variant.

## Build

From the workspace root:

```sh
cargo build --profile release-dev -p histo
```

Lamellar backend setup depends on the target system.  See the Lamellar runtime
documentation for the required environment for local, shared-memory, OFI, UCX,
or ROFI-C execution.

## Run

For a small local run:

```sh
LAMELLAR_BACKEND=local LAMELLAR_PES=1 \
  cargo run --profile release-dev -p histo -- \
  --variants safe-am atomic-array --local-size 1000 --local-updates 1000
```

Use `--variants`, `--am-index-size`, and `--array-distribution` to narrow the
Dev implementations under test. `--help` lists all supported values.

Each PE 0 run writes JSONL provenance/results to `benchmarking/`.

## Legacy implementations

The former standalone sources are retained in `src/deprecated/` for historical
reference. They are not Cargo targets and must not be invoked as
`histo_safe_am`, `histo_buffered_safe_am`, or other former binary names.
