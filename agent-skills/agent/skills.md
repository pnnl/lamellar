---
name: writing-lamellar-applications
description: >
  Use when writing, debugging, or reviewing Lamellar (distributed Rust) programs:
  active messages (AMs), distributed arrays, Darcs, memory regions/RDMA, and
  multi-PE + multi-threaded execution. Load this skill whenever a task involves
  parallelizing Rust across processing elements (PEs) with the `lamellar` crate,
  or when diagnosing serialization/handle/deadlock errors in Lamellar code.
version: 0.8.0
---

# Writing Lamellar Applications

> Every code claim in this skill was checked against the real sources under
> `examples/` and the crate manifest (`Cargo.toml` -> `lamellar = 0.8.0`; the
> README's `0.7.0-rc.1` is stale). Prefer the version that matches your target.

**The difficulty in Lamellar is almost never the parallelism — it is serialization boundaries.**
`Arc` is a process-local pointer and is meaningless on another PE.
Never try to "send" or "share" a live object across PEs.
Send inputs -> rebuild remotely -> return only data.

---

## 0. Terminology

Lamellar-specific terms and acronyms used throughout this skill. (Standard Rust
vocabulary — `Arc`, `Rc`, `Mutex`, `RwLock`, `Serialize`/`Deserialize`, `Option`,
`Result`, futures/`.await` — is assumed and not redefined here.)

| Term | Meaning |
|------|---------|
| **PE** | Processing Element — one process/rank in the distributed job. Each PE owns part of the data and runs its own worker threads. |
| **AM** | Active Message — a serializable struct shipped to and executed on a remote PE (via `#[lamellar::am]` + `exec`). |
| **Darc** | Distributed Arc — a reference-counted handle where **each PE holds its own local instance** of the value; the cross-PE analogue of `Arc`. |
| **RW lock** | Reader-writer lock: allows many concurrent readers **or** one exclusive writer. In Lamellar what differs between `LocalRwDarc` and `GlobalRwDarc` is the lock's *scope* (per-PE vs. collective), not the lock semantics. |
| **collective op** | An operation every PE must call together — it acts as a synchronization point across all PEs (e.g. `barrier()`, a `GlobalRwDarc` lock, `dist_iter` instantiation). |
| **one-sided op** | An operation a single PE performs without coordinating with the others (e.g. `local_iter`, an RDMA `get`/`put`). |
| **team** | A group of PEs an operation runs over. `world.team()` is the default team of all PEs; sub-teams address a subset. |
| **world** | The top-level handle to the whole Lamellar runtime/job, built by `LamellarWorldBuilder`. Inside an AM, use the injected `lamellar::world` global instead of the outer `main` handle. |
| **RDMA** | Remote Direct Memory Access — reading/writing another PE's memory directly, without involving that PE's CPU (the `memregion` APIs). |
| **shmem / rofi** | Runtime backends: `shmem` = shared memory (multiple PEs on one node); `rofi` = network backend for multi-node/distributed runs. `local` is the single-PE default. |
| **`typed_am_group!` / `AmGroup`** | A macro/type for aggregating many AMs into one batched launch; results come back as a `TypedAmGroupResult<T>` (see §6), NOT a `Vec<T>`. |

---

## 1. Procedure to follow BEFORE writing AM code

When asked to parallelize, **DO NOT code first.** Work through these steps and get
approval before writing the active message:

1. **INVENTORY** every value that crosses a PE boundary — inputs sent in and
   results returned out — with its concrete type.
2. **CLASSIFY** each value as:
   - `SERIALIZABLE` — plain data; can derive/already derives Serialize/Deserialize.
   - `NON-SERIALIZABLE` — contains Arc/Rc, raw pointers, locks, file/thread handles,
     closures/trait objects, or is a live domain object.
   - `UNKNOWN` — cannot tell from the code shown. Ask for the type definition; do
     NOT assume it is serializable.
   - Also flag `EXPENSIVE` — serializable but large/costly to send; prefer a
     cheaper representation (e.g. a raw `Vec` instead of a rich matrix type).
3. **PLAN the data flow**:
   - For each NON-SERIALIZABLE *input*: identify the serializable constructor
     inputs, ship those instead, and rebuild the object inside `exec()`.
   - For each NON-SERIALIZABLE *result*: extract the serializable data into a
     dedicated result struct and return that.
4. **PLAN the launch + gather**: choose the launch API and where results land
   (`exec_am_all` -> `Vec<T>` on caller; `typed_am_group!` -> `TypedAmGroupResult<T>`;
   `exec_am_pe` for a single PE). Confirm result-collection type before coding.

Then stop and get approval before writing code.

---

## 2. Quick Reference

### Imports by use-case

| Doing... | Import |
|----------|--------|
| Active messages | `use lamellar::active_messaging::prelude::*;` |
| Distributed arrays | `use lamellar::array::prelude::*;` |
| Darcs | `use lamellar::darc::prelude::*;` |
| Memory regions / RDMA | `use lamellar::memregion::prelude::*;` |

Prefer the prelude over bare `use lamellar::ActiveMessaging;` — the prelude also
brings in `LamellarWorldBuilder`, `LamellarTaskGroup`, `typed_am_group!`, and the
macros.

### The handle rule (applies to almost every remote op)

Array constructors, `sum()`, `exec_am_*`, iterator consumers, etc. all return a
**handle/future**. You must realize it with exactly one of:

- `.block()` — block the current (non-AM) thread until done.
- `.spawn()` — launch as a task, returns a handle to await later.
- `.await` — inside async code or an AM body.

Dropping a handle without driving it can silently skip the work (the runtime emits
an "unexecuted remote operation" warning).

### Do / Don't

| Do | Don't |
|-------|----------|
| `.await` other AMs inside `async fn exec` | call `.block()` inside an AM (**deadlocks**) |
| use `lamellar::world` / `lamellar::team` / `lamellar::current_pe` inside an AM | use the outer `main`'s `world` handle inside an AM |
| `.block()` / `block_on(...)` from `main` | block a worker thread with `std::thread::sleep` in async code |
| pick the array type by access pattern (see §5) | default to `UnsafeArray` without understanding the `unsafe` contract |
| `array.barrier()` before reading results written collectively | assume ordering without a barrier |
| carry the **global** `index` in an AM for per-item decisions | rely on a local per-PE counter |

### Failure signatures -> fix

| Symptom | Cause | Fix |
|---|---|---|
| `Arc<...>` / live object doesn't implement `Serialize` | trying to send a process-local pointer across PEs | send serializable inputs, rebuild inside `exec()` |
| `#[AmData]` derive fails: a field isn't `Serialize` | an owned field type has no `Serialize`/`Deserialize` derive (distinct from an un-serializable pointer) | derive Serialize/Deserialize on the field type, or convert it to a plain `Vec` and rebuild inside `exec()` |
| `Darc` doesn't put one value on PE 0 | `Darc` gives each PE a *local* instance; `fetch_add` "only updates atomic on executing pe" | use return-and-collect (`exec_am_all` / task group) |
| type mismatch: `let v: Vec<T> = group.exec()...` | `typed_am_group!` returns `TypedAmGroupResult<T>`, NOT `Vec<T>` | drain via `.len()` + `.at(i)`, match `AmGroupResult::Pe(_pe, val)` where `val: &T` (so `val.clone()`) |
| deadlock inside an AM | called `.block()` in `async fn exec` | use `.await` |
| "unexecuted remote operation" warning | dropped a handle without driving it | `.block()` / `.spawn()` / `.await` it |
| tasks serialize with `LAMELLAR_THREADS>1` | blocking work (e.g. `std::thread::sleep`) in an async AM body starves the worker | use `.await` on real futures (e.g. `async_std::task::sleep().await`) |
| per-item odd/even logic wrong on multi-PE | used a local per-PE counter | carry the global `index` in the AM, decide remotely |
| wrong `world` used inside `exec` | used outer `main` handle | use injected `lamellar::world` / `lamellar::team` / `lamellar::current_pe` |
| `UnsafeArray::sum()` won't compile | reduction needs the `unsafe` contract | wrap: `unsafe { array.sum().block() }` |

### Run commands

```bash
cargo run --release --example <name>                          # 1 PE (local backend)
./lamellar_run.sh -N=4 -T=8 ./target/release/examples/<name>  # 4 PEs x 8 threads (shmem)
srun -N 2 --mpi=pmi2 ./target/release/examples/<name>         # distributed
```

### Verification checklist (run before claiming done)

- [ ] Import matches the use-case (§2 table) — AM code uses the
      `active_messaging::prelude`, array code uses the `array::prelude`.
- [ ] Every handle/future is realized with exactly one of `.block()` /
      `.spawn()` / `.await` — no dropped handles.
- [ ] No `.block()` inside `async fn exec` (would deadlock); use `.await`.
- [ ] Inside `exec`, `world`/`team`/`current_pe` refer to the `lamellar::` globals,
      not the outer `main` handle.
- [ ] The result-collection type is correct: `exec_am_all` -> `Vec<T>`;
      `typed_am_group!` -> `TypedAmGroupResult<T>`; array `sum()` -> `Option<T>`;
      iterator `sum()` -> plain `T` (see §6).
- [ ] `UnsafeArray` reductions/ops are wrapped in `unsafe { .. }`.
- [ ] A `barrier()` precedes any read of results written collectively.
- [ ] It builds: `cargo build --release --examples`.

---

## 3. Canonical, copy-paste-correct Active Message

One end-to-end AM — imports through launch and collect — using the `self`-by-value
`exec` signature that the real examples use.

```rust
use lamellar::active_messaging::prelude::*;

// 1. Define the message payload. `AmData` replaces `derive` and makes the struct
//    serializable so it can travel to remote PEs.
#[AmData(Debug, Clone)]
struct SquareSum {
    n: usize, // compute the sum of squares 0..n on each PE
}

// 2. Implement LamellarAM. The #[lamellar::am] macro registers it for remote exec.
//    Inside exec, use the lamellar:: globals (NOT the outer `world`).
#[lamellar::am]
impl LamellarAM for SquareSum {
    async fn exec(self) -> usize {
        let pe = lamellar::current_pe;
        // do the per-PE work; .await other AMs here if needed (never .block())
        (0..self.n).map(|x| x * x).sum::<usize>() + pe
    }
}

// 3. Build the world and launch.
#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();
    world.barrier();

    // launch on ONE specific PE, block for its single result
    let one = world.exec_am_pe(num_pes - 1, SquareSum { n: 10 }).block();

    // launch on ALL PEs; exec_am_all returns Vec<usize> collected on THIS PE
    let all: Vec<usize> = world.exec_am_all(SquareSum { n: 10 }).block();

    if my_pe == 0 {
        println!("result from last PE: {one}");
        println!("results from all PEs (indexed by PE id): {all:?}");
    }
    world.barrier();
}
```

Checklist this snippet encodes:

- **Import:** `lamellar::active_messaging::prelude::*` (brings in the builder,
  macros, and launch methods).
- **Macros:** `#[AmData(..)]` on the struct, `#[lamellar::am]` on the `impl`.
- **Signature:** `async fn exec(self) -> T` (by value). A no-return AM just omits
  `-> T`. Use `#[lamellar::local_am]` + `#[AmLocalData(..)]` for a *local-only* AM
  that skips serialization (`am_local.rs`).
- **Launch (from `main`, outside any AM):** `exec_am_pe(pe, am).block()` for one PE,
  `exec_am_all(am).block()` for all (returns a `Vec<T>` on the caller).
- **Inside `exec`:** use `lamellar::current_pe` / `lamellar::world` / `lamellar::team`;
  `.await` nested AMs; never `.block()`.

> **`exec_am_all` and `typed_am_group!` return DIFFERENT types** (verified in
> `src/lamellar_task_group.rs`):
> - `world.exec_am_all(am).block()` -> **`Vec<T>`** (indexed by PE id).
> - `typed_am_group!(...).exec().await` / `block_on(..)` -> **`TypedAmGroupResult<T>`**,
>   consumed via `.len()` + `.at(i)` (each `.at(i)` is an `AmGroupResult<'_, T>` —
>   `AmGroupResult::Pe(pe, &T)` for an `add_am_pe` AM, `AmGroupResult::All(iter)` for
>   an `add_am_all` AM). It is **not** a `Vec<T>` — see §6.

Reference examples: `examples/active_message_examples/am_return_usize.rs`,
`examples/hello_world/hello_world_am.rs`, `examples/active_message_examples/am_local.rs`.

---

## 4. Multi-processing (PEs) + Multi-threading (within a PE)

Lamellar gives you **two nested layers** of parallelism over a distributed
collection (the "vector of things"):

1. **Multi-processing (across PEs):** the vector is a `LamellarArray` split across
   PEs via `Distribution::Block` or `Distribution::Cyclic`. Each PE owns a chunk.
2. **Multi-threading (within a PE):** when you drive a `dist_iter()` / `local_iter()`
   with `for_each(...)`, the closure runs **concurrently across that PE's worker
   threads** (controlled by `LAMELLAR_THREADS`).

### `dist_iter` vs `local_iter`

| Entry point | Scope | Cross-PE sync? | Reduction result |
|-------------|-------|----------------|------------------|
| `dist_iter()` / `dist_iter_mut()` | each PE's local chunk | **collective** (instantiation is a sync point) | combined **global** result |
| `local_iter()` / `local_iter_mut()` | each PE's local chunk | **one-sided** (no cross-PE sync) | **per-PE** partial result |

Both drive their closures across the PE's worker threads, so both are
multithreaded; they differ only in whether cross-PE coordination happens.
`enumerate()` yields the **global** index (`parallel_array_gemm.rs` uses this to
build an identity matrix from the global position).

### Snippet

```rust
use lamellar::array::prelude::*;

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    let num_pes = world.num_pes();

    // Distributed array split across PEs (multi-processing).
    let len = 100 * num_pes;
    let array = AtomicArray::<usize>::new(world.team(), len, Distribution::Block).block();

    // MULTI-PROCESSING + MULTI-THREADING: each PE processes only its local chunk,
    // for_each runs the closure concurrently across that PE's worker threads.
    // enumerate() yields the GLOBAL index.
    array
        .dist_iter_mut()
        .enumerate()
        .for_each(move |(i, elem)| elem.store(i * 2))
        .block(); // collective: waits for all PEs' threads to finish
    array.barrier();

    // MULTI-THREADED, PER-PE REDUCTION (one-sided): only this PE's chunk.
    // Returns a PER-PE partial (no cross-PE combine).
    let local_sum = array.local_iter().map(|e| e.load()).reduce(|a, b| a + b).block();
    println!("[PE {my_pe}] local partial sum = {:?}", local_sum);
    array.barrier();

    // MULTI-PROCESSING GLOBAL REDUCTION: reduces locally on each PE's threads,
    // then the runtime combines partials ACROSS PEs into one global result.
    let global_sum = array.dist_iter().map(|e| e.load()).sum().block();
    if my_pe == 0 {
        println!("global sum across all PEs = {global_sum}");
    }
}
```

- `-N` sets the number of PEs (processes / multi-processing layer).
- `-T` sets threads per PE; equivalently `LAMELLAR_THREADS`. With `cargo run` you
  get 1 PE but still multiple worker threads, so `dist_iter`/`local_iter` closures
  already run multithreaded.

### Other ways multithreading/multi-processing shows up in the examples

The `dist_iter`/`local_iter` + `for_each`/`sum`/`reduce` form above is the common
case, but the examples use several more patterns. Each is verified against the
cited source.

**A. Control how per-PE work is split across threads — `Schedule` + `*_with_schedule`.**
Every iterator consumer has a `_with_schedule` variant taking a `Schedule`
(`array_consumer_schedules.rs`):

```rust
use lamellar::array::prelude::*; // Schedule is in this prelude

array
    .local_iter()
    .filter(|e| e.load() % 2 == 0)
    .for_each_with_schedule(Schedule::Dynamic, move |e| { /* ... */ })
    .block();
```

Consumers with schedule variants: `for_each_with_schedule`, `reduce_with_schedule`,
`collect_with_schedule::<C>(sched, dist)`, `count_with_schedule`,
`sum_with_schedule`. Schedules: `Schedule::Static | Dynamic | Chunk(n) | Guided |
WorkStealing`. This is the knob for *how* the multithreading is distributed;
the plain consumers use the default schedule.

**B. Three ways to drive a parallel iterator (ties into the §2 handle rule).**
The same consumer can be realized three ways (`array_consumer_schedules.rs`):

- `.block()` — from `main`.
- `.spawn()` then `array.wait_all()` (join all outstanding tasks on this PE), and
  `handle.block()` later to read the result.
- inside `array.block_on(async move { let r = ...consumer....await;
  array.async_barrier().await; })` — note `async_barrier().await`, the async form
  of `barrier()`.

`wait_all()` (on `world` or an array) is the idiomatic "join everything spawned on
this PE" call, used heavily alongside `barrier()`.

**C. AM fan-out multithreading (no array).** Launch many AMs and let the PE's
worker threads execute them concurrently (`am_flops.rs`):

```rust
let mut reqs = vec![];
for _ in 0..num_tasks {
    reqs.push(world.spawn_am_all(FlopAM { iterations })); // spawn_am_all: lazy multi-PE launch
}
world.wait_all();
let total: usize = reqs.drain(..).map(|r| r.block().drain(..).sum::<usize>()).sum();
```

`spawn_am_all` is the `.spawn()`-style sibling of `exec_am_all` (see §3/§6). This is
multithreading via task fan-out rather than distributed iterators.

**D. Hybrid / nested parallelism.** Outer one-sided iterator over remote data
feeding an inner spawned local iterator (`parallel_array_gemm.rs`):

```rust
b.onesided_iter().chunks(p).into_iter().enumerate().for_each(|(j, col)| {
    let (col, c) = (col.clone(), c.clone());
    let _ = a.local_chunks(n).enumerate().for_each(move |(i, row)| {
        let sum = col.iter().zip(row).map(|(&x, &y)| x * y).sum::<f32>();
        c.mut_local_data().at(j + (i % rows_pe) * m).fetch_add(sum); // direct local write
    }).spawn();
});
world.wait_all();
world.barrier();
```

Uses `onesided_iter().chunks(..)`, `local_chunks(..)`, and
`c.mut_local_data().at(idx).fetch_add(..)` for direct local updates, all joined via
`wait_all()` + `barrier()`.

Reference examples: `examples/array_examples/distributed_iteration.rs`,
`examples/array_examples/local_iteration.rs`,
`examples/array_examples/dist_array_reduce.rs`,
`examples/array_examples/array_consumer_schedules.rs` (schedules + block/spawn/await),
`examples/kernels/am_flops.rs` (AM fan-out),
`examples/kernels/parallel_array_gemm.rs` (hybrid/nested).

---

## 5. Which array type to pick?

The single most consequential design decision. All are constructed the same way
(`Ty::<T>::new(world.team(), len, Distribution::Block).block()`) and differ only in
how you may access elements and whether operations need `unsafe`.

| Array type | Element access | `unsafe` for reductions/ops? | When to use |
|------------|----------------|------------------------------|-------------|
| `ReadOnlyArray<T>` | immutable after build | no | data that never changes after init; fastest reads |
| `AtomicArray<T>` | per-element atomic `load()`/`store()`/`fetch_*` | **no** | concurrent element updates from many threads/PEs |
| `LocalLockArray<T>` | `&`/`&mut` slices under a local RW lock | no | bulk local access; coarse-grained locking |
| `GlobalLockArray<T>` | global RW lock across all PEs | no | globally coordinated exclusive access |
| `UnsafeArray<T>` | raw `&`/`&mut` (no checks) | **yes** — wrap in `unsafe {}` | max performance; you guarantee no data races |

Verified gotcha:

- `AtomicArray` / `ReadOnlyArray`: `let s = array.sum().block();` — **no `unsafe`**
  (`array_batch_add.rs`, `parallel_array_gemm.rs`).
- `UnsafeArray`: `let s = unsafe { array.sum().block() };` — the `unsafe {}` block is
  **required** (`array_am.rs`, `dist_array_reduce.rs`).

Conversions exist between types, e.g. `array.into_read_only().block()` or
`array.into_unsafe().block()` (`array_batch_add.rs`, `dist_array_reduce.rs`).

Reference examples: `examples/array_examples/global_lock_array.rs`,
`examples/bandwidths/readonly_array_get_bw.rs`, `examples/array_examples/array_am.rs`.

---

## 6. Return-Type Cheat Sheet (don't confuse these)

Every claim below is checked against the runtime source (`src/active_messaging/handle.rs`,
`src/lamellar_task_group.rs`, `src/array/*.rs`, `src/array/iterator/**`).

- `world.exec_am_all(am).block()` -> **`Vec<T>`** on the calling PE, indexed by PE id.
  If only PE 0 calls it, this IS the gather-to-PE-0. (`MultiAmHandle<T>::block -> Vec<T>`)
- `world.exec_am_pe(pe, am).block()` -> single **`T`** from that PE.
  (`AmHandle<T>::block -> T`)
- `world.exec_am_local(am).block()` -> single **`T`**, executed locally.
  (`LocalAmHandle<T>::block -> T`)
- `.spawn()` on any `exec_am_*` returns a `LamellarTask<Output>` (same `Output` as
  `.block()`), and `.await` yields that same `Output`. All three are lazy — you MUST
  drive the handle or the work may be skipped (see §2 handle rule).
- `typed_am_group!(...).exec()` (via `.await` / `block_on`) -> **`TypedAmGroupResult<T>`**,
  consumed with `.len()` + `.at(i)`; each `.at(i)` is `AmGroupResult<'_, T>`:
  `Pe(usize, &T)` for `add_am_pe`, `All(iter)` for `add_am_all`. It is **NOT** a `Vec<T>`.
- **Array-level reductions are ALWAYS `Option<T>`.**
  `array.sum().block()` / `.prod()` / `.max()` / `.min()` / `.reduce(op)` ->
  **`Option<T>`** on every array type. Realize it with `.block().expect("len > 0")`
  (exactly what `histo.rs` does). `UnsafeArray::sum()` is additionally `unsafe`:
  `unsafe { array.sum().block() }.expect("len > 0")`.
- **Iterator-level reductions differ by consumer:**
  - `array.dist_iter().map(..).sum().block()` -> **plain `T`** (NOT `Option`; global combine).
  - `array.local_iter().map(..).sum().block()` -> **plain `T`** (per-PE partial).
  - `array.dist_iter()/local_iter().reduce(op).block()` -> **`Option<T>`**.
  - `.count().block()` -> **`usize`**.
  - `.collect::<C>().block()` -> **`C`** (the collection you name).
  - `.for_each(..).block()` -> **`()`**.
- Array constructor `Ty::<T>::new(team, len, dist).block()` -> the **array** itself
  (also a handle you must `.block()`/`.spawn()`/`.await`).

### The one that bites people

`array.sum()` (array-level, `Option<T>`) vs. `array.dist_iter().map(..).sum()`
(iterator-level, plain `T`) look almost identical but return different types. If the
compiler complains about `Option<T>` where you expected `T` (or vice versa), check
which `sum()` is actually called.

---

## 7. Gathering Per-PE Computations Onto PE 0

A very common pattern: every PE computes a partial over its local data, then you
collect all partials **on PE 0**. The clean idiom is an AM that **returns** its
partial, driven by `exec_am_all` from PE 0.

`world.exec_am_all(am).block()` runs the AM on **every** PE and returns a `Vec<T>`
**on the calling PE**, indexed by PE id. So if **only PE 0 calls it**, the whole
result vector is assembled on PE 0 — that is the gather. `am_return_usize.rs` proves
this: it asserts `res == (0..num_pes).collect::<Vec<usize>>()`.

```rust
use lamellar::active_messaging::prelude::*;

#[AmData(Debug, Clone)]
struct ComputePartial { base: usize }

#[lamellar::am]
impl LamellarAM for ComputePartial {
    async fn exec(self) -> usize {
        let pe = lamellar::current_pe;
        (0..100).map(|x| x + self.base + pe).sum()
    }
}

#[lamellar::main]
fn main() {
    let world = lamellar::LamellarWorldBuilder::new().build();
    let my_pe = world.my_pe();
    world.barrier();

    // Only PE 0 launches exec_am_all, so the Vec of results is COLLECTED ON PE 0.
    if my_pe == 0 {
        let partials: Vec<usize> = world.exec_am_all(ComputePartial { base: 1 }).block();
        let total: usize = partials.iter().sum();
        println!("PE 0 gathered partials {:?}", partials);
        println!("PE 0 combined total = {total}");
    }
}
```

### Alternatives / caveats

- **Push instead of pull:** any PE can send its result directly to PE 0 with
  `world.exec_am_pe(0, am)` (`am_return_usize.rs`).
- **`LamellarTaskGroup`:** collects returned results the same way as `exec_am_all` —
  useful when aggregating many AMs (`am_return_usize.rs`).
- **Darc caveat:** a plain `Darc<AtomicUsize>` is *not* a single global counter. As
  `examples/darc_examples/darc.rs` notes, `fetch_add` inside an AM "only updates
  atomic on the executing pe", so it does **not** substitute for the
  return-and-collect gather when you need one combined value on PE 0. The same
  local-instance caveat applies to `LocalRwDarc`/`GlobalRwDarc` — see §9.

Reference examples: `examples/active_message_examples/am_return_usize.rs`,
`examples/darc_examples/darc.rs`.

---

## 8. Async Patterns (rules + traps)

### Correct patterns (verified in the examples)

- **Await another AM from *inside* an AM.** `recursive_am.rs`:
  `let next = lamellar::world.exec_am_pe(next_pe, ..); let res = next.await;`
  Launching-and-awaiting nested AMs is fully supported.
- **Block from *outside* an AM** (e.g. in `main`): `request.block()` or
  `world.block_on(async move { grp.exec().await })` (`async_comparison.rs`).
- **Do real async work inside an AM** with `.await` on genuine futures:
  `async_std::task::sleep(Duration::from_secs(s)).await` (`async_comparison.rs`).

### Traps — the actual lessons

1. **`block()` inside an AM deadlocks.** `recursive_am.rs` literally comments the
   wrong line: `// let mut res = next.block().expect(...); // this will cause deadlock`.
   **Rule:** inside `async fn exec`, always `.await`; never `.block()`.
2. **Blocking the worker thread starves other tasks.** `async_comparison.rs`
   contrasts `std::thread::sleep` (blocks the worker) with
   `async_std::task::sleep().await` (yields it). With `LAMELLAR_THREADS=1` the
   blocking version serializes all tasks. **Rule:** don't do blocking I/O/compute
   inside async AM bodies if you want concurrency.
3. **Inside an AM, `world`/`team`/`current_pe`/`num_pes` are the runtime-injected
   `lamellar::` globals**, NOT the outer `main` handle. `recursive_am.rs` uses
   `lamellar::world.exec_am_pe(..)` and `lamellar::team.num_pes()`.
4. **Async-launched groups still need to be driven.** `AmGroup` /
   `typed_am_group!` don't run until you `.exec().await` (or `block_on`) them
   (`async_comparison.rs`). Dropping them triggers the "unexecuted remote
   operation" warning.
5. **Handles are lazy-ish:** `spawn()` returns a handle you must eventually
   `.block()`/`.await`; dropping it without awaiting can drop the work.

Reference examples: `examples/active_message_examples/recursive_am.rs`,
`examples/active_message_examples/async_comparison.rs`.

---

## 9. Darc Variants (`Darc` / `LocalRwDarc` / `GlobalRwDarc`)

`Darc<T>` (Distributed Arc) is a reference-counted handle to a value that each PE
holds a **local instance** of — the cross-PE analogue of `Arc`. Three variants
differ only in how you may mutate the inner value and the scope of the lock. All
are imported via `use lamellar::darc::prelude::*;`.

| Type | Interior mutability | Lock scope | Access API | Construct |
|------|--------------------|-----------|------------|-----------|
| `Darc<T>` | none (immutable shared) | — | deref to `&T`; use `Mutex`/`RwLock`/atomics inside for mutation | `Darc::new(&world, v).block().unwrap()` |
| `LocalRwDarc<T>` | `RwLock` | **process-local** (each PE locks its own copy) | `.read()` / `.write()` -> guard, driven by `.await` or `.block()` | `LocalRwDarc::new(&world, v).block().unwrap()` |
| `GlobalRwDarc<T>` | `RwLock` | **global** across all PEs (collective) | `.read()` / `.write()` -> guard, driven by `.block()` or `.await` | `GlobalRwDarc::new(world.team(), v).block().unwrap()` |

### Verified facts (from the example sources)

- **Construction returns a handle** you must realize (`.block()`/`.await`) and then
  `.unwrap()` (it yields a `Result`): e.g.
  `LocalRwDarc::new(&world, 0).block().unwrap()` (`lamellar_env.rs`),
  `GlobalRwDarc::new(world.team(), 0).block().unwrap()` (`darc.rs`).
- **`LocalRwDarc`** — `.write().await` / `.read().await` inside an AM
  (`string_darc.rs`, `dist_hashmap.rs`), or `.write().block()` / `.read().block()`
  from `main` (`darc.rs`). Its lock is **per-PE**: it serializes access to *that
  PE's* local copy only.
- **`GlobalRwDarc`** — `.read().block()` / `.write().block()` (`darc.rs`), or
  `.read().await` / `.write().await` inside an AM. Its lock is **collective**:
  acquiring it coordinates across all PEs.
- **Plain `Darc`** — no lock; deref for reads. Mutating shared state requires an
  inner `AtomicUsize`/`Mutex` etc. (`stress_test.rs`: `Darc<AtomicUsize>` with
  `fetch_add`). As `darc.rs` notes, that `fetch_add` "only updates atomic on the
  executing pe".

### Key gotcha

All three give each PE a **local instance** of the inner value — none of them
teleport a single shared value onto PE 0. `LocalRwDarc`/`GlobalRwDarc` add a lock
around that value; they do **not** change the fact that mutations are local to the
PE that performs them (`GlobalRwDarc` coordinates the *lock* globally, but each PE
still sees its own instance). To collect one combined value on PE 0, use the
return-and-collect gather (`exec_am_all` / task group, §7), not a Darc.

- Use `Darc<T>` for immutable shared data (or `Darc<Atomic*>` for per-PE atomics).
- Use `LocalRwDarc<T>` for a mutable per-PE structure (e.g. a local shard of a
  distributed map — `dist_hashmap.rs`).
- Use `GlobalRwDarc<T>` when you need globally coordinated exclusive access under a
  collective lock.

Reference examples: `examples/darc_examples/darc.rs` (all three side by side),
`examples/darc_examples/string_darc.rs` (`LocalRwDarc<String>`),
`examples/misc/dist_hashmap.rs` (`LocalRwDarc<HashMap<..>>`),
`examples/kernels/parallel_blocked_array_gemm.rs` (`GlobalRwDarc`),
`examples/misc/lamellar_env.rs` (all three constructed together).

---

## 10. Project Setup (Cargo.toml + build/run)

The in-repo crate is version **`0.8.0`** (`Cargo.toml`). The README still says
`0.7.0-rc.1`, which is stale — use the version that matches your target.

```toml
# Cargo.toml — workstation (local + shmem backends)
[dependencies]
lamellar = "0.8"

# distributed HPC (adds a network backend; requires libfabric/ROFI toolchain)
# lamellar = { version = "0.8", features = ["enable-rofi"] }
```

```bash
cargo build --release --examples                # build all examples
cargo run   --release --example hello_world_am  # 1 PE, local backend

# multi-PE on one node (shared memory): N processes, T threads each
./lamellar_run.sh -N=4 -T=8 ./target/release/examples/hello_world_am

# distributed (multi-node) via the cluster launcher
srun -N 2 --mpi=pmi2 ./target/release/examples/hello_world_am
```

Useful env vars (see `src/env_var.rs`):

- `LAMELLAR_THREADS` — worker threads per PE (the multithreading knob).
- `LAMELLAR_BACKEND` — `local` (default) / `shmem` / `rofi`.
- `LAMELLAR_EXECUTOR` — `lamellar` (default work-stealing) / `async_std` / `tokio`.
- `LAMELLAR_DEADLOCK_TIMEOUT` — helps surface stalled/deadlocked apps.
