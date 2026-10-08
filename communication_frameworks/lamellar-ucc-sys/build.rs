
use std::{env, path::PathBuf};
use std::path::Path;

/// Vendored UCC release: unlike UCX/libfabric/MPICH-PMI, upstream has never
/// published an official "make dist" tarball for UCC. This is a Lamellar
/// hand-generated tarball -- a fresh checkout of git tag v1.8.0, `./autogen.sh
/// && ./configure && make dist` run locally -- not an official release
/// artifact. See README.md for the full provenance disclosure. UCC's
/// configure.ac already carries upstream's own AM_MAINTAINER_MODE, so no
/// Lamellar patch was needed there (unlike libfabric/MPICH-PMI).
const UCC_VERSION: &str = "1.8.0";

fn remove_dst_forcibly(dst: &Path) {
    match std::fs::remove_file(dst) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mut perms) = std::fs::metadata(dst).map(|m| m.permissions()) {
                    perms.set_mode(0o666);
                    let _ = std::fs::set_permissions(dst, perms);
                }
            }
            let _ = std::fs::remove_file(dst);
        }
        Err(_) => {}
    }
}

fn copy_rec(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        if dst.exists() && !dst.is_dir() {
            remove_dst_forcibly(dst);
        }
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            copy_rec(&src_path, &dst_path)?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if dst.exists() {
            remove_dst_forcibly(dst);
        }
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

fn build_ucc(out_path: &PathBuf) -> PathBuf {
    println!("cargo:warning=building vendored UCC {}", UCC_VERSION);

    let ucx_root_path = std::env::var("DEP_UCX_ROOT").expect("Could not find UCX installation.");
    let _ucx_lib_path = std::path::PathBuf::from(&ucx_root_path).join("lib");
    let _ucx_include_path = std::path::PathBuf::from(&ucx_root_path).join("include");


    let dest = out_path.clone().join("ucc_src");
    copy_rec(&std::path::PathBuf::from("ucc"), &dest).expect("Failed to copy UCC source files");

    let src_path = std::fs::canonicalize(&dest).unwrap();

    // No autogen.sh / autoreconf here: the vendored tree already ships a
    // pre-generated `configure` (see the provenance note on UCC_VERSION above).
    //
    // UCC's config/m4/ucx.m4 probes for UCX via AC_CHECK_LIB(ucp, ...,
    // [-luct -lucm -lucs]) -- a plain link test against only UCX's own
    // libs, and static archives (unlike .so's SONAME resolution) require
    // exact dependency order left-to-right. That hardcoded order is wrong
    // for a static build in two ways: (1) ucs actually depends on ucm
    // (memtype_cache.c/rcache.c call ucm_set_event_handler/ucm_orig_mmap/
    // etc.), but ucm is listed *before* ucs, so ld has already moved past
    // ucm's archive by the time ucs's undefined refs to it appear; (2)
    // uct's ib transport needs ibv_*/rdma_* symbols the probe never lists
    // at all. Re-supply -lucm after the probe's own "-lucs" (so ucm's
    // archive gets a second, later chance to satisfy ucs's forward
    // reference) and add the missing ibverbs/rdmacm chain via LIBS, which
    // autoconf appends at the very end of every AC_CHECK_LIB link line.
    //
    // LIBS is global (every configure link test gets it, including the
    // very first "does the C compiler work" sanity check run before
    // --with-ucx's own LDFLAGS scoping kicks in), so -lucm needs a
    // matching global -L or that earliest sanity check fails outright
    // with "cannot find -lucm".
    //
    // ucm and ucs have a genuine CIRCULAR static dependency: ucs's
    // memtype_cache.c/rcache.c call ucm_set_event_handler/ucm_orig_mmap
    // etc (ucs needs ucm), while ucm's event.c calls ucs_load_modules /
    // ucs_sys_is_dynamic_lib (ucm needs ucs). No single left-to-right
    // ordering of two archives with mutual undefined refs can satisfy
    // both directions in one pass, and this bites at two different
    // points: UCC's own ./configure probe (config/m4/ucx.m4's
    // AC_CHECK_LIB(ucp,...,[-luct -lucm -lucs])) AND the later `make
    // install` link of the ucc_info tool (LDADD is just libucc.la, but
    // automake appends the global $(LIBS) Makefile variable -- sourced
    // from this same env -- to every program link). Wrap them in
    // --start-group/--end-group so ld re-scans until both directions
    // resolve, instead of depending on order.
    //
    // BUT: this --start-group/--end-group trick only works for UCC's own
    // ./configure probes (AC_CHECK_LIB), not for the later `make install`
    // link of tools/info's ucc_info binary. Verified via `make V=1`:
    // ucc_info links against libucc.la, and libtool recursively expands
    // *that* into its recorded dependency_libs (-lucp -luct -lucs -lucm,
    // in that unwrapped order -- libtool's .la format explicitly excludes
    // -Wl,* flags from dependency_libs, so the group markers never made it
    // in when libucc.la itself was linked). libtool then dedupes those
    // same -l names out of our LIBS entirely, leaving an empty, useless
    // `-Wl,--start-group -Wl,--end-group` sitting after the real (and
    // still circularly-unresolved) archives. No LDFLAGS/LIBS placement
    // can fix this because the dependency_libs expansion is spliced in at
    // libucc.la's own position in LDADD, ahead of anything we control.
    //
    // Actual fix: don't build/install tools/info (or contrib/cmake/perf/
    // mpi-test) at all -- the Rust bindings only need libucc.a + headers,
    // which `make install` under the top-level `src` SUBDIR already
    // produces correctly (confirmed working in the log above this point).
    // Scope `make install` to that subdir with `-C src` to skip the
    // ucc_info link entirely.
    let path = autotools::Config::new(&src_path)
        .env(
            "LIBS",
            "-Wl,--start-group -lucp -luct -lucs -lucm -Wl,--end-group -libverbs -lrdmacm -lpthread -lm -ldl -lrt",
        )
        .ldflag(format!("-L{}/lib", ucx_root_path))
        .enable_static()
        .disable_shared()
        // Same PIC requirement as UCX/PMIx/hwloc -- libtool skips -fPIC
        // object compilation for a static-only build unless told
        // otherwise, which breaks the final Rust PIE link.
        .with("pic", None)
        .with("rocm", Some("no"))
        .with("cuda", Some("no"))
        .enable("optimizations", None)
        .with("ucx", Some(&ucx_root_path))
        .make_args(vec!["-C".to_string(), "src".to_string()])
        .build();
    path
}

fn build_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-env-changed=UCC_DIR");
    let ucc_path = match env::var("UCC_DIR") {
        Ok(val) => std::path::PathBuf::from(val),
        Err(_) => {
            let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
             println!("cargo:warning=UCC_DIR environment variable not set, building UCC from source. This may take a while. To use a pre-built version of UCC, set the UCC_DIR environment variable to the path of the UCC installation.");
             build_ucc(&out_dir)
        }
    };
    println!("cargo:root={}", ucc_path.display());

    let mut builder = cc::Build::new();
    builder.file("wrapper.c");
    builder.include(&ucc_path.join("./include"));
    builder.compile("wrapper");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", ucc_path.join("include").display()))
        .generate()
        .expect("Unable to generate bindings for UCC");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let ucc_lib_path = ucc_path.join("lib");
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        ucc_lib_path.display()
    );
    println!("cargo:rustc-link-search={}", ucc_lib_path.display());
    if let Ok(ucx_root_path) = env::var("DEP_UCX_ROOT") {
        let ucx_lib_path = std::path::PathBuf::from(ucx_root_path).join("lib");
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            ucx_lib_path.display()
        );
        println!("cargo:rustc-link-search={}", ucx_lib_path.display());
    }
    // Prefer static if a static archive was produced (vendored build now
    // defaults to enable_static/disable_shared); fall back to dylib for a
    // system-provided (UCC_DIR) shared install. lamellar-ucx-sys's own
    // build script already emits the ucp/uct/ucs/ucm/ibverbs/rdmacm link
    // directives needed to satisfy libucc.a's transitive symbols.
    if ucc_lib_path.join("libucc.a").exists() {
        // UCC's CL/TL/MC/EC/sysinfo components are dlopen()ed plugins in a
        // shared build (lib/ucc/libucc_<fw>_<name>.so). The vendored
        // ucc/src/utils/ucc_component.c carries a Lamellar patch that also
        // picks up components linked into the executable (weak references to
        // their ucc_<fw>_<name> iface symbols); nothing else references those
        // objects, so every component archive and libucc.a itself (for its
        // config-table constructors) must be linked +whole-archive.
        println!("cargo:rustc-link-lib=static:+whole-archive,-bundle=ucc");
        let ucc_component_dir = ucc_lib_path.join("ucc");
        println!(
            "cargo:rustc-link-search=native={}",
            ucc_component_dir.display()
        );
        let mut components: Vec<String> = std::fs::read_dir(&ucc_component_dir)
            .map(|dir| {
                dir.filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().into_string().ok()?;
                        let lib = name.strip_prefix("lib")?.strip_suffix(".a")?;
                        lib.starts_with("ucc_").then(|| lib.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        components.sort();
        for component in components {
            println!("cargo:rustc-link-lib=static:+whole-archive,-bundle={component}");
        }
    } else {
        println!("cargo:rustc-link-lib=dylib=ucc");
    }
}


fn main() {
    println!("cargo:rerun-if-changed={}", "build.rs");
    println!("cargo:rerun-if-changed={}", "wrapper.h");
    println!("cargo:rerun-if-changed={}", "wrapper.c");
    println!("cargo:rerun-if-changed={}", "ucc/src/utils/ucc_component.c");
    build_bindings();
}