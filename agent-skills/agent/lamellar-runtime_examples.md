# Lamellar HPC Runtime — Example Catalog

A human-readable guide to the example programs in [`examples/`](examples/).
Lamellar is an asynchronous, distributed-memory runtime for HPC written in Rust,
built around **Active Messages (AMs)**, **distributed arrays**, **RDMA memory regions**,
 **distributed reference-counting (Darc)**, and **teams** of PEs
(processing elements).

Following building packages instruction located in `README.md`, each example can typically be run with:

```bash
# single process (local backend)
cargo run --release --example <name>

# shared memory, N PEs
./lamellar_run.sh -N=4 ./target/release/examples/<name>

# distributed (rofi / UCX backend via MPI launcher)
srun -N 2 --mpi=pmi2 ./target/release/examples/<name>
```

---

## Hello World (`examples/hello_world/`)

The best starting point for learning Lamellar.

- `hello_world/hello_world_am.rs` — Create a Lamellar Active Message and broadcast it to every PE, printing the PE, thread, and originating PE.
- `hello_world/hello_world_array.rs` — Create a distributed Lamellar Array and perform simple `add` operations on its elements.
- `hello_world/hello_world_array_iteration.rs` — Create a Lamellar Array and use its iterators to print and modify data.

---

## Active Messages (`examples/active_message_examples/`)

Demonstrates how to define and launch active messages locally, remotely, and on all PEs.

- `active_message_examples/am_local.rs` — A "local" AM that skips serialization/deserialization of the message struct.
- `active_message_examples/am_no_return.rs` — An AM that returns no data; also shows a ring pattern where each PE messages its right neighbor.
- `active_message_examples/am_return_usize.rs` — An AM with multiple input types that returns a `usize`.
- `active_message_examples/am_return_ordered.rs` — An AM returning a `usize`, illustrating ordered execution/results.
- `active_message_examples/am_return_am.rs` — An AM that returns another AM, which auto-executes on arrival but returns no data.
- `active_message_examples/am_return_am_usize.rs` — An AM that returns another AM which itself returns a `usize` final result.
- `active_message_examples/recursive_am.rs` — Active messages that launch further active messages recursively.
- `active_message_examples/async_comparison.rs` — Shows Lamellar's integration with Rust's `async`/`await` framework.
- `active_message_examples/am_batch_tests.rs` — Exercises batched active-message submission and grouping.
- `active_message_examples/am_local_memregions.rs` — Uses local memory regions inside active messages.

---

## Distributed Arrays (`examples/array_examples/`)

Working with `LamellarArray` types (unsafe, atomic, read-only, locked) and their distributed operations.

- `array_examples/array_am.rs` — Embed and use a memory region within an active message for remote get/put.
- `array_examples/array_ops.rs` — Common element-wise array operations (add, sub, etc.).
- `array_examples/array_batch_add.rs` — Batched add operations across a distributed array.
- `array_examples/array_put_get.rs` — RDMA-style put/get transfers on distributed arrays.
- `array_examples/array_first_last_global_indices.rs` — Query first/last global indices owned by each PE.
- `array_examples/array_consumer_schedules.rs` — Different consumer/scheduling strategies for array iterators.
- `array_examples/atomic_compare_exchange.rs` — Atomic compare-and-exchange on array elements.
- `array_examples/network_atomics.rs` — Benchmark comparing backend-native atomics vs. the active-messaging fallback path.
- `array_examples/dist_array_reduce.rs` — Define a user-defined reduction over a distributed array (experimental API).
- `array_examples/distributed_iteration.rs` — Distributed (global) iteration over array elements.
- `array_examples/local_iteration.rs` — Iteration over only the locally owned portion of an array.
- `array_examples/onesided_iteration.rs` — One-sided iteration that gathers remote elements on demand.
- `array_examples/generic_array.rs` — Using distributed arrays with generic/custom element types.
- `array_examples/global_lock_array.rs` — Coordinated access using a global-lock atomic array.
- `array_examples/histo.rs` — A histogram kernel built on distributed arrays.
- `array_examples/prefix_sum.rs` — A distributed prefix-sum (scan) computation.

---

## Bandwidth & Latency Benchmarks (`examples/bandwidths/`)

Micro-benchmarks measuring transfer bandwidth and latency between PEs.

