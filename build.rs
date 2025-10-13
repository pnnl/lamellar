fn main() {
    // let libevent_dir = std::env::var("LIBEVENT_DIR").map_or_else(|_|  None, |p| Some(p));
    let lib_event_dir = std::env::var("DEP_EVENT_ROOT").expect("Couldn't find libevent");
    let libhwloc_dir = std::env::var("DEP_HWLOC_ROOT").expect("Couldn't find libevent");


    let pmix_build = std::env::var("DEP_PMIX_ROOT").expect("Could not find DEP_PMIX_ROOT");
    let prrte_path = std::fs::canonicalize(std::path::PathBuf::from("prrte")).unwrap();

    std::process::Command::new("./autogen.pl")
        .current_dir(prrte_path.as_path())
        .status()
        .expect("Failed to autogen for PMIx");

    let mut prrte_build = autotools::Config::new(prrte_path.as_path());
    
    prrte_build.with("pmix", Some(&pmix_build));
    prrte_build.with("libevent", Some(&lib_event_dir));
    prrte_build.with("hwloc", Some(&libhwloc_dir));


    let prrte_build = prrte_build.build();

    let _prrte_inc = prrte_build.join("include");
    let _prrte_lib = prrte_build.join("lib");
    println!("cargo:rerun-if-changed={}", "build.rs");
}