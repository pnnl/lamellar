use std::os::unix::fs;


extern crate bindgen;
fn main(){
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    
    let src_inc_path = if out_path.join("pmi2.h").exists() {
            out_path.clone().join("pmi2.h")
        } 
        else { 
            let path = std::path::PathBuf::from(std::env::var("PMI2_INC_DIR").expect("PMI2 not found. Please provide path to PMI2 include dir in \"PMI2_INC_DIR\" environmental variable"));
            if path.join("pmi2.h").exists() {
                fs::symlink(path.join("pmi2.h"), out_path.join("pmi2.h")).unwrap();
            }
            else {
                panic!("Path {} does not exist.", path.join("pmi2.h").to_str().unwrap())
            }
            
            out_path.clone().join("pmi2.h")
        };
    let src_lib_path = if out_path.join("libpmi2.so").exists() {
            out_path.clone()
        } else {
            let path = std::path::PathBuf::from(std::env::var("PMI2_LIB_DIR").expect("PMI2 not found. Please provide path to PMI2 lib dir in \"PMI2_LIB_DIR\" environmental variable"));
            if path.join("libpmi2.so").exists() {
                fs::symlink(path.join("libpmi2.so"), out_path.join("libpmi2.so")).unwrap();
            }            
            else {
                panic!("Path {} does not exist.", path.join("libpmi2.so").to_str().unwrap() )
            }
            
            out_path.clone()
        };

    // let header_path = src_inc_path.join("/pmi22.h");
    // let lib_path = src_lib_path.join("/lib/");

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        // .clang_arg(format!("-I{}",header_pathlude/"))
        .header(src_inc_path.as_path().to_str().unwrap())
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link with the pmi2 to access its symbols. 
    println!("cargo:rustc-link-search={}", src_lib_path.as_path().to_str().unwrap());
    println!("cargo:rustc-link-lib=pmi2");
}
