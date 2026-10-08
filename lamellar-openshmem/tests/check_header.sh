#!/bin/bash
# Verifies that every function declared in include/shmem.h is exported by libshmem.so.
set -e
cd "$(dirname "$0")/.."
python3 include/gen_shmem_h.py include/shmem.h
lib=${1:-target/release/libshmem.so}
# function names = identifier directly before '(' in prototype lines
grep -E '^[A-Za-z_].*\(.*\);$' include/shmem.h | grep -v '^#' \
  | sed -E 's/^.*[ *]([A-Za-z_0-9]+)\(.*$/\1/' | sort -u > /tmp/declared.$$
nm -D --defined-only "$lib" | awk '{print $3}' | sort -u > /tmp/exported.$$
missing=$(comm -23 /tmp/declared.$$ /tmp/exported.$$)
rm -f /tmp/declared.$$ /tmp/exported.$$
if [ -n "$missing" ]; then echo "declared but not exported:"; echo "$missing"; exit 1; fi
echo "header OK: all declared symbols are exported"
