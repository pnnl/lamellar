extern crate bindgen;
#[cfg(any(feature = "vendored", feature = "openpmix-src"))]
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
    #[cfg(any(feature = "vendored", feature = "openpmix-src"))]
    {
        if env_inner("PMIX_NO_VENDORED").map_or(true, |v| v == "0") {
            // None means: no explicit override known here, so openpmix-src's own
            // ./configure is left to auto-detect a system libevent via pkg-config
            // (see openpmix-src's build.rs), and this link-search/link-lib emission
            // below falls back to trusting the linker's default search paths.
            let libevent_from_env = env_inner("LIBEVENT_DIR");
            let lib_event_dir = libevent_from_env.clone()
                .or_else(|| std::env::var("DEP_EVENT_ROOT").ok())
                .or_else(|| std::env::var("DEP_EVENT_INCLUDE").ok().map(|include_str| {
                    std::path::Path::new(&include_str).parent().unwrap().display().to_string()
                }));
            let artifacts =  openpmix_src::Build::new().build();
            // Propagate DEP_PMIX_ROOT to consumers (e.g. prrte-src) via the `links = "pmix"`
            // mechanism -- find_pmix_normal() below already does this for the non-vendored
            // path, but this vendored path never did, leaving DEP_PMIX_ROOT unset even on a
            // successful vendored build.
            println!("cargo:root={}", artifacts.lib_dir().parent().unwrap_or(artifacts.lib_dir()).display());
            // Re-export the hwloc openpmix-src actually built/used, so prrte-sys can
            // reuse the exact same hwloc (via DEP_PMIX_HWLOC_ROOT/DEP_PMIX_HWLOC_INCLUDE)
            // instead of needing its own hwloc-src vendoring.
            let hwloc_link_kind = if let Some(dir) = artifacts.hwloc_dir() {
                println!("cargo:hwloc_root={}", dir.display());
                println!("cargo:hwloc_include={}", dir.join("include").display());
                println!("cargo:rustc-link-search={}/lib", dir.display());
                "static"
            } else {
                "dylib"
            };
            if let Some(dir) = lib_event_dir.as_ref() {
                println!("cargo:rustc-link-search={}/lib", dir);
                // Re-export libevent's dir (mirrors hwloc_root/hwloc_include
                // above) so rofi-sys -- which links PMIx directly but has no
                // Cargo dependency edge to libevent-sys, hence no DEP_EVENT_*
                // of its own -- can find PMIx's own event_core/event_pthreads
                // transitive deps via DEP_PMIX_EVENT_ROOT.
                println!("cargo:event_root={}", dir);
            }
            // A system libevent (selected via LIBEVENT_DIR, or auto-detected by
            // openpmix-src's own configure via pkg-config when unset) is typically
            // only available as a shared library, unlike the vendored autotools
            // build (DEP_EVENT_ROOT case) which always produces static archives.
            let event_link_kind = if lib_event_dir.is_some() && libevent_from_env.is_none() { "static" } else { "dylib" };
            println!("cargo:rustc-link-lib={}=event_core", event_link_kind);
            println!("cargo:rustc-link-lib={}=event_pthreads", event_link_kind);
            println!("cargo:rustc-link-lib={}=hwloc", hwloc_link_kind);
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
    // Prefer static if a static archive was produced (vendored build now
    // defaults to enable_static/disable_shared); fall back to dylib for a
    // system-provided (PMIX_LIB_DIR) shared install.
    let pmix_link_kind = if artifacts.0.join("libpmix.a").exists() { "static" } else { "dylib" };
    println!("cargo:rustc-link-lib={}=pmix", pmix_link_kind);
    println!("cargo:rustc-link-search=native={}", artifacts.0.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}",  artifacts.0.display());
    // PMIx's configure auto-detects zlib via pkg-config when present and,
    // if so, compiles in the compress_zlib PSEC/modex codepath. A shared
    // libpmix.so would record libz as DT_NEEDED and the dynamic linker
    // resolves it transitively; a static libpmix.a does not, so an
    // unrelated consumer binary fails to link with undefined
    // inflate/deflate symbols unless something adds -lz explicitly. zlib
    // itself is essentially always available as a system shared library,
    // so link it dynamically here rather than vendoring it.
    if pmix_link_kind == "static" {
        println!("cargo:rustc-link-lib=dylib=z");
    }


}
