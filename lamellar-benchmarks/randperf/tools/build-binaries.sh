#!/bin/bash
# Build the benchmark binaries once, on the login node, and install them where hpc_driver.sh looks for them (builds/bin).
#
#   tools/build-binaries.sh [--ucx] [--libfabric] [variant ...]     default: every variant below, on both networks
#
# Variants:
#   lamellar            randperf-lamellar with every lamellae backend compiled in (picked at run time with LAMELLAR_BACKEND: ucx ucx-opt
#                       libfabric libfabric-sys libfabric-sys-opt rofi_c shmem-opt; shmem is always there)
#   sos-ucx sos-ofi     randperf-shmem on Sandia OpenSHMEM over UCX / libfabric
#   ompi-ucx            randperf-shmem on Open MPI OSHMEM over UCX (a vendored Open MPI with shared UCX / Open MPI libraries; the last step,
#                       tools/stage_runtime.py, copies them to builds/bin/runtime/ompi and rewrites the RPATHs, see below)
#   lamellar-openshmem  randperf-shmem on lamellar-openshmem (OpenSHMEM on Lamellar, from the git dependency); installed as
#                       randperf-shmem-lamellar-openshmem, backends ucx ucx-opt libfabric libfabric-sys libfabric-sys-opt rofi_c shmem shmem-opt
#                       are compiled in (--ucx / --libfabric choose the families; rofi_c is part of the libfabric one), LAMELLAR_BACKEND picks one
#
# Everything is built into the workspace's one cargo target directory from github.com/pnnl/lamellar (git dependencies, see Cargo.toml).
# The profile is chosen with RANDPERF_PROFILE (default release: fat LTO, use it for every measurement; release-dev has thin LTO and builds
# faster, use it for smoke builds). Both write to the same builds/bin names, so the profile is recorded in each .provenance file (hpc_driver.sh
# warns about a binary that is not a release one). Each variant is built and copied out in turn, so the binary of one variant is never
# overwritten by the next. Every binary except Open MPI's is fully static: no libfabric / UCX / PMIx shared library at run time (checked here,
# with ldd).
#
# Intermediates (the C builds of SOS, UCX, libfabric, PMIx, Open MPI; several minutes the first time) go to the cargo build directory (target/ by default).
# Node-local disk is faster and not on the NFS quota: set RANDPERF_BUILD_DIR=/some/dir (or CARGO_BUILD_BUILD_DIR; .cargo/config.toml has an example).
# Compute nodes cannot see node-local disk, so what the runs need from it is copied onto
# shared disk by tools/stage_runtime.py, which this script runs at the end (repeatable): the PRRTE launcher to builds/bin/runtime/prrte and
# the vendored Open MPI with its UCX to builds/bin/runtime/ompi (hpc_driver.sh sets OPAL_PREFIX to it). The final binaries are copied to builds/bin.
# Where cargo puts its outputs is read from cargo's own messages; nothing here assumes a path.
#
# Every binary gets a .sha256 and a .provenance (date, profile, repository revision from git rev-parse, features).
# Environment: BIN_DIR (default builds/bin), RANDPERF_PROFILE (default release), RANDPERF_BUILD_DIR. Logs: builds/build-<variant>.log.
set -euo pipefail
ws="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ws"
# --locked only when a Cargo.lock exists (none is shipped: the first build resolves one and later builds keep it)
LOCKED=""; [ -f "$ws/Cargo.lock" ] && LOCKED="--locked"
unset LD_LIBRARY_PATH
BIN_DIR="${BIN_DIR:-$ws/builds/bin}"
LOG_DIR="$ws/builds"
mkdir -p "$BIN_DIR" "$LOG_DIR"
PROFILE="${RANDPERF_PROFILE:-release}"
case "$PROFILE" in release|release-dev|release-debug) ;; *) echo "RANDPERF_PROFILE must be release, release-dev or release-debug (Cargo.toml)" >&2; exit 2 ;; esac
if [ -n "${RANDPERF_BUILD_DIR:-}" ]; then export CARGO_BUILD_BUILD_DIR="$RANDPERF_BUILD_DIR/{workspace-path-hash}"; fi

