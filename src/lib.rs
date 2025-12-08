use std::path::{Path, PathBuf};
use std::{env, fs};

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("mpich")
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
        std::process::Command::new("./autogen.sh")
                .current_dir(pmi_mpich_path.as_path())
                .arg("--with-pmi")
                .arg("--without-hydra")
                .arg("--without-romi")
                .arg("--without-hwloc")
                .arg("--without-ucx")
                .arg("--without-ofi")
                .arg("--without-json")
                .arg("--without-yaksa")
                .arg("--without-test")
                .arg("--without-fortran")
                .arg("--without-f77")
                .arg("--without-f08")
                .status()
                .expect("Failed to autogen for pmi_mpich");

        std::process::Command::new("patch")
            .current_dir(pmi_mpich_path.as_path())
            .arg("-p1")
            .arg("-i")
            .arg(src.join("..").join("pmi.patch").to_str().unwrap())
            .status()
            .expect("Failed to apply patch for pmi_mpich");

        let mut pmi_mpich_build = autotools::Config::new(pmi_mpich_path.join("src").join("pmi").as_path());
        let pmi_mpich_build = pmi_mpich_build.reconf("-ivf")
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
