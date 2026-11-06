#!/usr/bin/env python3
"""Plotting helper for hpc_driver.sh.

Every sample of every run is plotted (no trimming): bars show the mean and a
box-and-whiskers is drawn over each bar. Bars are grouped by runtime (shmem,
Lamellar, lamellar-openshmem, SOS, Open MPI) with dashed separators between groups; inside a
group the base backend comes before its -opt version. Atomic variants are hatched.
One hue per family (green = Lamellar, orange = lamellar-openshmem, blue = SOS,
purple = Open MPI, pink = Lamellar rofi_c); the -opt lamellae are the same hue darker.
Tables use the steady state (mean of iterations >= 3).

Usage:
    plot_results.py boxplot --data results_4.json --name results/nodes_4/run_ucx --fabric ucx --n 4
    plot_results.py scaling --prefix results/nodes_4/run_ucx --op put --fabric ucx
    plot_results.py tables --dir results_dd_mm_yy --out results_dd_mm_yy/tables.md
"""
import argparse
import json
import os
import re
import sys


# The libfabric lamellae backends (libfabric/libfabric-sys/libfabric-sys-opt) and rofi_c
# share one results file and one "libfabric" group in the shmem leg's keys (shmem's
# build/run never varies across them -- see result_fabric in hpc_driver.sh), so a
# key can end in either the exact fabric or its group. Keep in sync with
# results_group()/is_relevant_array_type() in hpc_driver.sh.
def results_group(fab):
    if fab in ("libfabric", "libfabric-sys", "libfabric-sys-opt", "rofi_c"):
        return "libfabric"
    if fab in ("ucx", "ucx-opt"):
        return "ucx"
    if fab in ("shmem", "shmem-opt"):
        return "shmem"
    return fab


def group_members(fab):
    """All exact fabric values sharing a results file with fab (see results_group()).
    Charting any one of them shows all of them together, so the three libfabric
    lamellae variants can be visually compared side by side."""
    group = results_group(fab)
    if group == "libfabric":
        return ["libfabric", "libfabric-sys", "libfabric-sys-opt", "rofi_c"]
    if group == "ucx":
        return ["ucx", "ucx-opt"]
    if group == "shmem":
        return ["shmem", "shmem-opt"]
    return [fab]


# Backend order inside a runtime group: base first, then its -opt version (never by performance).
BACKEND_ORDER = ["shmem", "shmem-opt", "ucx", "ucx-opt", "libfabric", "libfabric-sys", "libfabric-sys-opt", "rofi_c"]
# Space between one variant's (plain, atomic) pair of bars and the next pair, in bar widths.
PAIR_GAP = 0.6
# Runtime groups, left to right. "shmem" is Lamellar's single-node shmem lamellae.
GROUP_ORDER = ["shmem", "Lamellar", "Lamellar-OpenSHMEM", "SOS", "OMPI"]

# Legend names of the runtime groups (keys of GROUP_ORDER).
GROUP_LABEL = {"shmem": "Lamellar", "Lamellar": "Lamellar", "Lamellar-OpenSHMEM": "lamellar-openshmem", "SOS": "SOS", "OMPI": "Open MPI"}

FOOTNOTE = ("bar = mean, white box = Q1–Q3, line = median, whisker = true min to max (all samples)\n"
            "opt = the optimized *-opt lamellae (same hue, darker); shmem = Lamellar's single-node shared-memory lamellae; "
            "lamellar-openshmem = OpenSHMEM implemented on Lamellar; rofi_c = Lamellar's rofi (C) lamellae; hatched = atomic arrays")


