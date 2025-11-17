fn main() {
    // let libevent_dir = std::env::var("LIBEVENT_DIR").map_or_else(|_|  None, |p| Some(p));
    // let lib_event_dir = std::env::var("DEP_EVENT_ROOT").expect("Couldn't find libevent");
    // let libhwloc_dir = std::env::var("DEP_HWLOC_ROOT").expect("Couldn't find libevent");
    // let pmix_dir = std::env::var("DEP_PMIX_ROOT").expect("Could not find DEP_PMIX_ROOT");

    println!("cargo:rerun-if-changed={}", "build.rs");
    let _artifacts = prrte_src::Build::new().build();

        
}