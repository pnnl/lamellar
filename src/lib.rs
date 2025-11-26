mod create_inlined_wrappers;
use std::fs::{File, ReadDir};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::env;

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("libfabric")
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

                #[cfg(not(feature = "shared"))]
                let install_dest = autotools::Config::new(ofi_src_path)
                    .reconf("-ivf")
                    .disable_shared()
                    .enable_static()
                    .cflag("-O3")
                    .cxxflag("-O3")
                    .build();

                #[cfg(feature = "shared")]
                let install_dest = autotools::Config::new(ofi_src_path)
                    .reconf("-ivf")
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
            if file.as_ref().unwrap().file_type().unwrap().is_file() {
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
            } else if file.as_ref().unwrap().file_type().unwrap().is_dir() {
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
        let lib_dir = libfabric_build.join("lib");
        let bin_dir = libfabric_build.join("bin");
        let ofi_include_rdma_path = libfabric_build.join("include/rdma/");
        
        // libfabric has multiple header file that we are required to generate bindings for so we create a
        // new header named fabric_sys.h and #include all headers there
        let header_file =
            std::fs::File::create(include_dir.join("fabric_sys.h")).unwrap();
        let mut writer = std::io::BufWriter::new(header_file);
        
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
            target: target.to_string(),
        }

    }
}
