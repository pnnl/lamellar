## Part 2: Intro to lamellar-runtime (Active Messages + Distributed Arrays)

Picks up right after Part 1 (`../part1_rust_basics/`). Everything here runs on a single
node using multiple local PEs (processes) via the `shmem` backend — no cluster, no `libfabric`,`ucx`,etc.
But these can easily be enabled via passing
`--features enable-libfabric`
`--features enable-ucx`
or adding them to  features section in the lamellar entry in cargo.toml 

**macOS**: `enable-ucx` and `enable-libfabric` are not supported on macOS — UCX's
build requires `-lrt` (Linux-only), and the `libfabric`/`libfabric-sys` bindings assume
Linux-only libfabric headers/errno codes. Stick to the default `shmem` backend on macOS.

### 0. Setup / orientation (~15 min)

- `Cargo.toml` points `lamellar` at crates.io v0.8.0 plus `rand`. No `rayon` in this
  tutorial — part 1's `Arc`/
  `AtomicUsize` thread demo (topic 10) already covers single-process concurrency; part 2 is
  exclusively about the *lamellar (distributed)* story.
- `Cargo.toml` also ports `lamellar-runtime`'s own `[profile.dev.build-override]`/
  `[profile.release]`/`[profile.release-dev]` settings — these only take effect from a
  top-level manifest, so the tutorial (its own separate Cargo root) needs its own copy.
  This cuts the first build (which compiles PMIx/PRRTE from source — vendored, not
  prebuilt) from ~21 min down to ~7 min. Build/run everything with `--profile release-dev`.
- **macOS**: hwloc's vendored autotools build fails there. See the commented-out
  `lamellar` dependency line in `Cargo.toml` for the fix (link a system hwloc instead) —
  requires `brew install hwloc libevent pkgconf` (plus `brew reinstall autoconf automake
  libtool` if `autoreconf` fails with a "bad interpreter: /usr/bin/perl5.30" error).
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
  cargo run --profile release-dev --bin hello -- -- --pes 4
  ```

  (`src/hello.rs` just prints `my_pe()`/`num_pes()` and calls `world.barrier()` — no AMs,
  no intentional bugs, so it's a clean way to see `--pes`/`--lamellae` in isolation before
  section 1 introduces the AM bug.)

  Everything before the first `--` is cargo's own args, between the two `--`s is forwarded
  to the app (unused here), after the second `--` is forwarded to the launcher. `--pes` =
  number of PEs. `--pes-per-node` only matters once you're spanning multiple nodes — skip
  it for this single-node tutorial. Without `--lamellae` set (or set to `local`),
  omitting `--pes` just runs a single PE (`world.num_pes() == 1`) — useful for quick
  single-PE debugging.
  - **Try `--pes 4` right now, before setting anything else.** Every one of the 4 processes
    comes up as its own isolated single-PE world (all print PE 0, no shared data) — the
    default backend doesn't actually connect PEs together.
  - Now add `--lamellae shmem` and re-run the same command: the 4 processes join one
    real 4-PE world (distinct `my_pe()` values). This is the backend that makes local
    multi-PE runs real — pass it for the rest of the tutorial:

    ```bash
    cargo run --profile release-dev --bin hello -- -- --pes 4 --lamellae shmem
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
- Launch it with `world.exec_am_all(...)` / `world.spawn_am_all(...)` (every PE) or
  `world.exec_am_pe(pe, ...)` / `world.spawn_am_pe(pe, ...)` (one specific PE) — these
  differ in when the AM actually gets submitted to the scheduler:
  - `exec_am_all`/`exec_am_pe` are **lazy** — building the handle does not submit the AM.
    Nothing runs until you call `.spawn()` (fire-and-forget, track completion later via
    `world.wait_all()`), `.block()` (wait now, get any return value), or `.await` it.
    Drop the handle without doing one of those and the AM *never runs* — the runtime
    catches this for you (`Cargo.toml` enables the `runtime-warnings-panic` feature) and
    panics with `[LAMELLAR WARNING] You are dropping a MultiAmHandle that has not been
    'await'ed, 'spawn()'ed or 'block()'ed`.
  - `spawn_am_all`/`spawn_am_pe` are **eager** — the AM is submitted to the scheduler
    immediately, before the call even returns. The handle you get back is only for
    tracking/collecting results; dropping it doesn't stop the AM from running, it just
    means you never observe when it finishes.
