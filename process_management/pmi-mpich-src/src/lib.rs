use std::path::{Path, PathBuf};
use std::{env, fs};

/// Vendored MPICH-PMI release: upstream's standalone `libpmi` "make dist"
/// tarball (v5.0.0), NOT a full MPICH checkout. It ships with `configure`
/// (and `mpl/configure`) already generated, so no Flex/Autoconf/Automake/
/// Libtool/autogen.sh is required to build it. `configure.ac` (both the
/// top-level PMI package and the embedded `mpl` subpackage) carries a
/// Lamellar-added `AM_MAINTAINER_MODE` guard -- see `libpmi/configure.ac`.
const MPICH_PMI_VERSION: &str = "5.0.0";

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
            "default location baked in when pmi-mpich-src was compiled".to_string(),
        ),
    };
    if !dir.join("configure").is_file() {
        panic!(
            "pmi-mpich-src v{version}: vendored source tree not found or incomplete at {} ({}; no `configure` script there).\n\
             Likely causes:\n\
             \x20 - pmi-mpich-src was resolved from crates.io or a stale/moved registry copy instead of the Lamellar repo checkout; add the [patch.crates-io] block from lamellar-runtime/Cargo.toml to your top-level Cargo.toml and run `cargo update`.\n\
             \x20 - the source directory was moved or deleted after pmi-mpich-src was compiled; run `cargo clean -p pmi-mpich-src`.\n\
             To use a tree elsewhere, set {} to its path.",
            dir.display(), origin, env_var
        );
    }
    dir
}

pub fn source_dir() -> PathBuf {
    resolve_source_dir("LAMELLAR_PMI_MPICH_SRC_DIR", "libpmi", MPICH_PMI_VERSION)
}

pub fn version() -> &'static str {
    MPICH_PMI_VERSION
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

pub enum Protocol {
    V1,
    V2,
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

    pub fn build(&self, protocol: Protocol) -> Artifacts {
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let target = self.target.as_ref().expect("TARGET not set");

        let dest = out_dir.join("src");
        let src = source_dir();

        copy_rec(&src, &dest).expect(&format!("Failed to copy {} to {}", src.display(), dest.display()));
        
        let disable = match protocol {
            Protocol::V1 => "pmi2",
            Protocol::V2 => "pmi1",
        };

        let pmi_mpich_path = std::fs::canonicalize(dest).unwrap();

        // No autogen.sh / autoreconf here: the vendored tree came from
        // upstream's official standalone `libpmi` release tarball, which
        // ships pre-generated `configure` (and `mpl/configure`) -- that's
        // the whole point of vendoring the tarball instead of a git
        // checkout.
        std::process::Command::new("patch")
            .current_dir(pmi_mpich_path.as_path())
            .arg("-p0")
            .arg("-i")
            .arg(src.join("..").join("pmi.patch").to_str().unwrap())
            .status()
            .expect("Failed to apply patch for pmi_mpich");

        let mut pmi_mpich_build = autotools::Config::new(pmi_mpich_path.as_path());
        let pmi_mpich_build = pmi_mpich_build
            .disable(disable, None)
            .out_dir(out_dir)
            // .disable_static()
            // .enable_shared()
            .build();

        let include_dir = pmi_mpich_build.join("include");
        let lib_dir = pmi_mpich_build.join("lib");
        let bin_dir = pmi_mpich_build.join("bin");
        let lib_name = match protocol {
            Protocol::V1 => "libpmi.a",
            Protocol::V2 => "libpmi2.a",
        };
        

        fs::rename(lib_dir.join("libpmi.a"), lib_dir.join(lib_name)).expect("Failed to rename library");
        
        let artifact_name = match protocol {
            Protocol::V1 => "pmi",
            Protocol::V2 => "pmi2",
        };

        let libs = vec![
            artifact_name.to_string(),
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