def parse_key(key):
    """Split a result key into (group, backend, atomic) or None for unknown keys.

    Classified by prefix (not substring): "lamellar-shmem" is Lamellar's shmem lamellae,
    "shmem-<osh>-..." is the OpenSHMEM leg.
    Keys: lamellar[-atomic]-<backend>, shmem-sos|ompi[-atomic]-<group>, shmem-lamellar-openshmem[-atomic]-<backend>.
    """
    atomic = "-atomic" in key
    core = key.replace("-atomic", "")
    if core.startswith("lamellar-"):
        backend = core[len("lamellar-"):]
        if backend in ("shmem", "shmem-opt"):
            return ("shmem", backend, atomic)
        return ("Lamellar", backend, atomic)
    if core.startswith("shmem-"):
        rest = core[len("shmem-"):]
        for osh, grp in (("sos", "SOS"), ("ompi", "OMPI"), ("lamellar-openshmem", "Lamellar-OpenSHMEM")):
            if rest.startswith(osh + "-"):
                return (grp, rest[len(osh) + 1:], atomic)
    return None


def is_relevant_array_type(at, fab):
    parsed = parse_key(at)
    if parsed is None:
        return False
    group, backend, _ = parsed
    if "-am" in at or "buffered" in at:
        return False
    members = set(group_members(fab)) | {results_group(fab)}
    return backend in members


def filter_results(data, fab):
    return {op: {at: t for at, t in arrays.items() if is_relevant_array_type(at, fab)}
            for op, arrays in data.items()}


def variant_label(key, fabric):
    """Short legend label; the backend only appears where the chart mixes several."""
    group, backend, _ = parse_key(key)
    multi = len(group_members(fabric)) > 1 or backend == "rofi_c"
    if group == "shmem":
        return f"Lamellar {backend}"
    if group in ("Lamellar", "Lamellar-OpenSHMEM"):
        return f"{GROUP_LABEL[group]} {backend}" if multi else GROUP_LABEL[group]
    return GROUP_LABEL.get(group, group)


def ordered_array_types(array_types, fab):
    """Group order, then base before -opt, then non-atomic before atomic."""
    def rank(at):
        group, backend, atomic = parse_key(at)
        b = BACKEND_ORDER.index(backend) if backend in BACKEND_ORDER else len(BACKEND_ORDER)
        return (GROUP_ORDER.index(group), b, atomic)
    return sorted(set(array_types), key=rank)


# Family colours: one hue per implementation family, colour-blind safe. The -opt
# lamellae are the same hue darker; libfabric-sys is the same hue lighter, so the three libfabric lamellae of one chart stay apart
# (libfabric = hue, libfabric-sys = lighter, libfabric-sys-opt = darker). Keep in sync with key_color() in hpc_driver.sh (gnuplot fallback).
FAMILY_COLOR = {
    "Lamellar": "#009e73",              # green
    "shmem": "#009e73",                 # Lamellar's single-node shmem lamellae: native Lamellar
    "Lamellar-OpenSHMEM": "#d55e00",    # orange (vermillion)
    "SOS": "#0072b2",                   # blue
    "OMPI": "#a046a0",                  # purple
}
ROFI_COLOR = {"Lamellar": "#cc79a7", "Lamellar-OpenSHMEM": "#a08c00"}   # rofi_c is its own family: pink; the lamellar-openshmem one olive
OVERFLOW_COLOR = "#898781"
INK = "#0b0b0b"


def array_family(at):
    """Hue identity: runtime group + backend, so atomic shares the hue and differs by hatch."""
    group, backend, _ = parse_key(at)
    return f"{group}:{backend}"


def family_color(family):
    group, backend = family.split(":")
    hue = ROFI_COLOR.get(group) if backend == "rofi_c" else None
    hue = hue or FAMILY_COLOR.get(group, OVERFLOW_COLOR)
    if backend.endswith("-opt"):
        return darken(hue, 0.38)
    if backend == "libfabric-sys":
        return lighten(hue, 0.4)
    return hue


def family_colors(array_types):
    return {array_family(at): family_color(array_family(at)) for at in array_types}


def lighten(hex_color, amount=0.4):
    h = hex_color.lstrip("#")
    r, g, b = (int(h[i : i + 2], 16) for i in (0, 2, 4))
    return "#{:02x}{:02x}{:02x}".format(*(int(c + (255 - c) * amount) for c in (r, g, b)))


