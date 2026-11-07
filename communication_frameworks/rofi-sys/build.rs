extern crate bindgen;
extern crate subprocess;
//extern crate libfabric_src;

use autotools;
use glob::glob;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

// ---- system-library lookup helpers. The same code is in libfabric-src, pmix-sys, rofi-sys, lamellar-ucx-sys and
// lamellar-ucc-sys (their build scripts are standalone). ----

/// `<root>/lib64` instead of `<root>/lib` when only lib64 holds one of `names` (RHEL-style installs such as `/usr`).
fn lib_dir_of(root: &std::path::Path, names: &[&str]) -> std::path::PathBuf {
    let lib = root.join("lib");
    let lib64 = root.join("lib64");
    if !names.iter().any(|n| lib.join(n).exists()) && names.iter().any(|n| lib64.join(n).exists()) {
        lib64
    } else {
        lib
    }
}

/// True when every member of `archive` can be linked into a position-independent object. Probes by linking the whole
/// archive into a throwaway shared object: non-PIC code fails with "relocation ... recompile with -fPIC", while
/// undefined symbols are fine for `-shared`, so the library's own dependencies need not be present. If the probe
/// cannot run (no C compiler), the archive is assumed usable.
fn archive_is_pic(archive: &std::path::Path, scratch: &std::path::Path) -> bool {
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let out = scratch.join("pic_probe.so");
    let result = std::process::Command::new(cc)
        .arg("-shared")
        .arg("-o")
        .arg(&out)
        .arg("-Wl,--whole-archive")
        .arg(archive)
        .arg("-Wl,--no-whole-archive")
        .output();
    let _ = std::fs::remove_file(&out);
    match result {
        Ok(output) if output.status.success() => true,
        Ok(output) => !String::from_utf8_lossy(&output.stderr).contains("-fPIC"),
        Err(_) => true,
    }
}

