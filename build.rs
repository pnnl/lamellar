extern crate libfabric_src;

fn main() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let artifacts = libfabric_src::Build::new().build();
    
    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        .header(artifacts.include_dir().join("fabric_sys.h").to_str().unwrap())
        .clang_arg(format!("-I{}", artifacts.include_dir().to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed={}", "build.rs");
    println!("cargo:rustc-link-search={}", artifacts.lib_dir().display());
    println!("cargo:rustc-link-lib=static=fabric");
    println!("cargo:rustc-link-lib=rt");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=atomic");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
}