def darken(hex_color, amount=0.45):
    h = hex_color.lstrip("#")
    r, g, b = (int(h[i : i + 2], 16) for i in (0, 2, 4))
    return "#{:02x}{:02x}{:02x}".format(*(int(c * (1 - amount)) for c in (r, g, b)))


# The Rust plots specify strokes in pixels; matplotlib wants points. At the 100 dpi these figures are saved at, 1 px = 0.72 pt.
PX = 72.0 / 100.0


def quartiles(v):
    """(Q1, median, Q3) by linear interpolation between order statistics, over every sample
    (rank = p * (n - 1))."""
    import math

    s = sorted(v)

    def q(p):
        if len(s) == 1:
            return s[0]
        r = p * (len(s) - 1)
        lo, hi = int(math.floor(r)), int(math.ceil(r))
        return s[lo] + (s[hi] - s[lo]) * (r - lo)

    return q(0.25), q(0.5), q(0.75)


def draw_bar_glyph(ax, color, atomic, times, c, hw):
    """One bar: height = mean of every sample; white box
    Q1..Q3 (0.8 alpha, 2 px black outline) with a 3 px median line; whisker from the true minimum to the true maximum with
    caps. `c` is the bar centre and `hw` its half width; the box and the caps are 0.45 of that wide on each side."""
    from matplotlib.colors import to_rgba
    from matplotlib.patches import Rectangle

    mean = sum(times) / len(times)
    lo, hi = min(times), max(times)
    q1, med, q3 = quartiles(times)
    bw = hw * 0.45
    # bar: fill at 0.9 alpha (hatched when atomic), then a thin black outline at 0.55 alpha
    ax.add_patch(Rectangle((c - hw, 0.0), 2 * hw, mean, facecolor=to_rgba(color, 0.9),
                           edgecolor=darken(color) if atomic else "none", linewidth=0, hatch="//" if atomic else None))
    ax.add_patch(Rectangle((c - hw, 0.0), 2 * hw, mean, fill=False, edgecolor=(0, 0, 0, 0.55), linewidth=1 * PX))
    # whisker: true min to true max, capped
    wc = {"color": (0, 0, 0, 0.9), "linewidth": 2 * PX, "solid_capstyle": "butt", "zorder": 3}
    ax.plot([c, c], [lo, hi], **wc)
    ax.plot([c - bw, c + bw], [lo, lo], **wc)
    ax.plot([c - bw, c + bw], [hi, hi], **wc)
    # box: Q1 to Q3 in white at 0.8 alpha with a black outline, median line across it
    ax.add_patch(Rectangle((c - bw, q1), 2 * bw, q3 - q1, facecolor=(1, 1, 1, 0.8), edgecolor="none", zorder=4))
    ax.add_patch(Rectangle((c - bw, q1), 2 * bw, q3 - q1, fill=False, edgecolor="black", linewidth=2 * PX, zorder=5))
    ax.plot([c - bw, c + bw], [med, med], color="black", linewidth=3 * PX, solid_capstyle="butt", zorder=6)


def draw_clusters(ax, clusters, array_types, fabric, ylabel_max):
    """clusters: list of (x_center, {array_type: [samples]}). Draws one bar per variant in every
    cluster (mean bar + box-and-whiskers over all samples) and dashed separators between runtime
    groups. Returns legend handles."""
    from matplotlib.patches import Patch

    colors = family_colors(array_types)
    n_arr = max(len(array_types), 1)
    # A variant's plain and atomic bars sit next to each other (a pair); a gap of PAIR_GAP bar widths separates one pair from the next.
    pair_of = [parse_key(at)[:2] for at in array_types]
    n_gaps = sum(1 for i in range(1, n_arr) if pair_of[i] != pair_of[i - 1])
    width = 0.8 / (n_arr + n_gaps * PAIR_GAP)
    for xc, per_at in clusters:
        left = xc - 0.4
        prev_group = None
        gaps_before = 0
        for i, at in enumerate(array_types):
            times = per_at.get(at, [])
            new_pair = i > 0 and pair_of[i] != pair_of[i - 1]
            if new_pair:
                gaps_before += 1
            x0 = left + (i + gaps_before * PAIR_GAP) * width
            group = parse_key(at)[0]
            if prev_group is not None and group != prev_group:
                ax.axvline(x0 - PAIR_GAP * width / 2, color=INK, linestyle="--", linewidth=0.8, alpha=0.6)
            prev_group = group
            color = colors[array_family(at)]
            atomic = parse_key(at)[2]
            if times:
                draw_bar_glyph(ax, color, atomic, times, x0 + width * 0.475, width * 0.475)
    handles, seen = [], set()
    for at in array_types:
        atomic = parse_key(at)[2]
        label = variant_label(at, fabric) + (" (atomic)" if atomic else "")
        if label in seen:
            continue
        seen.add(label)
        color = colors[array_family(at)]
        handles.append(Patch(facecolor=color, edgecolor=darken(color) if atomic else color,
                             linewidth=0, hatch="//" if atomic else None, label=label))
    return handles


