use std::env;
use std::path::{Path, PathBuf};

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("ompi")
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
    // Path::ends_with compares whole path components, so a file-name suffix such as ".gz" needs the file name itself
    let skip = src.file_name().map_or(false, |n| {
        let n = n.to_string_lossy();
        n == ".git" || n == ".github" || n == ".ci" || n.ends_with(".gz")
    });
    if !skip {
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
    }

    fn with_hwloc(&self, config: &mut autotools::Config) {
        println!("cargo:warn=Locating hwloc installation...");
        if let Ok(hwloc_dir) = env::var("DEP_HWLOC_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", hwloc_dir);
            println!("cargo:rerun-if-changed={}/include", hwloc_dir);
            println!("cargo:rustc-link-search={}/lib", hwloc_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", hwloc_dir);
            config.with("hwloc", Some(&hwloc_dir.to_string()));
        } else {
            println!("cargo:error=hwloc not found via DEP_HWLOC_ROOT");
        }
    }

    fn with_libevent(&self, config: &mut autotools::Config) {
        println!("cargo:warn=Locating libevent installation...");
        if let Ok(event_dir) = env::var("DEP_EVENT_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", event_dir);
            println!("cargo:rerun-if-changed={}/include", event_dir);
            println!("cargo:rustc-link-search={}/lib", event_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", event_dir);
            config.with("libevent", Some(&event_dir.to_string()));
        } else {
            println!("cargo:error=libevent not found via DEP_LIBEVENT_ROOT");
        }
    }

    fn with_prrte(&self, config: &mut autotools::Config) {
        println!("cargo:warn=Locating PRRTE installation...");
        if let Ok(prrte_dir) = env::var("DEP_PRRTE_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", prrte_dir);
            println!("cargo:rerun-if-changed={}/include", prrte_dir);
            println!("cargo:rustc-link-search={}/lib", prrte_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", prrte_dir);
            config.with("prrte", Some(&prrte_dir.to_string()));
        } else {
            println!("cargo:error=PRRTE not found via DEP_PRRTE_ROOT");
        }
    }

    #[cfg(feature = "ofi")]
    fn with_ofi(&self, config: &mut autotools::Config) {
        println!("cargo:warn=Configuring with OFI support...");
        if let Ok(ofi_dir) = env::var("DEP_OFI_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", ofi_dir);
            println!("cargo:rerun-if-changed={}/include", ofi_dir);
            println!("cargo:rustc-link-lib=dylib=fabric");
            println!("cargo:rustc-link-search={}/lib", ofi_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", ofi_dir);
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
        println!("cargo:warn=Configuring ompi with UCX support...");
        if let Ok(ucx_dir) = env::var("DEP_UCX_ROOT") {
            println!("cargo:rerun-if-changed={}/lib", ucx_dir);
            println!("cargo:rerun-if-changed={}/include", ucx_dir);
            println!("cargo:rustc-link-search={}/lib", ucx_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", ucx_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", ucx_dir);
            config.with("ucx", Some(ucx_dir.as_str()));
            // The vendored UCX is a set of static archives. configure's `ucp_cleanup` link test only passes
            // -lucp -luct -lucm -lucs, so the archives' own dependencies (pthread spinlocks, dlopen, the 16-byte
            // atomics, verbs) and their mutual references have to come from LIBS, grouped so order does not matter.
            let mut libs = String::from(
                "-Wl,--start-group -lucp -luct -lucm -lucs -Wl,--end-group \
                 -latomic -lpthread -ldl -lrt -lz -lnuma -lmlx5 -libverbs -lrdmacm -lm -lutil",
            );
            if let Ok(user) = env::var("LIBS") {
                libs = format!("{user} {libs}");
            }
            config.env("LIBS", libs);
        } else {
            println!("cargo:error=DEP_UCX_ROOT not set");
        }
    }

    fn build_ompi(&self, ompi_dir: Option<&PathBuf>) -> std::path::PathBuf {
        println!("cargo:warn=Building OMPI from source. This may take a while...");
        match std::env::var("OMPI_DIR") {
            Ok(val) => {
                println!("cargo:rerun-if-changed={}/lib", val);
                println!("cargo:rerun-if-changed={}/include", val);
                std::path::PathBuf::from(val)
            }
            Err(_) => {
                let ompi_src_path = std::fs::canonicalize(ompi_dir.unwrap()).unwrap();

                match std::process::Command::new("./autogen.pl")
                    .arg("--no-3rdparty")
                    .arg("openpmix,prrte,hwloc,libevent,pympistandard")
                    .arg("--exclude")
                    .arg("ompi")
                    .current_dir(&ompi_src_path)
                    .status()
                {
                    Ok(status) if status.success() => {
                        println!("cargo:warn=autogen.pl completed successfully")
                    }
                    Ok(status) => panic!("autogen.pl failed with exit code: {}", status),
                    Err(e) => panic!("Failed to execute autogen.pl: {}", e),
                }
                let mut config = autotools::Config::new(ompi_src_path);
                config
                    .out_dir(self.out_dir.as_ref().unwrap())
                    .cflag("-O3")
                    .cxxflag("-O3")
                    
                    .disable("mpi-fortran", None)
                    .enable("mpi-fortran", Some("no"))
                    .disable("mpi", None)
                    .enable("mpi", Some("no"))
                    .disable("oshmem-fortran", None)
                    .enable("oshmem-fortran", Some("no"))
                    .enable_shared()
                    .disable_static();

                self.with_pmi(&mut config);
                self.with_hwloc(&mut config);
                self.with_libevent(&mut config);
                self.with_prrte(&mut config);

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

        let dest = out_dir.join("ompi_src");
        let src = source_dir();

        println!(
            "cargo:warn=Building OMPI from source. This may take a while... {} (source: {})",
            dest.display(),
            src.display()
        );

        copy_rec(&src, &dest).expect("Failed to copy source_dir() to OUT_DIR/src");

        let libfabric_path = std::fs::canonicalize(dest).unwrap();
        let libfabric_build = self.build_ompi(Some(&libfabric_path));

        let include_dir = libfabric_build.join("include");
        let lib_dir = libfabric_build.join("lib");
        let bin_dir = libfabric_build.join("bin");

        let libs = vec!["oshmem".to_string()];

        Artifacts {
            include_dir,
            lib_dir,
            bin_dir,
            libs,
            target: target.to_string(),
        }
    }
}
