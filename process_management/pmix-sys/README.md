# pmix-sys

`pmix-sys` exposes the PMIx API through `bindgen`. It either uses an existing installation (set `DEP_PMIX_ROOT`) or builds the bundled `openpmix-src` Autotools tree together with optional bundled `hwloc`/`libevent`.

## Features

- `vendored` triggers the `openpmix-src` build plus `vendored-hwloc`/`vendored-libevent` so the entire stack is self-contained.
- `vendored-hwloc` / `vendored-libevent` vendor just hwloc or libevent (via the bundled `hwlocality-sys`/`libevent-sys` features), while still vendoring PMIx itself. Set `HWLOC_DIR` / `LIBEVENT_DIR` to a system install prefix for whichever one you leave unvendored (e.g. `HWLOC_DIR=$(brew --prefix hwloc) cargo build --features vendored-libevent`). This is the escape hatch for macOS, where hwlocality's vendored autotools build fails.
- Leave `vendored` off to link against a system PMIx install; `DEP_PMIX_ROOT` must point to that tree so the build script can forward `include` and `lib` paths.