def finish_axes(fig, ax, handles, title, xlabel, ymax, ncol_hint=1):
    ax.set_ylim(0, max(ymax * 1.2, 1e-9))
    ax.set_xlabel(xlabel)
    ax.set_ylabel("Time (seconds)")
    ax.set_title(title)
    if handles:
        # Outside the axes, on the right: the group names are long and a legend inside covered bars.
        ax.legend(handles=handles, loc="upper left", bbox_to_anchor=(1.01, 1.0), borderaxespad=0,
                  fontsize=8, ncol=ncol_hint)
    # Footnote below the axis label with enough bottom margin not to overlap it.
    fig.tight_layout(rect=(0, 0.095, 1, 1))  # tight_layout makes room for the legend outside the axes
    fig.text(0.01, 0.012, FOOTNOTE, fontsize=7, ha="left", va="bottom", linespacing=1.5)


def plot_boxplot(data_path, name, fabric, n):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    with open(data_path) as f:
        data = filter_results(json.load(f), fabric)

    display_fab = results_group(fabric) if len(group_members(fabric)) > 1 else fabric
    out_name = f"{name}_{n}_{display_fab}.png" if n is not None else f"{name}_{display_fab}.png"

    array_types = ordered_array_types({at for arrays in data.values() for at in arrays}, fabric)
    ops = ["put", "get", "swap"]
    clusters = [(i + 1, data.get(op, {})) for i, op in enumerate(ops)]
    ymax = max((max(t) for arrays in data.values() for t in arrays.values() if t), default=0.0)

    fig, ax = plt.subplots(figsize=(14, 7))
    handles = draw_clusters(ax, clusters, array_types, fabric, ymax)
    ax.set_xlim(0.4, len(ops) + 0.6)
    ax.set_xticks([1, 2, 3])
    ax.set_xticklabels(ops)
    finish_axes(fig, ax, handles, f"{display_fab} (n={n} PEs per node)" if n is not None else display_fab,
                "Operation Type", ymax)
    fig.savefig(out_name)
    plt.close(fig)
    print(f"wrote {out_name}")


def find_result_files(prefix):
    dirpart = os.path.dirname(prefix) or "."
    base = os.path.basename(prefix)
    pattern = re.compile(rf"^{re.escape(base)}_(\d+)\.json$")
    results = []
    if not os.path.isdir(dirpart):
        return results
    for entry in sorted(os.listdir(dirpart)):
        m = pattern.match(entry)
        if m:
            results.append((int(m.group(1)), os.path.join(dirpart, entry)))
    return sorted(results)


