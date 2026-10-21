hpc_launch_codegen
==================

Not intended to be used directly. This crate holds the token generator behind the self-launching `main`/`test`
attribute macros:

- [`hpc_launch_impl`](../hpc_launch_impl/README.md) uses it for `#[hpc_launch::main]` / `#[hpc_launch::test]`.
- `lamellar_main_impl` uses it for `#[lamellar::main]` / `#[lamellar::test]`.

The generator is a plain library (a `proc-macro` crate can only export macros, so it cannot be shared between two
macro crates). A caller describes itself with a `LaunchConfig`: the tool name used in messages, the environment
variable prefix (`HPC_LAUNCH` for `hpc_launch`, `LAMELLAR` for lamellar), the crate path the generated code calls
helper functions through, extra launcher flags that are forwarded to the job as environment variables, and optional
code hooks. Everything else (argument parsing, `--pes`/`--nodes`/`--pes-per-node`/`--threads-per-pe` resolution,
`salloc`, `prterun`/`srun` flags, gdb wrapping, RPATH patching) exists once, here.
