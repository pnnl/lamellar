## Part 2: Intro to lamellar-runtime (Active Messages + Distributed Arrays)

Picks up right after Part 1 (`../part1_rust_basics/`). Everything here runs on a single
node using multiple local PEs (processes) via the `shmem` backend — no cluster, no `libfabric`,`ucx`,etc.
But these can easily be enabled via passing
`--features enable-libfabric`
`--features enable-ucx`
or adding them to  features section in the lamellar entry in cargo.toml 

### 0. Setup / orientation (~15 min)

- `Cargo.toml` points `lamellar` at the local `lamellar-runtime` checkout (v0.8.0, releasing
  before this tutorial) plus `rand`. No `rayon` in this tutorial — part 1's `Arc`/
  `AtomicUsize` thread demo (topic 10) already covers single-process concurrency; part 2 is
  exclusively about the *lamellar (distributed)* story.
- `Cargo.toml` also ports `lamellar-runtime`'s own `[profile.dev.build-override]`/
  `[profile.release]`/`[profile.release-dev]` settings — these only take effect from a
  top-level manifest, so the tutorial (its own separate Cargo root) needs its own copy.
  This cuts the first build (which compiles PMIx/PRRTE from source — vendored, not
  prebuilt) from ~21 min down to ~7 min. Build/run everything with `--profile release-dev`.
- Nomenclature:
  - **PE** (Processing Element) — one participant in the distributed program (a process, here).
  - **World** — the top-level handle representing all PEs; created once via
    `LamellarWorldBuilder::new().build()`.
  - **Team** — a subset of PEs that can collectively participate in operations (the World
    is itself the "world team"; arrays/AMs can be scoped to smaller teams too).
  - **Active Message (AM)** — a unit of code + data sent to (possibly) another PE for
    remote execution — "put work where the data is" rather than "bring data to the work."
  - **One-sided vs collective** — one-sided ops (e.g. sending an AM to one PE) don't require
    the target PE to explicitly participate in the call; collective ops (e.g. `world.barrier()`)
    require every PE in the team to call it.
- Every `fn main()` in this tutorial is annotated `#[lamellar::main]` — this attribute
  macro handles re-launching the binary under the PE launcher for you; you no longer need
  a separate `lamellar_run.sh` step.
- Running locally with multiple PEs: use the launcher-agnostic `--pes`/`--pes-per-node`
  flags, forwarded after a `--`:

  ```bash
  cargo run --profile release-dev --bin main -- -- --pes 4
  ```

  Everything before the first `--` is cargo's own args, between the two `--`s is forwarded
  to the app (unused here), after the second `--` is forwarded to the launcher. `--pes` =
  number of PEs. `--pes-per-node` only matters once you're spanning multiple nodes — skip
  it for this single-node local tutorial. Without `--lamellae` set (or set to `local`),
  omitting `--pes` just runs a single PE (`world.num_pes() == 1`) — useful for quick
  single-PE debugging.
  - **Try `--pes 4` right now, before setting anything else.** Every one of the 4 processes
    comes up as its own isolated single-PE world (all print PE 0, no shared data) — the
    default backend doesn't actually connect PEs together.
  - Now add `--lamellae shmem` and re-run the same command: the 4 processes join one
    real 4-PE world (distinct `my_pe()` values, shared/distributed array data). This is the
    backend that makes local multi-PE runs real — pass it for the rest of the tutorial:

    ```bash
    cargo run --profile release-dev --bin main -- -- --pes 4 --lamellae shmem
    ```
  - **Always pass `--pes N` explicitly once `--lamellae shmem` is set.** Omitting it
    doesn't default to 1 PE here — it fans out to one PE per NUMA domain on the node (e.g.
    16 PEs on a 16-NUMA-domain machine), which can surprise you with a much bigger run than
    intended.
  - **Run via `cargo run` at least once per binary before invoking it directly.** `cargo
    build` alone isn't enough — the RPATH/RUNPATH patching that lets a launched PE find
    libfabric/UCX/rofi (see main `lamellar-runtime` README) happens at `#[lamellar::main]`
    launch time, not at build time, so it only runs the first time you actually execute the
    binary. After that one `cargo run`, the binary's RPATH is patched in place and you can
    call it directly (e.g. `./target/release-dev/main -- --pes 4 --lamellae shmem`) without
    going through `cargo run` again — useful for scripting repeated runs without paying
    cargo's own startup overhead each time.

### 1. Active Messages basics (~45 min) — `src/main.rs`

- Build `LamellarWorldBuilder::new().build()`, inspect `world.my_pe()` / `world.num_pes()`.
- Define an AM: a plain struct annotated `#[AmData(Debug, Clone)]`, plus
  `#[am] impl LamellarAM for YourStruct { async fn exec(self) { ... } }`.
