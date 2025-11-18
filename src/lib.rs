use std::path::{Path, PathBuf};
use std::env;

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("prrte")
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
    if src.is_dir() {
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
        let lib_event_dir = std::env::var("DEP_EVENT_ROOT").expect("Couldn't find libevent");
        let libhwloc_dir = std::env::var("DEP_HWLOC_ROOT").expect("Couldn't find libhwloc");
        let libpmix_dir = std::env::var("DEP_PMIX_ROOT").expect("Couldn't find libpmix");

        let dest = out_dir.join("src");
        let src = source_dir();

        copy_rec(&src, &dest).expect("Failed to copy source_dir() to OUT_DIR/src");

        let prrte_path = std::fs::canonicalize(dest).unwrap();
        std::process::Command::new("./autogen.pl")
                .current_dir(prrte_path.as_path())
                .status()
                .expect("Failed to autogen for prrte");
        
        let prrte_build = autotools::Config::new(prrte_path.as_path())
                .enable_static()
                .disable_shared()
                .with("libevent", Some(&lib_event_dir))
                .with("hwloc", Some(&libhwloc_dir))
                .with("pmix", Some(&libpmix_dir))
                .build();


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
