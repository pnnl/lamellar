# Native dependencies

Lamellar is a Rust library, but distributed backends build and link native
communication and process-management libraries. This guide lists the packages
needed to build an application, and explains when Lamellar builds a bundled
copy instead of using a system installation.

For an ordinary single-node application using the `local` or `shmem` backend,
no transport library is needed.

## Quick start

All builds need a Rust toolchain, a C compiler/linker, GNU Make, CMake, Perl,
and Clang with the `libclang` development library (used by `bindgen`). Package
names vary by distribution; on Linux, the relevant packages are commonly named
`clang`, `libclang-dev`, `gcc`/`build-essential`, `make`, `cmake`, and `perl`.

The default Lamellar feature set (`with-pmix`, `vendored-pmi`,
`enable-lamellar-main`, `vendored-libevent`, `vendored-pmix`) builds the
required PMIx/PRRTE launch stack — PMIx, PRRTE, libevent, and hwloc — from
vendored sources. hwloc is built by the vendored PMIx build (`openpmix-src`)
from its bundled source and reused by PRRTE; set `HWLOC_DIR` to link a system
hwloc instead.

To use a distributed transport, enable one feature in your application's
`Cargo.toml`:

```toml
[dependencies]
lamellar = { version = "…", features = ["enable-libfabric"] }
# Or: "enable-ucx", "enable-rofi-c", or "enable-rofi-c-pmix"
```

## Build prerequisites

| Requirement | Needed for | Notes |
| --- | --- | --- |
| Rust (`rustc`, Cargo) | Every build | Lamellar does not currently declare a minimum Rust version in its manifest. |
| C compiler and linker | Native bindings and all vendored libraries | A standard GCC or Clang toolchain is sufficient. |
| GNU Make | Any vendored native library | Runs the generated Makefiles after configuration. |
| CMake | Default vendored libevent build | The transitive `libevent-sys` bundled build uses CMake. |
| Perl | Default vendored libevent build | Bundled libevent enables bundled OpenSSL; OpenSSL's source build runs Perl. |
| Clang and libclang development files | `*-sys` crates | Required by `bindgen` to generate Rust FFI bindings. |
| `pkg-config` / `pkgconf` | System PMIx, PRRTE, libevent, or hwloc discovery | Only relevant when linking system installs instead of the vendored ones. |
| hwloc development files | Only when linking a system hwloc | Set `HWLOC_DIR`, or needed with a system PMIx (see below); see the version guidance below. |
| RDMA development files (`libibverbs`, `librdmacm`) | libfabric, UCX, and ROFI | Required by Lamellar's current native link directives. |
| `libnuma`, `libuuid`, `libatomic` development files | libfabric and ROFI | Required by Lamellar's current native link directives. |

You do **not** need Git, Autoconf, Automake, Libtool, or Flex for the default
Lamellar vendored build. Lamellar's checked-in native sources use generated
configuration files, with their maintainer-mode regeneration rules disabled by
default. The default vendored libevent/OpenSSL dependency chain still requires
CMake and Perl, as listed above.

The bundled hwloc is built the same way, from a checked-in release tarball,
with no network access needed. Autotools programs are only needed when
regenerating or refreshing a vendored source archive.

## Backend dependencies

| Lamellar feature | Native transport libraries | Process-management layer | Bundled when a system prefix is not provided |
| --- | --- | --- | --- |
| `local`, `shmem` | None | None | N/A |
| `enable-libfabric`, `enable-libfabric-sys`, `enable-libfabric-async` | libfabric / OFI | PMI-1, PMI-2, or PMIx (PMIx by default) | libfabric and the selected PMI implementation |
| `enable-ucx` | UCX and UCC | PMI-1, PMI-2, or PMIx (PMIx by default) | UCX, UCC, and the selected PMI implementation |
| `enable-rofi-c`, `enable-rofi-c-shared` | ROFI and libfabric / OFI | ROFI's own PMI-1 client (`simple_pmi.c`); no PMIx | ROFI and libfabric |
| `enable-rofi-c-pmix`, `enable-rofi-c-shared-pmix` | ROFI and libfabric / OFI | PMIx (only; not selectable) | ROFI, libfabric, and PMIx |

The `enable-rofi-rust` feature currently does not activate a native dependency
by itself.

