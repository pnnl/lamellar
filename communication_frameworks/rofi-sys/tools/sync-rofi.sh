#!/usr/bin/env bash
# Keep rofi-sys/rofi/ (the copy of ROFI that rofisys builds, and that ships in
# the published crate) in step with communication_frameworks/rofi/ (the source
# of truth).
#
# rofi/ holds only the hand-written files. rofi-sys/rofi/ holds those same
# files plus the generated autotools output (configure, Makefile.in,
# aclocal.m4, ltmain.sh, m4/, ...), so build.rs does not need autoreconf.
# Only the hand-written files are mirrored here; generated files are never
# touched except by --regenerate.
#
#   sync-rofi.sh               copy changed hand-written files from rofi/
#   sync-rofi.sh --check       report drift and exit 1 if there is any (CI)
#   sync-rofi.sh --regenerate  sync, run autoreconf, refresh the stamp
#   sync-rofi.sh --stamp       record the stamp only, asserting the generated
#                              files are already current (bootstrap)
#
# Hand-written files whose change makes the generated files stale (configure.ac
# and every Makefile.am) are tracked by a hash in rofi-sys/rofi/.autotools-stamp.
set -euo pipefail

mode=sync
case "${1:-}" in
    "") ;;
    --check) mode=check ;;
    --regenerate) mode=regenerate ;;
    --stamp) mode=stamp ;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $1 (try --help)" >&2; exit 2 ;;
esac

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SRC=$(cd "$here/../../rofi" && pwd)
DST=$(cd "$here/../rofi" && pwd)
STAMP="$DST/.autotools-stamp"

# Hand-written files mirrored from rofi/ (paths relative to both roots). Files
# rofi/ keeps but that are not built or shipped are left out.
authored_files() {
    (
        cd "$SRC"
        find src tests -type f \( -name '*.c' -o -name '*.h' -o -name 'Makefile.am' \)
        printf '%s\n' configure.ac Makefile.am README.md docs/Makefile.am \
            docs/Doxyfile.in uthash/LICENSE uthash/src/uthash.h
    ) | grep -vE '^src/(context\.c|include/ft_osd\.h|include/unix/)' | sort
}

# "<destination><TAB><source>" pairs. make dist replaced the uthash/include
# symlink with a real copy of uthash.h, so that copy is mirrored as well.
pairs() {
    authored_files | awk '{ print $0 "\t" $0 }'
    printf 'uthash/include/uthash.h\tuthash/src/uthash.h\n'
}

# Hand-written-looking files in the destination that rofi/ no longer provides.
stale_files() {
    comm -23 \
        <(cd "$DST" && find src tests -type f \( -name '*.c' -o -name '*.h' -o -name 'Makefile.am' \) | sort) \
        <(authored_files)
}

build_hash() {
    (
        cd "$SRC"
        { echo configure.ac; echo Makefile.am; find src tests docs -name Makefile.am; } \
            | sort -u | xargs sha256sum
    ) | sha256sum | cut -d' ' -f1
}

stamp_state() { # prints current|stale|missing
    if [ ! -f "$STAMP" ]; then echo missing
    elif [ "$(cat "$STAMP")" = "$(build_hash)" ]; then echo current
    else echo stale
    fi
}

check_drift() {
    local drift=0 d s
    while IFS=$'\t' read -r d s; do
        if [ ! -e "$DST/$d" ]; then echo "MISSING $d"; drift=1
        elif ! cmp -s "$SRC/$s" "$DST/$d"; then echo "DIFF    $d"; drift=1
        fi
    done < <(pairs)
    while IFS= read -r d; do
        [ -n "$d" ] && { echo "STALE   $d (no longer in rofi/)"; drift=1; }
    done < <(stale_files)
    if [ "$(stamp_state)" != current ]; then
        echo "STAMP   generated autotools files are $(stamp_state) relative to configure.ac/Makefile.am; run $0 --regenerate"
        drift=1
    fi
    return $drift
}

sync_files() {
    local d s
    while IFS=$'\t' read -r d s; do
        if ! cmp -s "$SRC/$s" "$DST/$d" 2>/dev/null; then
            mkdir -p "$(dirname "$DST/$d")"
            cp "$SRC/$s" "$DST/$d"
            echo "updated $d"
        fi
    done < <(pairs)
    while IFS= read -r d; do
        [ -n "$d" ] && { rm -f "$DST/$d"; echo "removed $d (no longer in rofi/)"; }
    done < <(stale_files)
}

case "$mode" in
    check)
        if check_drift; then echo "rofi-sys/rofi is in sync with rofi"; else exit 1; fi
        ;;
    sync)
        sync_files
        [ "$(stamp_state)" = current ] || \
            echo "NOTE: configure.ac/Makefile.am changed since the generated files were built; run $0 --regenerate"
        ;;
    regenerate)
        command -v autoreconf >/dev/null || { echo "autoreconf not found" >&2; exit 1; }
        sync_files
        (cd "$DST" && autoreconf -ivf)
        rm -rf "$DST/autom4te.cache"
        find "$DST" -name '*~' -type f -delete # autoreconf backups, e.g. config.h.in~
        build_hash > "$STAMP"
        echo "regenerated; review 'git diff' (autoreconf may also refresh config.guess, ltmain.sh, ...)"
        ;;
    stamp)
        build_hash > "$STAMP"
        echo "stamp recorded: $(cat "$STAMP")"
        ;;
esac
