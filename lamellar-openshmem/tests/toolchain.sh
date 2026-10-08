#!/bin/bash
# End-to-end toolchain test: install into a temporary prefix with cargo (cargo-c for the library, `cargo install` for the
# wrappers), then build and run programs through every entry point (wrapper C/C++, static link, pkg-config, CMake).
#
#   bash tests/toolchain.sh                        the default (shmem) backend
#   FEATURES=shmem-opt bash tests/toolchain.sh     another backend's cargo features (the run-time backend follows)
#
# Needs cargo-c (`cargo install cargo-c`). Installing and compiling need no allocation; the stages that run programs are
# skipped unless the script runs inside a Slurm allocation or TOOLCHAIN_RUN=1 is set.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
# Scratch space on a shared filesystem, so compute nodes can see what a login node built.
mkdir -p "$root/target"
tmp="$(mktemp -d "${TOOLCHAIN_TMP:-$root/target}/toolchain.XXXXXX")"; trap 'rm -rf "$tmp"' EXIT
prefix="$tmp/prefix"
target="${TARGET:-target}"
FEATURES="${FEATURES:-}"
case "$FEATURES" in   # the run-time backend that goes with the cargo features
  *libfabric-sys-opt*) backend_default=libfabric-sys-opt ;;
  *libfabric*)         backend_default=libfabric ;;
  *ucx-opt*)           backend_default=ucx-opt ;;
  *ucx*)               backend_default=ucx ;;
  *shmem-opt*)         backend_default=shmem-opt ;;
  *)                   backend_default=shmem ;;
esac
[[ "$FEATURES" == *libfabric* || "$FEATURES" == *ucx* ]] && [ -x "$root/target/fab/prrte/bin/prterun" ] && export PATH="$root/target/fab/prrte/bin:$PATH"
export LAMELLAR_BACKEND="${LAMELLAR_BACKEND:-$backend_default}" SHMEM_SYMMETRIC_SIZE="${SHMEM_SYMMETRIC_SIZE:-67108864}" \
       LAMELLAR_HEAP_SIZE="${LAMELLAR_HEAP_SIZE:-1073741824}" LAMELLAR_THREADS="${LAMELLAR_THREADS:-2}"
can_run=0
if [ -n "${SLURM_JOB_ID:-}" ] || [ "${TOOLCHAIN_RUN:-0}" = 1 ]; then can_run=1
else
  echo "NOTE: not inside a Slurm allocation (and TOOLCHAIN_RUN is not 1) -- install and compile stages only; run stages SKIPPED."
fi
pass() { if [ $can_run = 1 ]; then echo "  ok: $1"; else echo "  ok (build only, run skipped): $1"; fi; }
fail() { echo "  FAIL: $1"; exit 1; }
# count PEs that print the success marker
expect() { local n="$1" pat="$2"; shift 2; [ $can_run = 1 ] || { echo "  (skipped run: $*)"; return 0; }; local out; out="$("$@" 2>&1 | tr -d '\0')" || { echo "$out"; fail "$* exited nonzero"; }
           [ "$(grep -c "$pat" <<<"$out")" = "$n" ] || { echo "$out"; fail "expected $n x '$pat' from $*"; }; }

cargo cinstall --help >/dev/null 2>&1 || { echo "tests/toolchain.sh needs cargo-c: cargo install cargo-c" >&2; exit 2; }

echo "== install to $prefix"
( cd "$root" && cargo cinstall --release --prefix "$prefix" --libdir "$prefix/lib" --target-dir "$target" \
    ${FEATURES:+--features "$FEATURES"} ${CARGOFLAGS:-} >/dev/null )
cargo install --path "$root/wrappers" --root "$prefix" --force ${CARGOFLAGS:-} >/dev/null 2>&1
# a libfabric.so build needs the library next to libshmem.so (cargo-c does not copy it)
if [[ "$FEATURES" == *libfabric*shared* ]]; then
  triple="$(rustc -vV | sed -n 's/^host: //p')"
  "$root/tools/copy-libfabric.sh" "$target/$triple" "$prefix/lib" >/dev/null
fi
req=(bin/shmemcc bin/oshcc bin/shmemcxx bin/oshcxx bin/oshrun bin/shmemrun include/shmem.h lib/libshmem.so lib/libshmem.a
     lib/pkgconfig/shmem.pc share/cmake/LamellarShmem/LamellarShmemConfig.cmake)
[[ "$FEATURES" == *libfabric*shared* ]] && req+=(lib/libfabric.so.1)
for f in "${req[@]}"; do
  [ -e "$prefix/$f" ] || fail "missing $f"; done; pass "installed layout"
export PATH="$prefix/bin:$PATH"
real_prefix="$(cd "$prefix" && pwd -P)"

echo "== wrapper introspection"
[ "$(oshcxx --showme:command)" = "${OSHCC_CXX:-${CXX:-c++}}" ] || fail "C++ wrapper driver"
oshcc --showme:compile | grep -q -- "-I$real_prefix/include" || fail "showme:compile"
oshcc --showme:link | grep -q -- "-lshmem" || fail "showme:link"
oshcc --shmem-static --showme:link | grep -q -- "--whole-archive" || fail "static showme"
oshcc --showme foo.c -o foo | grep -q "foo.c" || fail "showme passes args"
oshcc --showme:version | grep -q OpenSHMEM || fail "showme:version"; pass "showme flags"

echo "== C, shared"
oshcc -std=c11 -Wall "$root/tests/c/full.c" -o "$tmp/full" && expect 4 "all checks passed" oshrun -np 4 "$tmp/full"; pass "full.c x4"
echo "== C++, shared"
oshcxx -Wall "$root/tests/cxx/hello.cpp" -o "$tmp/hello" && expect 3 "(C++): OK" oshrun -np 3 "$tmp/hello"; pass "hello.cpp x3"
echo "== C, static (whole-archive)"
oshcc --shmem-static -std=c11 "$root/tests/c/full.c" -o "$tmp/full_static" && expect 3 "all checks passed" oshrun -np 3 "$tmp/full_static"; pass "static full.c x3"
echo "== pkg-config"
export PKG_CONFIG_PATH="$prefix/lib/pkgconfig"
cc $(pkg-config --cflags shmem) "$root/tests/c/ring.c" -o "$tmp/ring_pc" $(pkg-config --libs shmem) -Wl,-rpath,"$prefix/lib" \
  && expect 2 "OK" oshrun -np 2 "$tmp/ring_pc"; pass "pkg-config ring x2"
echo "== CMake"
if command -v cmake >/dev/null; then
  cmake -S "$root/tests/cmake" -B "$tmp/cm" -DCMAKE_PREFIX_PATH="$prefix" >/dev/null && cmake --build "$tmp/cm" >/dev/null
  expect 2 "OK" oshrun -np 2 "$tmp/cm/ring"; expect 2 "OK" oshrun -np 2 "$tmp/cm/ring_static"; pass "cmake shared + static"
else echo "  skipped (no cmake)"; fi
if [ $can_run = 1 ]; then echo "toolchain OK"; else echo "toolchain BUILD stages OK -- RUN stages SKIPPED"; fi
