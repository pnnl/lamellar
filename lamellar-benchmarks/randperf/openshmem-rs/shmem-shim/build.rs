use std::env;
use std::path::Path;
use std::process::Command;
// libfabric build is optional for now; keep the code but disabled.
// use libfabric_src;

// Optional build-time SOS helper (available as a build-dependency).
#[cfg(any(feature = "sos-ofi", feature = "sos-ucx"))]
use sos_src;

#[cfg(feature = "ompi-ucx")]
use ompi_src;



fn main() {
    if let Ok(ofi_root) = env::var("DEP_OFI_ROOT") {
        println!("cargo:ofi_root={}", ofi_root);
    }

    let project_dir =
        env::var("CARGO_MANIFEST_DIR").expect("ERROR: CARGO_MANIFEST_DIR not defined?!");

    let mut shmemdir: Option<String> = None;// = env::var("CRAY_SHMEM_DIR").ok();
    let outdir = env::var("OUT_DIR").expect("ERROR: OUT_DIR not defined?!");

    // lamellar-openshmem is a normal cargo dependency (the `lamellar-openshmem` crate, an rlib): it provides the shmem_* symbols, so
    // nothing is built or linked here. Only the bindings are generated, from a copy of its generated shmem.h (c-src/lamellar-openshmem).
    if ["LAMELLAR_OPENSHMEM", "LAMELLAR_OPENSHMEM_UCX", "LAMELLAR_OPENSHMEM_LIBFABRIC"].iter().any(|f| env::var_os(format!("CARGO_FEATURE_{f}")).is_some()) {
        println!("cargo:rerun-if-changed=c-src/lamellar-openshmem/shmem.h");
        bindgen::Builder::default()
            .header(format!("{project_dir}/c-src/shim.h"))
            .clang_arg(format!("-I{project_dir}/c-src/lamellar-openshmem"))
            .allowlist_function("(_?shmem_.*|start_pes|shmalloc|shfree)")
            .allowlist_var("_?SHMEM_.*")
            .generate()
            .expect("bindgen failed on lamellar-openshmem's shmem.h")
            .write_to_file(format!("{outdir}/bindings.rs"))
            .expect("could not write bindings");
        return;
    }

    let mut include_arg: Option<String> = None;
    #[cfg(any(feature = "sos-ofi", feature = "sos-ucx"))]
    {
        #[cfg(feature = "sos-ucx")]
        for file in ["transport_ucx.c", "transport_ucx.h"] {
            println!(
                "cargo:rerun-if-changed={}",
                sos_src::source_dir().join("src").join(file).display()
            );
        }
        let a = sos_src::Build::new().build();
        println!("cargo:rustc-link-search=native={}", a.lib_dir().display());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", a.lib_dir().display());
        for lib in a.libs() {
            println!("cargo:rustc-link-lib={}", lib);
            println!("cargo:warn=Linking with sos library: {}", lib);
        }
        include_arg = Some(format!("-I{}", a.include_dir().display()));
        shmemdir = a
            .include_dir()
            .parent()
            .map(|p| p.to_str().unwrap().to_string());

        // SOS now builds libsma.a statically (see sos-src's with_ucx/
        // build_sos comments) rather than as libsma.so. A shared libsma.so
        // resolved its own PMIx/hwloc/UCX deps at ITS OWN link step; a
        // plain static libsma.a defers all of that to whoever links it in
        // -- so this crate's final link now needs to supply PMIx, hwloc,
        // and (whole-archived, see below) UCX itself.
        if let Ok(pmix_dir) = env::var("DEP_PMIX_ROOT") {
            println!("cargo:rustc-link-search=native={}/lib", pmix_dir);
            println!("cargo:rustc-link-lib=static=pmix");
            if let Ok(hwloc_dir) = env::var("DEP_PMIX_HWLOC_ROOT") {
                println!("cargo:rustc-link-search=native={}/lib", hwloc_dir);
                println!("cargo:rustc-link-lib=static=hwloc");
            }
            // PMIx's own build (pmix-sys/build.rs) emits these same
            // event_core/event_pthreads/z directives, but only for ITS OWN
            // crate's link -- pmix-sys sits behind sos-src as a nested
            // build-dependency here, and Cargo does not propagate a build
            // script's rustc-link-lib up through build-dependency edges to
            // the final binary (only through normal dependency edges), even
            // though it does propagate `links=`-keyed DEP_* env vars (which
            // is how DEP_PMIX_ROOT/DEP_EVENT_ROOT reach this build.rs at
            // all). So this crate's final link must re-emit them itself.
            if let Ok(event_dir) = env::var("DEP_EVENT_ROOT") {
                println!("cargo:rustc-link-search=native={}/lib", event_dir);
                println!("cargo:rustc-link-lib=static=event_core");
                println!("cargo:rustc-link-lib=static=event_pthreads");
            }
            println!("cargo:rustc-link-lib=dylib=z");
        }
        // UCX's constructor-registered UCT transports/UCP protocols need
        // to be whole-archived directly into the FINAL hamr-shmem-ucx
        // binary here -- SOS's own LDFLAGS (set in with_ucx) only affect
        // its ./configure sanity-check links, not this crate's final
        // link, since a --disable-shared autotools build never actually
        // links libsma.a itself. This mirrors lamellar-ucx-sys/build.rs's
        // known-working recipe for the identical vendored UCX archive.
        #[cfg(feature = "sos-ucx")]
        {
            if let Ok(ucx_dir) = env::var("DEP_UCX_ROOT") {
                let ucx_lib_dir = Path::new(&ucx_dir).join("lib");
                let ucx_module_dir = ucx_lib_dir.join("ucx");
                if ucx_lib_dir.join("libucp.a").exists() {
                    println!(
                        "cargo:rustc-link-search=native={}",
                        ucx_lib_dir.display()
                    );
                    println!(
                        "cargo:rustc-link-search=native={}",
                        ucx_module_dir.display()
                    );
                    for lib in ["ucm", "ucs", "uct"] {
                        println!("cargo:rustc-link-lib=static:+whole-archive,-bundle={lib}");
                    }
                    for module in ["uct_ib", "uct_ib_mlx5", "uct_rdmacm", "uct_cma", "uct_knem"] {
                        if ucx_module_dir.join(format!("lib{module}.a")).exists() {
                            println!(
                                "cargo:rustc-link-lib=static:+whole-archive,-bundle={module}"
                            );
                        }
                    }
                    println!("cargo:rustc-link-lib=static:+whole-archive,-bundle=ucp");
                    if ucx_module_dir.join("libuct_ib_mlx5.a").exists() {
                        println!("cargo:rustc-link-lib=mlx5");
                    }
                    println!("cargo:rustc-link-lib=ibverbs");
                    println!("cargo:rustc-link-lib=rdmacm");
                }
            }
        }
    }
    #[cfg(feature = "ompi-ucx")]
    {
        let a = ompi_src::Build::new().build();
        println!("cargo:rustc-link-search=native={}", a.lib_dir().display());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", a.lib_dir().display());
        for lib in a.libs() {
            println!("cargo:rustc-link-lib={}", lib);
            println!("cargo:warn=Linking with ompi library: {}", lib);
        }
        include_arg = Some(format!("-I{}", a.include_dir().display()));
        shmemdir = a
            .include_dir()
            .parent()
            .map(|p| p.to_str().unwrap().to_string());
        // DEP_SHMEM_ROOT for randperf-shmem/build.rs: a rustc-link-arg of this library crate does not reach the binary, so the binary
        // gets the rpath to the shared Open MPI libraries from there (tools/stage_runtime.py rewrites it for the compute nodes).
        println!("cargo:root={}", shmemdir.as_ref().unwrap());
    }

    #[cfg(feature = "ompi-system")]
    {
        // Open MPI is not built here: OMPI_DIR is an existing installation (bin/, lib/liboshmem, include/).
        println!("cargo:rerun-if-env-changed=OMPI_DIR");
        let root = env::var("OMPI_DIR").expect("feature ompi-system needs OMPI_DIR=<Open MPI install prefix>");
        assert!(Path::new(&root).join("lib").exists(), "OMPI_DIR={root} has no lib/");
        println!("cargo:rustc-link-search=native={root}/lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{root}/lib");
        println!("cargo:rustc-link-lib=oshmem");
        println!("cargo:root={root}");   // DEP_SHMEM_ROOT: hamr-openshmem/build.rs puts it in the binary's rpath
        include_arg = Some(format!("-I{root}/include"));
        shmemdir = Some(root);
    }

    #[cfg(feature = "hpcx-ucx")]
    {
        //check if PMIX_NO_VENDORED is set, if not set it to avoid picking up PMIx from the system.
        match env::var("PMIX_NO_VENDORED")
        {
            Ok(val) => {
                if val != "1" {
                    panic!("PMIX_NO_VENDORED environment variable is not set. Please set it to '1' to avoid picking up PMIx from the system.");
                }
            },
            Err(_) => {
                panic!("PMIX_NO_VENDORED environment variable is not set. Please set it to '1' to avoid picking up PMIx from the system.");
            }
        }
        
        if let Ok(ucx_dir) = env::var("HPCX_UCX_DIR") {
            println!("cargo:rustc-link-search=native={}/lib", ucx_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", ucx_dir);
            include_arg = Some(format!("-I{}/include", ucx_dir));
        }
        if let Ok(osh_dir) = env::var("HPCX_OSHMEM_DIR") {
            println!("cargo:rustc-link-search=native={}/lib", osh_dir);
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", osh_dir);
            println!("cargo:rustc-link-lib=oshmem");
            println!("cargo:rustc-link-lib=pmix");
            include_arg = Some(format!("{} -I{}/include", include_arg.unwrap_or_default(), osh_dir));
            shmemdir = Some(osh_dir);
        }
    }

    let shmeminc = match shmemdir.clone() {
        Some(shdir) => Some(format!("-I{}/include", shdir)),
        None => None,
    };

    let mut makeshim = Command::new("gcc");
    makeshim.args(&["c-src/shim.c", "-c", "-fPIC"]);
    if let Some(shinc) = shmeminc.as_ref() {
        makeshim.arg(shinc);
    }

    if !makeshim
        .arg("-o")
        .arg(format!("{}/shim.o", outdir))
        .status()
        .unwrap()
        .success()
    {
        panic!("Could not build shim.o.");
    }
    println!("cargo:warning=built shim.o");
    if !Command::new("ar")
        .args(&["-rvc", "librpgasc.a", "shim.o"])
        .current_dir(&Path::new(&outdir))
        .status()
        .unwrap()
        .success()
    {
        panic!("Failure running ar.");
    }

    print!("cargo:rustc-link-search=native={}\n", outdir);
    print!("cargo:rustc-link-arg=-Wl,-rpath,{}\n", outdir);
    print!("cargo:rustc-link-lib=static=rpgasc\n");
    

    let mut bindings = bindgen::Builder::default().header(format!("{}/c-src/shim.h", project_dir));
    if let Some(shinc) = shmeminc.as_ref() {
        bindings = bindings.clang_arg(shinc);
    }
    if let Some(ref s) = include_arg {
        bindings = bindings.clang_arg(s);
    }
    let bindings_path = format!("{}/bindings.rs", outdir);
    bindings
        .opaque_type("std:.*")
        .generate()
        .expect("ERROR: Error generating bindings")
        .write_to_file(&bindings_path)
        .expect("Error writing bindings");

    // Open MPI's OSHMEM declares the wait/test comparison constants as `enum shmem_wait_ops`, which bindgen exposes as
    // `shmem_wait_ops_SHMEM_CMP_*`; SOS declares them as macros (`SHMEM_CMP_*`). Give both the macro names, taking the
    // value from the library's own enum (the numeric values differ between the two).
    let mut text = std::fs::read_to_string(&bindings_path).expect("Error reading bindings");
    let mut aliases = String::new();
    for op in ["EQ", "NE", "GT", "LE", "LT", "GE"] {
        let enum_name = format!("pub const shmem_wait_ops_SHMEM_CMP_{op}:");
        let macro_name = format!("pub const SHMEM_CMP_{op}:");
        if text.contains(&enum_name) && !text.contains(&macro_name) {
            aliases.push_str(&format!(
                "pub const SHMEM_CMP_{op}: u32 = shmem_wait_ops_SHMEM_CMP_{op} as u32;\n"
            ));
        }
    }
    if !aliases.is_empty() {
        text.push('\n');
        text.push_str(&aliases);
        std::fs::write(&bindings_path, text).expect("Error writing bindings");
    }
}
