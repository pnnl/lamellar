use std::path::{Path, PathBuf};
use std::env;

/// Vendored OpenPMIx release: the official "make dist" tarball from an
/// OpenPMIx GitHub release (v5.0.9), NOT a git checkout. It ships with
/// `configure` already generated, so no Flex/Autoconf/Automake/Libtool/
/// autogen.pl is required to build it. Its pre-built Sphinx docs bundle
/// (docs/_build, docs/images) has been stripped out -- it's ~90% of the
/// tarball's size and unneeded since we never install docs; `configure`
/// detects its absence and skips doc install cleanly (see
/// `oac_install_docs` in openpmix/config/pmix.m4).
const PMIX_VERSION: &str = "5.0.9";

/// Vendored hwloc release tarball (official `hwloc-2.11.2.tar.gz`), same
/// rationale as PMIx/PRRTE: `configure` ships pre-generated so no autoreconf
/// is required. Both PRRTE and PMIx hard-require hwloc in the 1.11.0-2.x
/// window (checked directly in their `config/autogen_found_items.m4`), so we
/// build it once here and export the install prefix to both consumers
/// (PMIx directly, PRRTE transitively via pmix-sys's DEP_PMIX_HWLOC_ROOT).
const HWLOC_VERSION: &str = "2.11.2";

/// Resolve a vendored source tree. The environment variable `env_var` overrides the default location
/// (the tree shipped next to this crate's manifest, whose absolute path is
/// baked in at compile time). The result is checked for `configure` so a
/// missing or wrong tree fails with an actionable message instead of a bare
/// "No such file or directory".
fn resolve_source_dir(env_var: &str, subdir: &str, version: &str) -> PathBuf {
    println!("cargo:rerun-if-env-changed={}", env_var);
    let (dir, origin) = match env::var_os(env_var) {
        Some(v) if !v.is_empty() => (PathBuf::from(v), format!("{} environment variable", env_var)),
        _ => (
            Path::new(env!("CARGO_MANIFEST_DIR")).join(subdir),
            "default location baked in when openpmix-src was compiled".to_string(),
        ),
    };
    if !dir.join("configure").is_file() {
        panic!(
            "openpmix-src v{version}: vendored source tree not found or incomplete at {} ({}; no `configure` script there).\n\
             Likely causes:\n\
             \x20 - openpmix-src was resolved from crates.io or a stale/moved registry copy instead of the Lamellar repo checkout; add the [patch.crates-io] block from lamellar-runtime/Cargo.toml to your top-level Cargo.toml and run `cargo update`.\n\
             \x20 - the source directory was moved or deleted after openpmix-src was compiled; run `cargo clean -p openpmix-src`.\n\
             To use a tree elsewhere, set {} to its path.",
            dir.display(), origin, env_var
        );
    }
    dir
}

pub fn source_dir() -> PathBuf {
    resolve_source_dir("LAMELLAR_OPENPMIX_SRC_DIR", "openpmix", PMIX_VERSION)
}

pub fn hwloc_source_dir() -> PathBuf {
    resolve_source_dir("LAMELLAR_HWLOC_SRC_DIR", "hwloc", HWLOC_VERSION)
}

pub fn version() -> &'static str {
    PMIX_VERSION
}

pub fn hwloc_version() -> &'static str {
    HWLOC_VERSION
}

pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
    #[allow(dead_code)]
    host: Option<String>,
}


#[allow(dead_code)]
pub struct Artifacts {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    bin_dir: PathBuf,
    libs: Vec<String>,
    #[allow(dead_code)]
    target: String,
    hwloc_dir: Option<PathBuf>,
}



impl Artifacts {
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    /// Install prefix of the hwloc actually used to build PMIx -- either the
    /// vendored build (OUT_DIR/hwloc-install) or an external override
    /// (HWLOC_DIR/DEP_HWLOC_ROOT/DEP_HWLOC_INCLUDE). None only if PMIx's own
    /// configure was left to auto-detect hwloc via pkg-config.
    pub fn hwloc_dir(&self) -> Option<&Path> {
        self.hwloc_dir.as_deref()
    }
}