- `bandwidths/am_bw.rs` — AM bandwidth: send an AM carrying N bytes that returns immediately.
- `bandwidths/am_bw_get.rs` — AM + RDMA bandwidth: AM carries a shared-memory-region handle and "gets" N bytes.
- `bandwidths/am_group_bw_get.rs` — Same AM+RDMA get bandwidth test using AM groups.
- `bandwidths/am_latency.rs` — Per-message AM latency ping (min/p50/p95/p99/max), comparing `exec_am_pe` vs `spawn_am_pe`.
- `bandwidths/put_bw.rs` — RDMA put bandwidth from a local array into a remote PE.
- `bandwidths/put_latency.rs` — Array operation latency (RDMA put_buffer, AM batch_store, AM batch_fetch_add) across array types.
- `bandwidths/get_bw.rs` — RDMA get bandwidth from a remote PE into a local memory region.
- `bandwidths/readonly_array_get_bw.rs` — Get bandwidth on a read-only array.
- `bandwidths/readonly_array_get_unchecked_bw.rs` — Unchecked get bandwidth on a read-only array.
- `bandwidths/atomic_array_get_bw.rs` — Get bandwidth on an atomic array.
- `bandwidths/atomic_array_put_bw.rs` — Put bandwidth on an atomic array.
- `bandwidths/local_lock_atomic_array_get_bw.rs` — Get bandwidth on a local-lock atomic array.
- `bandwidths/local_lock_atomic_array_put_bw.rs` — Put bandwidth on a local-lock atomic array.
- `bandwidths/global_lock_atomic_array_get_bw.rs` — Get bandwidth on a global-lock atomic array.
- `bandwidths/global_lock_atomic_array_put_bw.rs` — Put bandwidth on a global-lock atomic array.
- `bandwidths/unsafe_array_get_bw.rs` — Get bandwidth on an unsafe array.
- `bandwidths/unsafe_array_get_unchecked_bw.rs` — Unchecked get bandwidth on an unsafe array.
- `bandwidths/unsafe_array_put_bw.rs` — Put bandwidth on an unsafe array.
- `bandwidths/unsafe_array_put_unchecked_bw.rs` — Unchecked put bandwidth on an unsafe array.
- `bandwidths/unsafe_array_store_bw.rs` — Store bandwidth on an unsafe array.
- `bandwidths/task_group_am_bw.rs` — AM bandwidth using a `LamellarTaskGroup`.
- `bandwidths/task_group_futures_am_bw.rs` — AM bandwidth using task groups with futures.
- `bandwidths/task_group_typed_futures_am_bw.rs` — AM bandwidth using task groups with typed futures.

---

## Distributed Reference Counting — Darc (`examples/darc_examples/`)

Distributed atomic reference-counted pointers (`Darc`, `GlobalRwDarc`, `LocalRwDarc`).

- `darc_examples/darc.rs` — Use various Darc types (`Darc`, `GlobalRwDarc`, `LocalRwDarc`, wrapped/nested) inside active messages.
- `darc_examples/string_darc.rs` — Share a string across PEs via a Darc.
- `darc_examples/stress_test.rs` — Stress test for Darc allocation, cloning, and cleanup.

---

## Kernels (`examples/kernels/`)

Larger, application-style computational kernels.

- `kernels/serial_array_gemm.rs` — Simplest distributed GEMM: multiply performed serially on PE 0 (lots of data transfer).
- `kernels/parallel_array_gemm.rs` — Distributed GEMM via row-of-A × column-of-B dot products with local C updates.
- `kernels/parallel_blocked_array_gemm.rs` — Distributed blocked/tiled GEMM using submatrices with local-only C updates.
- `kernels/safe_parallel_blocked_array_gemm.rs` — Safe-API variant of the blocked distributed GEMM.
- `kernels/am_gemm.rs` — Naive AM-based tiled GEMM (remote blocks transferred every multiplication).
- `kernels/cached_am_gemm.rs` — AM-based blocked GEMM that caches/reuses a remote block to transfer it only once.
- `kernels/am_flops.rs` — AM "flops" micro-benchmark performing dummy multiply-add work per message.
- `kernels/dft_proxy.rs` — Naive DFT proxy kernel; distributed Lamellar version plus a shared-memory Rayon version.

---

## RDMA Memory Regions (`examples/rdma_examples/`)

Constructing and using remote-direct-memory-access memory regions.

- `rdma_examples/rdma_put.rs` — Put local data into a memory region located on a remote PE.
- `rdma_examples/rdma_get.rs` — Get remote data from a memory region into a local buffer.
- `rdma_examples/rdma_am.rs` — Embed a memory-region handle in an AM so remote PEs can get/put data on the launching PE.

---

## Teams (`examples/team_examples/`)

Grouping PEs into teams and defining custom architectures/layouts.

- `team_examples/team_am.rs` — Create and use `LamellarTeam`s to launch and execute active messages.
- `team_examples/custom_team_arch.rs` — Implement a custom team layout (block + strided) via the `LamellarArch` trait.
- `team_examples/random_team.rs` — Experimental custom arch that scrambles PE ids; uses recursive AMs and sub-teams.

---

## Miscellaneous (`examples/misc/`)

Assorted smaller examples and utilities.

- `misc/lamellar_env.rs` — Showcases the `LamellarEnv` trait for querying runtime environment info.
- `misc/simple_ptp.rs` — A simple Precision-Time-Protocol-style clock-offset demo using AMs that return AMs (illustrative only).
- `misc/ping_pong.rs` — A basic two-PE ping-pong message exchange.
- `misc/dist_hashmap.rs` — A minimal distributed hash map built on Lamellar. (Note: not currently registered as an example in `Cargo.toml`, so it is not runnable via `cargo run --example`.)