def plot_scaling(prefix, op, fabric):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    files = find_result_files(prefix)
    if not files:
        print(f"No result files found matching {prefix}_<n>.json", file=sys.stderr)
        return

    array_types, file_results = set(), []
    for n, path in files:
        with open(path) as f:
            results = json.load(f)
        for at in results.get(op, {}):
            if is_relevant_array_type(at, fabric):
                array_types.add(at)
        file_results.append((n, results.get(op, {})))
    array_types = ordered_array_types(array_types, fabric)

    # Scaling charts are bars too: one cluster per PE count, same ordering/legend as per-n charts.
    clusters = [(i + 1, {at: res.get(at, []) for at in array_types}) for i, (n, res) in enumerate(file_results)]
    ymax = max((max(t) for _, per in clusters for t in per.values() if t), default=0.0)

    display_fab = results_group(fabric) if len(group_members(fabric)) > 1 else fabric
    fig, ax = plt.subplots(figsize=(16, 7))
    handles = draw_clusters(ax, clusters, array_types, fabric, ymax)
    ax.set_xlim(0.4, len(clusters) + 0.6)
    ax.set_xticks(range(1, len(clusters) + 1))
    ax.set_xticklabels([str(n) for n, _ in file_results])
    finish_axes(fig, ax, handles, f"Time vs PEs per node for {op} - {display_fab}",
                "PEs per node", ymax)
    out_name = f"{prefix}_{op}_all_arrays_pe_scaling_{display_fab}.png"
    fig.savefig(out_name)
    plt.close(fig)
    print(f"wrote {out_name}")


def table_label(key):
    """Display name for a result key: group + backend (+ atomic), the same names as the charts."""
    parsed = parse_key(key)
    if parsed is None:
        return key
    group, backend, atomic = parsed
    name = f"{GROUP_LABEL.get(group, group)} {backend}"
    return name + (" (atomic)" if atomic else "")


def make_tables(root, out_path):
    """tables.md from every DIR/nodes_N/run_*_<n>.json: mean per row over steady-state samples
    (iterations >= 3), with the number of samples used. Rows are never dropped."""
    pat = re.compile(r"^run_(.+)_(\d+)\.json$")
    rows = {}  # (nodes, op) -> {(n, key): samples}
    for nd in sorted(os.listdir(root)) if os.path.isdir(root) else []:
        m = re.match(r"^nodes_(\d+)$", nd)
        if not m:
            continue
        nodes = int(m.group(1))
        for fn in sorted(os.listdir(os.path.join(root, nd))):
            mm = pat.match(fn)
            if not mm:
                continue
            with open(os.path.join(root, nd, fn)) as f:
                d = json.load(f)
            for op, arrays in d.items():
                for key, samples in arrays.items():
                    rows.setdefault((nodes, op), {})[(int(mm.group(2)), key)] = samples
    lines = ["# Results tables", "",
             "Mean time in seconds over iterations >= 3 (steady state); `its` is the samples used "
             "(reps x iterations, all repetitions in one file).", ""]
    for (nodes, op) in sorted(rows):
        lines += [f"## {nodes} node(s), {op}", "", "| variant | PEs/node | mean (s) | its |", "|---|---|---|---|"]
        def row_rank(r):
            n, key = r
            parsed = parse_key(key)
            if parsed is None:
                return (n, len(GROUP_ORDER), len(BACKEND_ORDER), key)
            group, backend, atomic = parsed
            return (n, GROUP_ORDER.index(group),
                    BACKEND_ORDER.index(backend) if backend in BACKEND_ORDER else len(BACKEND_ORDER), atomic, key)
        for (n, key) in sorted(rows[(nodes, op)], key=row_rank):
            samples = rows[(nodes, op)][(n, key)]
            steady = samples[2:] if len(samples) > 3 else samples
            mean = sum(steady) / len(steady) if steady else float("nan")
            lines.append(f"| {table_label(key)} | {n} | {mean:.4f} | {len(steady)} |")
        lines.append("")
    with open(out_path, "w") as f:
        f.write("\n".join(lines))
    print(f"wrote {out_path}")



# ---- node scaling at a fixed PEs per node ------------------------------------------------------------------------------
NET_BACKENDS = ["ucx", "ucx-opt", "libfabric", "libfabric-sys", "libfabric-sys-opt", "rofi_c"]


def drop_warmup(data, skip):
    """{op: {key: samples}} with the first `skip` samples of every row dropped (rows too short to keep any are left whole, as in tables.md)."""
    if not skip:
        return data
    return {op: {k: (v[skip:] if len(v) > skip + 1 else v) for k, v in arrays.items()} for op, arrays in data.items()}


