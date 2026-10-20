#!/bin/bash
# Build a self-contained static libshmem.a for one backend and install it with the header, pkg-config file, CMake package
# and compiler wrappers:
#
#   tools/build-static.sh <backend> [prefix]       prefix defaults to target/stage/<backend>-static
#
#   backend    cargo features                           LAMELLAR_BACKEND values the library offers
#   shmem      shmem-opt                                shmem, shmem-opt
#   ucx        ucx ucx-opt shmem-opt                    ucx, ucx-opt, shmem, shmem-opt
#   libfabric  libfabric libfabric-sys-opt shmem-opt    libfabric, libfabric-sys-opt, shmem, shmem-opt
#
# A backend names a family: the plain lamellae and their -opt version are always built together, and so is shmem-opt, so
# one library serves every choice of LAMELLAR_BACKEND at run time (single-node shmem and shmem-opt only run on one node).
# The old names ucx-opt, ucx-all, libfabric-sys-opt and shmem-opt are accepted as aliases for their family.
#
# The result needs only system libraries at link time (verbs, rdmacm, numa, libc, ...), listed in <prefix>/lib/shmem.libs
# and in Libs.private of shmem.pc. A program links it with
#
#   shmemcc --shmem-static prog.c -o prog                  (the wrappers in <prefix>/bin)
#   cc prog.c $(pkg-config --static --cflags --libs shmem)
#   find_package(LamellarShmem) + Lamellar::shmem_static   (CMake)
#
# The libfabric archives already contain libfabric. UCX and UCC do not: cargo builds them as static archives but does not
# put them into the Rust static library, and they register their transports and components from static constructors, which
# an ordinary archive link would drop. This script therefore merges them into one relocatable object (`ld -r
# --whole-archive`) and adds that object to libshmem.a, so the linker takes it whole.
#
# Needs cargo-c (`cargo install cargo-c`). Compiling is fine on a login node.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
name="${1:?usage: tools/build-static.sh <backend> [prefix]}"
prefix="${2:-$root/target/stage/$name-static}"
case "$name" in
  shmem|shmem-opt)                  family=shmem;     features="shmem-opt" ;;
  ucx|ucx-opt|ucx-all)              family=ucx;       features="ucx ucx-opt shmem-opt" ;;
  libfabric|libfabric-sys-opt)      family=libfabric; features="libfabric libfabric-sys-opt shmem-opt" ;;
  *) echo "tools/build-static.sh: unknown backend '$name' (see the header of this script)" >&2; exit 2 ;;
esac
cargo cinstall --help >/dev/null 2>&1 || { echo "tools/build-static.sh needs cargo-c: cargo install cargo-c" >&2; exit 2; }

# crabtime (a build dependency of lamellar) needs an ancestor directory named `target`
tdir="$root/target/build/static-$name"
triple="$(rustc -vV | sed -n 's/^host: //p')"

echo "== build the static library for $name -> $prefix"
python3 "$root/include/gen_shmem_h.py" "$root/include/shmem.h"
( cd "$root" && cargo cinstall --release --library-type staticlib --prefix "$prefix" --libdir "$prefix/lib" \
    --target-dir "$tdir" ${features:+--features "$features"} ${CARGOFLAGS:-} )
# the wrappers carry PRRTE (built against the vendored PMIx), so oshrun does not depend on whatever prterun is on PATH
cargo install --path "$root/wrappers" --root "$prefix" --force --features bundled-prrte \
    --target-dir "$tdir/wrappers" ${CARGOFLAGS:-}

echo "== install the bundled PRRTE -> $prefix/prrte"
# prrte-sys installs PRRTE into its build-script output directory; the newest one is this build's
prrte_src="$(ls -dt "$tdir"/wrappers/release/build/prrte-sys-*/out 2>/dev/null | while read -r d; do [ -x "$d/bin/prterun" ] && { echo "$d"; break; }; done)"
[ -n "$prrte_src" ] || { echo "tools/build-static.sh: the wrappers build did not install PRRTE under $tdir/wrappers" >&2; exit 1; }
[ -x "$prrte_src/bin/prterun" ] || { echo "tools/build-static.sh: no prterun in $prrte_src/bin" >&2; exit 1; }
rm -rf "$prefix/prrte"
mkdir -p "$prefix/prrte"
# prte and its tools are static (system libraries only), so bin/ and share/ are the whole runtime tree
cp -a "$prrte_src/bin" "$prrte_src/share" "$prefix/prrte/"
# the help text of PMIx (prterun embeds it): prterun --prefix <dir> makes PMIx look in <dir>/share/pmix
pmix_share="$(ls -dt "$tdir"/wrappers/release/build/pmix-sys-*/out/share/pmix 2>/dev/null | while read -r d; do [ -f "$d/help-cli.txt" ] && { echo "$d"; break; }; done)"
if [ -n "$pmix_share" ]; then cp -a "$pmix_share" "$prefix/prrte/share/"; else echo "tools/build-static.sh: warning: no PMIx help files found; PMIx messages will not be readable" >&2; fi

case "$family" in
  ucx)
    echo "== merge the vendored UCX and UCC archives into libshmem.a"
    bdir="$tdir/$triple/release/build"
    # the newest build-script output directory that holds the archives (older ones can linger in a reused target dir)
    newest() { ls -dt "$bdir"/$1-*/out/lib 2>/dev/null | while read -r d; do [ -f "$d/$2" ] && { echo "$d"; break; }; done; }
    ucx_lib="$(newest lamellar-ucx-sys libucp.a)"; ucc_lib="$(newest lamellar-ucc-sys libucc.a)"
    [ -n "$ucx_lib" ] && [ -n "$ucc_lib" ] || { echo "tools/build-static.sh: UCX/UCC archives not found under $bdir" >&2; exit 1; }
    work="$(mktemp -d "$tdir/merge.XXXXXX")"
    archives="$(find "$ucx_lib" "$ucc_lib" -maxdepth 2 -name '*.a' | sort)"
    # shellcheck disable=SC2086
    ${LD:-ld} -r -o "$work/uc_all.o" --whole-archive $archives --no-whole-archive
    ar r "$prefix/lib/libshmem.a" "$work/uc_all.o"
    rm -rf "$work"
    ;;
esac

# libraries that are now inside libshmem.a and must not appear on the link line
is_inside() {
  case "$family" in
    ucx) case "$1" in -lucc*|-lucp|-luct*|-lucs|-lucm) return 0 ;; esac ;;
  esac
  return 1
}

# what a link of libshmem.a still needs from the system: Libs.private of shmem.pc, minus the libraries now inside the archive
pc="$prefix/lib/pkgconfig/shmem.pc"
priv="$(sed -n 's/^Libs.private: *//p' "$pc")"
libs=""
for w in $priv; do
  is_inside "$w" && continue
  case " $libs " in *" $w "*) ;; *) libs="$libs $w" ;; esac
done
libs="${libs# }"
echo "$libs" > "$prefix/lib/shmem.libs"
sed -i "s|^Libs.private:.*|Libs.private: $libs|; s|^Libs:.*|Libs: -L\${libdir} -lshmem $libs|" "$pc"
echo "installed $name: $(du -h "$prefix/lib/libshmem.a" | cut -f1) libshmem.a; link with: shmemcc --shmem-static (in $prefix/bin) or $libs"
