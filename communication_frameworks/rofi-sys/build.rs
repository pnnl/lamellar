extern crate bindgen;
extern crate subprocess;

use autotools;
use glob::glob;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn build_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ofi_env = match env::var("OFI_DIR") {
        Ok(val) => std::path::PathBuf::from(val),
        Err(_) => {
            let dest = out_path.clone().join("ofi_src");
            Command::new("cp")
                .args(&["-r", "libfabric", &dest.to_string_lossy()])
                .status()
                .unwrap();

            let install_dest = autotools::Config::new(dest.clone())
                .reconf("-ivf")
                .disable("shared", None)
                .enable("only", None)
                .enable("verbs", None)
                .enable("atomics", None)
                .enable("rxm", None)
                .build();
            std::path::PathBuf::from(install_dest)
        }
    };
    let ofi_lib_dir = ofi_env.join("lib");
    let ofi_inc_dir = ofi_env.join("include");

    let rofi_env = match env::var("ROFI_DIR") {
        Ok(val) => std::path::PathBuf::from(val),
        Err(_) => {
            let dest = out_path.clone().join("rofi_src");
            Command::new("cp")
                .args(&["-r", "rofi", &dest.to_string_lossy()])
                .status()
                .unwrap();

            autotools::Config::new(dest)
                .reconf("-ivfWnone")
                .ldflag(
                    format! {"-L{} -libverbs -pthread -ldl -lrdmacm -lrt",ofi_lib_dir.display()},
                )
                .cflag(format! {"-I{}",ofi_inc_dir.display()})
                .build()
        }
    };
    for entry in glob("rofi/**/*.c").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("cargo:rerun-if-changed={}", path.display()),
            Err(_) => {}
        }
    }
    for entry in glob("rofi/**/*.h").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("cargo:rerun-if-changed={}", path.display()),
            Err(_) => {}
        }
    }

    let rofi_inc_dir = rofi_env.join("include");
    let rofi_lib_dir = rofi_env.join("lib");

    println!("cargo:rustc-link-search=native={}", rofi_lib_dir.display());
    println!("cargo:rustc-link-search=native={}", ofi_lib_dir.display());
    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-search=native=/usr/lib64/libibverbs");
    println!("cargo:rustc-link-lib=rofi");
    println!("cargo:rustc-link-lib=fabric");
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=pmi_simple");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include")
        .clang_arg(format! {"-I{}",ofi_inc_dir.display()})
        .clang_arg(format! {"-I{}/rdma",ofi_inc_dir.display()})
        .clang_arg(format!("-I{}", rofi_inc_dir.display()))
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
