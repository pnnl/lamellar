mod create_inlined_wrappers;
use std::fs::{File, ReadDir};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::env;

/// Vendored libfabric release: the official "make dist" tarball from a
/// libfabric GitHub release (v1.22.0), NOT a git checkout. It ships with
/// `configure` already generated, so no Flex/Autoconf/Automake/Libtool/
/// autogen.sh is required to build it. Unlike PMIx/PRRTE's Sphinx docs, the
/// tarball's `man/` pages are NOT stripped -- they're wired as an
/// unconditional prerequisite of the generated `install` target with no
/// configure-time opt-out, so removing them would break `make install`.
const LIBFABRIC_VERSION: &str = "1.22.0";

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
            "default location baked in when lamellar-libfabric-src was compiled".to_string(),
        ),
    };
    if !dir.join("configure").is_file() {
        panic!(
            "lamellar-libfabric-src v{version}: vendored source tree not found or incomplete at {} ({}; no `configure` script there).\n\
             Likely causes:\n\
             \x20 - lamellar-libfabric-src was resolved from crates.io or a stale/moved registry copy instead of the Lamellar repo checkout; add the [patch.crates-io] block from lamellar-runtime/Cargo.toml to your top-level Cargo.toml and run `cargo update`.\n\
             \x20 - the source directory was moved or deleted after lamellar-libfabric-src was compiled; run `cargo clean -p lamellar-libfabric-src`.\n\
             To use a tree elsewhere, set {} to its path.",
            dir.display(), origin, env_var
        );
    }
    dir
}

pub fn source_dir() -> PathBuf {
    resolve_source_dir("LAMELLAR_LIBFABRIC_SRC_DIR", "libfabric", LIBFABRIC_VERSION)
}

