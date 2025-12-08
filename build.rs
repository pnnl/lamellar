extern crate bindgen;
#[cfg(feature = "vendored")]
extern crate pmi_mpich_src;


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

fn find_pmi_normal() -> (std::path::PathBuf, std::path::PathBuf) {
    use std::path::PathBuf;

    let lib_dir = env_inner("PMI_LIB_DIR").map(PathBuf::from);
    let include_dir = env_inner("PMI_INCLUDE_DIR").map(PathBuf::from);

    if let (Some(lib_dir), Some(include_dir)) = (lib_dir, include_dir) {
        return (lib_dir, include_dir);
    }
    else {
        panic!("PMI_LIB_DIR and PMI_INCLUDE_DIR must be set to use a non-vendored PMI implementation");
    }
}

fn find_pmi() -> (std::path::PathBuf, std::path::PathBuf) {
    #[cfg(feature = "vendored")]
    {
        if env_inner("PMI_NO_VENDORED").map_or(true, |v| v == "0") {
            let artifacts =  pmi_mpich_src::Build::new().build(pmi_mpich_src::Protocol::V1);
            return (
                artifacts.lib_dir().to_path_buf(),
                artifacts.include_dir().to_path_buf(),
            );
        }
    }
    find_pmi_normal()
}

fn main(){
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", "build.rs");

    let artifacts = find_pmi();

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        .header(artifacts.1.join("pmi.h").to_str().unwrap())
        .clang_arg(format!("-I{}", artifacts.1.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Instruct cargo to rerun the build script if any of the relevant files change
    println!("cargo:rerun-if-changed=build.rs");

    // Link with the pmi to access its symbols. 
    println!("cargo:rustc-link-search={}", artifacts.0.display());

    println!("cargo:rustc-link-lib=pmi");
}
