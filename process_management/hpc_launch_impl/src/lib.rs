//! Proc-macro front end for `#[hpc_launch::main]` / `#[hpc_launch::test]`.
//! The code generation itself lives in `hpc_launch_codegen`, which is shared
//! with `lamellar_main_impl`.

use hpc_launch_codegen::{self as codegen, LaunchConfig, Launcher};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[cfg(not(any(feature = "use-prterun", feature = "use-srun")))]
compile_error!(
    "Either feature \"use-prterun\" or \"use-srun\" must be enabled for the hpc_launch proc macros."
);
#[cfg(all(feature = "use-prterun", feature = "use-srun"))]
compile_error!(
    "Only one of features \"use-prterun\" or \"use-srun\" can be enabled for the hpc_launch proc macros."
);

#[cfg(feature = "use-srun")]
const LAUNCHER: Launcher = Launcher::Srun;
#[cfg(not(feature = "use-srun"))]
const LAUNCHER: Launcher = Launcher::Prterun;

fn config() -> LaunchConfig {
    LaunchConfig::new(
        "hpc_launch",
        "HPC_LAUNCH",
        quote! { ::hpc_launch },
        LAUNCHER,
        cfg!(feature = "with-salloc"),
    )
}

#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let cfg = config();
    let body = codegen::default_main_body(&cfg, &func);
    TokenStream::from(codegen::expand_main(&cfg, &func, body))
}

#[proc_macro_attribute]
pub fn test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    TokenStream::from(codegen::expand_test(&config(), &func))
}
