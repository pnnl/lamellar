#!/usr/bin/env bash
#
# hpc_driver.sh - benchmark sweep driver for randperf_ws.
#
# Shell/Python port of hpc_driver.rs. Requires no rust-script or one-off
# crates: just bash, awk, cargo, and (for plotting) python3+matplotlib or
# gnuplot as a fallback. See README.md for full usage.
#
# Results JSON files are read and written only by this script's own
# load_results/save_results functions below (a small hand-rolled parser,
# not a general-purpose JSON library) using a fixed, simple layout. Don't
# hand-edit or reformat them with an external JSON pretty-printer.
#
# Intentional deviations from hpc_driver.rs:
#   - base dir resolution is simplified: outside Slurm the script submits itself
#     with sbatch (and exits); inside a job it runs directly. sbatch runs a copy of
#     the script, so the original path is handed over in RANDPERF_SCRIPT_PATH.
#   - `--cancel` runs `scancel -u "$(id -un)"` instead of a hardcoded
#     username, so it works for whoever runs this script.
#
set -euo pipefail

SCRIPT_PATH="${RANDPERF_SCRIPT_PATH:-$(readlink -f "${BASH_SOURCE[0]}")}"   # sbatch runs a spooled copy of this file; the submit step passes the real path
SCRIPT_DIR="$(dirname "$SCRIPT_PATH")"
MANIFEST="$SCRIPT_DIR/Cargo.toml"
PLOT_HELPER="$SCRIPT_DIR/plot_results.py"
ORIGINAL_ARGS=("$@")

usage() {
  cat <<'EOF'
Usage: hpc_driver.sh --fabric <ucx|ucx-opt|libfabric|libfabric-sys|libfabric-sys-opt|rofi_c|shmem|shmem-opt> --osh <sos|ompi|lamellar-openshmem|none> --nodes N [options]

Required:
  --fabric <ucx|ucx-opt|libfabric|libfabric-sys|libfabric-sys-opt|rofi_c|shmem|shmem-opt>
                            Lamellar lamellae backend / transport
                            (shmem and shmem-opt are single node only; rofi_c works with
                            --osh none and --osh lamellar-openshmem, not with sos / ompi)
  --osh <sos|ompi|lamellar-openshmem|none>
                            OpenSHMEM implementation for the randperf-shmem leg
                            (none = Lamellar leg only)
  --nodes N                Number of nodes to allocate / already-allocated

Options:
  --pes-per-node N         Override auto-picked threads-per-pe
  --ns "1 2 4 ..."         PEs-per-node values to sweep (default: 1 2 4 8 16 32)
  --bin-dir DIR            Directory of prebuilt binaries (default: builds/bin):
                            randperf-lamellar, randperf-shmem-sos-ucx,
                            randperf-shmem-sos-ofi, randperf-shmem-ompi-ucx,
                            randperf-shmem-lamellar-openshmem.
                            Release builds (the default of tools/build-binaries.sh) are the ones to
                            measure with; a binary built with another profile gets a warning.
                            The PRRTE launcher is taken from DIR/runtime/prrte/bin/prterun
                            (staged by tools/build-binaries.sh) unless LAMELLAR_PRTERUN or
                            PRRTE_BIN_DIR is set; nothing is looked up in the build directory.
  --iters N                RANDPERF_ITERS for each run (default 25; use fewer to smoke test)
  --ops "put get swap"     Operations to run (default all three); with --arrays this selects single rows
  --arrays "plain atomic"  Array types to run (default both); "atomic" = the atomic-array variants
  --run-timeout MIN        Kill a single run that takes longer than MIN minutes (default 20, 0 = no limit) and record it as failed.
                            Without it one hung run holds the whole job until the Slurm time limit.
  --init-retries N         Retry a run at most N times (default 1) when it died in OpenSHMEM start-up
                            ("SHMEM failed to initialize") before any timing line; every retry is kept in
                            DIR/outputs/.../retried_*.log and DIR/retries.tsv. 0 disables.
  --build                  Build any missing binary first with tools/build-binaries.sh (before
                            the job is submitted); without it a missing binary is an error
  --time MIN               Slurm time limit of the job in minutes (default 240). Only used when
                            the script submits itself; request what the sweep needs
  --save DIR               Save results as DIR/nodes_N/run_GROUP_<n>.json, and
                            plot a per-n boxplot + final scaling charts as you go.
                            GROUP is FABRIC, except the three libfabric backends
                            (libfabric/libfabric-sys/libfabric-sys-opt) share one
                            GROUP ("libfabric") and file, so repeat sweeps with a
                            different one of the three accumulate into the same
                            JSON instead of separate files.
  --plot FILE              Plot a single results JSON file and exit
  --plot-scaling DIR       Plot scaling charts for DIR/nodes_N/run_GROUP and exit
  --plot-dir DIR           Plot per-n boxplots + scaling charts for
                            DIR/nodes_N/run_GROUP and exit
  --cancel                 scancel all your jobs and exit
  --sbatch-opts ...        Remaining args are passed through to sbatch (for example:
                            --partition foo); the script always adds --exclusive
                            --no-requeue -J bench_<fabric> -t <--time>, and --exclude=$RANDPERF_EXCLUDE_NODES when
                            that variable (a Slurm node list to keep jobs off) is set
                            (--salloc-opts is accepted as the old name of this option)

Outside Slurm the script submits itself with sbatch (the job runs this same script with
the same arguments), prints the job id and exits; inside a job (or an salloc) it runs the
sweep directly. The job's output goes to DIR/nodes_N/slurm/ with --save, else to ./slurm/.
  -h, --help               Show this help

Examples:
  ./hpc_driver.sh --fabric ucx --osh sos --nodes 4 --save results/sos
  ./hpc_driver.sh --fabric libfabric --osh sos --nodes 4 --save results/sos
  ./hpc_driver.sh --fabric libfabric-sys --osh sos --nodes 4 --save results/sos
  ./hpc_driver.sh --fabric libfabric-sys-opt --osh sos --nodes 4 --save results/sos
  ./hpc_driver.sh --fabric ucx --osh ompi --nodes 4 --save results/ompi
  ./hpc_driver.sh --fabric rofi_c --osh lamellar-openshmem --nodes 4 --save results/rofi
  ./hpc_driver.sh --fabric libfabric-sys-opt --osh none --nodes 2 --ns "1 2" --iters 3 --time 30 --save results/smoke   # smoke test
  ./hpc_driver.sh --fabric ucx --osh sos --nodes 4 --plot-dir results/sos
  ./hpc_driver.sh --fabric ucx --osh sos --nodes 4 --cancel
EOF
}