pub fn version() -> &'static str {
    LIBFABRIC_VERSION
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
    generated_header: PathBuf,
    shared: bool,
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

    /// Whether to link `libfabric.so` rather than `libfabric.a`: the `shared` feature, or an `OFI_DIR` install that
    /// provides only a shared library (distro and vendor packages often ship no static archive, or one that is not
    /// built with -fPIC and cannot be linked into a PIE or cdylib).
    pub fn shared(&self) -> bool {
        self.shared
    }

    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    /// Path to the generated `fabric_sys.h` (and its inlined-function wrappers),
    /// which lives under `OUT_DIR` rather than `include_dir` so a system
    /// `OFI_DIR` install (potentially read-only) is never written into.
    pub fn generated_header(&self) -> &Path {
        &self.generated_header
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

    fn build_ofi(&self, ofi_dir: Option<&PathBuf>) -> std::path::PathBuf {
        match std::env::var("OFI_DIR") {
            Ok(val) => {
                println!("cargo:rerun-if-changed={}/lib", val);
                println!("cargo:rerun-if-changed={}/include", val);
                std::path::PathBuf::from(val)
            }
            Err(_) => {
                let ofi_src_path =
                    std::fs::canonicalize(ofi_dir.unwrap()).unwrap();

                // No autogen.sh / autoreconf here: the vendored tree came from the
                // official release tarball, which ships a pre-generated `configure`
                // -- that's the whole point of vendoring the tarball instead of a
                // git checkout. configure.ac carries a Lamellar-added
                // AM_MAINTAINER_MODE guard (see libfabric/configure.ac), so
                // Automake's timestamp-triggered regen rules stay off by default
                // regardless of mtimes lost during copy_rec/git checkout.
                // Static build: --enable-embedded turns off ELF symbol versioning (the `@@FABRIC_x.y` tags that
                // `.symver` puts on fi_getinfo etc.). With the tags, the objects cannot be linked into a cdylib: the
                // version script rustc generates for a cdylib has no FABRIC_* nodes and the linker reports
                // "symbol fi_getinfo@@FABRIC_1.7 has undefined version". Embedded mode builds the library as an
                // uninstalled convenience archive, so `make install` does not provide libfabric.a; it is copied from
                // the build tree below.
                #[cfg(not(feature = "shared"))]
                let install_dest = {
                    let dest = autotools::Config::new(ofi_src_path)
                        .disable_shared()
                        .enable_static()
                        .enable("embedded", None)
                        .cflag("-O3")
                        .cxxflag("-O3")
                        .build();
                    let lib_dir = dest.join("lib");
                    std::fs::create_dir_all(&lib_dir).unwrap();
                    std::fs::copy(
                        dest.join("build/src/.libs/libfabric.a"),
                        lib_dir.join("libfabric.a"),
                    )
                    .expect("embedded libfabric.a not found in the build tree");
                    dest
                };

                #[cfg(feature = "shared")]
                let install_dest = autotools::Config::new(ofi_src_path)
                    .enable_shared()
                    .disable_static()
                    .cflag("-O3")
                    .cxxflag("-O3")
                    .build();

                std::path::PathBuf::from(install_dest)
            }
        }
    }

    fn iter_dir(
        &self,
        dir: ReadDir,
        inlined_funcs: &mut Vec<String>,
        writer: &mut BufWriter<File>,
        writer_inlined: &mut BufWriter<File>,
    ) {
        for file in dir {
            // Only libfabric's own headers (fabric.h, fi_*.h, providers/). A system OFI_DIR such as /usr also holds
            // rdma-core's headers in rdma/ (rdma_cma.h, rdma_user_cm.h, *-abi.h, ...), which redefine each other when
            // they are all included in one translation unit and are not part of the libfabric API.
            let name = file.as_ref().unwrap().file_name().to_string_lossy().into_owned();
            // metadata() follows symlinks (DirEntry::file_type() does not), for installs that link their headers in
            let is_file = std::fs::metadata(file.as_ref().unwrap().path()).map_or(false, |m| m.is_file());
            let is_libfabric_header = name == "fabric.h" || (name.starts_with("fi_") && name.ends_with(".h"));
            if (is_file && !is_libfabric_header) || (!is_file && name != "providers") {
                continue;
            }
            if is_file {
                // Create the wrappers (prototype, definition) for all inlined functions
                let mut funcs = crate::create_inlined_wrappers::read_file(
                    file.as_ref().unwrap().path().to_str().unwrap(),
                );

                // Store the prototype for later
                inlined_funcs.append(&mut funcs.0);

                // Write the definition to inlined.c
                for f in funcs.1 {
                    let _ = writer_inlined.write_all((f + "\n").as_bytes());
                }

                // #include the header to fabric_sys.h
                let _ = writer.write_all(
                    ("#include<".to_owned() + file.as_ref().unwrap().path().to_str().unwrap() + ">\n")
                        .as_bytes(),
                );
            } else {
                // the filter above lets only the providers/ directory through
                self.iter_dir(
                    std::fs::read_dir(file.unwrap().path()).unwrap(),
                    inlined_funcs,
                    writer,
                    writer_inlined,
                )
            }
        }
}

    pub fn build(&self) -> Artifacts {
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let inlined_file_path = out_dir.join("inlined.c");
        let target = self.target.as_ref().expect("TARGET not set");

        let dest = out_dir.join("src");
        let src = source_dir();

        copy_rec(&src, &dest).expect("Failed to copy source_dir() to OUT_DIR/src");

        let libfabric_path = std::fs::canonicalize(dest).unwrap();
        let libfabric_build = self.build_ofi(Some(&libfabric_path));

        let include_dir = libfabric_build.join("include");
        let mut lib_dir = libfabric_build.join("lib");
        let has_lib = |dir: &Path| dir.join("libfabric.a").exists() || dir.join("libfabric.so").exists();
        let using_ofi_dir = std::env::var_os("OFI_DIR").is_some();
        // RHEL-style system installs (OFI_DIR=/usr) keep libraries in lib64
        if using_ofi_dir && !has_lib(&lib_dir) && has_lib(&libfabric_build.join("lib64")) {
            lib_dir = libfabric_build.join("lib64");
        }
        // A system libfabric (OFI_DIR) is linked shared whenever it has libfabric.so. Its libfabric.a, if any, has the
        // providers compiled in (verbs, and on Slingshot or AWS also cxi and efa), and a static link then needs every
        // library those providers use (libcxi, libefa, ...) and a -fPIC build; the shared library carries those
        // dependencies itself. This is the same policy as UCX_DIR and UCC_DIR. OFI_LINK=static|shared overrides it.
        println!("cargo:rerun-if-env-changed=OFI_LINK");
        let link_pref = std::env::var("OFI_LINK").ok();
        let archive = lib_dir.join("libfabric.a");
        let so_exists = lib_dir.join("libfabric.so").exists();
        let shared = cfg!(feature = "shared")
            || link_pref.as_deref() == Some("shared")
            || (using_ofi_dir && so_exists && link_pref.as_deref() != Some("static"));
        if shared && !cfg!(feature = "shared") && using_ofi_dir {
            println!(
                "cargo:warning=linking the shared libfabric.so from {}{}. Set OFI_LINK=static to link libfabric.a instead.",
                lib_dir.display(),
                if archive.exists() { " (a libfabric.a is there too, but a static link needs the libraries of every provider compiled into it)" } else { " (there is no libfabric.a)" }
            );
        }
        let bin_dir = libfabric_build.join("bin");
        let ofi_include_rdma_path = libfabric_build.join("include/rdma/");

        // libfabric has multiple header file that we are required to generate bindings for so we create a
        // new header named fabric_sys.h and #include all headers there. This is written into
        // OUT_DIR (not include_dir) since include_dir may be a system OFI_DIR install that we
        // shouldn't write into (and may not even be writable).
        let generated_header = out_dir.join("fabric_sys.h");
        let header_file =
            std::fs::File::create(&generated_header).unwrap();
        let mut writer = std::io::BufWriter::new(header_file);
        // Newer system libfabric (2.x, e.g. HPE Slingshot installs) has rdma/fi_profile.h use container_of()
        // before any header in this list defines it, so provide the same fallback fabric.h uses.
        writer
            .write_all(
                b"#include <stddef.h>\n\
                  #ifndef container_of\n\
                  #define container_of(ptr, type, field) ((type *) ((char *)(ptr) - offsetof(type, field)))\n\
                  #endif\n\
                  #ifndef count_of\n\
                  #define count_of(x) ((sizeof(x)/sizeof(0[x])) / ((size_t)(!(sizeof(x) % sizeof(0[x])))))\n\
                  #endif\n",
            )
            .unwrap();

        // Another problem is that there are several static inline functions in these header files which do
        // not create symbols in the libfabric.so and thus cannot be linked from rust.
        // For this reason, we create wrapper functions for each inline one and store them in a file called inlined.c
        // which resides in the /build/ directory
        let inlined_file = std::fs::File::create(inlined_file_path.clone()).unwrap();
        let mut writer_inlined = std::io::BufWriter::new(inlined_file);
        let _ = writer_inlined.write_all(b"#include<fabric_sys.h>\n");
    
        // We keep the prototypes of all the wrappers for inline functions here and will append them to fabric_sys.h
        // so that they are visible to bindgen
        let mut inlined_funcs: Vec<String> = Vec::new();
        let headers = std::fs::read_dir(ofi_include_rdma_path).unwrap();

        // For each file in libfabric/include/
        self.iter_dir(
            headers,
            &mut inlined_funcs,
            &mut writer,
            &mut writer_inlined,
        );


        // Append the prototypes of the wrappers for the inline functions to fabric_sys.h
        for f in inlined_funcs {
            let _ = writer.write_all((f + "\n").as_bytes());
        }

        // Make sure the files are written completely
        let _ = writer.flush();
        let _ = writer_inlined.flush();

        // Create a new lib, libinlined.a that we expose the wrappers for rust to use
        cc::Build::new()
            .warnings(false)
            .flag("-w") // Silence warnings
            .file(inlined_file_path.clone())
            .opt_level(3)
            .include(include_dir.clone())
            .include(out_dir.clone())
            .compile("inlined");

        let libs = vec![
            "libfabric".to_string(),
        ];

        Artifacts {
            include_dir,
            lib_dir,
            bin_dir,
            libs,
            generated_header,
            shared,
            target: target.to_string(),
        }

    }
}
