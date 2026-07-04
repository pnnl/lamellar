extern crate bindgen;
extern crate subprocess;
//extern crate libfabric_src;

use autotools;
use glob::glob;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

fn build_rofi(
    out_path: &PathBuf,
    ofi: &PathBuf,
    ofi_lib: &PathBuf,
    ofi_inc: &PathBuf,
    pmix: Option<&PathBuf>,
) -> (PathBuf,bool,bool) {
    
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
        config.reconf("-ivfWnone");

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
        if let Some(pmix_root) = pmix {
            config.with(format!("pmix={}", pmix_root.display()), None);
        }
        let pkgconfig_path = ofi_lib.join("pkgconfig");
        let existing = std::env::var("PKG_CONFIG_PATH").unwrap_or_default();
        let new_pkgconfig_path = if existing.is_empty() {
            pkgconfig_path.display().to_string()
        } else {
            format!("{}:{}", pkgconfig_path.display(), existing)
        };
        config.env("PKG_CONFIG_PATH", new_pkgconfig_path);
        config.ldflag(format!("-L{} -libverbs -pthread -ldl -lrdmacm -lrt", ofi_lib.display()));
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
    let ofi_lib_dir = ofi.join("lib");
    let ofi_inc_dir = ofi.join("include");
    let pmix = pmix_root();
    println!("cargo:warning =Building Rofisys with libfabric at {:?}", ofi_lib_dir);
    println!("cargo:warning =Building Rofisys with libfabric include at {:?}", ofi_inc_dir);
    println!("cargo:rustc-link-search=native={}", ofi_lib_dir.display());
    if cfg!(feature = "shared") {
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
            let rofi_lib_dir = rofi_path.join("lib");
            let rofi_a = rofi_lib_dir.join("librofi.a");
            let rofi_so = rofi_lib_dir.join("librofi.so");
            
            let rofi_a_valid =check_lib_for_function(rofi_a.clone(), "rofi_transport_init").is_some();
            let rofi_so_valid = check_lib_for_function(rofi_so.clone(), "rofi_transport_init").is_some();

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
                (rofi_path,rofi_a_valid,rofi_so_valid)
            } else {
                println!("cargo:warning=unable to detect rofi version at root path {:?} (lib path {:?}). SUGGESTED: Rofisys includes a bundled version of Rofi it can build itself, simply unset the ROFI_DIR env variable to use the bundled version. ALTERNATIVE: update to version 0.4 of Rofi manually.",rofi_path,rofi_lib_dir);
                exit(1);
            }

           
        }
        Err(_) => build_rofi(&out_path, &ofi, &ofi_lib_dir, &ofi_inc_dir, pmix.as_ref()),
    };
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
    let rofi_lib_dir = rofi_env.join("lib");

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
    println!("cargo:rerun-if-env-changed=DEP_PMIX_ROOT");
    build_bindings();
}