`enable-rofi-c`/`enable-rofi-c-shared` build ROFI against its own bundled
PMI-1 client — no PMIx, and nothing routed through Lamellar's `pmi`
process-management abstraction, so there's nothing to vendor beyond ROFI and
libfabric themselves. Layer `enable-rofi-c-pmix`/`enable-rofi-c-shared-pmix`
on top to switch ROFI's process management to PMIx instead: this turns on
`rofi-sys`'s own `pmix` Cargo feature, which passes ROFI's C-library
`configure` step a `--with-pmix` pointing at `pmix-sys` (see [rofi-sys's
README](communication_frameworks/rofi-sys/README.md)). ROFI has no PMI-2
client, so PMI-1/PMIx are the only two options.

`rofi-sys` itself separates using PMIx from vendoring it: `pmix` just turns
on PMIx support, and a separate `vendored-pmix` feature
(`pmix-sys?/openpmix-src`) is what vendors it. Lamellar's own
`enable-rofi-c-pmix`/`enable-rofi-c-shared-pmix` features only turn on
`rofisys/pmix` — vendoring is left to Lamellar's top-level `vendored-pmix`
feature (on by default, and already wired to `rofisys?/vendored-pmix`).
Drop `vendored-pmix` and set `PMIX_LIB_DIR`/`PMIX_INCLUDE_DIR` to link a
system PMIx for ROFI through the `lamellar` crate directly — no need to
depend on `rofisys` yourself.

## Choosing a process manager

The distributed transports use Lamellar's `pmi` abstraction. Choose the
provider that matches your cluster's launcher:

| Feature | Native library | System-library variables | Vendored source |
| --- | --- | --- | --- |
| `with-pmi1` | `libpmi` | `PMI_LIB_DIR`, `PMI_INCLUDE_DIR` | MPICH `libpmi` 5.0.0 |
| `with-pmi2` | `libpmi2` | `PMI2_LIB_DIR`, `PMI2_INCLUDE_DIR` | MPICH `libpmi` 5.0.0 |
| `with-pmix` (default) | `libpmix` | `PMIX_LIB_DIR`, `PMIX_INCLUDE_DIR` | OpenPMIx 5.0.9 |
| `enable-lamellar-main` (default) | `libprrte`, `prterun` | `PRRTE_LIB_DIR`, `PRRTE_INCLUDE_DIR`, `PRRTE_BIN_DIR` | PRRTE 3.0.14 |

The default feature set pairs `with-pmix` with a separate `vendored-pmi`
feature to vendor it, rather than a single combined switch — same effect,
just decomposed so vendoring can be dropped independently of picking PMIx.
The same generic `vendored-pmi` toggle pairs with whichever of `with-pmi1`/
`with-pmi2`/`with-pmix` is selected. The `pmi` sub-crate's own per-backend
aliases (`with-pmi1-vendored`, `with-pmi2-vendored`, `with-pmix-vendored`)
aren't all re-exposed at the `lamellar` crate level — only
`with-pmix-vendored` is. Use `with-pmi1`/`with-pmi2` + `vendored-pmi`
instead of guessing at a `with-pmi1-vendored`-style name.

PMIx and PRRTE also need libevent and hwloc. The default features vendor
libevent (`vendored-libevent`), and `vendored-pmix` vendors hwloc as part of
the PMIx build, which PRRTE then reuses. `HWLOC_DIR` overrides it with a
system hwloc. With a system PMIx (no `openpmix-src`), there is no bundled
hwloc: PRRTE's configure auto-detects a system hwloc, or uses `HWLOC_DIR`.

**Setting the `*_LIB_DIR`/`*_INCLUDE_DIR`/`*_BIN_DIR` variables above is not
enough by itself.** `enable-lamellar-main` always builds PRRTE from its
bundled source (`prrte-sys/prrte-src` is unconditional, independent of the
`vendored-*` toggles), and `pmix-sys`/`openpmix-src` behave the same way once
pulled in. Their build scripts only skip that vendored build and fall back to
the system directories when `PRRTE_NO_VENDORED=1` / `PMIX_NO_VENDORED=1` is
also set. Without those, the vendored source is still compiled even if
`PRRTE_LIB_DIR`/`PMIX_LIB_DIR` etc. are set. This only applies when the
vendored build is actually part of the feature set to begin with:
`enable-lamellar-main` always pulls in `prrte-sys/prrte-src` unconditionally,
so `PRRTE_NO_VENDORED` is always relevant there, but `pmix-sys`'s vendored
build (`openpmix-src`) is gated behind Lamellar's own `vendored-pmix`
feature. Drop `vendored-pmix` (or never select it — e.g. picking `with-pmix`
outside the default launcher stack, as when pairing it with `enable-libfabric`
alone) and `openpmix-src` is never compiled in, so `PMIX_NO_VENDORED` has no
effect either way — setting `PMIX_LIB_DIR`/`PMIX_INCLUDE_DIR` alone is
sufficient in that case. The NUMA/core-topology detection used by
`#[lamellar::main]`'s PE-placement defaults scans `/sys` and does not use
hwloc or need any feature.

