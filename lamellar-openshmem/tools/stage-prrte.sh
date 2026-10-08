#!/bin/bash
# Build the vendored PRRTE (prterun) and stage it, with its PMIx, under STAGE_DIR/{prrte,pmix}.
#   tools/stage-prrte.sh STAGE_DIR     (CARGOFLAGS=--offline etc. are passed through)
# This is a compile (autotools, several minutes): run it on a login node, not inside a job.
# It builds through a scratch crate so the monorepo [patch] paths apply (building prrte-sys on its
# own would pull the older crates.io pmix-sys, which needs a system hwloc).
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
repo="$(cd "$root/.." && pwd)"
stage="$1"; mkdir -p "$stage"; stage="$(cd "$stage" && pwd)"
work="$root/target/prrte-build"
mkdir -p "$work/src"
echo 'fn main(){}' > "$work/src/main.rs"
cat > "$work/Cargo.toml" <<TOML
[package]
name = "prrte_build"
version = "0.0.0"
edition = "2021"

[dependencies]
prrte-sys = { path = "$repo/process_management/prrte-sys", features = ["vendored"] }

[patch.crates-io]
pmi = { path = "$repo/process_management/pmi" }
pmi-sys = { path = "$repo/process_management/pmi-sys" }
pmi2-sys = { path = "$repo/process_management/pmi2-sys" }
pmi-mpich-src = { path = "$repo/process_management/pmi-mpich-src" }
pmix-sys = { path = "$repo/process_management/pmix-sys" }
prrte-sys = { path = "$repo/process_management/prrte-sys" }
openpmix-src = { path = "$repo/process_management/openpmix-src" }
prrte-src = { path = "$repo/process_management/prrte-src" }

[workspace]
TOML
(cd "$work" && ${CARGO:-cargo} build --release ${CARGOFLAGS:-} --target-dir "$work/target")
newest() { ls -dt "$work"/target/release/build/$1 2>/dev/null | while read -r d; do [ -e "$d/$2" ] && { echo "$d"; break; }; done; }
prrte="$(newest 'prrte-sys-*/out' bin/prterun)"; pmix="$(newest 'pmix-sys-*/out' include/pmix.h)"
[ -n "$prrte" ] && [ -n "$pmix" ] || { echo "stage-prrte: could not find the PRRTE/PMIx install dirs under $work" >&2; exit 1; }
rm -rf "$stage/prrte" "$stage/pmix"; cp -a "$prrte" "$stage/prrte"; cp -a "$pmix" "$stage/pmix"
echo "staged prterun: $stage/prrte/bin/prterun"
