hpc_launch
==========

`hpc_launch` provides `#[hpc_launch::main]` and `#[hpc_launch::test]`: attribute macros that turn an ordinary `fn main()` into a self-launching multi-process binary, re-executing itself under `prterun` (PRRTE) or `srun` (SLURM) with no separate launch script.

It is a standalone crate — it does **not** depend on the [`lamellar`](https://crates.io/crates/lamellar) runtime, or on anything lamellar-specific. Use it directly if you want the launch/process-management behavior below without pulling in the rest of Lamellar (e.g. for your own MPI-style multi-process tooling). It is also the foundation that [`lamellar_main`](../lamellar_main/README.md) builds on to power `#[lamellar::main]`/`#[lamellar::test]`.

# What it does

The generated `main` detects whether it's already running as a launched PE (e.g. under PRRTE/srun); if not, it re-executes itself under a launcher (`prterun` by default, or `srun` with the `use-srun` feature) using the arguments given after the launcher-args separator (`--`). This means a single `cargo run`/binary invocation transparently becomes a multi-process job.

Before launching, it also patches the binary's `RPATH`/`RUNPATH` (via `readelf`/`patchelf`) with the build-output library directories found on `LD_LIBRARY_PATH`, so launched processes can find shared library dependencies without you having to export `LD_LIBRARY_PATH` yourself.

It also exposes standalone helper functions usable outside the macros:
- `numa_domain_count()`, `package_count()`, `core_count()`, `pu_count()` — best-effort hardware topology detection via a `/sys` scan
- `init_tracing_from_env()` — initializes a [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) global subscriber from the `HPC_LAUNCH_LOG` environment variable (`init_tracing_from_env_var(name)` reads a variable of your choosing)

# Environment variables

Launch settings can also come from the environment, with the command-line flag taking precedence:

| Variable | Flag | Meaning |
|---|---|---|
| `HPC_LAUNCH_NODES` | `--nodes` | number of nodes |
| `HPC_LAUNCH_PES` | `--pes` | total number of processes (PEs) |
| `HPC_LAUNCH_PES_PER_NODE` | `--pes-per-node` | processes per node |
| `HPC_LAUNCH_THREADS` | `--threads-per-pe` | worker threads per process; also exported to the launched job |
| `HPC_LAUNCH_LOG` | `--log` | tracing filter (`RUST_LOG` syntax) for the launched job |
| `HPC_LAUNCH_MAIN_TIME` | `--time` | when set, the launched job reports its wall-clock application time |

The prefix is a parameter of the shared code generator (`hpc_launch_codegen`): this crate uses `HPC_LAUNCH`, and `lamellar_main` uses `LAMELLAR` for the same variables.

# Using standalone

Add it to your `Cargo.toml`, enabling exactly one launcher backend:

```toml
[dependencies]
hpc_launch = { version = "0.1.0", features = ["use-prterun"] }
```

(use `features = ["use-srun"]` instead if you're launching via SLURM's `srun`).

```rust
#[hpc_launch::main]
fn main() {
    // by the time this runs, we're already inside a launched process
    // (PRTE_LAUNCHED / SLURM_LOCALID is set)
    println!("hello from a launched process");
}
```

Build and run as normal:

```
cargo run --release -- <app args> -- <launcher args>
```

- everything before the first `--` is `cargo`'s own arguments
- everything between the two `--` is forwarded to your application unmodified
- everything after the second `--` is forwarded to the launcher (`prterun`/`srun`)

See `--help` after the second `--` for the full set of recognized launch flags (`--pes`, `--pes-per-node`, `--threads-per-pe`, `--nodes` with `with-salloc`, `--output-dir`, `--gdb`, `--time`, etc.).

With `prterun`, `--pes` without `--pes-per-node` (or `--nodes`) means the PEs fill the nodes at hand: all of them on this node, or spread evenly over the nodes of the current SLURM allocation (`SLURM_JOB_NUM_NODES`). The default thread count and the `--map-by node:PE=<threads>` binding are sized for that layout. Pass `--pes-per-node` (or `--nodes`) to say otherwise.

# Features

- `use-prterun` — launch via `prterun` (PRRTE). Mutually exclusive with `use-srun`. Pulls in and reexports `prrte_sys`.
- `use-srun` — launch via `srun` (SLURM). Mutually exclusive with `use-prterun`.
- `with-salloc` — enables `--nodes`/`HPC_LAUNCH_NODES` to request a SLURM allocation via `salloc` and re-invoke the binary inside it, rather than just being parsed and left for the launcher to interpret.

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.