fn build_rofi(
    out_path: &PathBuf,
    ofi: &PathBuf,
    ofi_lib: &PathBuf,
    ofi_inc: &PathBuf,
    pmix: Option<&PathBuf>,
    provider: &str,
    ofi_version: Option<&str>,
) -> (PathBuf,bool,bool) {
    
    // The CXI code path includes <rdma/fi_cxi_ext.h>; say so up front rather than failing inside ROFI's compile
    if provider == "cxi" && !ofi_inc.join("rdma/fi_cxi_ext.h").exists() {
        println!(
            "cargo:warning=ROFI was asked to build for the CXI provider, but {} has no rdma/fi_cxi_ext.h: that libfabric \
             was built without CXI support. Point OFI_DIR at a libfabric with the CXI provider (e.g. the system one on \
             Slingshot), or build for verbs.",
            ofi_inc.display()
        );
        exit(1);
    }
    // let ofi_env = ofi.lib_dir().to_path_buf();
    let dest = out_path.clone().join("rofi_src");
    if dest.exists() {
        fs::remove_dir_all(&dest).expect("failed to remove previous bundled ROFI source tree");
    }
    // println!("cargo:warning =Building Rofisys with libfabric at {ofi_env:?} ofi_env,and placing the rofi source code in {dest:?} "); 
    let cp_result = Command::new("cp")
        .args(&["-r", "rofi", &dest.to_string_lossy()])
        .status();
    if let Ok(_) = cp_result {
        let mut config = autotools::Config::new(dest);

        #[cfg(feature = "shared")]
        {
            config.enable("shared", None);
            config.disable("static", None);
        }

        #[cfg(not(feature = "shared"))]
        {
            config.disable("shared", None);
            config.enable("static", None);
        }

        #[cfg(feature = "debug")]
        {
            config.enable("debug", None);
        }

        #[cfg(not(feature = "debug"))]
        {
            config.disable("debug", None);
        }

        config.with(format!("ofi={}", ofi.display()), None);
        // ROFI compiles a different libfabric hint set per provider (__OFI_PROV_VERBS__ / __OFI_PROV_CXI__)
        config.with(format!("ofi-provider={}", provider), None);
        // Without this, configure reads the version from `pkg-config libfabric`, which ignores the prefix above
        if let Some(version) = ofi_version {
            config.with(format!("ofi-version={}", version), None);
        }
        // PMIx's own configure-time PMIx_Init link probe (`checking for
        // library containing PMIx_Init`) links only against -lpmix by
        // default. The vendored PMIx here is a static archive, so its own
        // transitive deps (hwloc, libevent) must be supplied explicitly via
        // -L/-l, same class of fix as lamellar-ucc-sys/lamellar-ucx-sys's
        // static-link issues. DEP_PMIX_HWLOC_ROOT/DEP_PMIX_EVENT_ROOT are
        // re-exported by pmix-sys's build.rs for exactly this purpose.
        //
        // -L/-Wl,-rpath go through LDFLAGS (order doesn't matter for search
        // paths), but the actual -l names must go through LIBS instead of
        // LDFLAGS: autoconf's conftest link line is `$CC $LDFLAGS conftest.c
        // $LIBS`, i.e. LDFLAGS is placed BEFORE the source file and LIBS
        // AFTER. A static archive's undefined symbols are only resolved
        // against objects/archives appearing LATER on the command line, so
        // -lpmix (a static archive) sitting in LDFLAGS -- before conftest.c
        // even exists on the line -- gets scanned and discarded before
        // main()'s call to PMIx_Init is ever seen, producing "undefined
        // reference to PMIx_Init" despite -lpmix being present. Same bug
        // class as lamellar-ucc-sys's --start-group/LIBS fix.
        let mut pmix_ldflags = String::new();
        let mut pmix_libs = String::new();
        if let Some(pmix_root) = pmix {
            config.with(format!("pmix={}", pmix_root.display()), None);
            let pmix_lib_dir = pmix_root.join("lib");
            pmix_ldflags.push_str(&format!(
                " -L{} -Wl,-rpath,{}",
                pmix_lib_dir.display(),
                pmix_lib_dir.display()
            ));
            // -lutil: PMIx's pmix_pty.c (used by its IOF/forwarding code)
            // calls openpty(), which lives in glibc's libutil, not libc.
            // Only surfaces once libpmix.a's undefined refs are actually
            // scanned against real objects (i.e. after fixing the LDFLAGS/
            // LIBS ordering bug above) -- a dynamic libpmix.so hides this
            // because -lutil is one of *its* transitive DT_NEEDED deps.
            // -lz: PMIx auto-detects zlib via pkg-config and compiles in
            // compress_zlib.c (modex/topology compression) whenever present
            // on the build machine -- same reason pmix-sys/build.rs adds
            // -lz=dylib for the final Rust link of a static libpmix.a.
            pmix_libs.push_str(" -lpmix -lutil -lz");
            if let Ok(hwloc_root) = std::env::var("DEP_PMIX_HWLOC_ROOT") {
                pmix_ldflags.push_str(&format!(
                    " -L{0}/lib -Wl,-rpath,{0}/lib",
                    hwloc_root
                ));
                pmix_libs.push_str(" -lhwloc");
            }
            if let Ok(event_root) = std::env::var("DEP_PMIX_EVENT_ROOT") {
                pmix_ldflags.push_str(&format!(
                    " -L{0}/lib -Wl,-rpath,{0}/lib",
                    event_root
                ));
                pmix_libs.push_str(" -levent_core -levent_pthreads");
            }
        }
        let pkgconfig_path = ofi_lib.join("pkgconfig");
        let existing = std::env::var("PKG_CONFIG_PATH").unwrap_or_default();
        let new_pkgconfig_path = if existing.is_empty() {
            pkgconfig_path.display().to_string()
        } else {
            format!("{}:{}", pkgconfig_path.display(), existing)
        };
        config.env("PKG_CONFIG_PATH", new_pkgconfig_path);
        config.ldflag(format!(
            "-L{}{}",
            ofi_lib.display(),
            pmix_ldflags
        ));
        // -libverbs/-pthread/-ldl/-lrdmacm/-lrt/-lpmix/-lhwloc/-levent_* all
        // moved into LIBS (appended after conftest.c on every conftest link
        // line) instead of LDFLAGS, for the same ordering reason as above --
        // libibverbs.so etc happened to work by luck before (shared libs
        // tolerate being scanned early far more often than static archives
        // do), but LIBS is the correct place for all of them regardless.
        config.env(
            "LIBS",
            format!("-libverbs -pthread -ldl -lrdmacm -lrt{}", pmix_libs),
        );
        config.cflag(format!("-O3 -I{}", ofi_inc.display()));
        config.cxxflag("-O3");

        let path = config.build();

         #[cfg(not(feature = "shared"))]
        return (path,true,false);
        #[cfg(feature = "shared")]
        return (path,false,true);
    } else {
        println!("cargo:warning =Failed to copy Rofi source code to {dest:?}");
        exit(1);
    }
}

