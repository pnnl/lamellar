// The shared Open MPI libraries (features ompi-ucx / ompi-system) are found at run time through an rpath in the binary. A rustc-link-arg
// of the library crate shmem-shim does not reach the binary, so the install prefix it publishes (DEP_SHMEM_ROOT) is used here.
// tools/stage_runtime.py later copies the tree to builds/bin/runtime/ompi and rewrites this rpath, so no build-directory path stays.
fn main() {
    println!("cargo:rerun-if-env-changed=DEP_SHMEM_ROOT");
    if let Ok(root) = std::env::var("DEP_SHMEM_ROOT") {
        println!("cargo:rustc-link-search=native={root}/lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{root}/lib");
    }
}
