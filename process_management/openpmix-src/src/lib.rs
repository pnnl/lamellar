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

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("openpmix")
}

pub fn version() -> &'static str {
    PMIX_VERSION
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
        let lib_event_dir = std::env::var("LIBEVENT_DIR")
            .or_else(|_| std::env::var("DEP_EVENT_ROOT"))
            .expect("Couldn't find libevent: set LIBEVENT_DIR to a system libevent install prefix, or enable the vendored-libevent feature");
        let libhwloc_dir = std::env::var("HWLOC_DIR")
            .or_else(|_| std::env::var("DEP_HWLOC_ROOT"))
            .expect("Couldn't find libhwloc: set HWLOC_DIR to a system hwloc install prefix, or enable the vendored-hwloc feature");

        let dest = out_dir.join("src");
        let src = source_dir();

        copy_rec(&src, &dest).expect("Failed to copy source_dir() to OUT_DIR/src");

        let pmix_path = std::fs::canonicalize(dest).unwrap();

        // No autogen.pl / autoreconf here: the vendored tree came from the
        // official release tarball, which ships a pre-generated `configure`
        // -- that's the whole point of vendoring the tarball instead of a
        // git checkout.
        let mut pmix_build = autotools::Config::new(pmix_path.as_path());
        let pmix_build = pmix_build
            .out_dir(out_dir)
            .disable_static()
            .enable_shared()
            // Library only: skip building/installing the pmix_info/plookup/
            // pps/pattrs/pquery/pevent/wrapper CLI tools and the test/example
            // programs. Neither is needed to link against libpmix.
            .disable("pmix-binaries", None)
            .with("tests-examples", Some("no"))
            .with("libevent", Some(&lib_event_dir))
            .with("hwloc", Some(&libhwloc_dir))
            .build();

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
        assert!(!dest.join("docs").join("_build").exists());
        assert!(!dest.join("docs").join("images").exists());
        assert!(dest.join("docs").join("Makefile.in").is_file());
        assert!(dest.join("contrib").join("Makefile.in").is_file());
        assert!(dest.join("test").join("Makefile.in").is_file());
        assert!(dest.join("bindings").join("python").join("Makefile.in").is_file());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