fn copy_rec(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        if dst.exists() && !dst.is_dir() {
            // If destination exists as a file, remove it so we can create a directory
            match std::fs::remove_file(dst) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(mut perms) = std::fs::metadata(dst).map(|m| m.permissions()) {
                            perms.set_mode(0o666);
                            let _ = std::fs::set_permissions(dst, perms);
                        }
                    }
                    // try again
                    let _ = std::fs::remove_file(dst);
                }
                Err(_) => {}
            }
        }
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            copy_rec(&src_path, &dst_path)?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if dst.exists() {
            // try to remove existing destination file first; if permission denied, attempt to relax permissions
            match std::fs::remove_file(dst) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(mut perms) = std::fs::metadata(dst).map(|m| m.permissions()) {
                            perms.set_mode(0o666);
                            let _ = std::fs::set_permissions(dst, perms);
                        }
                    }
                    let _ = std::fs::remove_file(dst);
                }
                Err(_) => {}
            }
        }
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

/// The vendored tree is an OpenPMIx `make dist` release tarball whose
/// pre-generated Flex outputs (`*_lex.c` / `*_lex.h`) are shipped *newer* than
/// their `*.l` sources, so `make` never needs Flex to (re)generate them -- that
/// is the whole point of vendoring the release tarball instead of a git
/// checkout. But git checkout doesn't record mtimes and `copy_rec`
/// (std::fs::copy) doesn't preserve them, so after the copy a `.l` source can
/// end up newer than its generated `.c`. `make` then tries to rerun Flex --
/// failing outright when Flex is absent, or forcing a Flex build dependency we
/// vendored specifically to avoid. Re-assert generated-newer-than-source under
/// the freshly-copied tree: push each `.l` into the past and its generated
/// `.c`/`.h` to now.
fn retime_generated_lexers(root: &Path) {
    fn set_mtime(path: &Path, t: std::time::SystemTime) {
        if let Ok(f) = std::fs::OpenOptions::new().write(true).open(path) {
            let _ = f.set_modified(t);
        }
    }
    fn walk(dir: &Path, now: std::time::SystemTime, older: std::time::SystemTime) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, now, older);
            } else if path.extension().map_or(false, |e| e == "l") {
                set_mtime(&path, older);
                for ext in ["c", "h"] {
                    let generated = path.with_extension(ext);
                    if generated.is_file() {
                        set_mtime(&generated, now);
                    }
                }
            }
        }
    }
    let now = std::time::SystemTime::now();
    let older = now - std::time::Duration::from_secs(60);
    walk(root, now, older);
}

/// Vendor-build hwloc into OUT_DIR/hwloc-install and return its install prefix.
/// hwloc's release tarball ships no `.l` flex sources (unlike PRRTE/PMIx), so
/// no lexer retiming is needed here. It DOES need the same class of fix as
/// `retime_generated_lexers`, though, for `configure`/`Makefile.in` vs their
/// `configure.ac`/`aclocal.m4`/`*.m4`/`Makefile.am` sources: `copy_rec`
/// (std::fs::copy) doesn't preserve mtimes, so after copying, `configure.ac`
/// can end up newer than the pre-generated `configure` purely due to
/// readdir-order luck. Automake's standard "missing"-script auto-remake rule
/// then tries to regenerate `configure` for real with whatever autoconf is
/// installed on the build machine -- silently discarding every hand-patch
/// made to the committed `configure` (this is exactly how the doc/tests/
/// utils/contrib trim patches kept reverting themselves whenever autoconf
/// happened to be installed). Re-assert generated-newer-than-source the same
/// way retime_generated_lexers does for Flex.
fn retime_generated_autotools(root: &Path) {
    fn set_mtime(path: &Path, t: std::time::SystemTime) {
        if let Ok(f) = std::fs::OpenOptions::new().write(true).open(path) {
            let _ = f.set_modified(t);
        }
    }
    fn walk(dir: &Path, now: std::time::SystemTime, older: std::time::SystemTime) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, now, older);
                continue;
            }
            match path.file_name().and_then(|n| n.to_str()) {
                Some("configure.ac") | Some("configure.in") | Some("aclocal.m4")
                | Some("Makefile.am") => set_mtime(&path, older),
                Some(name) if name.ends_with(".m4") => set_mtime(&path, older),
                Some("configure") | Some("Makefile.in") | Some("config.h.in") => {
                    set_mtime(&path, now)
                }
                _ => {}
            }
        }
    }
    let now = std::time::SystemTime::now();
    let older = now - std::time::Duration::from_secs(60);
    walk(root, now, older);
}