## Full system build of the default launch stack

The default feature set is `with-pmix`, `vendored-pmi`, `enable-lamellar-main`,
`vendored-libevent`, `vendored-pmix` — the `#[lamellar::main]` PMIx/PRRTE
launcher, with PMIx/PRRTE/libevent/hwloc vendored. A `system-pmix-launcher` convenience
alias expands to the same feature set minus the three vendoring features —
use it on a cluster with a system-installed, version-matched PMIx/PRRTE
(and libevent) already on `PKG_CONFIG_PATH` or pointed to via env vars:

```toml
lamellar = { version = "…", default-features = false, features = ["system-pmix-launcher"] }
```

```bash
PMIX_LIB_DIR=/path/to/pmix/lib PMIX_INCLUDE_DIR=/path/to/pmix/include PMIX_NO_VENDORED=1 \
PRRTE_LIB_DIR=/path/to/prrte/lib PRRTE_INCLUDE_DIR=/path/to/prrte/include PRRTE_BIN_DIR=/path/to/prrte/bin PRRTE_NO_VENDORED=1 \
LIBEVENT_DIR=/path/to/libevent \
cargo build
```

`PMIX_NO_VENDORED=1`/`PRRTE_NO_VENDORED=1` are required, not optional — dropping
`vendored-pmix`/`vendored-libevent` only stops those *sub*-dependencies of
PMIx/PRRTE from being vendored; PMIx and PRRTE themselves are still built
from bundled source unless their own `*_NO_VENDORED` variable says otherwise.
`LIBEVENT_DIR` can be omitted if a system libevent is already discoverable
via `pkg-config`. With a system PMIx there is no bundled hwloc, so PRRTE's
configure auto-detects a system hwloc; set `HWLOC_DIR` if it isn't found on
the default search paths.

To vendor only some of the three, restate the default list minus the one
feature you want to drop instead of using `system-pmix-launcher`, e.g. keep
PMIx/PRRTE vendored but link libevent from a system install:

```toml
lamellar = { version = "…", default-features = false, features = ["with-pmix", "vendored-pmi", "enable-lamellar-main", "vendored-pmix"] }
```

```bash
LIBEVENT_DIR=/path/to/libevent cargo build
```

## Using system installations for the transport libraries

Set an install prefix or include/lib directory before running Cargo. Lamellar
will prefer these values over its bundled source where applicable.

| Library | Configuration |
| --- | --- |
| libfabric | `OFI_DIR=/path/to/prefix` |
| UCX | `UCX_DIR=/path/to/prefix` |
| UCC | `UCC_DIR=/path/to/prefix` |
| ROFI | `ROFI_DIR=/path/to/prefix` |

`enable-rofi-c-pmix`/`enable-rofi-c-shared-pmix` vendor PMIx only through
Lamellar's `vendored-pmix` feature (on by default) — drop it and set
`PMIX_LIB_DIR`/`PMIX_INCLUDE_DIR` to link a system PMIx for ROFI too; see
the note in the backend dependencies table above.

## Version guidance

Lamellar validates symbols and headers, not most system-library version
numbers. The bundled versions below are the versions tested by this project.
Versions below a bundled version have **not been tested** and should not be
assumed to work, even where the build does not reject them. They are not
claimed minimum versions for a system installation.