def load_pes_per_node(root, pes, skip=0):
    """{nodes: {op: {key: samples}}} from every DIR/nodes_N/run_<group>_<pes>.json."""
    out = {}
    pat = re.compile(r"^run_(.+)_%d\.json$" % pes)
    for nd in sorted(os.listdir(root)) if os.path.isdir(root) else []:
        m = re.match(r"^nodes_(\d+)$", nd)
        if not m:
            continue
        merged = {}
        for fn in sorted(os.listdir(os.path.join(root, nd))):
            if pat.match(fn):
                with open(os.path.join(root, nd, fn)) as f:
                    merged = merge_op_dicts(merged, drop_warmup(json.load(f), skip))
        if merged:
            out[int(m.group(1))] = merged
    return out


def merge_op_dicts(a, b):
    for op, arrays in b.items():
        a.setdefault(op, {}).update(arrays)
    return a


def _mean(v):
    return sum(v) / len(v)


def node_scaling_series(data, op):
    """The bars of the node-scaling chart for one op: [(label, hue, {nodes: (plain samples, atomic samples)})]. Lamellar and
    lamellar-openshmem use their best backend: the lowest geometric mean, over the node counts, of the mean of the plain and the
    atomic bar's mean time; only backends with both array types at every node count compete."""
    import math

    node_counts = sorted(data)

    def samples(prefix, backend, atomic, n):
        key = f"{prefix}{'-atomic' if atomic else ''}-{backend}"
        return data[n].get(op, {}).get(key) or []

    def collect(prefix, backend):
        return {n: (samples(prefix, backend, False, n), samples(prefix, backend, True, n)) for n in node_counts}

    def complete(points):
        return all(points[n][0] and points[n][1] for n in node_counts)

    series, chosen = [], []
    transports = (("ucx", ["ucx", "ucx-opt"], False),
                  ("libfabric", ["libfabric", "libfabric-sys", "libfabric-sys-opt", "rofi_c"], True))
    for label, prefix, group in (("Lamellar", "lamellar", "Lamellar"),
                                 ("lamellar-openshmem", "shmem-lamellar-openshmem", "Lamellar-OpenSHMEM")):
        for tname, backends, lighter in transports:
            # variants of this transport with plain and atomic data at one node count at least
            cands = {b: collect(prefix, b) for b in backends}
            cands = {b: pts for b, pts in cands.items() if any(pts[n][0] and pts[n][1] for n in node_counts)}
            # compare on the node counts where every one of them has both array types
            common = [n for n in node_counts if cands and all(pts[n][0] and pts[n][1] for pts in cands.values())]
            if not common:
                print(f"warning: no {label} {tname} variants have plain and atomic data at a common node count for {op}", file=sys.stderr)
                continue
            best = None
            for b, pts in cands.items():
                score = math.exp(sum(math.log((_mean(pts[n][0]) + _mean(pts[n][1])) / 2) for n in common) / len(common))
                if best is None or score < best[0]:
                    best = (score, b, pts)
            hue = family_color(f"{group}:{best[1]}")
            series.append((f"{label} {tname}", lighten(hue, 0.4) if lighter else hue, best[2]))
            chosen.append(f"{label} {tname} = {best[1]}")
    for label, prefix, backend, hue in (("SOS ucx", "shmem-sos", "ucx", FAMILY_COLOR["SOS"]),
                                        ("SOS libfabric", "shmem-sos", "libfabric", lighten(FAMILY_COLOR["SOS"], 0.4)),
                                        ("Open MPI ucx", "shmem-ompi", "ucx", FAMILY_COLOR["OMPI"])):
        pts = collect(prefix, backend)
        if any(pts[n][0] or pts[n][1] for n in node_counts):
            series.append((label, hue, pts))
    return series, chosen