fn build_hwloc(out_dir: &Path) -> PathBuf {
    let dest = out_dir.join("hwloc-src");
    let src = hwloc_source_dir();
    copy_rec(&src, &dest).expect("Failed to copy hwloc_source_dir() to OUT_DIR/hwloc-src");
    retime_generated_autotools(&dest);
    let hwloc_path = std::fs::canonicalize(dest).unwrap();

    let hwloc_install = out_dir.join("hwloc-install");
    std::fs::create_dir_all(&hwloc_install).expect("Failed to create hwloc-install dir");
    let mut hwloc_build = autotools::Config::new(hwloc_path.as_path());
    hwloc_build
        .out_dir(&hwloc_install)
        .enable_static()
        .disable_shared()
        // Rust binaries link PIE by default; libtool skips -fPIC object
        // compilation for a static-only (--disable-shared) build unless
        // explicitly told otherwise, which breaks the final PIE link
        // (R_X86_64_32/32S relocations against non-PIC objects).
        .with("pic", None)
        .disable("io", None)
        .disable("libudev", None)
        .disable("cairo", None)
        .disable("cuda", None)
        .disable("opencl", None)
        .disable("nvml", None)
        .disable("libxml2", None);
    // Lamellar patch (not upstream): this vendored copy ships without doc/,
    // tests/, utils/, or contrib/ (see configure's own patched
    // ac_config_files calls, and the patched doc/-existence check that
    // used to force --enable-embedded-mode) to avoid shipping ~19MB of
    // doxygen docs and an unused test suite. --enable-embedded-mode itself
    // is NOT used here: it changes `make install` to expect textual
    // source embedding into a consumer's own build rather than a
    // standalone configure+make+install into its own prefix, which is
    // what this function actually does (then hands the resulting prefix
    // to PMIx's own --with-hwloc) -- embedded mode broke hwloc.pc/header
    // installation for that usage.
    hwloc_build.build()
}

