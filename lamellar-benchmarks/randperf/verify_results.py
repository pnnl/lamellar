#!/usr/bin/env python3
"""Verify a randperf campaign directory.

Reads DIR/nodes_N/run_<group>_<n>.json (the driver's results) and the matching
run_<group>.checks.tsv written by hpc_driver.sh (PE 0's Accum/Check per sample), and checks:

  1. every row has the expected number of samples (--iters, default 25);
  1b. completeness: for each node count and results group (run_<group>), every PE count in --ns has a
     file, and every (PE count, op, variant) combination that exists anywhere in that group exists (and is
     non-empty) everywhere. The missing rows are listed as the rerun list. OMPI atomic `put` is skipped
     by the driver on purpose and is not expected. A variant that was never run at all cannot be detected
     this way, since there is no spec of what the campaign should contain;
  2. for each (nodes, total PEs, op), PE 0's Check and Accum per iteration agree across all
     variants -- the first clean row is the reference, later rows are compared to it.
     Exception: Accum for `swap` with more than 1 PE is not deterministic (randperf-core says
     so: it sums values returned by concurrent swaps), so it is only reported as a note;
     Check is always compared strictly.

Rows that fail are listed, never dropped. Exit status is 1 if anything failed.

Usage: verify_results.py DIR [--iters 25]
"""
import argparse
import collections
import glob
import json
import os
import re
import sys


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("dir")
    ap.add_argument("--iters", type=int, default=25)
    ap.add_argument("--ns", default="1 2 4 8 16 32", help="PEs-per-node values every group should cover")
    args = ap.parse_args()

    problems = []
    n_rows = 0
    present = collections.defaultdict(dict)   # (nodes, group) -> {(n, op, key): sample count}
    for nd in sorted(glob.glob(os.path.join(args.dir, "nodes_*"))):
        nodes = int(re.search(r"nodes_(\d+)$", nd).group(1))
        for path in sorted(glob.glob(os.path.join(nd, "run_*_*.json"))):
            m = re.match(r"run_(.+)_(\d+)\.json$", os.path.basename(path))
            if not m:
                continue
            with open(path) as f:
                data = json.load(f)
            for op, rows in data.items():
                for key, samples in rows.items():
                    present[(nodes, m.group(1))][(int(m.group(2)), op, key)] = len(samples)
                    n_rows += 1
                    if len(samples) != args.iters:
                        problems.append(f"{nodes} nodes, {m.group(2)} PEs/node, {op}, {key}: "
                                        f"{len(samples)} samples, expected {args.iters}")

    # Completeness (the rerun list)
    want_ns = [int(x) for x in args.ns.split()]
    missing = []
    for (nodes, group), rows in sorted(present.items()):
        have_ns = {n for (n, _, _) in rows}
        keys = {k for (_, _, k) in rows}
        for n in want_ns:
            if n not in have_ns:
                missing.append(f"{nodes} nodes, group {group}: no results file for {n} PEs/node")
        for n in sorted(have_ns):
            for op in ("put", "get", "swap"):
                for k in sorted(keys):
                    if op == "put" and k.startswith("shmem-ompi") and "-atomic" in k:
                        continue
                    if rows.get((n, op, k), 0) == 0:
                        missing.append(f"{nodes} nodes, {n} PEs/node, {op}, {k}: "
                                       + ("empty" if (n, op, k) in rows else "missing"))

    # Result equivalence from the checks files: columns total_pes, op, key, iter, accum, check, rc
    ref = {}
    seen = collections.defaultdict(set)
    notes = collections.defaultdict(set)
    n_checked = 0
    for path in sorted(glob.glob(os.path.join(args.dir, "nodes_*", "*.checks.tsv"))):
        nodes = re.search(r"nodes_(\d+)", path).group(1)
        with open(path) as f:
            for line in f:
                parts = line.rstrip("\n").split("\t")
                if len(parts) != 7:
                    continue
                pes, op, key, it, accum, check, rc = parts
                if rc != "0":
                    problems.append(f"{nodes} nodes, {pes} PEs, {op}, {key}: exit code {rc} (samples discarded)")
                    continue
                n_checked += 1
                k = (nodes, pes, op, int(it))
                val = (accum, check)
                if k not in ref:
                    ref[k] = (key, val)
                    continue
                ref_accum, ref_check = ref[k][1]
                accum_free = op == "swap" and int(pes) > 1
                if check != ref_check or (accum != ref_accum and not accum_free):
                    seen[(nodes, pes, op, key)].add(int(it))
                elif accum != ref_accum:
                    notes[(nodes, pes, op, key)].add(int(it))
    for (nodes, pes, op, key), its in sorted(seen.items()):
        k0 = (nodes, pes, op, min(its))
        problems.append(f"{nodes} nodes, {pes} PEs, {op}, {key}: Accum/Check differs from reference "
                        f"{ref[k0][0]} in iterations {sorted(its)}")

    print(f"checked {n_rows} rows, {n_checked} Accum/Check samples")
    rf = os.path.join(args.dir, "retries.tsv")
    if os.path.exists(rf):
        with open(rf) as f:
            rl = [l.rstrip("\n").split("\t") for l in f][1:]
        by = collections.Counter(r[-1] for r in rl if len(r) >= 10)
        hosts = sorted({r[8] for r in rl if len(r) >= 10})
        print(f"{len(rl)} run(s) needed an OpenSHMEM start-up retry: {dict(by)}; first-failure hosts: {', '.join(hosts)} (see retries.tsv)")
    if missing:
        print(f"{len(missing)} missing or empty row(s) (rerun list):")
        for line in missing:
            print("  " + line)
        problems.extend(f"incomplete: {x}" for x in missing)
    if n_checked == 0:
        print("warning: no Accum/Check samples found (*.checks.tsv missing); equivalence NOT verified")
    for (nodes, pes, op, key), its in sorted(notes.items()):
        print(f"note: {nodes} nodes, {pes} PEs, {op}, {key}: Accum differs from reference in "
              f"{len(its)} iteration(s) (expected, swap Accum is nondeterministic); Check matches")
    if problems:
        print(f"{len(problems)} problem(s):")
        for p in problems:
            print("  " + p)
        return 1
    print("all checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
