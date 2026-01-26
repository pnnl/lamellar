use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=DEP_PMI_ROOT");
    #[cfg(feature = "with-pmi1")]
    {
        if let Ok(pmi_lib_dir) = env::var("DEP_PMI_ROOT") {
            let lib_path = PathBuf::from(&pmi_lib_dir).join("lib");
            println!("cargo:rustc-link-search=native={}", lib_path.display());
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
            println!("cargo:root={}", pmi_lib_dir);
        } else {
            panic!(
                "unable to set pmix backend, recompile with 'with-pmix' feature {:?}",
                env::vars()
            )
        }
    }
    println!("cargo:rerun-if-env-changed=DEP_PMI2_ROOT");
    #[cfg(feature = "with-pmi2")]
    {
        if let Ok(pmi2_lib_dir) = env::var("DEP_PMI2_ROOT") {
            let lib_path = PathBuf::from(&pmi2_lib_dir).join("lib");
            println!("cargo:rustc-link-search=native={}", lib_path.display());
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
            println!("cargo:root={}", pmi2_lib_dir);
        } else {
            panic!(
                "unable to set pmix backend, recompile with 'with-pmix' feature {:?}",
                env::vars()
            )
        }
    }
    println!("cargo:rerun-if-env-changed=DEP_PMIX_ROOT");
    #[cfg(feature = "with-pmix")]
    {
        if let Ok(pmix_lib_dir) = env::var("DEP_PMIX_ROOT") {
            let lib_path = PathBuf::from(&pmix_lib_dir).join("lib");
            println!("cargo:rustc-link-search=native={}", lib_path.display());
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
            println!("cargo:root={}", pmix_lib_dir);
        } else {
            panic!(
                "unable to set pmix backend, recompile with 'with-pmix' feature {:?}",
                env::vars()
            )
        }
    }

}