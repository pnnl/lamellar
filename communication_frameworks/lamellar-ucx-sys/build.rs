use std::{
    env,
    path::PathBuf,
    process::{Command, Stdio, exit},
};

use glob::glob;

/// Vendored UCX release: the official "make dist" tarball from a UCX GitHub
/// release (v1.20.0), NOT a git checkout. It ships with `configure` already
/// generated, so no Flex/Autoconf/Automake/Libtool/autogen.sh is required to
/// build it.
const UCX_VERSION: &str = "1.20.0";

fn build_ucx(out_path: &PathBuf) -> PathBuf {
    println!("cargo:warning=building vendored UCX {}", UCX_VERSION);
    let dest = out_path.clone().join("ucx_src");
    Command::new("cp")
        .args(&["-r", "ucx", &dest.to_string_lossy()])
        .status()
        .unwrap();

    // No autogen.sh / autoreconf here: the vendored tree came from the
    // official release tarball, which ships a pre-generated `configure`
    // -- that's the whole point of vendoring the tarball instead of a
    // git checkout.
    //
    // Only run `./configure` via the autotools crate, not `make`/`make
    // install`: UCX's top-level `make all`/`make install` also builds
    // src/tools/profile's `ucx_read_profile`, whose link step fails with
    // `undefined reference to 'ucs_load_modules'` in a static
    // (--disable-shared) build -- static archives don't auto-pull
    // transitive deps the way .so's SONAME resolution does, and that
    // diagnostic tool isn't needed to link against libucp/libuct/libucs/
    // libucm. Build/install just the library subdirs ourselves instead.
    // Feature `shared`: build libucp/libuct/libucs/libucm (and the lib/ucx transport plugins) as shared libraries
    // instead of static archives. The link section below already picks dylibs when no libucp.a exists.
    let shared = env::var_os("CARGO_FEATURE_SHARED").is_some();
    let mut config = autotools::Config::new(dest.clone());
    if shared {
        config.enable("shared", None).disable("static", None);
    } else {
        config.disable("shared", None).enable("static", None);
    }
    let build_dir = config
        // Rust binaries link PIE by default; libtool skips -fPIC object
        // compilation for a static-only (--disable-shared) build unless
        // explicitly told otherwise, which breaks the final PIE link.
        .with("pic", None)
        .disable("logging", None)
        .disable("debug", None)
        .disable("assertions", None)
        .disable("params-check", None)
        .enable("optimizations", None)
        .enable("mt", None)
        .without("rocm", None)
        .without("cuda", None)
        .without("go", None)
        .without("java", None)
        .without("fuse3", None)
        .without("fuse3-static", None)
        .with("march", None)
        .configure();
    // autotools does an out-of-source build; configure() returns the
    // install prefix (OUT_DIR), not the build tree -- the actual
    // generated Makefiles live in OUT_DIR/build.
    let build_dir = build_dir.join("build");

    let num_jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "1".to_string());
    for subdir in ["src/ucm", "src/ucs", "src/uct", "src/ucp"] {
        for target in ["all", "install"] {
            let status = Command::new("make")
                .current_dir(&build_dir)
                .args(["-C", subdir, "-j", &num_jobs, target])
                .status()
                .expect("failed to run make");
            if !status.success() {
                panic!("`make -C {subdir} {target}` failed for vendored UCX build");
            }
        }
    }

    out_path.clone()
}

fn check_lib_for_function(lib: PathBuf, func: &str) -> Option<()> {
    let nm_out = Command::new("nm")
        .args(&["-g", &lib.to_string_lossy()])
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    let grep1 = Command::new("grep")
        .arg(func)
        .stdin(Stdio::from(nm_out.stdout?)) // Pipe through.
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    let grep2 = Command::new("grep")
        .arg("T")
        .stdin(Stdio::from(grep1.stdout?)) // Pipe through.
        .output()
        .ok()?;
    if grep2.stdout.len() > 0 {
        return Some(());
    } else {
        return None;
    }
}

