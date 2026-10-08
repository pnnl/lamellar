#!/bin/bash
# Copy the vendored shared libfabric (libfabric.so*) built by the libfabric-shared feature into DEST.
#   tools/copy-libfabric.sh TARGET_DIR DEST
# Picks the newest build directory that actually contains libfabric.so (stale static-only builds
# leave sibling directories without one). Exits nonzero if none is found.
set -eu
target="$1"; dest="$2"
src=$(ls -dt "$target"/release/build/lamellar-libfabric-sys-*/out/lib 2>/dev/null | while read -r d; do [ -e "$d/libfabric.so" ] && { echo "$d"; break; }; done)
[ -n "$src" ] || { echo "copy-libfabric: no libfabric.so under $target/release/build (was it built with FEATURES=libfabric-shared?)" >&2; exit 1; }
mkdir -p "$dest"
cp -P "$src"/libfabric.so* "$dest"/
echo "copied libfabric.so from $src"