FABRIC=""
OSH=""
NS_LIST="1 2 4 8 16 32"
BIN_DIR="$SCRIPT_DIR/builds/bin"
ITERS=25
BUILD=0
INIT_RETRIES=1
RUN_TIMEOUT=20
OPS_LIST="put get swap"
ARRAYS_LIST="plain atomic"
NODES=""
PES_PER_NODE=""
PLOT=""
SAVE=""
PLOT_SCALING=""
PLOT_DIR=""
CANCEL=0
TIME_MIN=240
SBATCH_OPTS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fabric) FABRIC="$2"; shift 2 ;;
    --osh) OSH="$2"; shift 2 ;;
    --nodes) NODES="$2"; shift 2 ;;
    --pes-per-node) PES_PER_NODE="$2"; shift 2 ;;
    --ns) NS_LIST="$2"; shift 2 ;;
    --bin-dir) BIN_DIR="$(readlink -f "$2")"; shift 2 ;;
    --iters) ITERS="$2"; shift 2 ;;
    --build) BUILD=1; shift ;;
    --init-retries) INIT_RETRIES="$2"; shift 2 ;;
    --run-timeout) RUN_TIMEOUT="$2"; shift 2 ;;
    --ops) OPS_LIST="$2"; shift 2 ;;
    --arrays) ARRAYS_LIST="$2"; shift 2 ;;
    --plot) PLOT="$2"; shift 2 ;;
    --save) SAVE="$2"; shift 2 ;;
    --plot-scaling) PLOT_SCALING="$2"; shift 2 ;;
    --plot-dir) PLOT_DIR="$2"; shift 2 ;;
    --cancel) CANCEL=1; shift ;;
    --time) TIME_MIN="$2"; shift 2 ;;
    --sbatch-opts|--salloc-opts) shift; SBATCH_OPTS=("$@"); break ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 1 ;;
  esac
done

if [[ -z "$FABRIC" ]]; then echo "--fabric is required (ucx|ucx-opt|libfabric|libfabric-sys|libfabric-sys-opt|rofi_c|shmem|shmem-opt)" >&2; exit 1; fi
case "$FABRIC" in
  ucx|ucx-opt|libfabric|libfabric-sys|libfabric-sys-opt|rofi_c|shmem|shmem-opt) ;;
  *) echo "--fabric must be one of: ucx, ucx-opt, libfabric, libfabric-sys, libfabric-sys-opt, rofi_c, shmem, shmem-opt" >&2; exit 1 ;;
esac
[[ "$TIME_MIN" =~ ^[0-9]+$ ]] || { echo "--time takes minutes (a number), got '$TIME_MIN'" >&2; exit 1; }
[[ "$OSH" == "lls" ]] && OSH="lamellar-openshmem"   # old name
if [[ -z "$OSH" ]]; then echo "--osh is required (sos|ompi|lamellar-openshmem|none)" >&2; exit 1; fi
case "$OSH" in sos|ompi|lamellar-openshmem|none) ;; *) echo "--osh must be sos, ompi, lamellar-openshmem or none" >&2; exit 1 ;; esac
if [[ -z "$NODES" ]]; then echo "--nodes is required" >&2; exit 1; fi
for _o in $OPS_LIST; do case "$_o" in put|get|swap) ;; *) echo "--ops: unknown op '$_o' (put get swap)" >&2; exit 1 ;; esac; done
for _a in $ARRAYS_LIST; do case "$_a" in plain|atomic) ;; *) echo "--arrays: unknown array type '$_a' (plain atomic)" >&2; exit 1 ;; esac; done
if [[ ( "$FABRIC" == shmem || "$FABRIC" == shmem-opt ) && "$NODES" != "1" ]]; then
  echo "--fabric $FABRIC is single node only (use --nodes 1)" >&2; exit 1
fi

shmem_feature() {
  case "$1:$2" in
    sos:ucx) echo "sos-ucx" ;;
    sos:libfabric|sos:libfabric-sys|sos:libfabric-sys-opt) echo "sos-ofi" ;;
    ompi:ucx) echo "ompi-ucx" ;;
    # lamellar-openshmem has every lamellae compiled in (openshmem-rs/shmem-shim/Cargo.toml); plain shmem is always available
    lamellar-openshmem:ucx|lamellar-openshmem:ucx-opt|lamellar-openshmem:libfabric|lamellar-openshmem:libfabric-sys|lamellar-openshmem:libfabric-sys-opt|lamellar-openshmem:rofi_c|lamellar-openshmem:shmem|lamellar-openshmem:shmem-opt) echo "lamellar-openshmem" ;;
    # sos:rofi_c and ompi:rofi_c do not exist (rofi_c is a Lamellar lamellae): the shmem leg is skipped
    *) echo "" ;;
  esac
}

# All three libfabric lamellae backends (enable-libfabric/-sys/-sys-opt) are
# cfg-additive in the runtime -- they can all be compiled in at once and
# selected at runtime via LAMELLAR_BACKEND (see run_op_and_array_hpc below).
# So instead of building just the one `--fabric` feature requested, build
# all three together whenever any of them is requested: switching between
# libfabric/libfabric-sys/libfabric-sys-opt across invocations then hits
# cargo's build cache instead of forcing a rebuild each time.
lamellar_features() {
  case "$1" in
    libfabric|libfabric-sys|libfabric-sys-opt)
      echo "randperf-lamellar/libfabric,randperf-lamellar/libfabric-sys,randperf-lamellar/libfabric-sys-opt" ;;
    rofi_c) echo "randperf-lamellar/rofi-c" ;;   # the cargo feature is spelled with a dash
    *) echo "randperf-lamellar/$1" ;;
  esac
}

results_prefix() {
  echo "$1/nodes_$2/run_$3"
}

# All the libfabric lamellae backends (libfabric, libfabric-sys, libfabric-sys-opt) and rofi_c share one results file/chart group
# (run_libfabric_<n>.json) instead of one each -- their RESULTS keys already
# carry the exact backend in the suffix (see result_key below), so they can't
# collide within the shared file; this just lets --plot-dir/--plot-scaling
# and repeat sweeps compare all three side by side without extra bookkeeping.
results_group() {
  case "$1" in
    libfabric|libfabric-sys|libfabric-sys-opt|rofi_c) echo "libfabric" ;;
    ucx|ucx-opt) echo "ucx" ;;
    shmem|shmem-opt) echo "shmem" ;;
    *) echo "$1" ;;
  esac
}

# All exact fabric values sharing a results file/group with $1 (see results_group()).
# Charting any one of them shows all of them together, so the three libfabric
# lamellae variants can be visually compared side by side.
group_members() {
  case "$1" in
    libfabric|libfabric-sys|libfabric-sys-opt|rofi_c) echo "libfabric libfabric-sys libfabric-sys-opt rofi_c" ;;
    ucx|ucx-opt) echo "ucx ucx-opt" ;;
    shmem|shmem-opt) echo "shmem shmem-opt" ;;
    *) echo "$1" ;;
  esac
}

# --- results JSON: hand-rolled read/write for the fixed {op: {key: [floats]}} schema ---
# RESULTS is keyed by "<op>\x1f<key>" -> comma-separated floats (no library needed).
declare -A RESULTS=()

