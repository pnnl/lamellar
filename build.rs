
extern crate bindgen;
fn main(){
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let lib_event_dir = std::env::var("DEP_EVENT_ROOT").expect("Couldn't find libevent");
    let libhwloc_dir = std::env::var("DEP_HWLOC_ROOT").expect("Couldn't find libevent");

    let pmix_path = std::fs::canonicalize(std::path::PathBuf::from("openpmix")).unwrap();

    println!("cargo:rerun-if-changed={}", "build.rs");
    std::process::Command::new("./autogen.pl")
        .current_dir(pmix_path.as_path())
        .status()
        .expect("Failed to autogen for PMIx");

    let mut pmix_build = autotools::Config::new(pmix_path.as_path());
    
    pmix_build.reconf("-ivf");
    pmix_build.disable_static();
    pmix_build.enable_shared();
    pmix_build.with("libevent", Some(&lib_event_dir));
    pmix_build.with("hwloc", Some(&libhwloc_dir));
    

    let pmix_build = pmix_build.build();

    let pmix_inc = pmix_build.join("include");
    let pmix_lib = pmix_build.join("lib");
    // let header_path = src_inc_path.join("/pmi2.h");
    // let lib_path = src_lib_path.join("/lib/");

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        .header(pmix_inc.join("pmix.h").to_str().unwrap())
        .clang_arg(format!("-I{}",pmix_inc.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link with the pmi to access its symbols. 
    println!("cargo:rustc-link-search={}", pmix_lib.to_str().unwrap());
    println!("cargo:rustc-link-search={}/lib", lib_event_dir);
    println!("cargo:rustc-link-search={}/lib", libhwloc_dir);

    println!("cargo:rustc-link-lib=pmix");
    println!("cargo:rustc-link-lib=static=event_core");
    println!("cargo:rustc-link-lib=static=event_pthreads");
    println!("cargo:rustc-link-lib=static=hwloc");
}