# --ucx / --libfabric choose the network of the OpenSHMEM variants and add up (both if neither is given). A family has every lamellae of its
# network and always both single-node shmem lamellae: --ucx = ucx + ucx-opt, --libfabric = libfabric + libfabric-sys + libfabric-sys-opt.
net_ucx=0; net_ofi=0; names=()
for a in "$@"; do case "$a" in --ucx) net_ucx=1 ;; --libfabric) net_ofi=1 ;; *) names+=("$a") ;; esac; done
[ $net_ucx = 0 ] && [ $net_ofi = 0 ] && { net_ucx=1; net_ofi=1; }
los_feats=""
[ $net_ucx = 1 ] && los_feats="lamellar-openshmem-ucx"
[ $net_ofi = 1 ] && los_feats="${los_feats:+$los_feats,}lamellar-openshmem-libfabric"
variants=("${names[@]}")
if [ ${#variants[@]} -eq 0 ]; then
  variants=(lamellar)
  [ $net_ucx = 1 ] && variants+=(sos-ucx)
  [ $net_ofi = 1 ] && variants+=(sos-ofi)
  [ $net_ucx = 1 ] && variants+=(ompi-ucx)
  variants+=(lamellar-openshmem)
fi

# no non-system shared library may be needed (the Open MPI variant is exempt)
check_static() { # binary name
  local bad
  bad="$(ldd "$1" 2>&1 | awk '/=>/ && $3 !~ /^\/(usr\/)?lib(64)?\// {print $1}' | tr '\n' ' ')"
  if [ -n "$bad" ] || ldd "$1" 2>&1 | grep -q 'not found'; then
    echo "error: $2 depends on non-system shared libraries: $bad" >&2; exit 1
  fi
}

# Lamellar and its helper crates are path dependencies on this repository, so its HEAD revision identifies them. "-dirty" marks uncommitted
# changes to tracked files (untracked files are not detected); "unknown" means the tree is not inside a git checkout.
LAMELLAR_REV="$(git -C "$ws" rev-parse HEAD 2>/dev/null || echo unknown)"
if [ "$LAMELLAR_REV" != unknown ] && ! git -C "$ws" diff --quiet HEAD 2>/dev/null; then LAMELLAR_REV="$LAMELLAR_REV-dirty"; fi

# Identity of the sources a binary is built from: the hash of the workspace's own Rust and TOML sources,
# build scripts, the lock file and the staging script, recorded in every .provenance so a binary can be matched to the tree it came from.
SOURCE_LIST="$(cd "$ws" && { find randperf-core randperf-lamellar randperf-shmem openshmem-rs/shmem-shim -type f \( -name '*.rs' -o -name '*.toml' \) -not -path '*/target/*'; echo Cargo.toml; [ -f Cargo.lock ] && echo Cargo.lock; echo .cargo/config.toml; echo tools/stage_runtime.py; } | sort)"
SOURCE_FILES="$(echo "$SOURCE_LIST" | wc -l)"
SOURCE_HASH="$(cd "$ws" && echo "$SOURCE_LIST" | xargs sha256sum | sha256sum | cut -d' ' -f1)"

# build <variant> <package> <bin> <install name> <cargo features>
build() {
  local v="$1" pkg="$2" bin="$3" name="$4" feats="$5" log="$LOG_DIR/build-$1.log" exe
  echo "== building $v ($feats, profile $PROFILE)"
  cargo build $LOCKED --profile "$PROFILE" -p "$pkg" --bin "$bin" --features "$feats" --message-format=json-render-diagnostics \
    > "$log.json" 2> "$log" || { echo "build of $v failed, see $log" >&2; exit 1; }
  exe="$(python3 -c '
import json, sys
exe = None
for line in open(sys.argv[1]):
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["name"] == sys.argv[2]:
        exe = m["executable"]
print(exe or "")' "$log.json" "$bin")"
  [ -n "$exe" ] && [ -x "$exe" ] || { echo "error: cargo reported no executable for $bin (see $log)" >&2; exit 1; }
  rm -f "$log.json"
  cp "$exe" "$BIN_DIR/$name"
  [ "$v" = ompi-ucx ] || check_static "$BIN_DIR/$name" "$name"
  sha256sum "$BIN_DIR/$name" > "$BIN_DIR/$name.sha256"
  {
    echo "installed: $(date -Is)"
    echo "profile: $PROFILE"
    echo "lamellar: $LAMELLAR_REV"
    echo "features: $pkg/$feats"
    echo "iterations default: 25 (RANDPERF_ITERS)"
    echo "cargo-lock-sha256: $(sha256sum "$ws/Cargo.lock" | cut -d' ' -f1)"
    echo "source-sha256: $SOURCE_HASH ($SOURCE_FILES files: randperf-*/ and openshmem-rs/shmem-shim sources and TOML, Cargo.toml, Cargo.lock, .cargo/config.toml, tools/stage_runtime.py)"
  } > "$BIN_DIR/$name.provenance"
  echo "installed $BIN_DIR/$name"
}

for v in "${variants[@]}"; do
  case "$v" in
    lamellar)
      build lamellar randperf-lamellar randperf-lamellar randperf-lamellar ucx,ucx-opt,libfabric,libfabric-sys,libfabric-sys-opt,rofi-c,shmem-opt ;;
    sos-ucx|sos-ofi|ompi-ucx)
      build "$v" randperf-shmem randperf-shmem "randperf-shmem-$v" "$v" ;;
    lamellar-openshmem|lls-all)   # lls-all: the old name
      build lamellar-openshmem randperf-shmem randperf-shmem randperf-shmem-lamellar-openshmem "$los_feats" ;;
    *) echo "unknown variant '$v' (see the header of this script)" >&2; exit 2 ;;
  esac
done
python3 "$ws/tools/stage_runtime.py" "$BIN_DIR"
echo "done: binaries in $BIN_DIR"
