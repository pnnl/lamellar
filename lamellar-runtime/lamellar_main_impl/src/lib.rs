//! Proc-macro front end for `#[lamellar::main]` / `#[lamellar::test]`. The
//! launch code generation lives in `hpc_launch_codegen`, shared with
//! `hpc_launch_impl`; this crate supplies lamellar's settings (the `LAMELLAR`
//! environment prefix, backend/executor/... flags, libfabric environment) and
//! wraps the program body in lamellar's world-build / application timers.

use hpc_launch_codegen::{self as codegen, ExtraFlag, LaunchConfig, Launcher, MainBody};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[cfg(not(any(feature = "use-prterun", feature = "use-srun")))]
compile_error!(
    "Either feature \"use-prterun\" or \"use-srun\" must be enabled for the lamellar_main proc macros."
);
#[cfg(all(feature = "use-prterun", feature = "use-srun"))]
compile_error!(
    "Only one of features \"use-prterun\" or \"use-srun\" can be enabled for the lamellar_main proc macros."
);

#[cfg(feature = "use-srun")]
const LAUNCHER: Launcher = Launcher::Srun;
#[cfg(not(feature = "use-srun"))]
const LAUNCHER: Launcher = Launcher::Prterun;

/// Lamellar's launch settings. `explicit_libfabric` is what the program's own
/// `with_lamellae(Backend::..)` call picks, if its `main` contains one with a
/// literal backend; it takes precedence over `--lamellae` and `LAMELLAR_BACKEND`.
fn config(explicit_libfabric: Option<bool>) -> LaunchConfig {
    let mut cfg = LaunchConfig::new(
        "lamellar_main",
        "LAMELLAR",
        quote! { ::lamellar },
        LAUNCHER,
        cfg!(feature = "with-salloc"),
    );
    cfg.extra_flags = vec![
        ExtraFlag { flag: "--lamellae", local: "lamellae", env_var: "LAMELLAR_BACKEND" },
        ExtraFlag { flag: "--cmd-queue", local: "cmd_queue", env_var: "LAMELLAR_CMD_QUEUE" },
        ExtraFlag { flag: "--batcher", local: "batcher", env_var: "LAMELLAR_BATCHER" },
        ExtraFlag { flag: "--executor", local: "executor", env_var: "LAMELLAR_EXECUTOR" },
        ExtraFlag { flag: "--heap-size", local: "heap_size", env_var: "LAMELLAR_HEAP_SIZE" },
    ];
    cfg.help_extra = quote! {
        println!("  --lamellae <name>      lamellae backend for the launched job [env: LAMELLAR_BACKEND]");
        println!("      available: {} (default: {})", ::lamellar::available_backends().join(", "), ::lamellar::compiled_default_backend());
        println!("  --cmd-queue <variant>  command queue protocol for the launched job [env: LAMELLAR_CMD_QUEUE]");
        println!("      available: batched, get (default), geteager, getslots, put, putslots, puteager");
        println!("  --batcher <variant>    active-message batcher for the launched job [env: LAMELLAR_BATCHER]");
        println!("      available: simple (default), direct, team_am, vec_simple (experimental), vec_team_am (experimental)");
        println!("  --executor <variant>   executor used during execution for the launched job [env: LAMELLAR_EXECUTOR]");
        println!("      available: lamellar (default), single_thread, async_std, tokio (if tokio-executor feature enabled)");
        println!("  --heap-size <bytes>    initial size of the RDMAable runtime memory pool [env: LAMELLAR_HEAP_SIZE]");
    };
    cfg.single_pe_default = quote! { ::lamellar::config().backend == "local" };
    let explicit_libfabric = match explicit_libfabric {
        Some(b) => quote! { Some(#b) },
        None => quote! { None::<bool> },
    };
    cfg.extra_launcher_env = quote! {
        // UCX's memory hooks (installed when the program starts, if UCX is compiled in) break libfabric's memory
        // registration cache, so a libfabric run has them switched off. They can only be switched off before the
        // process starts, which is why it is done here. Left alone if the user has set it.
        let __explicit_libfabric: Option<bool> = #explicit_libfabric;
        let __uses_libfabric = __explicit_libfabric.unwrap_or_else(|| {
            lamellae.clone().unwrap_or_else(|| ::lamellar::config().backend.clone()).starts_with("libfabric")
        });
        if __uses_libfabric && std::env::var_os("UCX_MEM_EVENTS").is_none() {
            launcher_cmd.env("UCX_MEM_EVENTS", "no");
        }
    };
    cfg
}

/// Whether the `with_lamellae(Backend::X)` call in this function selects a libfabric backend: `Some(true/false)` for a
/// literal `Backend::X` argument, `None` if there is no such call or its argument is not a literal variant.
fn explicit_backend_is_libfabric(func: &ItemFn) -> Option<bool> {
    let text = quote!(#func).to_string();
    let call = text.find("with_lamellae")?;
    let rest = &text[call + "with_lamellae".len()..];
    let rest = rest.trim_start().strip_prefix('(')?.trim_start();
    // only the call's own argument: a variable there is not a literal backend
    let rest = &rest[..rest.find(')')?];
    // "Backend :: X" or "lamellar :: Backend :: X" -- tokens are stringified with spaces around "::"
    let mut parts = rest.split("::").map(str::trim);
    let mut prev = parts.next()?;
    for part in parts {
        if prev.ends_with("Backend") {
            let variant: String = part
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            return Some(variant.starts_with("Libfabric"));
        }
        prev = part;
    }
    None
}

// Find the LamellarWorldBuilder::new()...build() statement and split the
// block around it so we can emit a separate timer for world construction
// and one for the application code that follows. If the whole fn body is
// wrapped in a single bare/unsafe block (as many examples do), that
// wrapper is itself the only top-level statement and would otherwise be
// misidentified as "the world build statement" (its stringified tokens
// contain "LamellarWorldBuilder" too, since it contains everything) --
// leaving nothing after it to time. So unwrap lone wrapper blocks first
// and split on the statements actually inside them, then rebuild the
// same wrapper around the timed result.
fn unwrap_single_block(stmts: &[syn::Stmt]) -> Option<(bool, &syn::Block)> {
    if stmts.len() != 1 {
        return None;
    }
    match &stmts[0] {
        syn::Stmt::Expr(syn::Expr::Unsafe(e), _) => Some((true, &e.block)),
        syn::Stmt::Expr(syn::Expr::Block(e), _) if e.label.is_none() => Some((false, &e.block)),
        _ => None,
    }
}

fn build_timed_body(stmts: &[syn::Stmt]) -> proc_macro2::TokenStream {
    if let Some((is_unsafe, inner_block)) = unwrap_single_block(stmts) {
        let inner = build_timed_body(&inner_block.stmts);
        return if is_unsafe {
            quote! { unsafe { #inner } }
        } else {
            quote! { { #inner } }
        };
    }

    let world_build_idx = stmts
        .iter()
        .position(|stmt| quote!(#stmt).to_string().contains("LamellarWorldBuilder"));

    if let Some(idx) = world_build_idx {
        let pre = &stmts[..idx];
        let world_stmt = &stmts[idx];
        let post = &stmts[idx + 1..];
        quote! {
            #(#pre)*
            let __lamellar_world_build_start = std::time::Instant::now();
            let mut __lamellar_app_start;
            {
                #world_stmt
                if std::env::var("LAMELLAR_MAIN_TIME").is_ok() {
                    println!("[LAMELLAR_MAIN] world build time: {:?}", __lamellar_world_build_start.elapsed());
                }
                __lamellar_app_start = std::time::Instant::now();
                #(#post)*

            } // this should enforce that world is dropped across all PEs before we print the application time, which is important for accurate timing of the application code.
            if std::env::var("LAMELLAR_MAIN_TIME").is_ok() {
                println!("[LAMELLAR_MAIN] application time: {:?}", __lamellar_app_start.elapsed());
            }
        }
    } else {
        quote! {
            #(#stmts)*
        }
    }
}

#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let cfg = config(explicit_backend_is_libfabric(&func));

    let (pre, post) = if cfg!(feature = "enable-prof") {
        (
            quote! { ::lamellar::init_prof_bt!(); },
            quote! { ::lamellar::fini_prof!(); },
        )
    } else {
        (quote! {}, quote! {})
    };
    let body = MainBody {
        pre,
        body: build_timed_body(&func.block.stmts),
        post,
    };
    TokenStream::from(codegen::expand_main(&cfg, &func, body))
}

#[proc_macro_attribute]
pub fn test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    TokenStream::from(codegen::expand_test(&config(None), &func))
}
