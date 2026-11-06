extern crate libfabric_src;


fn main() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let artifacts = libfabric_src::Build::new().build();
    
    // tested: only the vendored libfabric 1.22
    check_api_version(
        "libfabric",
        &artifacts.include_dir().join("rdma/fabric.h"),
        "FI_MAJOR_VERSION",
        "FI_MINOR_VERSION",
        "1.22",
        |major, minor| major == 1 && minor == 22,
    );

    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        // .clang_arg(format!("-I{}", ofi_include_path.to_str().unwrap()))
        // .header(out_path.to_str().unwrap().to_owned() + "/fabric_sys.h")
        // .blocklist_function("qgcvt")
        // .blocklist_function("qgcvt_r")
        // .blocklist_function("qfcvt")
        // .blocklist_function("qfcvt_r")
        // .blocklist_function("qecvt")
        // .blocklist_function("qecvt_r")
        // .blocklist_function("strtold")
        .header(artifacts.generated_header().to_str().unwrap())
        .clang_arg(format!("-I{}", artifacts.include_dir().to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // // Link with the libfabric and libinlined libraries to access their symbols.
    // println!("cargo:rustc-link-search={}", ofi_lib_path.display());
    // println!("cargo:rustc-link-search={}", out_path.display());

    // println!("cargo:rustc-link-lib=fabric");
    println!("cargo:rerun-if-changed={}", "build.rs");
    // Exposed to dependents as DEP_OFI_MAJOR (this crate has `links = "ofi"`), so lamellar-libfabric can gate on API changes.
    if let Some(major) = header_define(&artifacts.include_dir().join("rdma/fabric.h"), "FI_MAJOR_VERSION") {
        println!("cargo:major={}", major);
    }
    println!("cargo:root={}", artifacts.lib_dir().parent().unwrap_or(artifacts.lib_dir()).display());
    // The directory holding libfabric (lib or lib64) and whether it is linked shared; rofi-sys links the same way.
    println!("cargo:lib_dir={}", artifacts.lib_dir().display());
    println!("cargo:shared={}", if artifacts.shared() { 1 } else { 0 });
    println!("cargo:rustc-link-search={}", artifacts.lib_dir().display());
    if artifacts.shared() {
        println!("cargo:rustc-link-lib=dylib=fabric");
    } else {
        println!("cargo:rustc-link-lib=static=fabric");
    }
    println!("cargo:rustc-link-lib=rt");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=atomic");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=numa");
    println!("cargo:rustc-link-lib=uuid");
}

// Reads `#define <name> <n>` from `header`, or None if the header or macro cannot be read.
fn header_define(header: &std::path::Path, name: &str) -> Option<u32> {
    let text = std::fs::read_to_string(header).ok()?;
    text.lines().find_map(|line| {
        let mut words = line.split_whitespace();
        if words.next()? != "#define" || words.next()? != name {
            return None;
        }
        words.next()?.parse().ok()
    })
}

// Warns, and never fails, when the library whose headers are about to be bound is outside the versions this crate was
// tested with: bindgen names anonymous C unions and structs by position (`..__bindgen_ty_N`), so a different release can
// move the names the Rust code uses and the build then fails far from here, in the code that uses the bindings. The same
// function is in lamellar-ucc-sys, lamellar-ucx-sys and libfabric-sys (their build scripts are standalone).
// Reads `#define <major_name> <n>` and `#define <minor_name> <n>` from `header`; a header or macro that cannot be read
// is skipped without a word.
fn check_api_version(
    library: &str,
    header: &std::path::Path,
    major_name: &str,
    minor_name: &str,
    tested: &str,
    in_range: impl Fn(u32, u32) -> bool,
) {
    println!("cargo:rerun-if-changed={}", header.display());
    let text = match std::fs::read_to_string(header) {
        Ok(text) => text,
        Err(_) => return,
    };
    let define = |name: &str| -> Option<u32> {
        text.lines().find_map(|line| {
            let mut words = line.split_whitespace();
            if words.next()? != "#define" || words.next()? != name {
                return None;
            }
            words.next()?.parse().ok()
        })
    };
    if let (Some(major), Some(minor)) = (define(major_name), define(minor_name)) {
        if !in_range(major, minor) {
            println!(
                "cargo:warning={} {}.{} ({}) is outside the versions {} was tested with ({}); if the build fails on a name \
                 like `..__bindgen_ty_N` the bindings do not match this version",
                library,
                major,
                minor,
                header.display(),
                std::env::var("CARGO_PKG_NAME").unwrap_or_default(),
                tested
            );
        }
    }
}
