extern crate bindgen;
extern crate openpmix_src;

fn main(){
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let lib_event_dir = std::env::var("DEP_EVENT_ROOT").expect("Couldn't find libevent");
    let libhwloc_dir = std::env::var("DEP_HWLOC_ROOT").expect("Couldn't find libhwloc");

    // let pmix_path = std::fs::canonicalize(std::path::PathBuf::from("openpmix")).unwrap();

    println!("cargo:rerun-if-changed={}", "build.rs");
    // std::process::Command::new("./autogen.pl")
    //     .current_dir(pmix_path.as_path())
    //     .status()
    //     .expect("Failed to autogen for PMIx");

    // let mut pmix_build = autotools::Config::new(pmix_path.as_path());
    
    // pmix_build.reconf("-ivf");
    // pmix_build.disable_static();
    // pmix_build.enable_shared();
    // pmix_build.with("libevent", Some(&lib_event_dir));
    // pmix_build.with("hwloc", Some(&libhwloc_dir));
    
    let artifacts = openpmix_src::Build::new().build();
    // let pmix_build = pmix_build.build();

    // let pmix_inc = artifacts.join("include");
    // let pmix_lib = artifacts.join("lib");
    // let header_path = src_inc_path.join("/pmi2.h");
    // let lib_path = src_lib_path.join("/lib/");

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        // .header(src_inc_path.as_path().to_str().unwrap().to_string() + "/pmix.h")
        // .clang_arg(format!("-I{}", src_inc_path.as_path().to_str().unwrap()))
        // .blocklist_function("qgcvt")
        // .blocklist_function("qgcvt_r")
        // .blocklist_function("qfcvt")
        // .blocklist_function("qfcvt_r")
        // .blocklist_function("qecvt")
        // .blocklist_function("qecvt_r")
        // .blocklist_function("strtold")
        .header(artifacts.include_dir().join("pmix.h").to_str().unwrap())
        .clang_arg(format!("-I{}", artifacts.include_dir().to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // // Link with the pmi to access its symbols.
    // println!(
    //     "cargo:rustc-link-search={}",
    //     src_lib_path.as_path().to_str().unwrap()
    // );
    // Instruct cargo to rerun the build script if any of the relevant files change
    println!("cargo:rerun-if-changed=build.rs");

    // Link with the pmi to access its symbols. 
    println!("cargo:rustc-link-search={}", artifacts.lib_dir().display());
    println!("cargo:rustc-link-search={}/lib", lib_event_dir);
    println!("cargo:rustc-link-search={}/lib", libhwloc_dir);

    println!("cargo:rustc-link-lib=pmix");
    println!("cargo:rustc-link-lib=static=event_core");
    println!("cargo:rustc-link-lib=static=event_pthreads");
    println!("cargo:rustc-link-lib=static=hwloc");
}
