# Results: triangle_count

SUMMARY: task=triangle_count, rev=1, baseline=generated, results=167, status_1pe=ok, status_npe=ok, time_1pe_in_sec=0.19, time_npe_in_sec=0.35, n_pes=2, threads=default(nproc/npes), speedup_vs_serial=N/A, verdit=pass

Generated: 2026-07-30, lamellar 0.8.0-rc.1, Backend shmem

| Task | Version | Rev | 1 PE | N PEs | Time 1 PE | Time N PEs | Speedup vs serial | RESULT | Verdict |
|------|---------|-----|------|-------|-----------|------------|-------------------|--------|---------|
| triangle_count | serial_agent_generated (agent-generated) | — | ok | n/a | 0.00s | n/a | 1.00x | `167` | baseline |
| triangle_count | agent_generated | 1 | ok | ok | 0.19s | 0.35s | N/A | `167` | pass |

## Checklist

skills.md §2 verification checklist, item by item against `agent/examples/triangle_count_agent_generated.rs`:

- [x] **Import matches the use-case** — PASS. Uses `use lamellar::active_messaging::prelude::*;` (AM code), matching the §2 table.
- [x] **Every handle/future realized with exactly one of `.block()`/`.spawn()`/`.await`** — PASS. The single remote op `world.exec_am_all(am).block()` is driven by `.block()` from `main`; no dropped handles.
- [x] **No `.block()` inside `async fn exec`** — PASS. `exec` does pure local compute (build/load graph + count), no nested remote calls, no `.block()`.
- [x] **Inside `exec`, `world`/`team`/`current_pe` refer to `lamellar::` globals** — PASS. Uses `lamellar::current_pe`; does not reference the outer `main` `world`.
- [x] **Result-collection type correct** — PASS. `exec_am_all(...).block()` → `Vec<u64>` (indexed by PE id), summed on PE 0. Matches skills §6.
- [x] **`UnsafeArray` reductions wrapped in `unsafe {}`** — N/A. No arrays used (AM fan-out design).
- [x] **`barrier()` precedes any read of collectively-written results** — PASS. `world.barrier()` before PE 0 reads `partials`/`total` and prints; a second barrier after printing.
- [x] **It builds** — PASS. `cargo build --release --examples` finished clean (no errors).
- [x] **No `.lock()` inside async** — PASS. No locks anywhere; each PE works on its own local graph copy.

Additional internal correctness checks built into the code (all passed at runtime):
- [x] `partials.len() == num_pes` (one result per PE).
- [x] Block partition of `0..n` is complete and non-overlapping (asserted: no gaps/overlaps, coverage == n).
- [x] Per-PE block bounds within range (asserted inside `exec`).
- [x] Serial self-check: ordered count == alternate-traversal count (asserted in the serial baseline).

## Notes

- rev 1: initial parallel implementation. Design = AM fan-out (`exec_am_all`). Each PE
  independently rebuilds the identical deterministic graph (splitmix64 seed) or loads the
  same `--graph` file, then counts only triangles whose smallest vertex `u` lies in its
  contiguous vertex block. Ordering `u < v < w` guarantees each triangle is counted exactly
  once, by the owner of its smallest vertex; summing per-PE partials on PE 0 gives the total.
- Correctness: RESULT=167 at 1 PE, 2 PE, and 16 PE (default local run) — PE-invariant and
  equal to the serial baseline (167). File-input mode validated: K4 complete graph → 4
  triangles at both 1 and 2 PEs.
- Graph-file input added per user request: `--graph <path>`, whitespace `u v` edge list,
  `#` comments, self-loops/duplicate edges dropped. Pass it before Lamellar's own `--`
  runtime flags.
- Timing caveat: at "medium" scale (2000 vertices, 10000 edges, 167 triangles) the actual
  triangle-counting compute is well under 10 ms; measured wall times are dominated by
  process/runtime startup (shmem launcher). The serial direct-binary time was below the
  0.01s timer resolution, so `speedup_vs_serial` is not meaningfully computable and is
  reported as N/A rather than fabricated. Data is too small to amortize runtime startup;
  a larger graph would be needed to show real parallel speedup.
- Measured times reported are launcher wall-clock from `/usr/bin/time -v` (1 PE ≈ 0.19s,
  2 PE ≈ 0.35s; both noisy and startup-bound). Only measured values are reported.

## Summary

SUMMARY: task=triangle_count, rev=1, baseline=generated, results=167, status_1pe=ok, status_npe=ok, time_1pe_in_sec=0.19, time_npe_in_sec=0.35, n_pes=2, threads=default(nproc/npes), speedup_vs_serial=N/A, verdit=pass