fn build_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ucx_env = match env::var("UCX_DIR") {
        Ok(val) => {
            let ucx_path = std::path::PathBuf::from(val);
            let ucx_lib_dir = ucx_path.join("lib");
            let ucm_a = ucx_lib_dir.join("libucm.a");
            let ucm_so = ucx_lib_dir.join("libucm.so");
            let ucp_a = ucx_lib_dir.join("libucp.a");
            let ucp_so = ucx_lib_dir.join("libucp.so");
            let ucs_a = ucx_lib_dir.join("libucs.a");
            let ucs_so = ucx_lib_dir.join("libucs.so");
            let uct_a = ucx_lib_dir.join("libuct.a");
            let uct_so = ucx_lib_dir.join("libuct.so");
            if check_lib_for_function(ucm_a, "ucm_set_global_opts").is_some()
                && check_lib_for_function(ucp_a, "ucp_lib_query").is_some()
                && check_lib_for_function(ucs_a, "ucs_init_ucm_opts").is_some()
                && check_lib_for_function(uct_a, "uct_query_components").is_some()
            {
                ucx_path
            } else if check_lib_for_function(ucm_so, "ucm_set_global_opts").is_some()
                && check_lib_for_function(ucp_so, "ucp_lib_query").is_some()
                && check_lib_for_function(ucs_so, "ucs_init_ucm_opts").is_some()
                && check_lib_for_function(uct_so, "uct_query_components").is_some()
            {
                ucx_path
            } else {
                println!(
                    "cargo:warning=unable to detect ucx version at {:?}. SUGGESTED: Rofisys includes a bundled version of Rofi it can build itself, simply unset the ROFI_DIR env variable to use the bundled version. ALTERNATIVE: update to version 0.3 of Rofi manually.",
                    ucx_path
                );
                exit(1);
            }
        }
        Err(_) => build_ucx(&out_path),
    };
    for entry in glob("ucx/**/*.c").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("cargo:rerun-if-changed={}", path.display()),
            Err(_) => {}
        }
    }
    for entry in glob("ucx/**/*.h").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("cargo:rerun-if-changed={}", path.display()),
            Err(_) => {}
        }
    }

    let ucx_inc_dir = ucx_env.join("include");
    let ucx_lib_dir_base = ucx_env.join("lib");

    println!("cargo:root={}", out_path.display());
    println!(
        "cargo:rustc-link-search=native={}",
        ucx_lib_dir_base.display()
    );
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        ucx_lib_dir_base.display()
    );
    // Prefer static if static archives were produced (vendored build now
    // defaults to enable_static/disable_shared); fall back to dylib for a
    // system-provided (UCX_DIR) shared install.
    if ucx_lib_dir_base.join("libucp.a").exists() {
        // UCX initializes itself entirely from constructors: ucs_init()
        // (ucs/sys/init.c) sets up the global topo/async/config state, and
        // every UCT component/transport (UCT_COMPONENT_REGISTER,
        // uct_ib_init, ...) and UCP protocol registers via UCS_STATIC_INIT.
        // Nothing references those objects by symbol, so a plain static
        // link drops them -- ucp_init() then spins forever on the
        // never-initialized ucs_topo_global_ctx lock. Link every archive
        // +whole-archive (-bundle so the archives are resolved at the final
        // link from the search path rather than repacked into the rlib).
        //
        // With every member kept, archive order no longer matters for
        // symbol resolution, but it still sets .init_array (constructor)
        // order: list them in the order the shared libraries initialize
        // (ucm -> ucs -> uct -> uct transport modules -> ucp) so ucs_init()
        // runs before the uct/ucp constructors that depend on it.
        //
        // The transport modules (lib/ucx/libuct_*.a) are dlopen()ed plugins
        // in a shared build; statically they only exist if linked here, and
        // register through the same constructors. ib_mlx5 must follow ib
        // (it adds its md ops ahead of verbs, as its dlopen from
        // uct_ib_init would).
        let ucx_module_dir = ucx_lib_dir_base.join("ucx");
        println!(
            "cargo:rustc-link-search=native={}",
            ucx_module_dir.display()
        );
        for lib in ["ucm", "ucs", "uct"] {
            println!("cargo:rustc-link-lib=static:+whole-archive,-bundle={lib}");
        }
        for module in ["uct_ib", "uct_ib_mlx5", "uct_rdmacm", "uct_cma", "uct_knem"] {
            if ucx_module_dir.join(format!("lib{module}.a")).exists() {
                println!("cargo:rustc-link-lib=static:+whole-archive,-bundle={module}");
            }
        }
        println!("cargo:rustc-link-lib=static:+whole-archive,-bundle=ucp");
        if ucx_module_dir.join("libuct_ib_mlx5.a").exists() {
            println!("cargo:rustc-link-lib=mlx5");
        }
    } else {
        println!("cargo:rustc-link-lib=dylib=ucp");
        println!("cargo:rustc-link-lib=dylib=uct");
        println!("cargo:rustc-link-lib=dylib=ucs");
        println!("cargo:rustc-link-lib=dylib=ucm");
    }
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=rdmacm");
    // println!("cargo:rustc-link-lib=pmi_simple");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", ucx_inc_dir.display()))
        .allowlist_item("uc[mpst].*")
        .allowlist_item("UC[MPST].*")
        .rustified_enum(".*")
        .bitfield_enum("ucp_feature")
        .bitfield_enum(".*_field")
        .bitfield_enum(".*_flags(_t)?")
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to src/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    build_bindings();
}
