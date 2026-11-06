use std::env;
use std::path::{Path, PathBuf};

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("SOS")
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
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
    if !src.ends_with(".git")
        && !src.ends_with(".github")
        && !src.ends_with(".gz")
        && !src.ends_with(".ci")
    {
        if src.is_dir() {
            // check if dst already exists and is a directory
            if !dst.exists() {
                std::fs::create_dir_all(dst)?;
            }
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
            std::fs::copy(src, dst)?;
        }
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

    fn with_pmi(&self, config: &mut autotools::Config) {
        println!("cargo:warn=Locating PMIx installation...");
        if let Ok(pmix_dir) = env::var("DEP_PMIX_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", pmix_dir);
            println!("cargo:rerun-if-changed={}/include", pmix_dir);
            config.with("pmix", Some(&pmix_dir.to_string()));
        } else {
            println!("cargo:error=PMIx not found via DEP_PMIX_ROOT");
        }
        self.with_hwloc(config);
    }

    // SOS's own configure has an independent --with-hwloc option (it doesn't
    // pick up hwloc transitively from --with-pmix). Reuse the hwloc that
    // pmix-sys's vendored openpmix-src build already built (and re-exports as
    // DEP_PMIX_HWLOC_ROOT), rather than requiring a system hwloc install or
    // relying on UCX's own (hwloc-less) static libs to satisfy SOS's link
    // check for ucp_cleanup.
    fn with_hwloc(&self, config: &mut autotools::Config) {
        if let Ok(hwloc_dir) = env::var("DEP_PMIX_HWLOC_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", hwloc_dir);
            println!("cargo:rerun-if-changed={}/include", hwloc_dir);
            config.with("hwloc", Some(&hwloc_dir));
        } else if let Ok(hwloc_dir) = env::var("HWLOC_DIR") {
            config.with("hwloc", Some(&hwloc_dir));
        }
    }

    #[cfg(feature = "ofi")]
    fn with_ofi(&self, config: &mut autotools::Config) {
        println!("cargo:warn=Configuring with OFI support...");
        if let Ok(ofi_dir) = env::var("DEP_OFI_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", ofi_dir);
            println!("cargo:rerun-if-changed={}/include", ofi_dir);
            // libfabric is linked statically (libfabric-sys builds libfabric.a, symbol versioning off)
            println!("cargo:rustc-link-lib=static=fabric");
            println!("cargo:rustc-link-search={}/lib", ofi_dir);
            // What the static libfabric still needs (libfabric.pc Libs.private), after it on the link line. libfabric-sys emits the same
            // flags, but shmem-shim never references that crate from Rust code, so rustc does not link it and its flags are lost.
            for lib in ["rt", "numa", "ibverbs", "uuid", "rdmacm", "atomic"] {
                println!("cargo:rustc-link-lib={}", lib);
            }
            config
                .with("ofi", Some(ofi_dir.as_str()))
                .enable("hard-polling", None)
                .enable("ofi-mr", Some("basic"));
        } else {
            println!("cargo:error=DEP_LIBFABRIC_ROOT not set");
        }
    }

    #[cfg(feature = "ucx")]
    fn with_ucx(&self, config: &mut autotools::Config) {
        println!("cargo:warn=Configuring with UCX support...");
        if let Ok(ucx_dir) = env::var("DEP_UCX_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", ucx_dir);
            println!("cargo:rerun-if-changed={}/include", ucx_dir);
            println!("cargo:rustc-link-search={}/lib", ucx_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", ucx_dir);
            config.with("ucx", Some(ucx_dir.as_str()));

            // UCX registers its UCT transports (ib, ib_mlx5, rdmacm, cma,
            // knem) and UCP protocols entirely through constructor
            // attributes (UCS_STATIC_INIT / UCT_COMPONENT_REGISTER) --
            // nothing else references those objects by symbol. In a shared
            // UCX build these live as separate lib/ucx/libuct_*.so plugins
            // that ucs dlopen()s at startup; this vendored UCX is
            // --disable-shared, so those modules exist only as
            // lib/ucx/libuct_*.a archives that no dlopen path can ever
            // load, and a plain static link drops their unreferenced
            // objects entirely (silently falling back to tcp/self-only
            // transports). Force every member of ucm/ucs/uct/ucp and each
            // present transport module into SOS's own link with
            // --whole-archive so their constructors are present and
            // actually register -- the same fix lamellar-ucx-sys/build.rs
            // already applies for hamr-lamellar's UCX path.
            let ucx_lib_dir = std::path::Path::new(&ucx_dir).join("lib");
            let ucx_module_dir = ucx_lib_dir.join("ucx");
            // libucm/libucs/libuct/libucp themselves live in the parent lib/
            // dir, not lib/ucx/ (that subdir holds only the transport
            // modules) -- the pre-existing cargo:rustc-link-search line above
            // only affects the final Rust crate's own link, not SOS's
            // separately-scoped LDFLAGS, so both dirs need an explicit -L
            // here too.
            config.ldflag(format!("-L{}", ucx_lib_dir.display()));
            config.ldflag(format!("-L{}", ucx_module_dir.display()));
            config.ldflag("-Wl,--whole-archive");
            for lib in ["ucm", "ucs", "uct", "ucp"] {
                config.ldflag(format!("-l{lib}"));
            }
            for module in ["uct_ib", "uct_ib_mlx5", "uct_rdmacm", "uct_cma", "uct_knem"] {
                if ucx_module_dir.join(format!("lib{module}.a")).exists() {
                    config.ldflag(format!("-l{module}"));
                }
            }
            config.ldflag("-Wl,--no-whole-archive");
            if ucx_module_dir.join("libuct_ib_mlx5.a").exists() {
                config.ldflag("-lmlx5");
            }
            // Forcing every object out of the archives above (--whole-archive)
            // pulls in references to these system libs immediately -- even
            // during configure's own "does the C compiler work" sanity check,
            // which links a trivial program using only this LDFLAGS (SOS's
            // own configure.ac only adds these later, once it actually probes
            // for UCX, which is too late for that first check). List them
            // here unconditionally so every link against this LDFLAGS, trivial
            // or not, resolves.
            config.ldflag("-libverbs");
            config.ldflag("-lrdmacm");
            config.ldflag("-lpthread");
            config.ldflag("-ldl");
            config.ldflag("-lrt");
            config.ldflag("-lm");

            // libucm.a and libucs.a reference each other's symbols (circular),
            // which a one-pass static link can't resolve in either archive
            // order. Open a group and deliberately never close it -- GNU ld
            // treats an unmatched --start-group as extending to the end of
            // the command line, so it covers the -lucm/-lucs SOS appends
            // later in LIBS without needing to control SOS's own configure.ac.
            config.ldflag("-Wl,--start-group");
        } else {
            println!("cargo:error=DEP_UCX_ROOT not set");
        }
    }

    fn build_sos(&self, sos_dir: Option<&PathBuf>) -> std::path::PathBuf {
        println!("cargo:warn=Building SOS from source. This may take a while...");
        match std::env::var("SOS_DIR") {
            Ok(val) => {
                println!("cargo:rerun-if-changed={}/lib", val);
                println!("cargo:rerun-if-changed={}/include", val);
                std::path::PathBuf::from(val)
            }
            Err(_) => {
                let sos_src_path = std::fs::canonicalize(sos_dir.unwrap()).unwrap();

                match std::process::Command::new("./autogen.sh")
                    .current_dir(&sos_src_path)
                    .status()
                {
                    Ok(status) if status.success() => {
                        println!("cargo:warn=autogen.sh completed successfully")
                    }
                    Ok(status) => panic!("autogen.sh failed with exit code: {}", status),
                    Err(e) => panic!("Failed to execute autogen.sh: {}", e),
                }
                let mut config = autotools::Config::new(sos_src_path);
                config
                    .out_dir(self.out_dir.as_ref().unwrap())
                    .cflag("-O3")
                    .cxxflag("-O3")
                    .disable("fortran", None)
                    .disable("error-checking", None)
                    .disable("threads", None)
                    // Build SOS as a static archive (libsma.a), not a
                    // shared library. As a .so, every UCX function SOS
                    // whole-archives in (ucs_topo_find_device_by_bus_id,
                    // ucs_topo_init, ...) is GLOBAL DEFAULT in .dynsym --
                    // intra-library calls to them go through PLT/GOT
                    // indirection that a plain static executable link
                    // (e.g. lamellar-ucx-sys's, which is known-working
                    // against the identical archive) never has to do.
                    // Rust binaries link PIE by default; libtool skips
                    // -fPIC object compilation for a static-only
                    // (--disable-shared) build unless told otherwise,
                    // which breaks the final PIE link (same reason
                    // lamellar-ucx-sys/build.rs sets --with-pic for its
                    // own vendored UCX build).
                    .disable_shared()
                    .enable_static()
                    .with("pic", None);

                self.with_pmi(&mut config);

                #[cfg(all(feature = "ucx", feature = "ofi"))]
                {
                    println!(
                        "cargo:error=Cannot enable both UCX and OFI features at the same time"
                    );
                    std::process::exit(1);
                }
                #[cfg(not(any(feature = "ucx", feature = "ofi")))]
                {
                    println!("cargo:error=Must select at least one of UCX or OFI features");
                    std::process::exit(1);
                }

                #[cfg(feature = "ofi")]
                self.with_ofi(&mut config);

                #[cfg(feature = "ucx")]
                self.with_ucx(&mut config);

                let install_dest = config.build();

                std::path::PathBuf::from(install_dest)
            }
        }
    }

    pub fn build(&self) -> Artifacts {
        if let Ok(project_dir) = env::var("CARGO_MANIFEST_DIR") {
            println!("cargo:warn=CARGO_MANIFEST_DIR={:?}", project_dir);
        }
        if let Ok(out_dir) = env::var("OUT_DIR") {
            println!("cargo:warn=OUT_DIR={:?}", out_dir);
        }
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let target = self.target.as_ref().expect("TARGET not set");

        for env_var in std::env::vars() {
            if env_var.0.starts_with("DEP_") {
                println!("cargo:warn={:?}", env_var);
                println!("cargo:rerun-if-changed={}", env_var.1);
            }
        }

        let dest = out_dir.join("sos_src");
        let src = source_dir();

        println!(
            "cargo:warn=Building SOS from source. This may take a while... {} (source: {})",
            dest.display(),
            src.display()
        );

        copy_rec(&src, &dest).expect("Failed to copy source_dir() to OUT_DIR/src");

        let libfabric_path = std::fs::canonicalize(dest).unwrap();
        let libfabric_build = self.build_sos(Some(&libfabric_path));

        let include_dir = libfabric_build.join("include");
        let lib_dir = libfabric_build.join("lib");
        let bin_dir = libfabric_build.join("bin");

    
        let libs = vec!["sma".to_string()];

        Artifacts {
            include_dir,
            lib_dir,
            bin_dir,
            libs,
            target: target.to_string(),
        }
    }
}
