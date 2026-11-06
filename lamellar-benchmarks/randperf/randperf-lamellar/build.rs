// use std::env;
// use libfabric_src;

fn main() {
    // if let Ok(project_dir) = std::env::var("CARGO_MANIFEST_DIR") {
    //     println!("cargo:warning=CARGO_MANIFEST_DIR={:?}", project_dir);
    // }
    // if let Ok(out_dir) = std::env::var("OUT_DIR") {
    //     println!("cargo:warning=OUT_DIR={:?}", out_dir);
    // }

    for env_var in std::env::vars() {
        // println!("cargo:warning={:?}={:?}", env_var.0, env_var.1);
        if env_var.0.starts_with("DEP_") {
            
            println!("cargo:rustc-link-search={}/lib", env_var.1);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", env_var.1);
        }
    }
    // if let Ok(dep_shmem_root) = std::env::var("DEP_SHMEM_ROOT") {
    //     println!("cargo:warn=DEP_SHMEM_ROOT={:?}", dep_shmem_root);
    //     // print!("cargo:rustc-link-lib=sma\n");
    //     println!("cargo:rustc-link-search={}/lib", dep_shmem_root);
    //     println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", dep_shmem_root);
    // }

    // libfabric build/linking disabled for now. To re-enable, uncomment
}
