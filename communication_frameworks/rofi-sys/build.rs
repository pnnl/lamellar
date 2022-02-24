extern crate bindgen;
extern crate subprocess;

use autotools;
use std::env;
use std::path::PathBuf;
use glob::glob;

fn build_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ofi_env = match env::var("OFI_DIR"){
        Ok(val) => {
            println!("cargo:rustc-link-lib=dylib=fabric");
            std::path::PathBuf::from(val)
        },
        Err(_) => {
            panic!("Error: OFI_DIR not set. Please specify the root directory of your libfabrics installation.");
        }
    };
    let ofi_lib_dir = ofi_env.join("lib");
    let ofi_inc_dir = ofi_env.join("include");
    
    let rofi_env = match env::var("ROFI_DIR"){
        Ok(val) => {
            println!("cargo:rustc-link-lib=static=rofi");
            println!("cargo:rustc-link-lib=static=pmi_simple");
            std::path::PathBuf::from(val)
        },
        Err(_) => {
            println!("cargo:rustc-link-lib=static=pmi_simple");
            println!("cargo:rustc-link-lib=static=rofi");
           
            autotools::Config::new("rofi")
            .reconf("-ivfWnone")
            .ldflag(format!{"-L{}",ofi_lib_dir.display()})
            .cflag(format!{"-I{}",ofi_inc_dir.display()})
            .build()
        }
        
    };
    for entry in glob("rofi/**/*.c").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("cargo:rerun-if-changed={}", path.display()),
            Err(_) => {},
        }
    }
    for entry in glob("rofi/**/*.h").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("cargo:rerun-if-changed={}", path.display()),
            Err(_) => {},
        }
    }
    

    let rofi_inc_dir = rofi_env.join("include");
    let rofi_lib_dir = rofi_env.join("lib");

    println!("cargo:rustc-link-search=native={}", rofi_lib_dir.display());
    println!("cargo:rustc-link-search=native={}", ofi_lib_dir.display());

    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-search=native=/usr/lib64/libibverbs");
    println!("cargo:rustc-link-lib=dylib=ibverbs");
    

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include")
        .clang_arg(format!{"-I{}",ofi_inc_dir.display()})
        .clang_arg(format!{"-I{}/rdma",ofi_inc_dir.display()})
        .clang_arg(format!("-I{}",rofi_inc_dir.display()))
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