def plot_node_scaling(root, pes, out_dir, skip=0):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from matplotlib.patches import Patch

    data = load_pes_per_node(root, pes, skip)
    if not data:
        print(f"No run_*_{pes}.json files under {root}/nodes_N", file=sys.stderr)
        return
    node_counts = sorted(data)
    os.makedirs(out_dir, exist_ok=True)
    for op in ("put", "get", "swap"):
        series, chosen = node_scaling_series(data, op)
        if not series:
            continue
        n_bars = 2 * len(series)
        width = 0.8 / (n_bars + (len(series) - 1) * PAIR_GAP)
        ymax = max((max(t) for _, _, pts in series for pair in pts.values() for t in pair if t), default=0.0)
        fig, ax = plt.subplots(figsize=(14, 7))
        for ci, nodes in enumerate(node_counts):
            left = ci + 1 - 0.4
            for si, (label, hue, pts) in enumerate(series):
                for ai, times in enumerate(pts[nodes]):
                    x0 = left + (2 * si + ai + si * PAIR_GAP) * width
                    if times:
                        draw_bar_glyph(ax, hue, bool(ai), times, x0 + width * 0.475, width * 0.475)
        handles = []
        for label, hue, _ in series:
            handles.append(Patch(facecolor=hue, linewidth=0, label=label))
            handles.append(Patch(facecolor=hue, edgecolor=darken(hue), linewidth=0, hatch="//", label=f"{label} (atomic)"))
        ax.set_xlim(0.4, len(node_counts) + 0.6)
        ax.set_xticks(range(1, len(node_counts) + 1))
        ax.set_xticklabels([str(n) for n in node_counts])
        finish_axes(fig, ax, handles, f"Time vs nodes for {op} at {pes} PEs per node" + (f" (steady state: first {skip} iterations dropped)" if skip else ""),
                    "Nodes", ymax)
        fig.text(0.01, 0.052, "Lamellar and lamellar-openshmem: per transport (ucx: ucx, ucx-opt; libfabric: libfabric, libfabric-sys, libfabric-sys-opt, rofi_c) the variant with the lowest "
                              "geometric-mean time (mean of its plain and atomic bars) over the node counts where all of its variants have data.\n"
                              "Shown: " + "; ".join(chosen),
                 fontsize=7, ha="left", va="bottom")
        out_name = os.path.join(out_dir, f"node_scaling_{pes}pes_{op}{'_steady' if skip else ''}.png")
        fig.savefig(out_name)
        plt.close(fig)
        print(f"wrote {out_name}  ({', '.join(label for label, _, _ in series)})")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    # required=True as a kwarg needs Python 3.7+; setting the attribute after
    # the fact works on 3.6 too, which is what this cluster's system python3
    # actually is (see find_python() in hpc_driver.sh).
    sub = parser.add_subparsers(dest="command")
    sub.required = True

    p_box = sub.add_parser("boxplot")
    p_box.add_argument("--data", required=True)
    p_box.add_argument("--name", default="bar_boxplot_chart")
    p_box.add_argument("--fabric", required=True)
    p_box.add_argument("--n", type=int, default=None)

    p_scale = sub.add_parser("scaling")
    p_scale.add_argument("--prefix", required=True)
    p_scale.add_argument("--op", required=True, choices=["put", "get", "swap"])
    p_scale.add_argument("--fabric", required=True)

    p_ns = sub.add_parser("nodescaling")
    p_ns.add_argument("--dir", required=True)
    p_ns.add_argument("--pes", type=int, default=32, help="PEs per node")
    p_ns.add_argument("--out", default=None, help="output directory (default: --dir)")
    p_ns.add_argument("--skip-first", type=int, default=0, help="drop the first N iterations of every run (steady state: 2)")

    p_tab = sub.add_parser("tables")
    p_tab.add_argument("--dir", required=True)
    p_tab.add_argument("--out", required=True)

    args = parser.parse_args()

    if args.command == "boxplot":
        plot_boxplot(args.data, args.name, args.fabric, args.n)
    elif args.command == "scaling":
        plot_scaling(args.prefix, args.op, args.fabric)
    elif args.command == "tables":
        make_tables(args.dir, args.out)
    elif args.command == "nodescaling":
        plot_node_scaling(args.dir, args.pes, args.out or args.dir, args.skip_first)


if __name__ == "__main__":
    main()
