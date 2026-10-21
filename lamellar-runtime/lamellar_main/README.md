lamellar_main
=============

`lamellar_main` provides `#[lamellar_main::main]` and `#[lamellar_main::test]` — a thin, lamellar-specific wrapper around [`hpc_launch`](../../process_management/hpc_launch/README.md) that adds:

- `--lamellae <name>`, `--cmd-queue <variant>`, `--batcher <variant>`, `--executor <variant>`, `--heap-size <bytes>` launch flags (setting `LAMELLAR_BACKEND`/`LAMELLAR_CMD_QUEUE`/`LAMELLAR_BATCHER`/`LAMELLAR_EXECUTOR`/`LAMELLAR_HEAP_SIZE` on the launched process)
- Lamellar's available-backend list in `--help`
- Backend-aware PE defaulting (defaults to a single PE when `lamellar::config().backend == "local"`)
- `enable-prof` support (`lamellar::init_prof_bt!`/`fini_prof!` around the wrapped `main` body)

Everything else (launch mechanics, RPATH patching) is the same code `hpc_launch` uses: both macro crates generate their launch preamble through [`hpc_launch_codegen`](../../process_management/hpc_launch_codegen/README.md), configured here with the `LAMELLAR` environment prefix. Topology detection and tracing init come from `hpc_launch`'s runtime helpers.

Most users don't need to depend on this crate directly — `lamellar` reexports `#[lamellar::main]`/`#[lamellar::test]` from it when built with the `enable-lamellar-main` feature (on by default). See the [`lamellar`](https://crates.io/crates/lamellar) crate's documentation for full application-level usage.

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.
