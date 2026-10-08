#!/usr/bin/env python3
"""Summarize shmem_bench results (target/bench/shmem-<backend>-<jobid>/results.csv) into markdown tables.

    tests/bench/summarize_bench.py RESULTS_DIR [--ceilings target/bench/ceilings-<jobid>]

Each cell is the median over repetitions of the per-run p50 (latency) or bandwidth, with the min..max spread
across repetitions in brackets when there is more than one. With --ceilings, the inter-node latency and
bandwidth tables also show the raw-transport ceilings (IB write / UCX put and get) and the ratio.
"""
import csv, os, statistics, sys
from collections import defaultdict

SIZES = [1, 8, 64, 512, 4096, 65536, 1048576, 4194304]


def label(b):
    return f"{b >> 20} MiB" if b >= 1 << 20 else (f"{b >> 10} KiB" if b >= 1024 else f"{b} B")


def load(path):
    rows = defaultdict(list)  # key -> list of row dicts (one per repetition)
    for r in csv.DictReader(open(path)):
        try:
            key = (r["mode"], r["bench"], int(r["size_bytes"]), int(r["np"]), r["mem"])
        except (KeyError, ValueError):
            continue
        rows[key].append(r)
    return rows


def cell(rows, key, field, nd=2):
    rs = rows.get(key)
    if not rs:
        return "-"
    v = [float(r[field]) for r in rs]
    m = statistics.median(v)
    s = f"{m:.{nd}f}"
    if len(v) > 1 and (max(v) - min(v)) > 0:
        s += f" [{min(v):.{nd}f}..{max(v):.{nd}f}]"
    return s


def med(rows, key, field):
    rs = rows.get(key)
    return statistics.median([float(r[field]) for r in rs]) if rs else None


def ceilings(path):
    out = {}
    f = os.path.join(path, "ceilings.csv")
    if os.path.exists(f):
        for r in csv.DictReader(open(f)):
            def g(k):
                try:
                    return float(r[k])
                except (KeyError, ValueError):
                    return None
            out[int(r["size_bytes"])] = {k: g(k) for k in r if k != "size_bytes"}
    return out