| Component | Version / range |
| --- | --- |
| libfabric | Bundled 1.22.0 plus a backport of the MR-cache deadlock fix (see below); no system minimum enforced. |
| UCX | Bundled 1.20.0; no system minimum enforced. |
| UCC | Bundled 1.8.0; no system minimum enforced. |
| ROFI | Bundled from upstream commit `d112544`; no system minimum enforced. |
| OpenPMIx | Bundled 5.0.9. When using a system PMIx with bundled PRRTE: `4.2.4 <= PMIx < 6.0.0`. |
| PRRTE | Bundled 3.0.14; no system minimum enforced. |
| libevent | 2.0.21 or newer, built with pthread support. |
| hwloc | 1.11.0 through 2.x; 3.x is rejected by the bundled PMIx/PRRTE configuration. |

**libfabric MR-cache deadlock (fixed upstream in 2.6.0+).** libfabric before
2.6.0 can `malloc()` while holding the memory-monitor lock (`mm_lock`) when it
inserts a new MR cache entry. With the default memhooks monitor this deadlocks
against a concurrent `free()` that trims the heap (the allocator's lock is held
while the intercepted `madvise`/`brk` waits on `mm_lock`), hanging RMA/memory
registration under multithreaded load. The bundled 1.22.0 carries a backport of
upstream commit `d27cfffdced9` ("util: Drop mm lock before memory operations",
in `prov/util/src/util_mr_cache.c` and `src/tree.c`); drop the backport when the
bundled libfabric moves to 2.6.0 or newer. A system libfabric supplied via
`OFI_DIR` should be 2.6.0 or newer to avoid this hang. With an older system
libfabric the hang can also be avoided by disabling the MR cache
(`FI_MR_CACHE_MAX_COUNT=0` or `FI_MR_CACHE_MONITOR=disabled`), at a significant
performance cost, since every registration then goes to the provider.

**Static UCX and UCC.** The bundled UCX and UCC are built as static archives
and linked into the executable. UCX initializes itself entirely from
constructors (`ucs_init`, and the registration of each UCT component,
transport, and UCP protocol). Nothing references those objects by symbol, so
`lamellar-ucx-sys` links `libucm`, `libucs`, `libuct`, the transport modules
(`lib/ucx/libuct_*.a`), and `libucp` with `+whole-archive`, in that order,
which is the order the shared libraries initialize in. Without this,
`ucp_init()` hangs on uninitialized global state.

Upstream UCC can load its CL/TL/MC/EC/sysinfo components only by `dlopen()`ing
`libucc_<framework>_<name>.so`, so it has no static mode. The bundled 1.8.0
carries a Lamellar patch that is not upstream: in `src/utils/ucc_component.c`,
`ucc_components_load()` also registers any component linked into the
executable, found through weak references to its `ucc_<framework>_<name>` iface
symbol. The same function skips the `.so` search when it is called from a
component constructor before `ucc_init()` (which happens only in a static
build). `lamellar-ucc-sys` links `libucc.a` and every `lib/ucc/libucc_*.a`
with `+whole-archive` so that those components are present. A system
`UCX_DIR`/`UCC_DIR` that has only shared libraries is linked dynamically and is
unaffected.

**Third-party autotools libraries linking the bundled static UCX directly.**
`lamellar-ucx-sys`'s `+whole-archive` link directives above only cover the
*final Rust binary* link. They do nothing for a separate C library that also
links against this same static UCX during its own `./configure`/`make` step
(e.g. a vendored OpenSHMEM implementation built by its own `*-src` crate) —
that C-level link needs the identical treatment applied independently,
through that crate's own `autotools::Config`, or it will silently drop UCX's
unreferenced transport modules and fall back to tcp/self with no error. This
has bitten a real downstream consumer once already, building a separate
OpenSHMEM implementation against `DEP_UCX_ROOT`.

Required, in order, via `DEP_UCX_ROOT`:

1. `-L<DEP_UCX_ROOT>/lib/ucx` (the transport-module directory), then
   `-Wl,--whole-archive -lucm -lucs -luct -lucp`, plus each present
   `lib/ucx/libuct_<transport>.a` (`uct_ib`, `uct_ib_mlx5`, `uct_rdmacm`,
   `uct_cma`, `uct_knem` — gate each on the file actually existing, since not
   every transport module is always built), then `-Wl,--no-whole-archive`.
   Same reasoning as above: UCX self-registers via constructors, so a normal
   static link drops anything nothing else symbol-references.
