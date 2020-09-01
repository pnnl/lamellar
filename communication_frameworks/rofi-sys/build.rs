extern crate bindgen;
extern crate subprocess;

use std::env;
use std::path::PathBuf;

fn build_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ofi_env = env::var("OFI_DIR").expect("The OFI_DIR environment variable must be specified with the location of the OFI installation");
    assert!(ofi_env.len() > 0,"The OFI_DIR environment variable must be specified with the location of the OFI installation");

    let rofi_env = env::var("ROFI_DIR").expect("The ROFI_DIR environment variable must be specified with the location of the Rust-OFI installation");
    assert!(rofi_env.len() > 0,"The ROFI_DIR environment variable must be specified with the location of the OFI installation");

    println!("cargo:rustc-link-lib=dylib=fabric");

    let ca_1 = "-I".to_string() + ofi_env.as_str() + "/include";
    let ca_2 = ca_1.to_string() + "/rdma";

    let ofi_lib_dir = ofi_env.as_str().to_owned() + "/lib";

    let rofi_inc_dir = "-I".to_string() + rofi_env.as_str() + "/include";
    let rofi_lib_dir = rofi_env + "/lib";
    println!("cargo:rustc-link-search=native={}", rofi_lib_dir);
    println!("cargo:rustc-link-search=native={}", ofi_lib_dir);

    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-search=native=/usr/lib64/libibverbs");


    println!("cargo:rustc-link-lib=dylib=fabric");
    println!("cargo:rustc-link-lib=dylib=ibverbs");

    println!("cargo:rustc-link-lib=dylib=rofi");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include")
        .clang_arg(ca_1)
        .clang_arg(ca_2)
        .clang_arg(rofi_inc_dir)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to src/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    build_bindings();
}