- `cargo run --profile release-dev --bin main` — has an intentional bug: the first AM is
  launched with `world.exec_am_all(...)` and its handle dropped without `.spawn()`/
  `.block()`, so it never runs at all, and the runtime panics with the `MultiAmHandle`
  warning above. Fix by adding `.block()` (or `.spawn()` + a later `wait_all()`).
  Once that's fixed, notice the *second* AM in the file — launched with
  `world.spawn_am_all(...)` and also dropped — does not panic (it's eager, so it already
  ran). Its `println!` is still guaranteed to land before `main` returns: dropping
  `LamellarWorld` runs an implicit `barrier(); wait_all(); barrier();`, so any AM already
  submitted to the scheduler (eager `spawn_am_all`/`spawn_am_pe`) is waited on even if you
  never touch its handle. That implicit wait is exactly why the *lazy* `exec_am_all` case
  is the dangerous one: an AM that was never submitted has nothing for `wait_all()` to
  wait on, so the dropped-handle panic is your only signal something didn't run.
  Check `solutions/main.rs` for both fixed.

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
- `spawn_am_local` is **eager** — like `spawn_am_all`/`spawn_am_pe`, it's already submitted
  to the scheduler when the call returns; dropping the handle doesn't stop it running.
- Bug: the fan-out loop never waits for the fanned-out `LocalSumAm`s to finish before
  reading `total` — a race. The final print may show a partial (or even zero) sum depending
  on how fast the AMs land relative to that line. Fix by adding a wait (e.g.
  `world.wait_all()`) before the print, or check `solutions/main_am_deep_dive.rs`.

### 4. Array type survey (~30 min) — `src/main_array_types.rs`

Safety/use-case spectrum, foundation to most-guarded:

- `UnsafeArray` — no access control at all; every other array type is built from this.
- `ReadOnlyArray` — no writes permitted once created.
- `AtomicArray` — per-element atomic read/write.
- `LocalLockArray` — per-PE `RwLock` (multi-element transactions within a PE's local data).
- `GlobalLockArray` — one global `RwLock` across all PEs (strongest guarantee, most contention).

Convert between them with `.into_atomic()`, `.into_read_only()`, `.into_local_lock()`,
`.into_global_lock()` — each returns a new handle wrapped in `.block()`.

Bug: `atomic_array.into_read_only();` — `into_read_only()` takes `self` by value, so this
consumes `atomic_array`. Dropping the returned handle instead of binding it doesn't compile:
the code below's use of `atomic_array` is a moved-value error (E0382). Fix by binding it
(`let read_only_array = atomic_array.into_read_only().block();`), or check
`solutions/main_array_types.rs`.

### 5. Capstone (~35 min) — `src/main_capstone.rs`

Histogram, four ways (adapted from `../April_2025/src/main_histo.rs`, minus rayon):

1. **Serial** — plain `Vec<usize>`, single PE, single thread — baseline.
2. **UnsafeArray** — `UnsafeArray::batch_add(indices, 1)` — same one-line batch op as step 3,
   but on the unsynchronized foundation array type every other array type is built on top of.
3. **Lamellar array** — `AtomicArray::batch_add(indices, 1)` — one line replaces the
   entire histogram loop, automatically distributed across every PE.
4. **Hand-rolled AM** — each PE's `HistoLaunch` (a local AM) routes indices to their owning
   PE, dispatching a `HistoAm` that increments counts in a `Darc<Vec<AtomicUsize>>` — the
   distributed analog of the `Arc<AtomicUsize>` from part 1 topic 10.

Run multi-PE to see it actually distribute:

```bash
cargo run --profile release-dev --bin main_capstone -- -- --pes 4 --lamellae shmem
```

Bug 1: `lamellar_unsafe_histogram` doesn't compile as written — `UnsafeArray`'s ops
(`batch_add`, `sum`) are `unsafe fn`, unlike `AtomicArray`'s. Fix by wrapping both calls
in `unsafe { ... }`, or check `solutions/main_capstone.rs`.

Bug 2: `lamellar_am_histogram` never calls `world.wait_all()` / `world.barrier()` after
launching the `HistoLaunch` AMs, so the final sum can be read before every `HistoAm` has
landed and incremented the table — flaky/undercounted totals, more visible at `-N 4+`.
Fix, or check `solutions/main_capstone.rs`.

### Where to go next

`../../lamellar-runtime/examples/kernels/` has larger worked examples (GEMM, a DFT proxy)
if you want to keep going past this session.
