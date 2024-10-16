mod create_inlined_wrappers;
use autotools;
use std::{
    fs::{File, ReadDir},
    io::{BufWriter, Write},
};

extern crate bindgen;
fn iter_dir(
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
            iter_dir(
                std::fs::read_dir(file.unwrap().path()).unwrap(),
                inlined_funcs,
                writer,
                writer_inlined,
            )
        }
    }
}

fn build_ofi() -> std::path::PathBuf {
    match std::env::var("OFI_DIR") {
        Ok(val) => {
            println!("cargo:rerun-if-changed={}/lib", val);
            println!("cargo:rerun-if-changed={}/include", val);
            std::path::PathBuf::from(val)
        }
        Err(_) => {
            let ofi_src_path =
                std::fs::canonicalize(std::path::PathBuf::from("libfabric")).unwrap();
            println!(
                "cargo:rerun-if-changed={}",
                ofi_src_path.join("src").display()
            );
            println!(
                "cargo:rerun-if-changed={}",
                ofi_src_path.join("include").display()
            );

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

fn main() {
    let build_path = build_ofi();
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let inlined_file_path = out_path.join("inlined.c");
    let ofi_include_path = build_path.join("include/");
    let ofi_lib_path = build_path.join("lib/");
    let ofi_include_rdma_path = build_path.join("include/rdma/");

    // libfabric has multiple header file that we are required to generate bindings for so we create a
    // new header named fabric_sys.h and #include all headers there
    let header_file =
        std::fs::File::create(out_path.to_str().unwrap().to_owned() + "/fabric_sys.h").unwrap();
    let mut writer = std::io::BufWriter::new(header_file);

    // Another problem is that there are several static inline functions in these header files which do
    // not create symbols in the libfabric.so and thus cannot be linked from rust.
    // For this reason, we create wrapper functions for each inline one and store them in a file called inlined.c
    // which resides in the /build/ directory
    let inlined_file = std::fs::File::create(inlined_file_path.clone()).unwrap();
    let mut writer_inlined = std::io::BufWriter::new(inlined_file);
    let _ = writer_inlined.write_all(b"#include<fabric_sys.h>\n");

    // We keep the prototypes of all the wrappers fo rinline functions here and will append them to fabric_sys.h
    // so that they are visible to bindgen
    let mut inlined_funcs: Vec<String> = Vec::new();

    let headers = std::fs::read_dir(ofi_include_rdma_path).unwrap();

    // For each file in libfabric/include/
    iter_dir(
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
        .include(ofi_include_path.clone())
        .include(out_path.clone())
        .compile("inlined");

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", ofi_include_path.to_str().unwrap()))
        .header(out_path.to_str().unwrap().to_owned() + "/fabric_sys.h")
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link with the libfabric and libinlined libraries to access their symbols.
    println!("cargo:rustc-link-search={}", ofi_lib_path.display());
    println!("cargo:rustc-link-search={}", out_path.display());

    println!("cargo:rustc-link-lib=static=fabric");
    println!("cargo:rustc-link-lib=rt");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=atomic");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
}
