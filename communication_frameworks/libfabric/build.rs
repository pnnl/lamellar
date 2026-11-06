// Turns the libfabric major version exported by lamellar-libfabric-sys (DEP_OFI_MAJOR) into `cfg(ofi2)` for libfabric 2.x
// and newer, whose C API dropped or moved several 1.x items.
fn main() {
    println!("cargo:rustc-check-cfg=cfg(ofi2)");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DEP_OFI_MAJOR");
    if let Ok(major) = std::env::var("DEP_OFI_MAJOR") {
        if major.parse::<u32>().map_or(false, |m| m >= 2) {
            println!("cargo:rustc-cfg=ofi2");
        }
    }
}
