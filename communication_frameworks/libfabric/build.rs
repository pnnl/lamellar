// libfabric's C API differs between releases (fields added or moved, constants removed). lamellar-libfabric-sys probes the
// bindings it generated and reports which optional items exist (DEP_OFI_API_CFGS, with every possible name in
// DEP_OFI_ALL_API_CFGS); this turns them into `cfg(ofi_*)` for the code in this crate.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DEP_OFI_API_CFGS");
    let list = |var: &str| -> Vec<String> {
        std::env::var(var)
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    };
    for cfg in list("DEP_OFI_ALL_API_CFGS") {
        println!("cargo:rustc-check-cfg=cfg({})", cfg);
    }
    for cfg in list("DEP_OFI_API_CFGS") {
        println!("cargo:rustc-cfg={}", cfg);
    }
}
