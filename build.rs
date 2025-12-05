// mod create_inlined_wrappers;
// use autotools;
// use std::{
//     fs::{File, ReadDir},
//     io::{BufWriter, Write},
// };

// extern crate bindgen;
// fn iter_dir(
//     dir: ReadDir,
//     inlined_funcs: &mut Vec<String>,
//     writer: &mut BufWriter<File>,
//     writer_inlined: &mut BufWriter<File>,
// ) {
//     for file in dir {
//         if file.as_ref().unwrap().file_type().unwrap().is_file() {
//             // Create the wrappers (prototype, definition) for all inlined functions
//             let mut funcs = crate::create_inlined_wrappers::read_file(
//                 file.as_ref().unwrap().path().to_str().unwrap(),
//             );

//             // Store the prototype for later
//             inlined_funcs.append(&mut funcs.0);

//             // Write the definition to inlined.c
//             for f in funcs.1 {
//                 let _ = writer_inlined.write_all((f + "\n").as_bytes());
//             }

//             // #include the header to fabric_sys.h
//             let _ = writer.write_all(
//                 ("#include<".to_owned() + file.as_ref().unwrap().path().to_str().unwrap() + ">\n")
//                     .as_bytes(),
//             );
//         } else if file.as_ref().unwrap().file_type().unwrap().is_dir() {
//             iter_dir(
//                 std::fs::read_dir(file.unwrap().path()).unwrap(),
//                 inlined_funcs,
//                 writer,
//                 writer_inlined,
//             )
//         }
//     }
// }

// fn build_ofi() -> std::path::PathBuf {
//     match std::env::var("OFI_DIR") {
//         Ok(val) => {
//             println!("cargo:rerun-if-changed={}/lib", val);
//             println!("cargo:rerun-if-changed={}/include", val);
//             std::path::PathBuf::from(val)
//         }
//         Err(_) => {
//             let ofi_src_path =
//                 std::fs::canonicalize(std::path::PathBuf::from("libfabric")).unwrap();
//             println!(
//                 "cargo:rerun-if-changed={}",
//                 ofi_src_path.join("src").display()
//             );
//             println!(
//                 "cargo:rerun-if-changed={}",
//                 ofi_src_path.join("include").display()
//             );

//             #[cfg(not(feature = "shared"))]
//             let install_dest = autotools::Config::new(ofi_src_path)
//                 .reconf("-ivf")
//                 .disable_shared()
//                 .enable_static()
//                 .cflag("-g -O3")
//                 .cxxflag("-g -O3")
//                 .build();

//             #[cfg(feature = "shared")]
//             let install_dest = autotools::Config::new(ofi_src_path)
//                 .reconf("-ivf")
//                 .enable_shared()
//                 .disable_static()
//                 .cflag("-O3")
//                 .cxxflag("-O3")
//                 .build();

//             std::path::PathBuf::from(install_dest)
//         }
//     }
// }
extern crate libfabric_src;

fn main() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let artifacts = libfabric_src::Build::new().build();
    
    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        // .clang_arg(format!("-I{}", ofi_include_path.to_str().unwrap()))
        // .header(out_path.to_str().unwrap().to_owned() + "/fabric_sys.h")
        // .blocklist_function("qgcvt")
        // .blocklist_function("qgcvt_r")
        // .blocklist_function("qfcvt")
        // .blocklist_function("qfcvt_r")
        // .blocklist_function("qecvt")
        // .blocklist_function("qecvt_r")
        // .blocklist_function("strtold")
        .header(artifacts.include_dir().join("fabric_sys.h").to_str().unwrap())
        .clang_arg(format!("-I{}", artifacts.include_dir().to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // // Link with the libfabric and libinlined libraries to access their symbols.
    // println!("cargo:rustc-link-search={}", ofi_lib_path.display());
    // println!("cargo:rustc-link-search={}", out_path.display());

    // println!("cargo:rustc-link-lib=fabric");
    println!("cargo:rerun-if-changed={}", "build.rs");
    println!("cargo:rustc-link-search={}", artifacts.lib_dir().display());
    println!("cargo:rustc-link-lib=static=fabric");
    println!("cargo:rustc-link-lib=rt");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=atomic");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
    
    println!("cargo:rustc-link-lib=numa");
    println!("cargo:rustc-link-lib=uuid");
}
