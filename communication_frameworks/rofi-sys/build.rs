extern crate bindgen;
extern crate subprocess;

use std::env;
use std::path::PathBuf;

fn build_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ofi_env = env::var("OFI_DIR").unwrap();

    println!("cargo:rustc-link-search=native={}/lib", ofi_env);
    println!("cargo:rustc-link-lib=dylib=fabric");    
    
    let mpi_root = env::var("MPI_ROOT").unwrap();

    let ca_1 = "-I".to_string() + ofi_env.as_str() + "/include";
   
    let ca_2 = ca_1.to_string() + "/rdma";

    let rofi_inc_dir = "-I/lustre/lamellar/deps/rofi/include";

    let mpi_1 = mpi_root.as_str().to_owned() + "/lib";
    let mpi_2 = mpi_1.to_string() + "/openmpi";

    let mpi_i1 = "-I".to_string() + mpi_root.as_str() + "/include";
    let mpi_i2 = mpi_i1.to_string() + "/openmpi";

    let rofi_lib_dir = "/lustre/lamellar/deps/rofi/lib";

    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-search=native=/usr/lib64/libibverbs");

    println!("cargo:rustc-link-search=native={}", mpi_1);
    println!("cargo:rustc-link-search=native={}", mpi_2);

    println!("cargo:rustc-link-search=native={}", rofi_lib_dir);

    println!("cargo:rustc-link-lib=dylib=fabric");
    println!("cargo:rustc-link-lib=dylib=ibverbs");
    println!("cargo:rustc-link-lib=dylib=mpi");

    println!("cargo:rustc-link-lib=dylib=rofi");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include")
        .clang_arg(ca_1)
        .clang_arg(ca_2)
        .clang_arg(rofi_inc_dir)
        .clang_arg(mpi_i1)
        .clang_arg(mpi_i2)
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