/// The libfabric provider ROFI is compiled for, in order of precedence: the `cxi` feature; `ROFI_OFI_PROVIDER` (`verbs`,
/// `cxi` or `auto`); then auto-detection, which picks cxi when the libfabric install has `rdma/fi_cxi_ext.h` (installed
/// exactly when libfabric was built with the CXI provider) and verbs otherwise.
fn rofi_provider(ofi_inc: &std::path::Path, announce: bool) -> &'static str {
    println!("cargo:rerun-if-env-changed=ROFI_OFI_PROVIDER");
    let from_env = env::var("ROFI_OFI_PROVIDER").ok();
    if cfg!(feature = "cxi") {
        if matches!(from_env.as_deref(), Some(p) if p != "cxi" && p != "auto" && !p.is_empty()) {
            println!("cargo:warning=the cxi feature is enabled, so ROFI_OFI_PROVIDER={} is ignored", from_env.unwrap());
        }
        return "cxi";
    }
    match from_env.as_deref() {
        Some("verbs") => "verbs",
        Some("cxi") => "cxi",
        None | Some("") | Some("auto") => {
            if ofi_inc.join("rdma/fi_cxi_ext.h").exists() {
                if !announce {
                    return "cxi";
                }
                // verbs shows up as the InfiniBand/RoCE development headers on the build machine
                let verbs_too = ["/usr/include/infiniband/verbs.h", "/usr/local/include/infiniband/verbs.h"]
                    .iter()
                    .any(|h| std::path::Path::new(h).exists());
                println!(
                    "cargo:warning=ROFI is being built for the CXI provider (HPE Slingshot): {} has rdma/fi_cxi_ext.h, so \
                     this libfabric has the CXI provider.{} To build for the verbs provider instead, set \
                     ROFI_OFI_PROVIDER=verbs for the build (e.g. `ROFI_OFI_PROVIDER=verbs cargo build ...`); to silence this \
                     message, set ROFI_OFI_PROVIDER=cxi or enable the `rofi-cxi` feature.",
                    ofi_inc.display(),
                    if verbs_too { " The verbs provider also looks available here (infiniband/verbs.h is installed); cxi is the default when both are." } else { "" }
                );
                "cxi"
            } else {
                "verbs"
            }
        }
        Some(other) => {
            println!(
                "cargo:warning=unknown ROFI_OFI_PROVIDER '{}' (valid values: verbs, cxi, auto)",
                other
            );
            exit(1);
        }
    }
}

/// The provider an already-built ROFI library was compiled for, from the marker symbol it defines. None when the
/// library cannot be inspected or is not a ROFI library.
fn compiled_provider_of(lib: &PathBuf) -> Option<&'static str> {
    // ROFI defines exactly one of these marker functions (rofi_compiled_provider() in api.c)
    if check_lib_for_function(lib.clone(), "rofi_built_for_provider_cxi").is_some() {
        return Some("cxi");
    }
    if check_lib_for_function(lib.clone(), "rofi_built_for_provider_verbs").is_some() {
        return Some("verbs");
    }
    // An older ROFI without the markers: the CXI build has functions a verbs build lacks
    const CXI_ONLY: [&str; 3] = ["rofi_barrier_p2p_internal", "rofi_transport_barrier_msg", "rofi_transport_wait_on_cq"];
    if CXI_ONLY.iter().any(|f| check_lib_for_function(lib.clone(), f).is_some()) {
        Some("cxi")
    } else if check_lib_for_function(lib.clone(), "rofi_transport_init").is_some() {
        Some("verbs")
    } else {
        None
    }
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