def ratio(a, b):
    return "-" if not a or not b else f"{a / b:.1f}x"


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    d = args[0]
    ceil = ceilings(args[args.index("--ceilings") + 1]) if "--ceilings" in args else {}
    rows = load(os.path.join(d, "results.csv"))
    env = open(os.path.join(d, "env.txt")).read().splitlines()[0] if os.path.exists(os.path.join(d, "env.txt")) else ""
    out = [f"# {os.path.basename(d.rstrip('/'))}", "", f"`{env}`", ""]
    modes = sorted({k[0] for k in rows})

    def lat_table(mode, mem, title):
        out.append(f"### {title}\n")
        hdr = "| size | put p50 (us) | get p50 (us) |"
        sep = "|---|---|---|"
        if ceil and mode == "inter":
            hdr += " ceiling: IB write / UCX put / UCX get (us) | put vs best ceiling |"
            sep += "---|---|"
        out.extend([hdr, sep])
        for s in SIZES:
            np_ = 2
            p = cell(rows, (mode, "put_lat", s, np_, mem), "p50_us")
            g = cell(rows, (mode, "get_lat", s, np_, mem), "p50_us")
            line = f"| {label(s)} | {p} | {g} |"
            if ceil and mode == "inter":
                c = ceil.get(s, {})
                best = min([x for x in (c.get("ib_write_typical_us"), c.get("ucx_put_p50_us")) if x] or [0]) or None
                pm = med(rows, (mode, "put_lat", s, np_, mem), "p50_us")
                line += f" {c.get('ib_write_typical_us') or '-'} / {c.get('ucx_put_p50_us') or '-'} / {c.get('ucx_get_p50_us') or '-'} | {ratio(pm, best)} |"
            out.append(line)
        out.append("")

    if "inter" in modes:
        lat_table("inter", "heap", "Inter-node (1 PE per node) one-sided latency, heap target")
    if "intra" in modes:
        lat_table("intra", "heap", "Intra-node (2 PEs on one node) one-sided latency, heap target")

    for mode in [m for m in ("inter", "intra") if m in modes]:
        out.append(f"### {mode}-node put bandwidth / message rate (window of non-blocking puts)\n")
        hdr = "| size | MB/s | Mmsg/s |"
        sep = "|---|---|---|"
        if ceil and mode == "inter":
            hdr += " ceiling MB/s: IB write / UCX put | achieved vs best |"
            sep += "---|---|"
        out += [hdr, sep]
        for s in SIZES:
            bw = cell(rows, (mode, "put_bw", s, 2, "heap"), "bw_MBps", 0)
            rt = cell(rows, (mode, "put_bw", s, 2, "heap"), "rate_Mops", 4)
            line = f"| {label(s)} | {bw} | {rt} |"
            if ceil and mode == "inter":
                c = ceil.get(s, {})
                best = max([x for x in (c.get("ib_write_bw_MBps"), c.get("ucx_put_bw_MBps")) if x] or [0]) or None
                a = med(rows, (mode, "put_bw", s, 2, "heap"), "bw_MBps")
                line += f" {c.get('ib_write_bw_MBps') or '-'} / {c.get('ucx_put_bw_MBps') or '-'} | {ratio(a, best) if a and best else '-'} |"
            out.append(line)
        out.append("")

    if "inter" in modes:
        out.append("### Heap vs global (data segment) target, inter-node p50 (us)\n")
        out += ["| size | put heap | put global | get heap | get global |", "|---|---|---|---|---|"]
        for s in (8, 4096, 65536, 1048576):
            out.append("| %s | %s | %s | %s | %s |" % (label(s), cell(rows, ("inter", "put_lat", s, 2, "heap"), "p50_us"),
                       cell(rows, ("inter", "put_lat", s, 2, "global"), "p50_us"),
                       cell(rows, ("inter", "get_lat", s, 2, "heap"), "p50_us"),
                       cell(rows, ("inter", "get_lat", s, 2, "global"), "p50_us")))
        out.append("")

    out.append("### AMO latency, 8 B, p50 (us)\n")
    out += ["| op | inter heap | inter global | intra heap |", "|---|---|---|---|"]
    for a in ("amo_fadd", "amo_cswap", "amo_swap", "amo_fetch", "amo_add"):
        out.append("| %s | %s | %s | %s |" % (a[4:], cell(rows, ("inter", a, 8, 2, "heap"), "p50_us"),
                   cell(rows, ("inter", a, 8, 2, "global"), "p50_us"), cell(rows, ("intra", a, 8, 2, "heap"), "p50_us")))
    out.append("")

    scale = sorted({k[0] for k in rows if k[0].startswith("scale")})
    allp = [("inter", 2)] if "inter" in modes else []
    allp += [(m, None) for m in scale]
    if scale or "inter" in modes:
        out.append("### Scaling with PE count (p50 us unless noted)\n")
        pes = sorted({k[3] for k in rows if k[1] == "barrier_all"})
        out += ["| benchmark | " + " | ".join(f"np={p}" for p in pes) + " |", "|---|" + "---|" * len(pes)]

        def row(bench, size, field="p50_us", mem="heap", nd=2):
            cells = []
            for p in pes:
                v = None
                for m in [m for m in modes if m != "intra" or p == 2]:
                    key = (m, bench, size, p, mem)
                    if key in rows:
                        v = cell(rows, key, field, nd)
                        break
                cells.append(v or "-")
            return cells

        out.append("| barrier_all | " + " | ".join(row("barrier_all", 0)) + " |")
        out.append("| contended fetch-add (Mops/s, all PEs) | " + " | ".join(row("amo_contended_fadd", 8, "rate_Mops", nd=4)) + " |")
        for b in ("broadcast64", "fcollect64", "sum_to_all"):
            for s in (8, 1024, 65536):
                out.append(f"| {b} {label(s)} | " + " | ".join(row(b, s)) + " |")
        out.append("")
    ops = sorted({(k[1], k[2]) for k in rows if k[1].startswith("op_") and k[0] in ("inter", "intra")})
    if ops:
        out.append("### Other operations, uncontended, p50 (us); window row is Mops/s\n")
        out += ["| op | size | inter heap | intra heap |", "|---|---|---|---|"]
        for b, s in ops:
            fld, nd = ("rate_Mops", 3) if b.endswith("_window") else ("p50_us", 2)
            out.append(f"| {b[3:]} | {label(s)} | {cell(rows, ('inter', b, s, 2, 'heap'), fld, nd)} | {cell(rows, ('intra', b, s, 2, 'heap'), fld, nd)} |")
        out.append("")

    multi = sorted({(k[1], k[2]) for k in rows if k[1].startswith("multi_")})
    if multi:
        mpes = sorted({k[3] for k in rows if k[1].startswith("multi_")})
        out.append("### Many PEs active at once: slowest-PE p50 (us), aggregate Mops/s in brackets\n")
        out += ["| benchmark | size | " + " | ".join(f"np={p}" for p in mpes) + " |", "|---|---|" + "---|" * len(mpes)]
        for b, s in multi:
            cells = []
            for p in mpes:
                v = None
                for m in sorted({k[0] for k in rows}):
                    key = (m, b, s, p, "heap")
                    if key in rows and (m.startswith("scale") or m == "inter"):
                        v = f"{med(rows, key, 'p50_us'):.2f} ({med(rows, key, 'rate_Mops'):.3f})"
                        break
                cells.append(v or "-")
            out.append(f"| {b[6:]} | {label(s)} | " + " | ".join(cells) + " |")
        out.append("")
    print("\n".join(out))
    with open(os.path.join(d, "summary.md"), "w") as f:
        f.write("\n".join(out) + "\n")


if __name__ == "__main__":
    main()
