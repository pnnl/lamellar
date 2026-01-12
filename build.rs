extern crate bindgen;
#[cfg(feature = "vendored")]
extern crate openpmix_src;


fn env_inner(name: &str) -> Option<String> {
    let var = std::env::var(name);
    println!("cargo:rerun-if-env-changed={name}");

    match var {
        Ok(v) => {
            println!("{} = {}", name, v);
            Some(v)
        }
        Err(_) => {
            println!("{name} unset");
            None
        }
    }
}

fn find_pmix_normal(out_path: &std::path::PathBuf) -> (std::path::PathBuf, std::path::PathBuf) {
    use std::path::PathBuf;

    let lib_dir = env_inner("PMIX_LIB_DIR").map(PathBuf::from);
    let include_dir = env_inner("PMIX_INCLUDE_DIR").map(PathBuf::from);

    if let (Some(lib_dir), Some(include_dir)) = (lib_dir, include_dir) {
        match std::os::unix::fs::symlink(&lib_dir, out_path.join("lib")) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    std::fs::remove_file(out_path.join("lib")).unwrap();
                    std::os::unix::fs::symlink(&lib_dir, out_path.join("lib")).unwrap();
                }
            }
        }
        match std::os::unix::fs::symlink(&include_dir, out_path.join("include")) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    std::fs::remove_file(out_path.join("include")).unwrap();
                    std::os::unix::fs::symlink(&include_dir, out_path.join("include")).unwrap();
                }
            }
        }
        println!("cargo:root={}", out_path.display());
        return (out_path.join("lib"), out_path.join("include"));
    }
    else {
        panic!("PMIX_LIB_DIR and PMIX_INCLUDE_DIR must be set to use a non-vendored PMI implementation");
    }
}

fn find_pmix(out_path: &std::path::PathBuf) -> (std::path::PathBuf, std::path::PathBuf) {
    #[cfg(feature = "vendored")]
    {
        if env_inner("PMIX_NO_VENDORED").map_or(true, |v| v == "0") {
            let lib_event_dir = std::env::var("DEP_EVENT_ROOT").expect("Couldn't find libevent");
            let libhwloc_dir = std::env::var("DEP_HWLOC_ROOT").expect("Couldn't find libhwloc");
            let artifacts =  openpmix_src::Build::new().build();
            println!("cargo:rustc-link-search={}/lib", lib_event_dir);
            println!("cargo:rustc-link-search={}/lib", libhwloc_dir);
            println!("cargo:rustc-link-lib=static=event_core");
            println!("cargo:rustc-link-lib=static=event_pthreads");
            println!("cargo:rustc-link-lib=static=hwloc");
            return (
                artifacts.lib_dir().to_path_buf(),
                artifacts.include_dir().to_path_buf(),
            );
        }
    }
    find_pmix_normal(out_path)
}


fn main(){
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", "build.rs");
    let artifacts = find_pmix(&out_path);

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        .header(artifacts.1.join("pmix.h").to_str().unwrap())
        .clang_arg(format!("-I{}", artifacts.1.to_str().unwrap()))
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

    // Link with the pmix to access its symbols. 
    println!("cargo:rustc-link-search={}", artifacts.0.display());

    println!("cargo:rustc-link-lib=pmix");

}
