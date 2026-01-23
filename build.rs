#[cfg(feature = "vendored")]
extern crate prrte_src;

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

fn find_prrte_normal(out_path: &std::path::PathBuf) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    use std::path::PathBuf;

    let lib_dir = env_inner("PRRTE_LIB_DIR").map(PathBuf::from);
    let include_dir = env_inner("PRRTE_INCLUDE_DIR").map(PathBuf::from);
    let bin_dir = env_inner("PRRTE_BIN_DIR").map(PathBuf::from);

    if let (Some(lib_dir), Some(include_dir), Some(bin_dir)) = (lib_dir, include_dir, bin_dir) {
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
        match std::os::unix::fs::symlink(&bin_dir, out_path.join("bin")) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    std::fs::remove_file(out_path.join("bin")).unwrap();
                    std::os::unix::fs::symlink(&bin_dir, out_path.join("bin")).unwrap();
                }
            }
        }
        println!("cargo:root={}", out_path.display());
        return (out_path.join("lib"), out_path.join("include"), out_path.join("bin"));
    }
    else {
        panic!("PRRTE_LIB_DIR and PRRTE_INCLUDE_DIR must be set to use a non-vendored PRRTE implementation");
    }
}

fn find_prrte(out_path: &std::path::PathBuf) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    #[cfg(feature = "vendored-prrte")]
    {
        if env_inner("PRRTE_NO_VENDORED").map_or(true, |v| v == "0") {
            let artifacts = prrte_src::Build::new().build();
            return (
                artifacts.lib_dir().to_path_buf(), 
                artifacts.include_dir().to_path_buf(), 
                artifacts.bin_dir().to_path_buf(),
            );
        }
    }
    find_prrte_normal(out_path)
}

fn main() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", "build.rs");
    let artifacts = find_prrte(&out_path);

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        .header(artifacts.1.join("prte.h").to_str().unwrap())
        .clang_arg(format!("-I{}", artifacts.1.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link with the pmi to access its symbols. 
    println!("cargo:rustc-link-search={}", artifacts.0.display());

    println!("cargo:rustc-link-lib=prrte");
}