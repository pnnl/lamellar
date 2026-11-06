extern crate libfabric_src;


fn main() {
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let artifacts = libfabric_src::Build::new().build();
    
    // tested: the vendored libfabric 1.22, and 2.3 through 2.7 (items that differ between releases are probed below)
    check_api_version(
        "libfabric",
        &artifacts.include_dir().join("rdma/fabric.h"),
        "FI_MAJOR_VERSION",
        "FI_MINOR_VERSION",
        "1.22 and 2.3 to 2.7",
        |major, minor| (major == 1 && minor == 22) || (major == 2 && (3..=7).contains(&minor)),
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

    emit_api_cfgs(&out_path.join("bindings.rs"));

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

// libfabric's C API differs between releases: structs gain or lose fields (`xpu_ctx`, `max_group_id`, ...), fields move
// into anonymous unions (`data_progress`), and constants are removed (`FI_REG_MR`, `FI_MR_BASIC`, ...). The C ecosystem
// handles that with #ifdefs, and the Rust equivalent is a cfg per item, decided by looking at what bindgen actually
// generated rather than at the version number (2.3 and 2.7, for example, disagree about `xpu_ctx`).
//
// Exposed to dependents (this crate has `links = "ofi"`) as DEP_OFI_API_CFGS (the names that hold) and
// DEP_OFI_ALL_API_CFGS (every name that can be reported, for check-cfg); lamellar-libfabric's build script turns them
// into `cfg(...)`.
const STRUCT_FIELD_CFGS: &[(&str, &str, &str)] = &[
    // (cfg, struct, field): holds when the struct has the field directly
    ("ofi_ep_xpu_ctx", "fi_ep_attr", "xpu_ctx"),
    ("ofi_cq_xpu_ctx", "fi_cq_attr", "xpu_ctx"),
    ("ofi_cntr_xpu_ctx", "fi_cntr_attr", "xpu_ctx"),
    ("ofi_domain_max_group_id", "fi_domain_attr", "max_group_id"),
    ("ofi_domain_max_cntr_value", "fi_domain_attr", "max_cntr_value"),
    ("ofi_domain_max_err_cntr_value", "fi_domain_attr", "max_err_cntr_value"),
    ("ofi_domain_max_xpu_ctx_cnt", "fi_domain_attr", "max_xpu_ctx_cnt"),
    // 1.x: a plain field; 2.x: inside an anonymous union (`__bindgen_anon_1.data_progress`)
    ("ofi_domain_data_progress_flat", "fi_domain_attr", "data_progress"),
];
const CONST_CFGS: &[(&str, &str)] = &[
    // (cfg, constant): holds when bindgen generated the constant
    ("ofi_const_fi_reg_mr", "FI_REG_MR"),
    ("ofi_const_fi_local_mr", "FI_LOCAL_MR"),
    ("ofi_const_fi_buffered_recv", "FI_BUFFERED_RECV"),
    ("ofi_const_fi_order_strict", "FI_ORDER_STRICT"),
    ("ofi_const_fi_order_data", "FI_ORDER_DATA"),
    ("ofi_const_fi_mr_unspec", "fi_mr_mode_FI_MR_UNSPEC"),
    ("ofi_const_fi_mr_basic", "fi_mr_mode_FI_MR_BASIC"),
    ("ofi_const_fi_mr_scalable", "fi_mr_mode_FI_MR_SCALABLE"),
    ("ofi_const_fi_datatype_last", "fi_datatype_FI_DATATYPE_LAST"),
];

// The text of `pub struct <name> { ... }` in bindgen output, if present.
fn struct_body<'a>(bindings: &'a str, name: &str) -> Option<&'a str> {
    let start = bindings.find(&format!("pub struct {} {{", name))?;
    let rest = &bindings[start..];
    Some(&rest[..rest.find("\n}")?])
}

fn emit_api_cfgs(bindings_path: &std::path::Path) {
    let bindings = std::fs::read_to_string(bindings_path).expect("Couldn't read the generated bindings");
    let mut all = Vec::new();
    let mut present = Vec::new();
    for (cfg, struct_name, field) in STRUCT_FIELD_CFGS {
        all.push(*cfg);
        if struct_body(&bindings, struct_name).map_or(false, |body| body.contains(&format!("pub {}:", field))) {
            present.push(*cfg);
        }
    }
    for (cfg, constant) in CONST_CFGS {
        all.push(*cfg);
        if bindings.contains(&format!("pub const {}:", constant)) {
            present.push(*cfg);
        }
    }
    println!("cargo:api_cfgs={}", present.join(","));
    println!("cargo:all_api_cfgs={}", all.join(","));
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
