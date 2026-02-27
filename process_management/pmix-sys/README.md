# pmix-sys

`pmix-sys` exposes the PMIx API through `bindgen`. It either uses an existing installation (set `DEP_PMIX_ROOT`) or builds the bundled `openpmix-src` Autotools tree together with optional bundled `hwloc`/`libevent`.

## Features

- `vendored` triggers the `openpmix-src` build and the bundled `hwlocality-sys`/`libevent-sys` features so the entire stack is self-contained.
- Leave `vendored` off to link against a system PMIx install; `DEP_PMIX_ROOT` must point to that tree so the build script can forward `include` and `lib` paths.