impl Build {
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| PathBuf::from(s)),
            target: env::var("TARGET").ok(),
            host: env::var("HOST").ok(),
        }
    }

    pub fn build(&self) -> Artifacts {
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let target = self.target.as_ref().expect("TARGET not set");
        // None means: don't pass --with-libevent/--with-hwloc at all, and let PMIx's own
        // ./configure fall through to its built-in pkg-config auto-detection (OAC_CHECK_PACKAGE)
        // instead of forcing an explicit path.
        // libevent-sys emits cargo:include= but never cargo:root=, so DEP_EVENT_ROOT
        // alone is always unset -- derive the prefix from DEP_EVENT_INCLUDE's parent
        // instead (mirrors prrte-src's equivalent lookup).
        let lib_event_dir = std::env::var("LIBEVENT_DIR")
            .or_else(|_| std::env::var("DEP_EVENT_ROOT"))
            .ok()
            .or_else(|| {
                std::env::var("DEP_EVENT_INCLUDE").ok().map(|include_str| {
                    std::path::Path::new(&include_str)
                        .parent()
                        .unwrap()
                        .display()
                        .to_string()
                })
            });
        let libhwloc_dir = std::env::var("HWLOC_DIR")
            .or_else(|_| std::env::var("DEP_HWLOC_ROOT"))
            .ok()
            .or_else(|| {
                std::env::var("DEP_HWLOC_INCLUDE").ok().map(|include_str| {
                    std::path::Path::new(&include_str)
                        .parent()
                        .unwrap()
                        .display()
                        .to_string()
                })
            });
        // No override given: vendor-build hwloc ourselves so PMIx (and, via
        // pmix-sys's DEP_PMIX_HWLOC_ROOT, PRRTE) always has one that satisfies
        // their shared 1.11.0-2.x version requirement.
        let libhwloc_dir = libhwloc_dir.or_else(|| {
            Some(build_hwloc(out_dir).display().to_string())
        });

        let dest = out_dir.join("src");
        let src = source_dir();

        copy_rec(&src, &dest).expect("Failed to copy source_dir() to OUT_DIR/src");
        retime_generated_lexers(&dest);

        let pmix_path = std::fs::canonicalize(dest).unwrap();

        // No autogen.pl / autoreconf here: the vendored tree came from the
        // official release tarball, which ships a pre-generated `configure`
        // -- that's the whole point of vendoring the tarball instead of a
        // git checkout.
        let mut pmix_build = autotools::Config::new(pmix_path.as_path());
        pmix_build
            .out_dir(out_dir)
            .enable_static()
            .disable_shared()
            // Same PIC requirement as hwloc above -- see that comment.
            .with("pic", None)
            // Library only: skip building/installing the pmix_info/plookup/
            // pps/pattrs/pquery/pevent/wrapper CLI tools and the test/example
            // programs. Neither is needed to link against libpmix.
            .disable("pmix-binaries", None)
            .with("tests-examples", Some("no"))
            // munge: optional PSEC credential component, auto-detected via
            // pkg-config if present on the build machine (native uid/gid
            // check is the fallback). A shared libpmix.so would record it
            // as DT_NEEDED and the dynamic linker resolves it transitively;
            // a static libpmix.a does not, so an unrelated consumer binary
            // fails to link with undefined munge_* symbols unless it
            // explicitly adds -lmunge itself. Disabled explicitly instead,
            // since the vendored static build's whole point is not
            // depending on what happens to be installed on the build
            // machine. zlib is left auto-detected (optional modex/topology
            // compression) -- pmix-sys's build.rs adds the corresponding
            // -lz explicitly for the static-link case.
            .without("munge", None);
        if let Some(dir) = lib_event_dir.as_ref() {
            pmix_build.with("libevent", Some(dir));
        }
        if let Some(dir) = libhwloc_dir.as_ref() {
            pmix_build.with("hwloc", Some(dir));
        }
        let pmix_build = pmix_build.build();

        let include_dir = pmix_build.join("include");
        let lib_dir = pmix_build.join("lib");
        let bin_dir = pmix_build.join("bin");

        let libs = vec![
            "pmix".to_string(),
        ];

        Artifacts {
            include_dir,
            lib_dir,
            bin_dir,
            libs,
            target: target.to_string(),
            hwloc_dir: libhwloc_dir.map(PathBuf::from),
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_vendored_source() {
        let tmp = std::env::temp_dir().join("openpmix-src-vendor-test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let dest = tmp.join("src");
        copy_rec(&source_dir(), &dest).unwrap();

        assert!(dest.join("configure").is_file());
        assert!(dest.join("src").join("mca").is_dir());
        // docs/ and test/ are trimmed from the vendored tree
        assert!(!dest.join("docs").exists());
        assert!(!dest.join("test").exists());
        assert!(dest.join("contrib").join("Makefile.in").is_file());
        assert!(dest.join("bindings").join("python").join("Makefile.in").is_file());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