- Launch it with `world.spawn_am_all(...)` (every PE) or `world.spawn_am_pe(pe, ...)`
  (one specific PE) — both return a lazy request handle that does nothing until you call
  `.spawn()` (fire-and-forget, track completion later via `world.wait_all()`) or `.block()`
  (wait now, get any return value).
- `cargo run --profile release-dev --bin main` — has an intentional bug (missing
  `request.block()`): the runtime
  itself catches this (`Cargo.toml` enables the `runtime-warnings-panic` feature) and panics
  with `[LAMELLAR WARNING] You are dropping a MultiAmHandle that has not been 'await'ed,
  'spawn()'ed or 'block()'ed` — a live demo of the runtime telling you exactly what you
  forgot. Fix, or check `solutions/main.rs`.

### 2. Distributed Arrays basics (~45 min) — `src/main_array_basics.rs`

- `AtomicArray::<T>::new(world.team(), global_len, Distribution::Block | Distribution::Cyclic).block()`.
- `dist_iter_mut().enumerate().for_each(...).block()` — data-parallel iteration, each PE only
  touches its local slice (compare with part 1 topic 4's `.iter().map()...` chains).
- `onesided_iter()` gathers the whole array to one PE (data movement — use sparingly);
  `array.print()` shows each PE's own local slice without moving data.
- Bug in `src/main_array_basics.rs`: `onesided_iter()` runs on every PE instead of being
  guarded by `if world.my_pe() == 0`, so the full array prints once per PE. Fix, or check
  `solutions/main_array_basics.rs`.
- Make sure `--lamellae shmem` is passed (per Setup) before running this with `--pes > 1`
  — otherwise you're back to the isolated-PEs behavior from the setup demo.

### 3. AM deep dive (~30 min) — `src/main_am_deep_dive.rs`

- AMs can return values: `async fn exec(self) -> usize { ... }`, retrieved via
  `request.block()`.
- Local-only AMs (`#[AmLocalData]` / `#[local_am]`, launched with `world.spawn_am_local(...)`)
  never leave the issuing PE — no serialization needed, good for fanning work out across a
  PE's own worker threads.
- Bug: the fan-out loop drops each `spawn_am_local(...)` handle without `.spawn()`, so the
  runtime panics with `[LAMELLAR WARNING] You are dropping a LocalAmHandle that has not
  been 'await'ed, 'spawn()'ed or 'block()'ed` — the AM never ran, and `total` would have
  stayed 0. Fix, or check `solutions/main_am_deep_dive.rs`.

### 4. Array type survey (~30 min) — `src/main_array_types.rs`

Safety/use-case spectrum, foundation to most-guarded:

- `UnsafeArray` — no access control at all; every other array type is built from this.
- `ReadOnlyArray` — no writes permitted once created.
- `AtomicArray` — per-element atomic read/write.
- `LocalLockArray` — per-PE `RwLock` (multi-element transactions within a PE's local data).
- `GlobalLockArray` — one global `RwLock` across all PEs (strongest guarantee, most contention).

Convert between them with `.into_atomic()`, `.into_read_only()`, `.into_local_lock()`,
`.into_global_lock()` — each returns a new handle wrapped in `.block()`.

Bug: `atomic_array.into_read_only();` — the return value (the new `ReadOnlyArray` handle) is
dropped instead of bound, so the code below still (incorrectly) uses the old `atomic_array`
binding. Fix, or check `solutions/main_array_types.rs`.

### 5. Capstone (~35 min) — `src/main_capstone.rs`

Histogram, three ways (adapted from `../April_2025/src/main_histo.rs`, minus rayon):

1. **Serial** — plain `Vec<usize>`, single PE, single thread — baseline.
2. **Lamellar array** — `AtomicArray::batch_add(indices, 1)` — one line replaces the
   entire histogram loop, automatically distributed across every PE.
3. **Hand-rolled AM** — each PE's `HistoLaunch` (a local AM) routes indices to their owning
   PE, dispatching a `HistoAm` that increments counts in a `Darc<Vec<AtomicUsize>>` — the
   distributed analog of the `Arc<AtomicUsize>` from part 1 topic 10.

Run multi-PE to see it actually distribute:

```bash
cargo run --profile release-dev --bin main_capstone -- -- --pes 4 --lamellae shmem
```

Bug: `lamellar_am_histogram` never calls `world.wait_all()` / `world.barrier()` after
launching the `HistoLaunch` AMs, so the final sum can be read before every `HistoAm` has
landed and incremented the table — flaky/undercounted totals, more visible at `-N 4+`.
Fix, or check `solutions/main_capstone.rs`.

### Where to go next

`../../lamellar-runtime/examples/kernels/` has larger worked examples (GEMM, a DFT proxy)
if you want to keep going past this session.
