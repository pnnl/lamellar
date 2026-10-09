#!/bin/bash
# Install one backend into its own prefix and build the tests against it:
#
#   tests/stage.sh <backend> [extra cmake arguments]
#
#   backend             run-time backend   cargo features
#   shmem               shmem              (none)
#   shmem-opt           shmem-opt          shmem-opt
#   ucx                 ucx                ucx
#   ucx-opt             ucx-opt            ucx-opt
#   ucx-shared          ucx                ucx-shared        (UCX as shared libraries, copied next to libshmem.so)
#   libfabric           libfabric          libfabric-shared  (libfabric.so copied next to libshmem.so)
#   libfabric-sys-opt   libfabric-sys-opt  libfabric-sys-opt-shared
#
# Result: the library, header, pkg-config file, CMake package and the wrappers (shmemcc, oshrun, ...) in
# target/stage/<backend>, and the built tests in target/tests-build/<backend>. Then, inside a Slurm allocation:
#
#   ctest --test-dir target/tests-build/<backend> -L single-node --output-on-failure
#   ctest --test-dir target/tests-build/<backend> -L multinode   --output-on-failure   # >= 2 nodes, network backends
#
# Needs cargo-c (`cargo install cargo-c`) and CMake >= 3.20. Compiling is fine on a login node. The multi-node tests need
# prterun unless Slurm has PMIx: build it once with `make prrte`; it is used automatically when target/fab/prrte/bin exists.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
name="${1:?usage: tests/stage.sh <backend> [extra cmake arguments]}"; shift
case "$name" in
  shmem)             features=""; backend=shmem ;;
  shmem-opt)         features="shmem-opt"; backend=shmem-opt ;;
  ucx)               features="ucx"; backend=ucx ;;
  ucx-opt)           features="ucx-opt"; backend=ucx-opt ;;
  ucx-shared)        features="ucx-shared"; backend=ucx ;;
  libfabric)         features="libfabric-shared"; backend=libfabric ;;
  libfabric-sys-opt) features="libfabric-sys-opt-shared"; backend=libfabric-sys-opt ;;
  *) echo "tests/stage.sh: unknown backend '$name' (see the header of this script)" >&2; exit 2 ;;
esac
cargo cinstall --help >/dev/null 2>&1 || { echo "tests/stage.sh needs cargo-c: cargo install cargo-c" >&2; exit 2; }

prefix="$root/target/stage/$name"
# crabtime (a build dependency of lamellar) needs an ancestor directory named `target`
tdir="$root/target/build/$name"
triple="$(rustc -vV | sed -n 's/^host: //p')"

echo "== install $name -> $prefix"
python3 "$root/include/gen_shmem_h.py" "$root/include/shmem.h"
( cd "$root" && cargo cinstall --release --prefix "$prefix" --libdir "$prefix/lib" --target-dir "$tdir" \
    ${features:+--features "$features"} ${CARGOFLAGS:-} )
cargo install --path "$root/wrappers" --root "$prefix" --force ${CARGOFLAGS:-}
case "$name" in
  ucx-shared) "$root/tools/copy-ucx.sh" "$tdir/$triple" "$prefix/lib" ;;
  libfabric|libfabric-sys-opt) "$root/tools/copy-libfabric.sh" "$tdir/$triple" "$prefix/lib" ;;
esac

echo "== build the tests"
globals=ON; [ "$name" = shmem ] && globals=OFF    # the original shmem backend cannot register the data segment
# UCX and libfabric need a self-contained static library (tools/build-static.sh, a second build of the backend without the
# *-shared features); SHMEM_SKIP_STATIC=1 skips it and the static test
static=ON; static_prefix=""
case "$name" in
  ucx|ucx-opt|ucx-shared|libfabric|libfabric-sys-opt)
    if [ "${SHMEM_SKIP_STATIC:-0}" = 1 ]; then
      static=OFF
    else
      static_backend="$backend"
      static_prefix="$root/target/stage/$static_backend-static"
      echo "== static library for $static_backend -> $static_prefix"
      "$root/tools/build-static.sh" "$static_backend" "$static_prefix"
    fi ;;
esac
prrte=""; [ -x "$root/target/fab/prrte/bin/prterun" ] && prrte="$root/target/fab/prrte/bin"
cmake -S "$root/tests" -B "$root/target/tests-build/$name" -DCMAKE_PREFIX_PATH="$prefix" -DSHMEM_BACKEND="$backend" \
      -DSHMEM_TEST_GLOBALS="$globals" -DSHMEM_TEST_STATIC="$static" ${static_prefix:+-DSHMEM_STATIC_PREFIX="$static_prefix"} \
      ${prrte:+-DSHMEM_PRRTE_DIR="$prrte"} "$@" >/dev/null
cmake --build "$root/target/tests-build/$name" -j 8 >/dev/null
echo "staged $name: ctest --test-dir target/tests-build/$name -L single-node --output-on-failure"
