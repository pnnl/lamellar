
extern crate bindgen;
fn main(){
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let src_inc_path = std::path::PathBuf::from(std::env::var("PMI_INC_DIR").expect("PMI not found. Please provide path to PMI include dir in \"PMI_INC_DIR\" environmental variable"));
    let src_lib_path = std::path::PathBuf::from(std::env::var("PMI_LIB_DIR").expect("PMI not found. Please provide path to PMI lib dir in \"PMI_LIB_DIR\" environmental variable"));

    // let header_path = src_inc_path.join("/pmi2.h");
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

    // Link with the pmi to access its symbols. 
    println!("cargo:rustc-link-search={}", src_lib_path.as_path().to_str().unwrap());
    println!("cargo:rustc-link-lib=pmi");
}
