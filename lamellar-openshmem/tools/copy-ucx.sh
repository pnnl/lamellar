#!/bin/bash
# Copy the vendored shared UCX (libucp/libuct/libucs/libucm .so* and the lib/ucx transport plugins) built by the
# ucx-shared feature into DEST, keeping UCX's layout: core libraries in DEST, plugins in DEST/ucx (UCX looks for
# them next to libucs.so). Put libshmem.so in DEST too: its $ORIGIN runpath then finds these copies first.
#   tools/copy-ucx.sh TARGET_DIR DEST
# Picks the newest build directory that actually contains libucp.so (stale static-only builds leave sibling
# directories without one). Exits nonzero if none is found.
set -eu
target="$1"; dest="$2"
src=$(ls -dt "$target"/release/build/lamellar-ucx-sys-*/out/lib 2>/dev/null | while read -r d; do [ -e "$d/libucp.so" ] && { echo "$d"; break; }; done)
[ -n "$src" ] || { echo "copy-ucx: no libucp.so under $target/release/build (was it built with FEATURES=ucx-shared?)" >&2; exit 1; }
mkdir -p "$dest/ucx"
cp -P "$src"/libuc*.so* "$dest"/
if [ -d "$src/ucx" ]; then cp -P "$src"/ucx/*.so* "$dest/ucx"/; fi
# libtool bakes an RPATH to the cargo build directory into every UCX library. RPATH is searched before $ORIGIN and
# LD_LIBRARY_PATH, so it would shadow these copies (and break once the build directory is cleaned). Point the
# libraries at their own directory instead: core libraries at $ORIGIN, plugins at the directory above them.
if command -v patchelf >/dev/null; then
  for f in "$dest"/libuc*.so.*.*.*; do [ -L "$f" ] || patchelf --force-rpath --set-rpath '$ORIGIN' "$f"; done
  for f in "$dest"/ucx/*.so.*.*.*; do [ -L "$f" ] || patchelf --force-rpath --set-rpath '$ORIGIN:$ORIGIN/..' "$f"; done
else
  echo "copy-ucx: patchelf not found; the copies still carry an RPATH to the cargo build directory" >&2
fi
echo "copied UCX shared libraries ($(ls "$dest"/libuc*.so.0 | wc -l) libs, $(ls "$dest"/ucx/*.so.0 2>/dev/null | wc -l) plugins) from $src"