2. `-lmlx5` if `libuct_ib_mlx5.a` exists.
3. `-libverbs -lrdmacm -lpthread -ldl -lrt -lm` — required even though these
   look unrelated to the whole-archive block, because forcing every object
   out of `ucm`/`ucs`/`uct`/`ucp` pulls in references to these system libs
   immediately, including during the very first "does the C compiler work"
   sanity check `./configure` runs — before the library's own configure.ac
   has even added its normal UCX probe flags.
4. `libucm.a`/`libucs.a` have a genuine circular symbol dependency (same
   class as the UCC `ucm`/`ucs` issue below) that no single archive order can
   resolve in one pass. Don't rely on a matched `--start-group`/
   `--end-group` pair the way `lamellar-ucc-sys` does for its own configure
   probes — if this whole LDFLAGS blob ever gets threaded through a libtool
   `.la` file (e.g. this crate is itself linked as a dependency of another
   `.la`), libtool strips `-Wl,*` flags out of the recorded `dependency_libs`
   and re-expands the plain `-lucm -lucs` unwrapped, silently breaking the
   group (see the `lamellar-ucc-sys`/`ucc_info` case above). Instead, open an
   **unclosed** `-Wl,--start-group` and never emit the matching
   `--end-group`: GNU ld treats an unmatched `--start-group` as extending to
   the end of the whole command line, so it keeps covering `-lucm`/`-lucs`
   regardless of whatever the consumer's own `LIBS` appends afterward,
   without needing to touch that library's own configure.ac.

**Third-party autotools libraries linking the bundled static UCC directly.**
The same gap applies to UCC: `lamellar-ucc-sys`'s `+whole-archive` link
directives only cover the final Rust binary. A separate C library that calls
`ucc_init()`/`ucc_context_create()` directly against `DEP_UCC_ROOT` (rather
than going through `lamellar-ucc-sys`) needs to replicate the same treatment
itself, or UCC's CL/TL/MC/EC/sysinfo components — registered only via the
constructor/weak-symbol pattern described above — never get linked in and
`ucc_components_load()` silently finds none.

Required, in order, via `DEP_UCC_ROOT`/`DEP_UCX_ROOT`:

1. `-Wl,--whole-archive -lucc`, plus every present `lib/ucc/libucc_*.a`
   component archive, then `-Wl,--no-whole-archive` — mirroring
   `lamellar-ucc-sys`'s own `build.rs`.
2. UCC's `libucc_tl_ucp.a` component pulls in UCX transitively, so the same
   UCX whole-archive block above (`ucm`/`ucs`/`uct`/`ucp` + transport
   modules) is also needed, with the same unclosed `-Wl,--start-group` for
   the `ucm`/`ucs` circular dependency.
3. `-libverbs -lrdmacm -lpthread -ldl -lrt -lm` for the same "first sanity
   check runs before the library's own probe flags exist" reason as the UCX
   case.

## Optional accelerators and providers

The vendored UCX and UCC configurations disable CUDA and ROCm. CUDA, ROCm,
GDRCopy, XPMEM, KNEM, and similar provider libraries are therefore not needed
for Lamellar's bundled build. They may be required if you point `UCX_DIR`,
`UCC_DIR`, or `OFI_DIR` at a custom system installation built with those
providers enabled.

## Troubleshooting

- **`bindgen` cannot find libclang:** install the libclang development package
  and ensure its library directory is visible to the dynamic loader.
- **hwloc is not found:** only happens when linking a system hwloc (a system
  PMIx, or `HWLOC_DIR` set) — install its development package and point
  `HWLOC_DIR` at the install prefix.
- **Linker errors for `ibverbs`, `rdmacm`, `numa`, or `uuid`:** install the
  matching RDMA/NUMA/UUID development packages for the enabled transport.
- **Build still compiles PMIx/PRRTE from source despite `*_LIB_DIR` being
  set:** also set `PMIX_NO_VENDORED=1`/`PRRTE_NO_VENDORED=1` — the directory
  variables alone don't skip the vendored build.
- **Cluster-provided PMIx or PRRTE is incompatible:** use `system-pmix-launcher`
  (or restate the default list minus the vendoring feature for just the one
  piece that's incompatible) with the env vars above instead of the vendored
  default.

For implementation details and exact feature wiring, see
`lamellar-runtime/Cargo.toml` and the `README.md` files next to each `*-sys`
or `*-src` crate.