load_results() {
  RESULTS=()
  local file="$1" cur_op="" line
  [[ -f "$file" ]] || return 0
  while IFS= read -r line; do
    line="${line#"${line%%[![:space:]]*}"}"
    if [[ "$line" =~ ^\"([a-zA-Z_-]+)\":\ *\{$ ]]; then
      cur_op="${BASH_REMATCH[1]}"
    elif [[ "$line" =~ ^\"([^\"]+)\":\ *\[([^]]*)\],?$ ]]; then
      if [[ -n "$cur_op" ]]; then
        RESULTS["${cur_op}"$'\x1f'"${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}"
      fi
    elif [[ "$line" == "}" || "$line" == "}," ]]; then
      cur_op=""
    fi
  done < "$file"
}

# Several hpc_driver.sh invocations (different fabrics / OpenSHMEM implementations, each its own sbatch job on its own node) write the
# same results files, so every read-modify-write is serialized with a lock. flock cannot be used for that: on this NFS it does not
# exclude between processes on different nodes (measured: two workers on two nodes doing 150 locked increments each ended at 39, not
# 300), and rows were silently lost. mkdir is atomic on NFS, so the lock is a directory, FILE.lockdir. A lock still held after
# LOCK_STALE_S seconds of waiting belongs to a dead holder (a live one keeps it for well under a second) and is broken.
LOCK_STALE_S=60
lock_acquire() {   # $1 = the file to protect
  local d="$1.lockdir" t0=$SECONDS
  while ! mkdir "$d" 2>/dev/null; do
    if (( SECONDS - t0 > LOCK_STALE_S )); then
      echo "warning: breaking stale lock $d" >&2
      rm -rf "$d"
      t0=$SECONDS
      continue
    fi
    sleep "0.$(( RANDOM % 9 + 1 ))"
  done
}
lock_release() { rm -rf "$1.lockdir"; }

# Add or replace one row. save_results rewrites the file atomically (temp file + rename). Afterwards the row is read back: if it is not
# there, something wrote over it, so try again; if it still is not there the caller reports the row as failed instead of "saved".
merge_and_save_result() {
  local file="$1" op="$2" key="$3" csv="$4" attempt
  for attempt in 1 2 3 4 5; do
    lock_acquire "$file"
    load_results "$file"
    RESULTS["${op}"$'\x1f'"${key}"]="$csv"
    save_results "$file"
    lock_release "$file"
    load_results "$file"
    [[ "${RESULTS["${op}"$'\x1f'"${key}"]-}" == "$csv" ]] && return 0
    echo "warning: row ${op}/${key} not found in $file after saving it (attempt ${attempt} of 5)" >&2
  done
  return 1
}

# Fresh existence check under the same lock, so a concurrent process for a different exact variant (sharing this file) skips rows the
# other one just finished, instead of relying on RESULTS as loaded once at the top of the n loop (which goes stale the moment another
# process saves).
result_exists() {
  local file="$1" op="$2" key="$3" found=1
  [[ -n "$file" ]] || return 1
  lock_acquire "$file"
  load_results "$file"
  [[ -n "${RESULTS["${op}"$'\x1f'"${key}"]+x}" ]] && found=0
  lock_release "$file"
  return $found
}

save_results() {
  local file="$1" k op key
  local -A seen_ops=()
  local -a ops_list=()
  for k in "${!RESULTS[@]}"; do
    op="${k%%$'\x1f'*}"
    if [[ -z "${seen_ops[$op]:-}" ]]; then
      seen_ops[$op]=1
      ops_list+=("$op")
    fi
  done
  if [[ ${#ops_list[@]} -gt 0 ]]; then
    mapfile -t ops_list < <(printf '%s\n' "${ops_list[@]}" | sort)
  fi

  {
    echo "{"
    local n_ops=${#ops_list[@]} oi=0
    for op in "${ops_list[@]}"; do
      oi=$((oi+1))
      echo "\"${op}\": {"
      local -a keys_list=()
      for k in "${!RESULTS[@]}"; do
        [[ "$k" == "${op}"$'\x1f'* ]] && keys_list+=("${k#*$'\x1f'}")
      done
      mapfile -t keys_list < <(printf '%s\n' "${keys_list[@]}" | sort)
      local n_keys=${#keys_list[@]} ki=0
      for key in "${keys_list[@]}"; do
        ki=$((ki+1))
        if [[ $ki -lt $n_keys ]]; then
          echo "  \"${key}\": [${RESULTS["${op}"$'\x1f'"${key}"]}],"
        else
          echo "  \"${key}\": [${RESULTS["${op}"$'\x1f'"${key}"]}]"
        fi
      done
      if [[ $oi -lt $n_ops ]]; then echo "},"; else echo "}"; fi
    done
    echo "}"
  } > "${file}.tmp.${HOSTNAME:-host}.$$"
  mv -f "${file}.tmp.${HOSTNAME:-host}.$$" "$file"
}

# remove_best_and_worst_inner, ported: drop 3 smallest + up to 3 largest when len>2.
# Reads/writes comma-separated values; prints one trimmed value per line.
trim_times() {
  local csv="$1"
  [[ -z "$csv" ]] && return 0
  local -a arr
  IFS=',' read -r -a arr <<< "$csv"
  local n=${#arr[@]}
  if (( n <= 2 )); then
    printf '%s\n' "${arr[@]}"
    return 0
  fi
  local -a sorted
  mapfile -t sorted < <(printf '%s\n' "${arr[@]}" | sort -g)
  sorted=("${sorted[@]:3}")
  local len=${#sorted[@]} drop=3
  (( drop > len )) && drop=$len
  (( drop > 0 )) && sorted=("${sorted[@]:0:$((len-drop))}")
  printf '%s\n' "${sorted[@]}"
}

is_relevant_array_type() {
  local at="$1" fab="$2" group m matched
  # result_key always ends in "-<fabric>": anchor to that suffix, not a bare
  # substring match. Lamellar rows match any exact variant sharing this
  # fabric's group (group_members), so charting one of the three libfabric
  # variants shows all three together. Shmem rows are deduped onto the
  # shared group suffix since shmem's build/run never varies across them.
  # Classify by key prefix: the Lamellar `shmem` lamellae gives keys like
  # "lamellar-shmem", which a bare *shmem* match would mistake for the OpenSHMEM leg.
  if [[ "$at" == shmem-* ]]; then
    group="$(results_group "$fab")"
    matched=1
    for m in "$fab" "$group" $(group_members "$fab"); do
      [[ "$at" == *-"$m" ]] && { matched=0; break; }
    done
    [[ $matched -eq 0 ]] || return 1
  elif [[ "$at" == lamellar* ]]; then
    matched=1
    for m in $(group_members "$fab"); do
      [[ "$at" == *-"$m" ]] && { matched=0; break; }
    done
    [[ $matched -eq 0 ]] || return 1
  else
    return 1
  fi
  [[ "$at" == *-am* || "$at" == *buffered* ]] && return 1
  return 0
}

# --- plotting backend selection (python3+matplotlib preferred, gnuplot fallback) ---
PLOTTER=""
PYTHON=""
# Don't trust the binary name: `python3` may be a stub/wrapper and `python` may be
# 2.x or 3.x depending on the distro. Check `--version` (python2 prints it to stderr,
# hence 2>&1) and require >= 3.6 (plot_results.py uses f-strings), then matplotlib.
find_python() {
  local cand ver major minor
  for cand in python3 python; do
    command -v "$cand" >/dev/null 2>&1 || continue
    ver="$("$cand" --version 2>&1)" || continue
    [[ "$ver" =~ ^Python\ ([0-9]+)\.([0-9]+) ]] || continue
    major="${BASH_REMATCH[1]}"; minor="${BASH_REMATCH[2]}"
    (( major == 3 && minor >= 6 || major > 3 )) || continue
    "$cand" -c "import matplotlib" >/dev/null 2>&1 || continue
    PYTHON="$cand"
    return 0
  done
  return 1
}
choose_plotter() {
  if [[ -n "$PLOTTER" ]]; then return; fi
  if find_python; then
    PLOTTER="python"
  elif command -v gnuplot >/dev/null 2>&1; then
    PLOTTER="gnuplot"
    echo "note: python3+matplotlib not found; using gnuplot (reduced-fidelity charts)" >&2
  else
    echo "error: plotting requires python3 with matplotlib, or gnuplot. Install one, e.g.:" >&2
    echo "  pip install --user matplotlib   # preferred, full-fidelity charts" >&2
    echo "  sudo yum install gnuplot        # or: sudo apt-get install gnuplot" >&2
    return 1
  fi
}

# Family colours, the same as in FAMILY_COLOR of plot_results.py (keep them in sync): green = native Lamellar, orange =
# lamellar-openshmem, blue = SOS, purple = Open MPI, pink = Lamellar rofi_c (lamellar-openshmem rofi_c: olive). The -opt lamellae
# are the same hue darker; libfabric-sys is the same hue lighter (so libfabric / -sys / -sys-opt of one chart stay apart).
shade_hex() { # #rrggbb dark|light
  local h="${1#\#}" r g b
  r=$((16#${h:0:2})); g=$((16#${h:2:2})); b=$((16#${h:4:2}))
  if [[ "$2" == dark ]]; then
    r=$((r * 62 / 100)); g=$((g * 62 / 100)); b=$((b * 62 / 100))
  else
    r=$((r + (255 - r) * 40 / 100)); g=$((g + (255 - g) * 40 / 100)); b=$((b + (255 - b) * 40 / 100))
  fi
  printf '#%02x%02x%02x' "$r" "$g" "$b"
}
# key_color RESULT_KEY -> hex colour of the key's family (atomic shares the hue)
key_color() {
  local k="${1/-atomic/}" hue backend
  case "$k" in
    lamellar-*)             backend="${k#lamellar-}"; hue="#009e73"; [[ "$backend" == rofi_c ]] && hue="#cc79a7" ;;
    shmem-lamellar-openshmem-*) backend="${k#shmem-lamellar-openshmem-}"; hue="#d55e00"; [[ "$backend" == rofi_c ]] && hue="#a08c00" ;;
    shmem-sos-*)            backend="${k#shmem-sos-}"; hue="#0072b2" ;;
    shmem-ompi-*)           backend="${k#shmem-ompi-}"; hue="#a046a0" ;;
    *) echo "#898781"; return ;;
  esac
  case "$backend" in
    *-opt) shade_hex "$hue" dark ;;
    libfabric-sys) shade_hex "$hue" light ;;
    *) echo "$hue" ;;
  esac
}

# Sweep-mode plotting must never abort the experiment run: warn and keep going.
plot_nonfatal() {
  [[ "$PLOTTER" == "none" ]] && return 0
  "$@" || echo "warning: plotting failed ($*); continuing with the next experiments" >&2
}

# gnuplot fallback: basic per-(op,array) boxplot using gnuplot's native boxplot style.
# Pure bash/awk data prep -- no python3 dependency, so this fallback is genuinely
# independent of the python3+matplotlib path.
gnuplot_boxplot() {
  local data_file="$1" name="$2" fabric="$3" n="${4:-}"
  local display_fab="$fabric" out_name
  # Multi-variant group (the three libfabric lamellae): name/title after the
  # group, not the one exact variant that triggered this plot -- it now shows
  # all of them (see is_relevant_array_type).
  [[ $(group_members "$fabric" | wc -w) -gt 1 ]] && display_fab="$(results_group "$fabric")"
  if [[ -n "$n" ]]; then out_name="${name}_${n}_${display_fab}.png"; else out_name="${name}_${display_fab}.png"; fi
  mkdir -p "$(dirname "$out_name")"

  load_results "$data_file"

  local tmp_data tmp_script
  tmp_data="$(mktemp)"
  tmp_script="$(mktemp)"

  local -a labels=()
  local k op key val
  for k in "${!RESULTS[@]}"; do
    op="${k%%$'\x1f'*}"
    key="${k#*$'\x1f'}"
    is_relevant_array_type "$key" "$fabric" || continue

    local label="${op}:${key}" idx=-1 li
    for li in "${!labels[@]}"; do
      [[ "${labels[$li]}" == "$label" ]] && { idx=$li; break; }
    done
    if [[ $idx -eq -1 ]]; then
      labels+=("$label")
      idx=$((${#labels[@]} - 1))
    fi

    while IFS= read -r val; do
      printf '%s\t%s\t%s\n' "$((idx + 1))" "$val" "$label" >> "$tmp_data"
    done < <(trim_times "${RESULTS[$k]}")
  done

  if [[ ! -s "$tmp_data" ]]; then
    echo "no data to plot for $out_name" >&2
    rm -f "$tmp_data" "$tmp_script"
    return
  fi

  local ncats
  ncats=$(cut -f1 "$tmp_data" | sort -un | tail -1)

  cat > "$tmp_script" <<EOF
set terminal pngcairo size 1400,800
set output "${out_name}"
set title "${display_fab} (gnuplot fallback)"
set style data boxplot
set style boxplot outliers pointtype 7
set boxwidth 0.5
set xtics rotate by -45
set xlabel "op:array"
set ylabel "Time (seconds)"
set xrange [0:${ncats}+1]
plot "${tmp_data}" using (\$1):(\$2):(0.5):xtic(3) notitle lc rgb "#4472C4"
EOF
  gnuplot "$tmp_script"
  rm -f "$tmp_data" "$tmp_script"
  echo "wrote $out_name"
}

# gnuplot fallback: basic scaling line chart (mean vs n, all arrays overlaid).
# Pure bash/awk data prep -- no python3 dependency.
gnuplot_scaling() {
  local prefix="$1" op="$2" fabric="$3"
  local display_fab="$fabric" dirpart basepart
  [[ $(group_members "$fabric" | wc -w) -gt 1 ]] && display_fab="$(results_group "$fabric")"
  dirpart="$(dirname "$prefix")"
  basepart="$(basename "$prefix")"
  [[ "$dirpart" == "" ]] && dirpart="."

  local -a ns=() files=()
  if [[ -d "$dirpart" ]]; then
    local f bn
    for f in "$dirpart"/*.json; do
      [[ -e "$f" ]] || continue
      bn="$(basename "$f")"
      if [[ "$bn" =~ ^${basepart}_([0-9]+)\.json$ ]]; then
        ns+=("${BASH_REMATCH[1]}")
        files+=("$f")
      fi
    done
  fi
  if [[ ${#files[@]} -eq 0 ]]; then
    echo "no scaling data found for prefix ${prefix}, op ${op}" >&2
    return
  fi

  local -a order
  mapfile -t order < <(for i in "${!ns[@]}"; do echo "${ns[$i]} $i"; done | sort -n -k1,1)

  local -a array_types=()
  declare -A means=()

  local entry n idx file k kop kkey mean
  for entry in "${order[@]}"; do
    n="${entry%% *}"; idx="${entry##* }"
    file="${files[$idx]}"
    load_results "$file"
    for k in "${!RESULTS[@]}"; do
      kop="${k%%$'\x1f'*}"
      kkey="${k#*$'\x1f'}"
      [[ "$kop" == "$op" ]] || continue
      is_relevant_array_type "$kkey" "$fabric" || continue

      local present=0 at
      for at in "${array_types[@]:-}"; do [[ "$at" == "$kkey" ]] && { present=1; break; }; done
      [[ $present -eq 0 ]] && array_types+=("$kkey")

      mean="$(trim_times "${RESULTS[$k]}" | awk '{s+=$1; c++} END{if(c>0) printf "%.9g", s/c; else print 0}')"
      means["${n}"$'\t'"${kkey}"]="$mean"
    done
  done

  if [[ ${#array_types[@]} -eq 0 ]]; then
    echo "no scaling data found for prefix ${prefix}, op ${op}" >&2
    return
  fi

  local tmp_wide tmp_script at
  tmp_wide="$(mktemp)"
  tmp_script="$(mktemp)"
  {
    printf 'n'
    for at in "${array_types[@]}"; do printf '\t%s' "$at"; done
    printf '\n'
    for entry in "${order[@]}"; do
      n="${entry%% *}"
      printf '%s' "$n"
      for at in "${array_types[@]}"; do
        printf '\t%s' "${means["${n}"$'\t'"${at}"]:-0}"
      done
      printf '\n'
    done
  } > "$tmp_wide"

  local ncols=$(( ${#array_types[@]} + 1 ))
  local out_name="${prefix}_${op}_all_arrays_pe_scaling_${display_fab}.png"
  mkdir -p "$(dirname "$out_name")"

  {
    echo 'set terminal pngcairo size 1400,800'
    echo "set output \"${out_name}\""
    echo "set title \"Performance vs n for ${op} (all arrays) - ${display_fab} (gnuplot fallback)\""
    echo 'set xlabel "n (number of PEs)"'
    echo 'set ylabel "Mean Time (seconds)"'
    echo 'set key outside'
    echo -n 'plot '
    for ((i = 2; i <= ncols; i++)); do
      [[ $i -gt 2 ]] && echo -n ', '
      at="${array_types[$((i - 2))]}"
      local color style
      color="$(key_color "$at")"
      # Same scheme as plot_results.py: hue per family (key_color), atomic = dashed + square marker.
      if [[ "$at" == *atomic* ]]; then style="dt 2 pt 5"; else style="dt 1 pt 7"; fi
      echo -n "\"${tmp_wide}\" using 1:${i} with linespoints lc rgb \"${color}\" lw 2 ps 1.5 ${style} title columnhead(${i})"
    done
    echo
  } > "$tmp_script"

  gnuplot "$tmp_script"
  rm -f "$tmp_wide" "$tmp_script"
  echo "wrote $out_name"
}

do_boxplot() {
  local data_file="$1" name="$2" fabric="$3" n="${4:-}"
  choose_plotter || return 1
  if [[ "$PLOTTER" == "python" ]]; then
    if [[ -n "$n" ]]; then
      "$PYTHON" "$PLOT_HELPER" boxplot --data "$data_file" --name "$name" --fabric "$fabric" --n "$n"
    else
      "$PYTHON" "$PLOT_HELPER" boxplot --data "$data_file" --name "$name" --fabric "$fabric"
    fi
  else
    gnuplot_boxplot "$data_file" "$name" "$fabric" "$n"
  fi
}

do_scaling() {
  local prefix="$1" op="$2" fabric="$3"
  choose_plotter || return 1
  if [[ "$PLOTTER" == "python" ]]; then
    "$PYTHON" "$PLOT_HELPER" scaling --prefix "$prefix" --op "$op" --fabric "$fabric"
  else
    gnuplot_scaling "$prefix" "$op" "$fabric"
  fi
}

# --- --plot / --plot-scaling / --plot-dir modes: exit before touching SLURM ---
DID_PLOT=0

if [[ -n "$PLOT" ]]; then
  do_boxplot "$PLOT" "bar_boxplot_chart" "$FABRIC"
  DID_PLOT=1
fi

if [[ -n "$PLOT_SCALING" ]]; then
  prefix="$(results_prefix "$PLOT_SCALING" "$NODES" "$(results_group "$FABRIC")")"
  for op in put get swap; do
    do_scaling "$prefix" "$op" "$FABRIC"
  done
  DID_PLOT=1
fi

if [[ -n "$PLOT_DIR" ]]; then
  results_dir="$(results_prefix "$PLOT_DIR" "$NODES" "$(results_group "$FABRIC")")"
  for n in $NS_LIST; do
    f="${results_dir}_${n}.json"
    if [[ -f "$f" ]]; then
      do_boxplot "$f" "$results_dir" "$FABRIC" "$n"
    fi
  done
  for op in put get swap; do
    do_scaling "$results_dir" "$op" "$FABRIC"
  done
  DID_PLOT=1
fi

if [[ "$DID_PLOT" -eq 1 ]]; then
  exit 0
fi

if [[ "$CANCEL" -eq 1 ]]; then
  scancel -u "$(id -un)"
  exit 0
fi

# Binaries are built once (tools/build-binaries.sh) and reused; the driver only
# builds, through tools/build-binaries.sh, when asked with --build. randperf-lamellar has every lamellae backend compiled in and picks one
# at run time via LAMELLAR_BACKEND.
binary_for() {
  local rt="$1" osh="$2" fabric="$3" feat
  if [[ "$rt" == "lamellar" ]]; then echo "$BIN_DIR/randperf-lamellar"; return; fi
  feat="$(shmem_feature "$osh" "$fabric")"
  case "$feat" in
    sos-ucx)  echo "$BIN_DIR/randperf-shmem-sos-ucx" ;;
    sos-ofi)  echo "$BIN_DIR/randperf-shmem-sos-ofi" ;;
    ompi-ucx) echo "$BIN_DIR/randperf-shmem-ompi-ucx" ;;
    lamellar-openshmem) echo "$BIN_DIR/randperf-shmem-lamellar-openshmem" ;;   # every lamellae backend compiled in, picked with LAMELLAR_BACKEND
    *) echo "" ;;
  esac
}

check_binaries() {
  local rt bin missing=0 prof
  for rt in lamellar shmem; do
    [[ "$rt" == "shmem" && -z "$(shmem_feature "$OSH" "$FABRIC")" ]] && continue
    bin="$(binary_for "$rt" "$OSH" "$FABRIC")"
    if [[ ! -x "$bin" ]]; then echo "missing binary for $rt: $bin" >&2; missing=1; continue; fi
    # tools/build-binaries.sh records the cargo profile it was built with; measurements are release (fat LTO) builds
    prof="$(sed -n 's/^profile: //p' "$bin.provenance" 2>/dev/null | head -1 || true)"
    if [[ "$prof" != release ]]; then
      echo "warning: $(basename "$bin") was built with profile '${prof:-unknown}', not release; its timings are not comparable with release builds (RANDPERF_PROFILE=release tools/build-binaries.sh)" >&2
    fi
  done
  if (( missing )); then echo "build the binaries first: --build, or tools/build-binaries.sh <variant> (see its header), or point --bin-dir at them" >&2; exit 1; fi
}

# --build: build exactly the variants whose binary is missing. Runs before the job is submitted so the
# build does not hold an idle allocation.
build_missing_binaries() {
  local rt bin name
  local -a variants=()
  for rt in lamellar shmem; do
    [[ "$rt" == "shmem" && -z "$(shmem_feature "$OSH" "$FABRIC")" ]] && continue
    bin="$(binary_for "$rt" "$OSH" "$FABRIC")"
    [[ -x "$bin" ]] && continue
    name="$(basename "$bin")"
    if [[ "$name" == randperf-lamellar ]]; then variants+=(lamellar); else variants+=("${name#randperf-shmem-}"); fi
  done
  (( ${#variants[@]} )) || return 0
  echo "Building missing binaries: ${variants[*]}" >&2
  BIN_DIR="$BIN_DIR" "$SCRIPT_DIR/tools/build-binaries.sh" "${variants[@]}" || { echo "build failed" >&2; exit 1; }
}

# Append one line per driver invocation to DIR/sweep_submissions.tsv: nodes, job id,
# purpose, minutes, exit code. Runs as the EXIT trap of the invocation that holds the allocation, so it knows the
# job id and the final status; concurrent invocations share the file under the mkdir lock.
log_submission() {
  local rc=$? f
  trap - EXIT
  if [[ -n "$SAVE" ]]; then
    f="$SAVE/sweep_submissions.tsv"
    mkdir -p "$SAVE"
    lock_acquire "$f"
    [[ -s "$f" ]] || printf 'nodes\tjob_id\tpurpose\tminutes\texit\n' > "$f"
    printf '%s\t%s\t%s\t%s\t%s\n' "$NODES" "${SLURM_JOB_ID:-?}" \
      "fabric=$FABRIC osh=$OSH ns=\"$NS_LIST\" iters=$ITERS$([[ "$OPS_LIST|$ARRAYS_LIST" == "put get swap|plain atomic" ]] || echo " ops=\"$OPS_LIST\" arrays=\"$ARRAYS_LIST\"")" "$(( SECONDS / 60 ))" "$rc" >> "$f"
    lock_release "$f"
  fi
  exit "$rc"
}


# --- job handling: outside Slurm, submit this same script with sbatch and exit; inside a job (or an salloc), run directly ---
submit_self() {
  [[ -n "${SLURM_JOB_ID:-}" ]] && return 0
  command -v sbatch >/dev/null || { echo "sbatch not found: this script needs Slurm (or run it inside an allocation)" >&2; exit 1; }
  local logdir jid
  if [[ -n "$SAVE" ]]; then logdir="$SAVE/nodes_${NODES}/slurm"; else logdir="$PWD/slurm"; fi
  mkdir -p "$logdir"
  # Always exclusive, never requeued (a requeued crashing binary takes down more nodes); nodes in RANDPERF_EXCLUDE_NODES are excluded. The job name
  # carries the fabric only.
  # sbatch exports the caller's environment (--export=ALL, the default) to the job; the original script path is handed over because
  # sbatch runs a spooled copy of the file.
  local -a excl=()
  [[ -n "${RANDPERF_EXCLUDE_NODES:-}" ]] && excl=(--exclude="$RANDPERF_EXCLUDE_NODES")
  jid="$(sbatch --parsable -N "$NODES" --exclusive --no-requeue ${excl[@]+"${excl[@]}"} -J "bench_${FABRIC}" -t "$TIME_MIN" \
           -o "$logdir/run_%j.slurm.out" "${SBATCH_OPTS[@]}" --export=ALL,RANDPERF_SCRIPT_PATH="$SCRIPT_PATH" \
           "$SCRIPT_PATH" "${ORIGINAL_ARGS[@]}")" || { echo "sbatch failed" >&2; exit 1; }
  echo "submitted job ${jid%%;*} (bench_${FABRIC}, ${NODES} node(s), ${TIME_MIN} min); output: $logdir/run_${jid%%;*}.slurm.out" >&2
  echo "${jid%%;*}"
  exit 0
}

# The PRRTE launcher the self-launching binaries start. Compute nodes cannot see the (node-local) cargo build directory, so unless
# LAMELLAR_PRTERUN / PRRTE_BIN_DIR name one, use the copy tools/build-binaries.sh staged next to the binaries (tools/stage_runtime.py).
# Open MPI binaries keep the prterun that matches their own PMIx (found through the path embedded in them).
resolve_prterun() {
  [[ -n "${LAMELLAR_PRTERUN:-}" && -x "${LAMELLAR_PRTERUN}" ]] && { echo "$LAMELLAR_PRTERUN"; return; }
  [[ -n "${PRRTE_BIN_DIR:-}" && -x "${PRRTE_BIN_DIR}/prterun" ]] && { echo "$PRRTE_BIN_DIR/prterun"; return; }
  [[ -x "$BIN_DIR/runtime/prrte/bin/prterun" ]] && { echo "$BIN_DIR/runtime/prrte/bin/prterun"; return; }
  return 1
}
# The Open MPI binary has the path of the prterun of its own build embedded (a node-local build directory that compute nodes
# cannot see). Use that one where it exists, preferring the PRRTE named in the staged Open MPI's build/config.log; otherwise the PRRTE
# staged next to the binaries (tools/stage_runtime.py), which is the same build.
resolve_ompi_prterun() {
  local bin="$BIN_DIR/randperf-shmem-ompi-ucx" cfg="$BIN_DIR/runtime/ompi/build/config.log" hash="" p first=""
  # the PRRTE Open MPI was configured with, staged by tools/stage_runtime.py (runtime/prrte-ompi), is the safest match for its PMIx
  [[ -x "$BIN_DIR/runtime/prrte-ompi/bin/prterun" ]] && { echo "$BIN_DIR/runtime/prrte-ompi/bin/prterun"; return; }
  [[ -f "$cfg" ]] && hash="$(grep -o 'prrte-sys-[0-9a-f]*' "$cfg" | head -1)"
  if [[ -f "$bin" ]]; then
    while IFS= read -r p; do
      [[ -x "$p" ]] || continue
      [[ -n "$hash" && "$p" == */"$hash"/* ]] && { echo "$p"; return; }
      [[ -n "$first" ]] || first="$p"
    done < <(grep -aoE '/[^[:space:][:cntrl:]]*/prrte-sys-[0-9a-f]+/out/bin/prterun' "$bin" | sort -u)
  fi
  [[ -n "$first" ]] && { echo "$first"; return; }
  [[ -x "$BIN_DIR/runtime/prrte/bin/prterun" ]] && { echo "$BIN_DIR/runtime/prrte/bin/prterun"; return; }
  return 1
}
check_prterun() {
  if [[ "$OSH" == ompi ]]; then
    if [[ -x "$BIN_DIR/randperf-shmem-ompi-ucx" ]] && ! resolve_ompi_prterun >/dev/null; then
      echo "no PRRTE launcher for the Open MPI binary (its build directory is not visible and none is staged in $BIN_DIR/runtime/prrte): run tools/stage_runtime.py $BIN_DIR" >&2; exit 1
    fi
    return 0
  fi
  if ! resolve_prterun >/dev/null; then
    echo "no staged PRRTE launcher in $BIN_DIR/runtime: run tools/stage_runtime.py $BIN_DIR (or set LAMELLAR_PRTERUN)" >&2; exit 1
  fi
}

(( BUILD )) && build_missing_binaries
check_binaries   # fail before requesting an allocation, not after
check_prterun
submit_self

if [[ -n "$SAVE" ]] && ! choose_plotter; then
  echo "warning: no plotter available; results will still be saved as JSON" \
       "(plot them later with --plot-dir)" >&2
  PLOTTER="none"
fi

trap log_submission EXIT

# Between runs: stop leftover launchers and clear /dev/shm -- of THIS allocation only. This script runs on the submit
# host, where launchers of other concurrent sweeps (other jobs of the same user) live too, so never match by user or
# process name alone: that killed other sweeps' runs mid-iteration.
cleanup_between_runs() {
  local nodes="$1" pid
  if [[ -n "${SLURM_JOB_ID:-}" ]]; then
    # Leftovers of a failed run include the launcher's `srun ... prted` step, which keeps the allocation's
    # resources busy: a later srun (including the cleanup below) then waits for them forever. So srun counts too.
    for pid in $(pgrep -u "$(id -un)" -x 'prterun|prted|srun' 2>/dev/null || true); do
      [[ "$pid" == "$$" || "$pid" == "$PPID" ]] && continue
      if tr '\0' '\n' < "/proc/$pid/environ" 2>/dev/null | grep -qx "SLURM_JOB_ID=${SLURM_JOB_ID}"; then
        kill "$pid" 2>/dev/null || true
      fi
    done
    sleep 5
    for pid in $(pgrep -u "$(id -un)" -x 'prterun|prted|srun' 2>/dev/null || true); do
      [[ "$pid" == "$$" || "$pid" == "$PPID" ]] && continue
      if tr '\0' '\n' < "/proc/$pid/environ" 2>/dev/null | grep -qx "SLURM_JOB_ID=${SLURM_JOB_ID}"; then
        kill -9 "$pid" 2>/dev/null || true
      fi
    done
    sleep 5
    # Bounded: a hung cleanup must never stall the sweep.
    timeout -k 5 60 srun --nodes="$nodes" --ntasks-per-node=1 \
      bash -c 'find /dev/shm -maxdepth 1 -user "$(id -un)" -type f -delete' >/dev/null 2>&1 || true
  else
    sleep 10
  fi
}

run_op_and_array_hpc() {
  local rt="$1" osh="$2" fabric="$3" op="$4" array="$5" total_pes="$6" nodes="$7" threads_per_pe="$8" key="${9:-}"

  local shmem_feat bin
  shmem_feat="$(shmem_feature "$osh" "$fabric")"
  if [[ "$rt" == "shmem" && -z "$shmem_feat" ]]; then
    echo "Skipping unsupported osh/fabric combo for shmem: ${osh}/${fabric}" >&2
    return 0
  fi
  bin="$(binary_for "$rt" "$osh" "$fabric")"

  local cmd=("$bin" --op "$op" --array-type "$array" --scale-nodes "$nodes" -- --nodes "$nodes" --pes "$total_pes" --threads-per-pe "$threads_per_pe")

  # No UCX_TLS: compare backends with their default transport selection.
  local -a envv=(SMA_OFI_PROVIDER=verbs LAMELLAR_ROFI_PROVIDER=verbs OMPI_MCA_btl='^sm' RANDPERF_ITERS="$ITERS")
  # Variables the launcher is told to forward to every rank explicitly (prterun -x NAME, appended to the binary's launch options below);
  # the rest of the environment reaches the ranks through the job (srun -> prted -> rank), as it always did.
  local -a fwd=()
  local prterun_path
  if [[ "$osh" != ompi || "$rt" == lamellar ]]; then
    if prterun_path="$(resolve_prterun)"; then envv+=(LAMELLAR_PRTERUN="$prterun_path"); fi
  else
    # Open MPI leg: its own prterun if visible from this node, else the staged PRRTE; OPAL_PREFIX relocates the staged Open MPI
    # (plugins in lib/openmpi, help files in share/openmpi). Without a staged tree (a system Open MPI) it is left alone.
    if prterun_path="$(resolve_ompi_prterun)"; then envv+=(LAMELLAR_PRTERUN="$prterun_path"); fi
    if [[ -d "$BIN_DIR/runtime/ompi" ]]; then envv+=(OPAL_PREFIX="$BIN_DIR/runtime/ompi"); fwd+=(OPAL_PREFIX); fi
  fi
  if [[ "$rt" == "lamellar" ]]; then
    envv+=(LAMELLAR_BACKEND="$fabric" LAMELLAR_EXECUTOR=single_thread LAMELLAR_HEAP_SIZE=1073741824 LAMELLAR_HEAP_MODE=static)
    # libfabric and rofi_c rows with UCX also compiled in: keep the libfabric MR cache enabled.
    if [[ "$fabric" == libfabric* || "$fabric" == rofi_c ]]; then envv+=(UCX_MEM_EVENTS=no); fwd+=(UCX_MEM_EVENTS); fi
  elif [[ "$osh" == "lamellar-openshmem" ]]; then
    # lamellar-openshmem: backend picked at run time; symmetric size must be plain bytes.
    envv+=(LAMELLAR_BACKEND="$fabric" SHMEM_SYMMETRIC_SIZE=536870912)
    if [[ "$fabric" == libfabric* || "$fabric" == rofi_c ]]; then envv+=(UCX_MEM_EVENTS=no); fwd+=(UCX_MEM_EVENTS); fi
  fi
  local v
  for v in ${fwd[@]+"${fwd[@]}"}; do cmd+=(-x "$v"); done   # anything after the launch options is handed to prterun verbatim

  echo "running: ${cmd[*]}" >&2

  local outputs_dir="${RESULTS_ROOT:-$SCRIPT_DIR}/outputs/nodes_${nodes}"
  local stdout_file stderr_file rc attempt=0 first_host=""
  local -a times=()

  # Capped retry (--init-retries): Open MPI's OpenSHMEM sometimes fails to agree on the symmetric heap's base address
  # during start-up ("SHMEM failed to initialize"), on random hosts, run sizes and ops, before the benchmark has timed
  # anything. Only that signature, with no timing line at all, is retried, so retries never change measured data.
  while :; do
    stdout_file="$(mktemp)"
    stderr_file="$(mktemp)"

    set +e
    local -a tmo=()
    (( RUN_TIMEOUT > 0 )) && tmo=(timeout -k 30 "$(( RUN_TIMEOUT * 60 ))")
    env -u LD_LIBRARY_PATH "${envv[@]}" ${tmo[@]+"${tmo[@]}"} "${cmd[@]}" >"$stdout_file" 2>"$stderr_file"
    rc=$?
    set -e

    times=()
    while IFS= read -r t; do
      times+=("$t")
    done < <(grep -oP 'FINAL TIME: \K[0-9]+(\.[0-9]+)?(?= seconds)' "$stdout_file" || true)

    if (( ${#times[@]} == 0 && attempt < INIT_RETRIES )) && grep -qs 'SHMEM failed to initialize' "$stdout_file" "$stderr_file"; then
      attempt=$((attempt + 1))
      mkdir -p "$outputs_dir"
      local rlog="${outputs_dir}/retried_${total_pes}_${rt}_${osh}_${fabric}_${op}_${array}_attempt${attempt}.log"
      {
        echo "${cmd[*]}"
        echo "exit code: $rc"
        echo
        echo "=== STDOUT ==="
        cat "$stdout_file"
        echo
        echo "=== STDERR ==="
        cat "$stderr_file"
      } > "$rlog"
      first_host="$(grep -aoh 'SHMEM_ABORT was invoked on rank [0-9]* (pid [0-9]*, host=[A-Za-z0-9_.-]*' "$stdout_file" "$stderr_file" | head -1 | sed 's/.*host=//' || true)"
      echo "SHMEM start-up failed (host ${first_host:-?}); retrying (retry ${attempt} of ${INIT_RETRIES}); first attempt kept in $rlog" >&2
      rm -f "$stdout_file" "$stderr_file"
      cleanup_between_runs "$nodes"
      continue
    fi
    break
  done

  # Keep PE 0's Accum/Check per sample next to the results, for result-equivalence checks.
  if [[ -n "$CHECKS_FILE" && ${#times[@]} -gt 0 ]]; then
    local chk_tmp; chk_tmp="$(mktemp)"
    grep -oP 'FINAL TIME: [0-9.]+ seconds +\(PE 0 Accum:\K[0-9]+  Check:[0-9]+' "$stdout_file" \
      | awk -v t="$total_pes" -v o="$op" -v k="$key" -v rc="$rc" '{sub(/  Check:/," "); print t "\t" o "\t" k "\t" NR-1 "\t" $1 "\t" $2 "\t" rc}' \
      > "$chk_tmp" || true
    lock_acquire "$CHECKS_FILE"; cat "$chk_tmp" >> "$CHECKS_FILE"; lock_release "$CHECKS_FILE"; rm -f "$chk_tmp"
  fi

  if [[ ${#times[@]} -eq 0 || $rc -ne 0 ]]; then
    mkdir -p "$outputs_dir"
    local fname="${outputs_dir}/failed_${total_pes}_${rt}_${osh}_${fabric}_${op}_${array}.log"
    {
      echo "${cmd[*]}"
      echo "exit code: $rc"
      if (( rc == 124 || rc == 137 )); then echo "TIMEOUT: killed after ${RUN_TIMEOUT} minute(s) (--run-timeout)"; fi
      echo
      echo "=== STDOUT ==="
      cat "$stdout_file"
      echo
      echo "=== STDERR ==="
      cat "$stderr_file"
    } > "$fname"
    echo "Saved failed experiment output to $fname" >&2
  fi

  # One line per run that needed a retry: what failed, where, and how it ended.
  if (( attempt > 0 )); then
    local outcome="ok" rf="${RESULTS_ROOT:-$SCRIPT_DIR}/retries.tsv"
    if (( ${#times[@]} == 0 || rc != 0 )); then outcome="failed_again"; fi
    mkdir -p "$(dirname "$rf")"
    lock_acquire "$rf"
    [[ -s "$rf" ]] || printf 'nodes\tpes\truntime\tosh\tfabric\top\tarray\tretries\tfirst_failure_host\toutcome\n' > "$rf"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$nodes" "$total_pes" "$rt" "$osh" "$fabric" "$op" "$array" \
      "$attempt" "${first_host:-?}" "$outcome" >> "$rf"
    lock_release "$rf"
  fi

  rm -f "$stdout_file" "$stderr_file"
  cleanup_between_runs "$nodes"

  # A run that exited non-zero is not data: drop its samples.
  if (( rc != 0 )); then return 0; fi
  # Never print an empty list: printf of an empty array still emits one blank line, which the caller
  # would store as a row with an empty sample list that resume then treats as done.
  if (( ${#times[@]} )); then printf '%s\n' "${times[@]}"; fi
}

SAVE_NAME=""
CHECKS_FILE=""
RESULTS_ROOT=""
if [[ -n "$SAVE" ]]; then
  SAVE_NAME="$(results_prefix "$SAVE" "$NODES" "$(results_group "$FABRIC")")"
  RESULTS_ROOT="$SAVE"
  mkdir -p "$SAVE/nodes_${NODES}"
  CHECKS_FILE="${SAVE_NAME}.checks.tsv"
fi

ops=($OPS_LIST)
runtimes=(shmem lamellar)
atomics=()
for _a in $ARRAYS_LIST; do
  if [[ "$_a" == atomic ]]; then atomics+=("-atomic"); else atomics+=(""); fi
done

n_saved=0; n_skipped=0; n_failed=0; failed_rows=()
for n in $NS_LIST; do
  results_file=""
  if [[ -n "$SAVE_NAME" ]]; then
    results_file="${SAVE_NAME}_${n}.json"
    mkdir -p "$(dirname "$results_file")"
  fi
  load_results "${results_file:-/dev/null}"

  total_pes=$(( n * NODES ))
  if [[ -n "$PES_PER_NODE" ]]; then
    threads_per_pe="$PES_PER_NODE"
  elif (( n >= 32 )); then
    threads_per_pe=2
  else
    threads_per_pe=4
  fi

  for o in "${ops[@]}"; do
    echo "Running operation: $o for n=$n" >&2
    for rt in "${runtimes[@]}"; do
      # shmem's build/run never varies across the three libfabric lamellae
      # variants (they all map to the same sos-ofi feature via
      # shmem_feature) -- key its rows by the shared group so a sweep over
      # libfabric-sys/-sys-opt reuses the shmem data already collected under
      # plain libfabric instead of re-running and re-storing identical rows
      # under a third, redundant key.
      if [[ "$rt" == "shmem" ]]; then
        feat="$(shmem_feature "$OSH" "$FABRIC")"
        if [[ -z "$feat" ]]; then
          echo "Skipping shmem leg: no feature wired for osh=${OSH} fabric=${FABRIC}" >&2
          continue
        fi
        osh_suffix="-${OSH}"
        # sos/ompi rows do not vary across ucx/ucx-opt or the libfabric lamellae; lamellar-openshmem does.
        if [[ "$OSH" == "lamellar-openshmem" ]]; then result_fabric="$FABRIC"; else result_fabric="$(results_group "$FABRIC")"; fi
      else
        osh_suffix=""
        result_fabric="$FABRIC"
      fi

      for at in "${atomics[@]}"; do
        # Open MPI OSHMEM has no atomic put row: its shmem_atomic_set issues ucp_atomic_op_nbx(UCP_ATOMIC_OP_SWAP) with no reply buffer,
        # which UCX 1.20 sends as the non-fetching amo64/post protocol; the transport answers "Unsupported operation" and UCX aborts
        # the process ("'abort' is not implemented for protocol amo64/post"). It runs with UCX_PROTO_ENABLE=n, but that would change
        # how UCX handles every Open MPI operation. SOS implements atomic_set as a fetching swap and is not affected.
        if [[ "$OSH" == "ompi" && "$rt" == "shmem" && "$at" == "-atomic" && "$o" == "put" ]]; then
          continue
        fi

        array_type="${rt}${at}"
        result_key="${rt}${osh_suffix}${at}-${result_fabric}"

        if [[ -n "$SAVE_NAME" ]] && result_exists "$results_file" "$o" "$result_key"; then
          echo "Skipping existing result for op '${o}' key '${result_key}'" >&2
          n_skipped=$((n_skipped + 1))
          continue
        fi

        mapfile -t times < <(run_op_and_array_hpc "$rt" "$OSH" "$FABRIC" "$o" "$array_type" "$total_pes" "$NODES" "$threads_per_pe" "$result_key")

        if [[ ${#times[@]} -gt 0 ]]; then
          if [[ -n "$SAVE_NAME" ]]; then
            csv="$(IFS=,; echo "${times[*]}")"
            if merge_and_save_result "$results_file" "$o" "$result_key" "$csv"; then
              echo "saved to $results_file ($o $result_key)" >&2
              n_saved=$((n_saved + 1))
            else
              echo "ERROR: could not save op '${o}' key '${result_key}' to $results_file" >&2
              n_failed=$((n_failed + 1)); failed_rows+=("n=${n} ${o} ${result_key} (not saved)")
            fi
          fi
        else
          echo "No times recorded for op '${o}' key '${result_key}'" >&2
          n_failed=$((n_failed + 1)); failed_rows+=("n=${n} ${o} ${result_key}")
        fi
      done
    done
  done

  if [[ -n "$SAVE_NAME" && -f "$results_file" ]]; then
    plot_nonfatal do_boxplot "$results_file" "$SAVE_NAME" "$FABRIC" "$n"
  fi
done

if [[ -n "$SAVE_NAME" ]]; then
  for op in put get swap; do
    plot_nonfatal do_scaling "$SAVE_NAME" "$op" "$FABRIC"
  done
fi

# Say how it went, and let the job state and sweep_submissions.tsv agree: exit status 3 when any row failed.
echo "=== summary (${FABRIC} / ${OSH}, ${NODES} node(s)): ${n_saved} saved, ${n_skipped} already present, ${n_failed} failed" >&2
if (( n_failed > 0 )); then
  printf '  failed: %s\n' "${failed_rows[@]}" >&2
  exit 3
fi
