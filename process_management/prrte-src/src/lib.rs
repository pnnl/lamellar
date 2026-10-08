use std::path::{Path, PathBuf};
use std::env;

/// Vendored PRRTE release: the official "make dist" tarball from a PRRTE
/// GitHub release (v3.0.14), NOT a git checkout. It ships with `configure`
/// already generated, so no Flex/Autoconf/Automake/Libtool/autogen.pl is
/// required to build it. Its pre-built Sphinx docs bundle (docs/_build) has
/// been stripped out -- it's most of the tarball's size and unneeded since
/// we never install docs; `configure` detects its absence and skips doc
/// install cleanly (see `OAC_SETUP_SPHINX` in prrte/configure.ac).
const PRRTE_VERSION: &str = "3.0.14";

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("prrte")
}

pub fn version() -> &'static str {
    PRRTE_VERSION
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

/// The vendored tree is a PRRTE `make dist` release tarball whose pre-generated
/// Flex outputs (`*_lex.c` / `*_lex.h`) are shipped *newer* than their `*.l`
/// sources, so `make` never needs Flex to (re)generate them -- that is the
/// whole point of vendoring the release tarball instead of a git checkout.
/// But git checkout doesn't record mtimes and `copy_rec` (std::fs::copy)
/// doesn't preserve them, so after the copy a `.l` source can end up newer than
/// its generated `.c`. `make` then tries to rerun Flex -- failing outright when
/// Flex is absent, or forcing a Flex build dependency we vendored specifically
/// to avoid. Re-assert generated-newer-than-source under the freshly-copied
/// tree: push each `.l` into the past and its generated `.c`/`.h` to now.
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
        // None means: don't pass --with-libevent/--with-hwloc at all, and let PRRTE's own
        // ./configure fall through to its built-in pkg-config auto-detection (see
        // OAC_CHECK_PACKAGE in config/oac/oac_check_package.m4) instead of forcing an
        // explicit path.
        let lib_event_dir = if let Ok(dir) = std::env::var("LIBEVENT_DIR") {
            Some(dir)
        } else if let Ok(root_dir) = std::env::var("DEP_EVENT_ROOT") {
            Some(root_dir)
        } else if let Ok(include_str) = std::env::var("DEP_EVENT_INCLUDE") {
            let include_dir = std::path::Path::new(&include_str);
            Some(include_dir.parent().unwrap().display().to_string())
        } else {
            None
        };
        // DEP_PMIX_HWLOC_ROOT/DEP_PMIX_HWLOC_INCLUDE: openpmix-src vendor-builds hwloc
        // itself and pmix-sys re-exports that prefix under these keys, so PRRTE reuses
        // the exact same hwloc PMIx already linked against instead of needing its own
        // hwloc-src vendoring (PRRTE requires an external PMIx unconditionally anyway --
        // see libpmix_dir below -- so pmix-sys's build script always runs first).
        let libhwloc_dir = if let Ok(dir) = std::env::var("HWLOC_DIR") {
            Some(dir)
        } else if let Ok(root_dir) = std::env::var("DEP_HWLOC_ROOT") {
            Some(root_dir)
        } else if let Ok(include_str) = std::env::var("DEP_HWLOC_INCLUDE") {
            let include_dir = std::path::Path::new(&include_str);
            Some(include_dir.parent().unwrap().display().to_string())
        } else if let Ok(root_dir) = std::env::var("DEP_PMIX_HWLOC_ROOT") {
            Some(root_dir)
        } else if let Ok(include_str) = std::env::var("DEP_PMIX_HWLOC_INCLUDE") {
            let include_dir = std::path::Path::new(&include_str);
            Some(include_dir.parent().unwrap().display().to_string())
        } else {
            None
        };
        // None means: don't pass --with-pmix at all, and let PRRTE's own ./configure
        // fall through to its built-in pkg-config auto-detection (PRTE_CHECK_PMIX ->
        // OAC_CHECK_PACKAGE, config/prte_setup_pmix.m4) instead of forcing an explicit
        // path -- same fallthrough as libevent/hwloc above. PRRTE always requires some
        // pmix (--with-pmix=no is a hard configure error), it just doesn't have to be
        // this workspace's vendored pmix-sys/openpmix-src.
        let libpmix_dir = if let Ok(dir) = std::env::var("PMIX_DIR") {
            Some(dir)
        } else if let Ok(root_dir) = std::env::var("DEP_PMIX_ROOT") {
            Some(root_dir)
        } else {
            None
        };

        let dest = out_dir.join("src");
        let src = source_dir();

        copy_rec(&src, &dest).expect("Failed to copy source_dir() to OUT_DIR/src");
        retime_generated_lexers(&dest);

        let prrte_path = std::fs::canonicalize(dest).unwrap();

        // No autogen.pl / autoreconf here: the vendored tree came from the
        // official release tarball, which ships a pre-generated `configure`
        // -- that's the whole point of vendoring the tarball instead of a
        // git checkout.
        let mut prrte_config = autotools::Config::new(prrte_path.as_path());
        prrte_config.enable_static().disable_shared();
        if let Some(dir) = lib_event_dir.as_ref() {
            prrte_config.with("libevent", Some(dir));
        }
        if let Some(dir) = libhwloc_dir.as_ref() {
            prrte_config.with("hwloc", Some(dir));
        }
        if let Some(dir) = libpmix_dir.as_ref() {
            prrte_config.with("pmix", Some(dir));
        }
        let prrte_build = prrte_config.build();


        let include_dir = prrte_build.join("include");
        let lib_dir = prrte_build.join("lib");
        let bin_dir = prrte_build.join("bin");

        let libs = vec![
            "prrte".to_string(),
        ];

        Artifacts {
            include_dir,
            lib_dir,
            bin_dir,
            libs,
            target: target.to_string(),
        }

    }
}
