extern crate bindgen;
extern crate subprocess;

use autotools;
use glob::glob;
use std::env;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

fn build_rofi(
    out_path: &PathBuf,
    ofi_env: &PathBuf,
    ofi_lib_dir: &PathBuf,
    ofi_inc_dir: &PathBuf,
) -> PathBuf {
    let dest = out_path.clone().join("rofi_src");
    Command::new("cp")
        .args(&["-r", "rofi", &dest.to_string_lossy()])
        .status()
        .unwrap();

    #[cfg(feature = "shared")]
    let path = autotools::Config::new(dest)
        .reconf("-ivfWnone")
        .enable("shared", None)
        .disable("static", None)
        .with(format!("ofi={}", ofi_env.display()), None)
        .ldflag(format! {" -L{} -libverbs -pthread -ldl -lrdmacm -lrt",ofi_lib_dir.display()})
        .cflag(format! {"-O3  -I{}",ofi_inc_dir.display()})
        .cxxflag(format! {"-O3"})
        .build();
    #[cfg(not(feature = "shared"))]
    let path = autotools::Config::new(dest)
        .reconf("-ivfWnone")
        .disable("shared", None)
        .enable("static", None)
        .with(format!("ofi={}", ofi_env.display()), None)
        .ldflag(format! {"-L{} -libverbs -pthread -ldl -lrdmacm -lrt",ofi_lib_dir.display()})
        .cflag(format! {"-O3 -I{} ",ofi_inc_dir.display()})
        .cxxflag(format! {"-O3"})
        .build();
    path
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

    let ofi_env = match env::var("OFI_DIR") {
        Ok(val) => std::path::PathBuf::from(val),
        Err(_) => {
            let dest = out_path.clone().join("ofi_src");
            Command::new("cp")
                .args(&["-r", "libfabric", &dest.to_string_lossy()])
                .status()
                .unwrap();

            #[cfg(not(feature = "shared"))]
            let install_dest = autotools::Config::new(dest.clone())
                .reconf("-ivf")
                .disable("shared", None)
                .enable("only", None)
                .enable("verbs", None)
                .enable("atomics", None)
                .enable("rxm", None)
                .enable("xpmem", Some("no"))
                .cflag("-O3")
                .cxxflag("-O3")
                .build();
            #[cfg(feature = "shared")]
            let install_dest = autotools::Config::new(dest.clone())
                .reconf("-ivf")
                .enable("shared", None)
                .disable("static", None)
                .enable("only", None)
                .enable("verbs", None)
                .enable("atomics", None)
                .enable("rxm", None)
                .enable("xpmem", Some("no"))
                .cflag("-O3")
                .cxxflag("-O3")
                .build();
            std::path::PathBuf::from(install_dest)
        }
    };
    let ofi_lib_dir = ofi_env.join("lib");
    let ofi_inc_dir = ofi_env.join("include");

    let rofi_env = match env::var("ROFI_DIR") {
        Ok(val) => {
            let rofi_path = std::path::PathBuf::from(val);
            let rofi_lib_dir = rofi_path.join("lib");
            let rofi_a = rofi_lib_dir.join("librofi.a");
            let rofi_so = rofi_lib_dir.join("librofi.so");
            if check_lib_for_function(rofi_a, "rofi_transport_init").is_some() {
                rofi_path
            } else if check_lib_for_function(rofi_so, "rofi_transport_init").is_some() {
                rofi_path
            } else {
                println!("cargo:warning=unable to detect rofi version at {:?}. SUGGESTED: Rofisys includes a bundled version of Rofi it can build itself, simply unset the ROFI_DIR env variable to use the bundled version. ALTERNATIVE: update to version 0.3 of Rofi manually.",rofi_path);
                exit(1);
            }
        }
        Err(_) => build_rofi(&out_path, &ofi_env, &ofi_lib_dir, &ofi_inc_dir),
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

    println!("cargo:rustc-link-search=native={}", rofi_lib_dir.display());
    println!("cargo:rustc-link-search=native={}", ofi_lib_dir.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", rofi_lib_dir.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", ofi_lib_dir.display());

    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-search=native=/usr/lib64/libibverbs");
    println!("cargo:rustc-link-lib=rofi");
    println!("cargo:rustc-link-lib=fabric");
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=pmi_simple");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include")
        .clang_arg(format! {"-I{}",ofi_inc_dir.display()})
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
    build_bindings();
}
