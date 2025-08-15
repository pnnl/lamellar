extern crate bindgen;
fn main() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let src_inc_path = std::path::PathBuf::from(std::env::var("PMIX_INC_DIR").expect("PMI2 not found. Please provide path to PMIX include dir in \"PMIX_INC_DIR\" environmental variable"));
    let src_lib_path = std::path::PathBuf::from(std::env::var("PMIX_LIB_DIR").expect("PMIX not found. Please provide path to PMIX lib dir in \"PMIX_LIB_DIR\" environmental variable"));

    // let header_path = src_inc_path.join("/pmi2.h");
    // let lib_path = src_lib_path.join("/lib/");

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        .header(src_inc_path.as_path().to_str().unwrap().to_string() + "/pmix.h")
        .clang_arg(format!("-I{}", src_inc_path.as_path().to_str().unwrap()))
        .blocklist_function("qgcvt")
        .blocklist_function("qgcvt_r")
        .blocklist_function("qfcvt")
        .blocklist_function("qfcvt_r")
        .blocklist_function("qecvt")
        .blocklist_function("qecvt_r")
        .blocklist_function("strtold")
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link with the pmi to access its symbols.
    println!(
        "cargo:rustc-link-search={}",
        src_lib_path.as_path().to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=pmix");
}