fn pmix_root() -> Option<PathBuf> {
    if !cfg!(feature = "pmix") {
        return None;
    }

    match env::var("DEP_PMIX_ROOT") {
        Ok(val) => Some(PathBuf::from(val)),
        Err(_) => {
            println!(
                "cargo:warning=PMIx support was requested, but DEP_PMIX_ROOT is not set. Ensure the vendored pmix-sys build dependency completed successfully."
            );
            exit(1);
        }
    }
}

fn build_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:warning =Building Rofisys with libfabric at {:?}", out_path);

     for var in env::vars() {
        println!("cargo:warning ={}={}", var.0, var.1);
    }
    let ofi =PathBuf::from(env::var("DEP_OFI_ROOT").expect("DEP_OFI_ROOT not set"));
    let ofi_version = env::var("DEP_OFI_VERSION").ok();
    // libfabric-sys reports where the library is (lib or lib64) and whether it linked it shared
    let ofi_lib_dir = env::var("DEP_OFI_LIB_DIR").map(PathBuf::from).unwrap_or_else(|_| ofi.join("lib"));
    let ofi_shared = match env::var("DEP_OFI_SHARED") {
        Ok(v) => v == "1",
        Err(_) => cfg!(feature = "shared"),
    };
    let ofi_inc_dir = ofi.join("include");
    let prebuilt_rofi = env::var_os("ROFI_DIR").is_some();
    let provider = rofi_provider(&ofi_inc_dir, !prebuilt_rofi);
    // What a prebuilt ROFI (ROFI_DIR) is compiled for is read from the library below; the bundled build uses `provider`.
    let mut built_provider = provider;
    let pmix = pmix_root();
    println!("cargo:warning =Building Rofisys with libfabric at {:?}", ofi_lib_dir);
    println!("cargo:warning =Building Rofisys with libfabric include at {:?}", ofi_inc_dir);
    println!("cargo:rustc-link-search=native={}", ofi_lib_dir.display());
    if ofi_shared {
        println!("cargo:rustc-link-lib=dylib=fabric");
    } else {
        println!("cargo:rustc-link-lib=static=fabric");
    }
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", ofi_lib_dir.display());

    if let Some(pmix_root) = pmix.as_ref() {
        let pmix_lib_dir = pmix_root.join("lib");
        println!("cargo:warning =Building Rofisys with vendored PMIx at {:?}", pmix_root);
        println!("cargo:rustc-link-search=native={}", pmix_lib_dir.display());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", pmix_lib_dir.display());
        println!("cargo:rustc-link-lib=pmix");
    }
    

   

    let (rofi_env,rofi_a,rofi_so) = match env::var("ROFI_DIR") {
        Ok(val) => {
            println!("cargo:warning =Trying to use Rofi at {:?}", val);
            let rofi_path = std::path::PathBuf::from(val);
            let rofi_lib_dir = lib_dir_of(&rofi_path, &["librofi.so", "librofi.a"]);
            let rofi_a = rofi_lib_dir.join("librofi.a");
            let rofi_so = rofi_lib_dir.join("librofi.so");
            
            let mut rofi_a_valid =check_lib_for_function(rofi_a.clone(), "rofi_transport_init").is_some();
            let rofi_so_valid = check_lib_for_function(rofi_so.clone(), "rofi_transport_init").is_some();
            // an archive built without -fPIC cannot be linked into a PIE or cdylib: use the shared library instead
            if rofi_a_valid && rofi_so_valid && !archive_is_pic(&rofi_a, &out_path) {
                println!("cargo:warning=librofi.a at {:?} is not position-independent; linking the shared librofi.so instead", &rofi_a);
                rofi_a_valid = false;
            }

            if rofi_a_valid || rofi_so_valid {
                if cfg!(feature = "shared") {
                    if !rofi_so_valid {
                        println!("cargo:warning=Rofi shared library not found at {:?}, but static library found at {:?}. Using static library instead. (to supress compile without the shared feature, or unset ROFI_DIR to use bundled version", &rofi_so, &rofi_a);
                    }
                }else{
                    if !rofi_a_valid {
                        println!("cargo:warning=Rofi static library not found at {:?}, but shared library found at {:?}. Using shared library instead. (to supress warning compile with the shared feature, or unset ROFI_DIR to use bundled version)", &rofi_a, &rofi_so);
                    }
                }
                // A prebuilt ROFI's provider was fixed when it was configured (--with-ofi-provider); read it from the library
                let inspected = if rofi_a_valid { &rofi_a } else { &rofi_so };
                let requested = if cfg!(feature = "cxi") {
                    Some("cxi")
                } else {
                    match env::var("ROFI_OFI_PROVIDER").ok().as_deref() {
                        Some("verbs") => Some("verbs"),
                        Some("cxi") => Some("cxi"),
                        _ => None,
                    }
                };
                match compiled_provider_of(inspected) {
                    Some(detected) => {
                        built_provider = detected;
                        println!("cargo:warning=the prebuilt ROFI at {:?} is compiled for the {} provider", inspected, detected);
                        if requested.map_or(false, |r| r != detected) {
                            println!(
                                "cargo:warning=a {} ROFI was requested (feature/ROFI_OFI_PROVIDER) but ROFI_DIR provides one compiled for {}; using it as is. Rebuild ROFI with --with-ofi-provider={} or unset ROFI_DIR to use the bundled one.",
                                requested.unwrap(), detected, requested.unwrap()
                            );
                        }
                    }
                    None => {
                        built_provider = "verbs";
                        println!("cargo:warning=could not tell which provider the prebuilt ROFI at {:?} is compiled for (nm unavailable?); assuming verbs", inspected);
                    }
                }
                (rofi_path,rofi_a_valid,rofi_so_valid)
            } else {
                println!("cargo:warning=unable to detect rofi version at root path {:?} (lib path {:?}). SUGGESTED: Rofisys includes a bundled version of Rofi it can build itself, simply unset the ROFI_DIR env variable to use the bundled version. ALTERNATIVE: update to version 0.4 of Rofi manually.",rofi_path,rofi_lib_dir);
                exit(1);
            }

           
        }
        Err(_) => build_rofi(
            &out_path,
            &ofi,
            &ofi_lib_dir,
            &ofi_inc_dir,
            pmix.as_ref(),
            provider,
            ofi_version.as_deref(),
        ),
    };
    // Exported as `rofisys::OFI_PROVIDER` so the runtime's default LAMELLAR_ROFI_PROVIDER matches this build
    fs::write(
        out_path.join("provider.rs"),
        format!("/// The libfabric provider ROFI was compiled for (`verbs` or `cxi`).\npub const OFI_PROVIDER: &str = \"{}\";\n", built_provider),
    )
    .expect("failed to write provider.rs");
    for entry in glob("rofi/**/*.c").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("cargo:rerun-if-changed={}", path.display()),
            Err(_) => {}
        }
    }
    for entry in glob("rofi/**/*.h").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("cargo:rerun-if-changed={}", path.display()),
            Err(_) => {}
        }
    }

    let rofi_inc_dir = rofi_env.join("include");
    let rofi_lib_dir = lib_dir_of(&rofi_env, &["librofi.so", "librofi.a"]);

    println!("cargo:root={}", out_path.display());
    println!("cargo:rustc-link-search=native={}", rofi_lib_dir.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", rofi_lib_dir.display());

    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-search=native=/usr/lib64/libibverbs");
    
    
    if rofi_a {
        println!("cargo:rustc-link-lib=static=rofi");
    } else if rofi_so {
        println!("cargo:rustc-link-lib=dylib=rofi");
    } else {
        println!("cargo:warning =Rofi library not found, please ensure Rofi is installed or set ROFI_DIR to a valid Rofi installation.");
        exit(1);
    }
   
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=pmi_simple");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include")
        .clang_arg(format! {"-I{}", ofi_inc_dir.display()})
        .clang_arg(format! {"-I{}/rdma",ofi_inc_dir.display()})
        .clang_arg(format!("-I{}", rofi_inc_dir.display()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to src/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    println!("cargo:rerun-if-changed={}", "build.rs");
    println!("cargo:rerun-if-env-changed=DEP_OFI_ROOT");
    println!("cargo:rerun-if-env-changed=DEP_OFI_VERSION");
    println!("cargo:rerun-if-env-changed=DEP_PMIX_ROOT");
    build_bindings();
}
