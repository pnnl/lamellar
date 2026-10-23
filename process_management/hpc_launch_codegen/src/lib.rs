//! Token generator shared by `hpc_launch_impl` (`#[hpc_launch::main]`) and
//! `lamellar_main_impl` (`#[lamellar::main]`).
//!
//! A `proc-macro` crate can only export macros, so the logic that both macro
//! crates need lives here, in an ordinary library. A caller describes itself
//! with a [`LaunchConfig`] and gets back the tokens for the launch preamble
//! ([`launch_block`]), the whole rewritten `main` ([`expand_main`]) or a
//! launched `#[test]` ([`expand_test`]).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Attribute, ItemFn};

/// Which launcher the generated binary re-executes itself under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Launcher {
    Prterun,
    Srun,
}

/// An extra launch flag (beyond the generic ones) whose value is forwarded to
/// the launched job as an environment variable, e.g. `--lamellae` ->
/// `LAMELLAR_BACKEND`.
#[derive(Clone, Debug)]
pub struct ExtraFlag {
    /// The flag as typed on the command line, e.g. `"--lamellae"`.
    pub flag: &'static str,
    /// Name of the generated local (an `Option<String>`) holding the value,
    /// visible to [`LaunchConfig::extra_launcher_env`], e.g. `"lamellae"`.
    pub local: &'static str,
    /// Environment variable set on the launched job, e.g. `"LAMELLAR_BACKEND"`.
    pub env_var: &'static str,
}

/// Everything that differs between the users of the generator.
#[derive(Clone)]
pub struct LaunchConfig {
    /// Name used in messages, e.g. `"hpc_launch"` or `"lamellar_main"`.
    pub tool_name: &'static str,
    /// Prefix of the environment variables read and set: `{prefix}_NODES`,
    /// `_PES`, `_PES_PER_NODE`, `_THREADS`, `_LOG`, `_MAIN_TIME`.
    pub env_prefix: &'static str,
    /// Path of the crate the generated code calls helper functions
    /// (`numa_domain_count`, `init_tracing_from_env`, `prrte_sys`, ...)
    /// through, e.g. `::hpc_launch` or `::lamellar`.
    pub crate_path: TokenStream,
    /// Launcher the binary re-executes itself under.
    pub launcher: Launcher,
    /// Whether `--nodes` may request a SLURM allocation through `salloc`.
    pub with_salloc: bool,
    /// Extra flags forwarded to the job as environment variables.
    pub extra_flags: Vec<ExtraFlag>,
    /// Statements run when `--help` is given, after the generic option list
    /// and before the `--help, -h` line; used to print `extra_flags` help.
    pub help_extra: TokenStream,
    /// A `bool` expression: whether to default to a single PE when neither
    /// `--pes` nor `--pes-per-node` was given (otherwise one PE per NUMA domain).
    pub single_pe_default: TokenStream,
    /// Statements run on `launcher_cmd` (the `std::process::Command` for the
    /// launcher) just before the output-dir handling; they can read the
    /// locals named by [`ExtraFlag::local`].
    pub extra_launcher_env: TokenStream,
}

impl LaunchConfig {
    /// A config with no extras: `{tool_name}`, `{env_prefix}`, single PE
    /// defaulting off.
    pub fn new(
        tool_name: &'static str,
        env_prefix: &'static str,
        crate_path: TokenStream,
        launcher: Launcher,
        with_salloc: bool,
    ) -> Self {
        LaunchConfig {
            tool_name,
            env_prefix,
            crate_path,
            launcher,
            with_salloc,
            extra_flags: Vec::new(),
            help_extra: quote! {},
            single_pe_default: quote! { false },
            extra_launcher_env: quote! {},
        }
    }

    fn env(&self, suffix: &str) -> String {
        format!("{}_{}", self.env_prefix, suffix)
    }

    /// (env var set by the launcher on its children to detect "already
    /// launched", tokens for the launcher program path)
    fn launcher_info(&self) -> (TokenStream, TokenStream) {
        let cp = &self.crate_path;
        match self.launcher {
            Launcher::Prterun => (quote! {"PRTE_LAUNCHED"}, quote! {#cp::prrte_sys::prterun_path()}),
            Launcher::Srun => (quote! {"SLURM_LOCALID"}, quote! {"srun"}),
        }
    }

    /// Points a prterun launcher at the install tree it lives in, so a copied
    /// or deployed PRRTE tree doesn't fall back to its compiled-in build
    /// prefix. Not needed for srun.
    fn prefix_env_block(&self) -> TokenStream {
        let cp = &self.crate_path;
        match self.launcher {
            Launcher::Prterun => quote! { #cp::prrte_sys::apply_prefix_env(&mut launcher_cmd); },
            Launcher::Srun => quote! {},
        }
    }
}

/// Checks that `func` is a plain `fn main()` (optionally returning
/// `Result<..>`) and returns the tokens the launching branch evaluates to:
/// `Some(Ok(()))` when `main` returns a `Result`, `None` otherwise.
pub fn check_main_signature(func: &ItemFn) -> Option<TokenStream> {
    assert!(func.sig.asyncness.is_none(), "async not supported");
    assert!(func.sig.constness.is_none(), "const not supported");
    assert!(func.sig.unsafety.is_none(), "unsafety not supported");
    assert!(
        func.sig.generics.lt_token.is_none(),
        "generics not supported"
    );
    assert!(func.sig.variadic.is_none(), "variadics not supported");
    assert!(func.sig.inputs.is_empty(), "inputs not supported");

    match func.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_rarrow, ref t) => match t.as_ref() {
            syn::Type::Path(ref type_path) => {
                assert!(
                    type_path.path.segments.first().unwrap().ident == "Result",
                    "Only Result<(),...> is supported as return type"
                );
                Some(quote! {Ok(())})
            }
            _other => panic!("Only Result<(),..> is supported as return type"),
        },
    }
}

/// The tokens that replace a launched-or-not decision at the top of `main`:
/// an `if !<already launched> { <re-execute under the launcher> }` expression,
/// meant to be followed by an `else { .. }` holding the program body.
/// `ret` is evaluated after the launcher returns (see [`check_main_signature`]).
pub fn launch_block(cfg: &LaunchConfig, ret: Option<TokenStream>) -> TokenStream {
    let cp = &cfg.crate_path;
    let tool = cfg.tool_name;
    let (env_var, launcher_path) = cfg.launcher_info();
    let prefix_env_block = cfg.prefix_env_block();

    let nodes_env = cfg.env("NODES");
    let pes_env = cfg.env("PES");
    let ppn_env = cfg.env("PES_PER_NODE");
    let threads_env = cfg.env("THREADS");
    let log_env = cfg.env("LOG");
    let time_env = cfg.env("MAIN_TIME");

    let msg_mkdir = format!("{tool}: failed to create output directory");
    let msg_inconsistent = format!(
        "{tool}: inconsistent --nodes {{}} / --pes {{}} / --pes-per-node {{}} (expected pes == nodes * pes_per_node)"
    );
    let msg_resolved = format!(
        "{tool}: resolved launch parameters: nodes={{:?}}, pes={{:?}}, pes_per_node={{:?}}, threads_per_pe={{:?}}, numa_domains={{:?}}, packages={{:?}}"
    );
    let msg_salloc_fail = format!(
        "{tool}: --nodes given but failed to spawn salloc ({{}}), continuing without allocation"
    );
    let msg_salloc_missing = format!(
        "{tool}: --nodes given but this binary was built without the \"with-salloc\" feature; not requesting a SLURM allocation."
    );
    let help_header = format!("{tool} launch options (pass after `--`):");

    let output_dir_block = match cfg.launcher {
        Launcher::Srun => quote! {
            if !output_dir.is_empty() {
                let __dir = output_dir.trim_end_matches('/');
                std::fs::create_dir_all(__dir)
                    .expect(#msg_mkdir);
                prterun_args.insert(0, format!("--error={}/pe_%t.err", __dir));
                prterun_args.insert(0, format!("--output={}/pe_%t.out", __dir));
            }
        },
        Launcher::Prterun => quote! {
            if !output_dir.is_empty() {
                let __dir = output_dir.trim_end_matches('/');
                std::fs::create_dir_all(__dir)
                    .expect(#msg_mkdir);
                prterun_args.insert(
                    0,
                    format!("--merge-stderr-to-stdout"),
                );
                prterun_args.insert(
                    0,
                    format!("--output=file={}/hpc_launch", __dir),
                );
            }
        },
    };

    let pe_injection_block = match cfg.launcher {
        Launcher::Srun => quote! {
            if pes.is_some() || pes_per_node.is_some() {
                if let Some(ppn) = pes_per_node {
                    if !has_flag(&prterun_args, &["--ntasks-per-node"]) {
                        prterun_args.push(format!("--ntasks-per-node={}", ppn));
                    }
                } else if let Some(p) = pes {
                    if !has_flag(&prterun_args, &["--ntasks"]) {
                        prterun_args.push(format!("--ntasks={}", p));
                    }
                }
                if !has_flag(&prterun_args, &["--cpus-per-task"]) {
                    prterun_args.push(format!("--cpus-per-task={}", threads_per_pe));
                }
                if let Some(d) = numa_domains {
                    if d > 1 && !has_flag(&prterun_args, &["--cpu-bind"]) {
                        prterun_args.push("--cpu-bind=ldoms".to_string());
                    }
                }
            }
        },
        Launcher::Prterun => quote! {
            if pes.is_some() || pes_per_node.is_some() {
                if let Some(p) = pes {
                    if !has_flag(&prterun_args, &["--np", "-np"]) {
                        prterun_args.push("--np".to_string());
                        prterun_args.push(p.to_string());
                    }
                }
                if pes_per_node.is_some() && !has_flag(&prterun_args, &["--map-by"]) {
                    // Keep the mapping *policy* as "node" so ranks still spread across
                    // all allocated nodes. `PE=<n>` implicitly binds each rank to <n>
                    // cpus, and PRRTE rejects combining a PE= directive with any
                    // --bind-to other than "core"/"hwt" -- so NUMA-awareness here
                    // comes entirely from sizing PE= to threads_per_pe. But if a single
                    // PE needs more cores than fit in one package (socket), that
                    // per-core binding would have to span multiple packages, which
                    // PRRTE always refuses ("bound to CPUs in more than one package").
                    // In that case drop PE= entirely and let the rank float across the
                    // whole node. This is a package boundary, not a NUMA domain one --
                    // a package can contain multiple NUMA domains, so pes_per_node can
                    // be well below numa_domains while still fitting in one package.
                    // pes_per_node < packages is only a necessary precondition for that,
                    // not sufficient -- only actually span if threads_per_pe wouldn't
                    // fit within a single package's cores.
                    let spans_multiple_domains = match packages {
                        Some(p) if p > 1 && pes_per_node.unwrap_or(1) < p => {
                            match #cp::cores_per_package() {
                                Some(cores_per_pkg) => threads_per_pe > cores_per_pkg,
                                None => true,
                            }
                        }
                        _ => false,
                    };
                    prterun_args.push("--map-by".to_string());
                    if spans_multiple_domains {
                        prterun_args.push("node".to_string());
                        if !has_flag(&prterun_args, &["--bind-to"]) {
                            prterun_args.push("--bind-to".to_string());
                            prterun_args.push("none".to_string());
                        }
                    } else {
                        prterun_args.push(format!("node:PE={}", threads_per_pe));
                    }
                }
            }
        },
    };

    // `--pes N` on its own says nothing about how the PEs spread over nodes. Left
    // unset, the thread count below assumes one PE per node (every PE gets all
    // the cores' worth of threads) while prterun's default binding gives each
    // rank a single core, so the PEs oversubscribe that core and can hang. Assume
    // the PEs fill the nodes at hand instead -- the allocation's node count when
    // inside SLURM, else just this node -- so threads and `--map-by node:PE=`
    // are sized for that layout. Not done for srun: its `--ntasks-per-node`
    // would launch more than N tasks when N isn't a multiple of the node count.
    let pes_per_node_default_block = match cfg.launcher {
        Launcher::Prterun => quote! {
            let pes_per_node = match (pes, pes_per_node, nodes) {
                (Some(p), None, None) => {
                    let alloc_nodes: u32 = ["SLURM_JOB_NUM_NODES", "SLURM_NNODES"]
                        .iter()
                        .find_map(|v| std::env::var(v).ok().and_then(|s| s.parse().ok()))
                        .unwrap_or(1)
                        .max(1);
                    Some((p + alloc_nodes - 1) / alloc_nodes)
                }
                _ => pes_per_node,
            };
        },
        Launcher::Srun => quote! {},
    };

    let salloc_block = if cfg.with_salloc {
        quote! {
            // If we're not already inside a SLURM allocation and a node count was
            // resolved, request one via `salloc` and re-invoke this same binary
            // (with its original argv) inside it.
            let in_allocation = std::env::var("SLURM_JOB_ID").is_ok();
            if !in_allocation {
                if let Some(n) = nodes {
                    let salloc_result = std::process::Command::new("salloc")
                        .arg("-N")
                        .arg(n.to_string())
                        .arg("--exclusive")
                        .args(&salloc_opts)
                        .args(&original_args)
                        .status();
                    match salloc_result {
                        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
                        Err(e) => {
                            eprintln!(#msg_salloc_fail, e);
                        }
                    }
                }
            }
        }
    } else {
        quote! {
            let _ = &salloc_opts;
            let _ = &original_args;
            if nodes.is_some() && std::env::var("SLURM_JOB_ID").is_err() {
                eprintln!(#msg_salloc_missing);
            }
        }
    };

    // Extra flags: locals, reserved-flag list, parse branches, env forwarding.
    let extra_locals = cfg.extra_flags.iter().map(|f| {
        let local = format_ident!("{}", f.local);
        quote! { let mut #local: Option<String> = None; }
    });
    let extra_reserved = cfg.extra_flags.iter().map(|f| {
        let flag = f.flag;
        quote! { #flag, }
    });
    let extra_parse = cfg.extra_flags.iter().map(|f| {
        let flag = f.flag;
        let local = format_ident!("{}", f.local);
        quote! {
            if x == #flag {
                #local = extra.next();
                continue;
            }
        }
    });
    let extra_forward = cfg.extra_flags.iter().map(|f| {
        let local = format_ident!("{}", f.local);
        let var = f.env_var;
        quote! {
            if let Some(ref __v) = #local {
                launcher_cmd.env(#var, __v);
            }
        }
    });

    let help_extra = &cfg.help_extra;
    let single_pe_default = &cfg.single_pe_default;
    let extra_launcher_env = &cfg.extra_launcher_env;
    let binary_update_block = binary_update_block_tokens();

    quote! {
        let prte_launched = std::env::var(#env_var).is_ok();
        if !prte_launched {
            let mut args: Vec<String> = std::env::args().collect();
            // Full original argv, preserved so we can re-invoke this same binary+args
            // under `salloc` if an allocation needs to be requested first.
            let original_args = args.clone();

            let exec = args.remove(0);

            let mut prterun_args = Vec::<String>::new();

            let mut time=false;
            let mut output_dir = String::new();
            let mut gdb_mode: Option<String> = None;
            let mut nodes: Option<u32> = None;
            let mut pes: Option<u32> = None;
            let mut pes_per_node: Option<u32> = None;
            let mut salloc_opts: Vec<String> = Vec::new();
            let mut threads_per_pe_flag: Option<u32> = None;
            let mut log_filter: Option<String> = None;
            #(#extra_locals)*

            // `cargo run --example X -- <args>` only lets one literal "--" through
            // to the binary (cargo consumes its own args/binary-args separator), so
            // requiring users to type a second "--" to mark the launch-options
            // section is easy to forget. Fall back to splitting at the first
            // recognized launch flag when no literal "--" is present at all, so
            // `cargo run --example X -- --pes 2` still works.
            const RESERVED_LAUNCH_FLAGS: &[&str] = &[
                "--time", "--output-dir", "--gdb", "--nodes", "--pes", "--pes-per-node",
                "--threads-per-pe", "--log", "--salloc-opts", "--help", "-h",
                #(#extra_reserved)*
            ];
            let pos = args
                .iter()
                .position(|x| x == "--")
                .or_else(|| args.iter().position(|x| RESERVED_LAUNCH_FLAGS.contains(&x.as_str())));
            if let Some(pos) = pos {
                let has_explicit_sep = args.get(pos).map(|x| x == "--").unwrap_or(false);
                let extra_args: Vec<String> = if has_explicit_sep {
                    args.split_off(pos).into_iter().skip(1).collect()
                } else {
                    args.split_off(pos)
                };
                if extra_args.iter().any(|a| a == "--help" || a == "-h") {
                    println!(#help_header);
                    print!("{}", #cp::generic_help_text());
                    #help_extra
                    println!("  --help, -h             print this message and exit");
                    println!("Any other flags are passed through verbatim to the underlying launcher (prterun/srun).");
                    std::process::exit(0);
                }
                let mut extra = extra_args.into_iter().peekable();
                while let Some(x) = extra.next() {
                    #(#extra_parse)*
                    if x == "--time" {
                        time = true;
                    } else if x == "--output-dir" {
                        if let Some(dir) = extra.next() {
                            output_dir = dir;
                        }
                    } else if x == "--gdb" {
                        if extra.peek().map(|s| s.as_str()) == Some("bt") {
                            extra.next();
                            gdb_mode = Some("bt".to_string());
                        } else {
                            gdb_mode = Some("plain".to_string());
                        }
                    } else if x == "--nodes" {
                        nodes = extra.next().and_then(|n| n.parse().ok());
                    } else if x == "--pes" {
                        pes = extra.next().and_then(|n| n.parse().ok());
                    } else if x == "--pes-per-node" {
                        pes_per_node = extra.next().and_then(|n| n.parse().ok());
                    } else if x == "--threads-per-pe" {
                        threads_per_pe_flag = extra.next().and_then(|n| n.parse().ok());
                    } else if x == "--log" {
                        log_filter = extra.next();
                    } else if x == "--salloc-opts" {
                        // Everything up to the next "--" (or end of args) is passed
                        // through to `salloc` verbatim, e.g.
                        // -- --nodes 2 --salloc-opts --partition foo --time 01:00:00 -- <launcher args>
                        while let Some(y) = extra.next() {
                            if y == "--" {
                                break;
                            }
                            salloc_opts.push(y);
                        }
                    } else {
                        prterun_args.push(x.to_string());
                    }
                }
            }

            // Resolve --nodes/--pes/--pes-per-node, falling back to env vars, then
            // deriving whichever of the three wasn't given from the other two.
            let nodes = nodes.or_else(|| std::env::var(#nodes_env).ok().and_then(|s| s.parse().ok()));
            let pes = pes.or_else(|| std::env::var(#pes_env).ok().and_then(|s| s.parse().ok()));
            let pes_per_node = pes_per_node.or_else(|| std::env::var(#ppn_env).ok().and_then(|s| s.parse().ok()));

            let (nodes, pes, pes_per_node): (Option<u32>, Option<u32>, Option<u32>) = match (nodes, pes, pes_per_node) {
                (Some(n), Some(p), Some(ppn)) if n > 0 && ppn > 0 => {
                    if p != n * ppn {
                        eprintln!(#msg_inconsistent, n, p, ppn);
                        std::process::exit(1);
                    }
                    (Some(n), Some(p), Some(ppn))
                }
                (None, Some(p), Some(ppn)) if ppn > 0 => (Some((p + ppn - 1) / ppn), Some(p), Some(ppn)),
                (Some(n), None, Some(ppn)) if n > 0 => (Some(n), Some(n * ppn), Some(ppn)),
                (Some(n), Some(p), None) if n > 0 => (Some(n), Some(p), Some((p + n - 1) / n)),
                other => other,
            };

            #salloc_block

            #pes_per_node_default_block

            // Best-effort NUMA domain count on the current host, used to decide
            // whether to bind PEs by NUMA domain instead of plain node. Absence of
            // a usable /sys topology just disables NUMA-aware binding.
            let numa_domains: Option<u32> = #cp::numa_domain_count();

            // Best-effort physical package (socket) count. PRRTE's binding
            // restriction ("bound to CPUs in more than one package") is keyed on
            // package boundaries, not NUMA domain boundaries -- a package can
            // contain several NUMA domains, so numa_domains must not be used as a
            // stand-in for this check.
            let packages: Option<u32> = #cp::package_count();

            println!(#msg_resolved,
                nodes, pes, pes_per_node, threads_per_pe_flag, numa_domains, packages
            );

            // If neither --pes nor --pes-per-node was given, pick a sensible default:
            // one PE per NUMA domain (multiplied by node count if --nodes was given).
            let (pes, pes_per_node) = if pes.is_none() && pes_per_node.is_none() {
                #cp::resolve_pe_defaults(nodes, numa_domains, #single_pe_default)
            } else {
                (pes, pes_per_node)
            };

            let threads_per_pe: u32 = threads_per_pe_flag
                .or_else(|| std::env::var(#threads_env).ok().and_then(|s| s.parse().ok()))
                // a PE bound explicitly (e.g. `--map-by node:PE=4`) should run that many threads,
                // not one per core of the whole node
                .or_else(|| #cp::explicit_cores_per_pe(&prterun_args))
                .unwrap_or_else(|| {
                    let cores = std::thread::available_parallelism()
                        .map(|n| n.get() as u32)
                        .unwrap_or(4);
                    (cores / pes_per_node.unwrap_or(1).max(1)).max(1)
                });

            let has_flag = |args: &Vec<String>, needles: &[&str]| {
                args.iter().any(|a| needles.iter().any(|n| a.starts_with(n)))
            };

            #pe_injection_block

            #binary_update_block

            // rust-gdb wrapping goes immediately before the executable itself, not
            // mixed in with prterun's own launcher flags (--np/--map-by/etc. above) --
            // otherwise prterun sees "rust-gdb ... --args --np N ..." and gdb tries to
            // treat "--np"/"N" as its own target program.
            if let Some(ref mode) = gdb_mode {
                prterun_args.push("rust-gdb".to_string());
                if mode == "bt" {
                    prterun_args.push("--ex".to_string());
                    prterun_args.push("run".to_string());
                    prterun_args.push("--ex".to_string());
                    prterun_args.push("thread apply all bt full".to_string());
                    prterun_args.push("--ex".to_string());
                    prterun_args.push("quit".to_string());
                }
                prterun_args.push("--args".to_string());
            }

            // After the launcher arguments, add the executable name, then the
            // arguments targeting the application.
            prterun_args.push(exec);

            prterun_args.extend(args.into_iter());

            let mut launcher_cmd = std::process::Command::new(#launcher_path);
            #prefix_env_block
            if time {
                launcher_cmd.env(#time_env, "1");
            }
            #(#extra_forward)*
            #extra_launcher_env
            // Ensure the launched job sees the same threads-per-pe value used to
            // size the launcher's own PE=/--cpus-per-task binding above, even when
            // it was auto-computed here rather than given via --threads-per-pe or
            // an inherited thread-count variable.
            launcher_cmd.env(#threads_env, threads_per_pe.to_string());
            if let Some(ref filter) = log_filter {
                launcher_cmd.env(#log_env, filter);
            }
            #output_dir_block
            println!("Launching with {:?}: {:?} {:?}", #env_var, #launcher_path, prterun_args.join(" "));
            let launcher_status = launcher_cmd
                .args(prterun_args)
                .status()
                .expect("failed to launch process");
            if !launcher_status.success() {
                // propagate the failure (128 + signal for a signalled child)
                #[cfg(unix)]
                let code = {
                    use std::os::unix::process::ExitStatusExt;
                    launcher_status
                        .code()
                        .or_else(|| launcher_status.signal().map(|s| 128 + s))
                        .unwrap_or(1)
                };
                #[cfg(not(unix))]
                let code = launcher_status.code().unwrap_or(1);
                std::process::exit(code);
            }
            #ret
        }
    }
}

/// What goes inside the `else` branch of the generated `main`, around the
/// program's own statements.
pub struct MainBody {
    /// Statements run before the body (after tracing initialisation).
    pub pre: TokenStream,
    /// The tokens of the body itself, evaluated inside a closure whose value
    /// becomes the return value of `main`.
    pub body: TokenStream,
    /// Statements run after the body; `result` holds its value.
    pub post: TokenStream,
}

/// The plain body: the function's statements as they are, plus a wall-clock
/// report when `{env_prefix}_MAIN_TIME` is set (what `--time` does).
pub fn default_main_body(cfg: &LaunchConfig, func: &ItemFn) -> MainBody {
    let stmts = &func.block.stmts;
    let time_env = cfg.env("MAIN_TIME");
    let report = format!("[{}] application time: {{:?}}", cfg.env_prefix);
    MainBody {
        pre: quote! { let __launch_app_start = std::time::Instant::now(); },
        body: quote! { #(#stmts)* },
        post: quote! {
            if std::env::var(#time_env).is_ok() {
                println!(#report, __launch_app_start.elapsed());
            }
        },
    }
}

/// Rewrites `func` (a plain `fn main()`) into one that re-executes itself
/// under the launcher when it is not already running inside one, and
/// otherwise runs `body`.
pub fn expand_main(cfg: &LaunchConfig, func: &ItemFn, body: MainBody) -> TokenStream {
    let ret = check_main_signature(func);
    let ret_type = &func.sig.output;
    let launch = launch_block(cfg, ret);
    let cp = &cfg.crate_path;
    let MainBody { pre, body, post } = body;
    quote! {
        fn main() #ret_type {
            #launch
            else {
                #cp::init_tracing_from_env();
                #pre
                let result = (|| { #body })();
                #post
                result
            }
        }
    }
}

/// The body of the launching `#[test]` wrapper generated by [`expand_test`]:
/// re-invokes the compiled test binary under the launcher, filtered down to
/// just the single launched test.
fn test_block(
    cfg: &LaunchConfig,
    test_attr: &Attribute,
    vis: &syn::Visibility,
    sig: &syn::Signature,
    name: &syn::Ident,
) -> TokenStream {
    let (env_var, launcher_path) = cfg.launcher_info();
    let prefix_env_block = cfg.prefix_env_block();
    quote! {
        #test_attr #vis #sig {
        fn type_name_of_val<T: ?Sized>(_val: &T) -> &'static str {
            std::any::type_name::<T>()
        }

        let full_func_name = type_name_of_val(&#name);
        let func_name = full_func_name
            .split("::")
            .into_iter()
            .skip(1)
            .collect::<Vec<&str>>()
            .join("::");

        let prte_launched = std::env::var(#env_var).is_ok();
        if !prte_launched {
            // Collect command line arguments
            let mut args: Vec<String> = std::env::args().collect();

            // Remove first argument (executable name) and maintain it for later
            let exec = args.remove(0);

            // Prepare arguments for the launcher
            let mut prterun_args = Vec::<String>::new();

            // Collect any additional arguments after "--" to pass to the launcher
            let pos = args.iter().position(|x| x == "--");
            if let Some(pos) = pos {
                args.split_off(pos).into_iter().skip(1).for_each(|x| {
                    prterun_args.push(x.to_string());
                });
            }

            // After the launcher arguments, add the executable name
            prterun_args.push(exec);
            prterun_args.push(func_name);

            // Add the arguments targeting the application
            prterun_args.extend(args.into_iter());
            prterun_args.push("--ignored".to_string());
            prterun_args.push("--exact".to_string());
            let mut launcher_cmd = std::process::Command::new(#launcher_path);
            #prefix_env_block
            let launcher_status = launcher_cmd
                .args(prterun_args)
                .status()
                .expect("failed to launch process");
            assert!(
                launcher_status.success(),
                "launched test failed: {}",
                launcher_status
            );
        }
        }
    }
}

/// Expands `#[test] fn t() {..}` into a `#[test]` wrapper that re-runs the
/// test binary under the launcher, plus the original function renamed
/// `t_launched` and marked `#[ignore]` (the launched processes run only that).
pub fn expand_test(cfg: &LaunchConfig, func: &ItemFn) -> TokenStream {
    assert!(func.sig.asyncness.is_none(), "async not supported");
    assert!(func.sig.constness.is_none(), "const not supported");
    assert!(func.sig.unsafety.is_none(), "unsafety not supported");
    assert!(
        func.sig.generics.lt_token.is_none(),
        "generics not supported"
    );
    assert!(func.sig.variadic.is_none(), "variadics not supported");
    assert!(func.sig.inputs.is_empty(), "inputs not supported");

    let test_attr: Attribute = parse_quote! {
        #[test]
    };
    let name = syn::Ident::new(
        format!("{}_launched", func.sig.ident).as_str(),
        func.sig.ident.span(),
    );
    let wrapper = test_block(cfg, &test_attr, &func.vis, &func.sig, &name);

    let launched_attrs: Vec<Attribute> = parse_quote! {
        #[test]
        #[ignore]
    };
    let mut sig_launched = func.sig.clone();
    sig_launched.ident = name;
    let mut func_copy = func.clone();
    func_copy.sig = sig_launched;
    func_copy.attrs = launched_attrs;

    quote! {
        #wrapper
        #func_copy
    }
}

/// Codegen for patching the launched binary's RPATH/RUNPATH so build-script
/// output shared libraries (found via `LD_LIBRARY_PATH`) are resolvable at
/// runtime, even after the binary has been re-invoked under
/// `prterun`/`srun`.
pub fn binary_update_block_tokens() -> TokenStream {
    quote! {
        {
            let ld_library_path = std::env::var("LD_LIBRARY_PATH").unwrap_or_else(|_| String::new());
            let mut shared_libs_dirs = Vec::new();
            for p in ld_library_path.split(':') {
                // Collect all build output library directories matching the pattern .../target/<profile>/build/.../out/lib
                if p.contains("/target/") && p.contains("/build/") && p.contains("/out/lib") {
                    shared_libs_dirs.push(p.to_string());
                }
            }
            let shared_libs_dir = shared_libs_dirs.join(":");
            // Only touch RPATH if the binary actually has unresolved shared
            // libraries right now. LD_LIBRARY_PATH's build/out/lib entries carry
            // cargo's per-build hash suffix, so string-comparing them against the
            // already-embedded RPATH is unreliable across separate builds
            // (a stale hash never matches, even when the binary already resolves
            // fine via $ORIGIN/shared_libs or another baked-in dir).
            let has_missing_libs = std::env::current_exe()
                .ok()
                .and_then(|p| std::process::Command::new("ldd").arg(p).output().ok())
                .map(|out| {
                    String::from_utf8_lossy(&out.stdout).contains("=> not found")
                })
                .unwrap_or(false);
            if !shared_libs_dir.is_empty() && has_missing_libs {
                if let Ok(exe_path) = std::env::current_exe() {
                    let readelf_out = std::process::Command::new("readelf")
                        .arg("-d")
                        .arg(&exe_path)
                        .output();
                    match readelf_out {
                        Ok(readelf_out) if readelf_out.status.success() => {
                            let readelf_stdout = String::from_utf8_lossy(&readelf_out.stdout);
                            let mut missing_lib_dirs: Vec<String> = Vec::new();
                            let mut existing_rpath = String::new();

                            for line in readelf_stdout.lines() {
                                if line.contains("(RPATH)") || line.contains("(RUNPATH)") {
                                    if let (Some(start), Some(end)) = (line.find('['), line.rfind(']')) {
                                        let cur = line[start + 1..end].trim();
                                        if !cur.is_empty() {
                                            existing_rpath = cur.to_string();
                                            missing_lib_dirs = shared_libs_dir
                                                .split(':')
                                                .filter(|lib_dir| !existing_rpath.contains(lib_dir))
                                                .map(|lib_dir| lib_dir.to_string())
                                                .collect();
                                        }
                                    }
                                }
                            }
                            if existing_rpath.is_empty() {
                                missing_lib_dirs = shared_libs_dir
                                    .split(':')
                                    .map(|lib_dir| lib_dir.to_string())
                                    .collect();
                            }

                            if !missing_lib_dirs.is_empty() {
                                let new_rpath = if existing_rpath.is_empty() {
                                    shared_libs_dir.to_string()
                                } else {
                                    format!("{}:{}", existing_rpath, shared_libs_dir)
                                };

                                let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
                                let temp_exe = exe_dir.join(format!("hpc_launch_exe_{}.tmp", std::process::id()));
                                let temp_exe_str = temp_exe.to_string_lossy().to_string();

                                if let Err(err) = std::fs::copy(&exe_path, &temp_exe) {
                                    eprintln!(
                                        "hpc_launch: failed to copy binary from {:?} to {}: {}",
                                        exe_path,
                                        temp_exe_str,
                                        err
                                    );
                                    eprintln!(
                                        "hpc_launch: unable to add missing RPATH entries {:?}; manually setting LD_LIBRARY_PATH may be needed.",
                                        missing_lib_dirs
                                    );
                                } else {
                                    let timer = std::time::Instant::now();
                                    while let Ok(false) = std::fs::exists(&temp_exe) {
                                        std::thread::sleep(std::time::Duration::from_millis(100));
                                        if timer.elapsed() > std::time::Duration::from_secs(1) {
                                            eprintln!(
                                                "hpc_launch: timeout waiting for temp binary {:?} to appear after copying: {}",
                                                temp_exe,
                                                temp_exe_str
                                            );
                                            break;
                                        }
                                    }
                                    let _ = std::process::Command::new("chmod")
                                        .arg("+x")
                                        .arg(&temp_exe_str)
                                        .status();

                                    let patch_status = std::process::Command::new("patchelf")
                                        .arg("--set-rpath")
                                        .arg(&new_rpath)
                                        .arg(&temp_exe_str)
                                        .status();

                                    match patch_status {
                                        Ok(status) if status.success() => {
                                            if let Err(err) = std::fs::rename(&temp_exe, &exe_path) {
                                                eprintln!(
                                                    "hpc_launch: failed to replace original binary {:?} with patched copy: {}",
                                                    exe_path,
                                                    err
                                                );
                                                eprintln!(
                                                    "hpc_launch: unable to add missing RPATH entries {:?}; manually setting LD_LIBRARY_PATH may be needed.",
                                                    missing_lib_dirs
                                                );
                                                let _ = std::fs::remove_file(&temp_exe);
                                            }
                                        }
                                        Ok(status) => {
                                            eprintln!(
                                                "hpc_launch: patchelf failed with status {} for {:?}",
                                                status,
                                                temp_exe_str
                                            );
                                            eprintln!(
                                                "hpc_launch: unable to add missing RPATH entries {:?}; manually setting LD_LIBRARY_PATH may be needed.",
                                                missing_lib_dirs
                                            );
                                            let _ = std::fs::remove_file(&temp_exe);
                                        }
                                        Err(err) => {
                                            eprintln!(
                                                "hpc_launch: failed to run patchelf for {:?}: {}",
                                                temp_exe_str,
                                                err
                                            );
                                            eprintln!(
                                                "hpc_launch: unable to add missing RPATH entries {:?}; manually setting LD_LIBRARY_PATH may be needed.",
                                                missing_lib_dirs
                                            );
                                            let _ = std::fs::remove_file(&temp_exe);
                                        }
                                    }
                                }
                            }
                        }
                        Ok(status_out) => {
                            eprintln!(
                                "hpc_launch: readelf -d failed for {:?} with status {}",
                                exe_path,
                                status_out.status
                            );
                            eprintln!(
                                "hpc_launch: unable to check/update RPATH; candidate library dirs {:?} may need to be added manually via LD_LIBRARY_PATH.",
                                shared_libs_dir
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "hpc_launch: failed to run readelf for {:?}: {}",
                                exe_path,
                                err
                            );
                            eprintln!(
                                "hpc_launch: unable to check/update RPATH; candidate library dirs {:?} may need to be added manually via LD_LIBRARY_PATH.",
                                shared_libs_dir
                            );
                        }
                    }
                } else {
                    eprintln!(
                        "hpc_launch: unable to determine current executable path; unable to update RPATH. Candidate library dirs {:?} may need to be added manually via LD_LIBRARY_PATH.",
                        shared_libs_dir
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfgs() -> Vec<LaunchConfig> {
        let mut out = Vec::new();
        for launcher in [Launcher::Prterun, Launcher::Srun] {
            for with_salloc in [false, true] {
                let mut c = LaunchConfig::new("hpc_launch", "HPC_LAUNCH", quote! {::hpc_launch}, launcher, with_salloc);
                out.push(c.clone());
                c.extra_flags.push(ExtraFlag {
                    flag: "--lamellae",
                    local: "lamellae",
                    env_var: "LAMELLAR_BACKEND",
                });
                c.help_extra = quote! { println!("  --lamellae <name>"); };
                c.extra_launcher_env = quote! { let _ = &lamellae; };
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn launch_block_parses_as_expression() {
        for cfg in cfgs() {
            for ret in [None, Some(quote! {Ok(())})] {
                let ts = launch_block(&cfg, ret);
                syn::parse2::<syn::ItemFn>(quote! { fn f() { #ts } })
                    .expect("launch block should parse");
            }
        }
    }

    #[test]
    fn launch_block_uses_prefix_and_tool_name() {
        let cfg = &cfgs()[1];
        let s = launch_block(cfg, None).to_string();
        for needle in ["HPC_LAUNCH_NODES", "HPC_LAUNCH_PES", "HPC_LAUNCH_PES_PER_NODE", "HPC_LAUNCH_THREADS", "HPC_LAUNCH_LOG", "HPC_LAUNCH_MAIN_TIME", "hpc_launch: resolved launch parameters", "LAMELLAR_BACKEND"] {
            assert!(s.contains(needle), "missing {needle}");
        }
        assert!(!s.contains("LAMELLAR_NODES"));
    }

    #[test]
    fn expand_main_and_test_parse() {
        let func: ItemFn = syn::parse_quote! { fn main() -> Result<(), String> { println!("hi"); Ok(()) } };
        for cfg in cfgs() {
            let body = default_main_body(&cfg, &func);
            let ts = expand_main(&cfg, &func, body);
            syn::parse2::<syn::ItemFn>(ts).expect("expanded main should be an fn");
            let t: ItemFn = syn::parse_quote! { fn my_test() { assert!(true); } };
            let ts = expand_test(&cfg, &t);
            syn::parse2::<syn::File>(ts).expect("expanded test should be items");
        }
    }

    #[test]
    fn pes_only_defaults_pes_per_node_for_prterun_only() {
        let prterun = LaunchConfig::new("hpc_launch", "HPC_LAUNCH", quote! {::hpc_launch}, Launcher::Prterun, false);
        let srun = LaunchConfig::new("hpc_launch", "HPC_LAUNCH", quote! {::hpc_launch}, Launcher::Srun, false);
        let p = launch_block(&prterun, None).to_string();
        assert!(p.contains("SLURM_JOB_NUM_NODES"));
        assert!(p.contains("(Some (p) , None , None)"));
        assert!(!launch_block(&srun, None).to_string().contains("SLURM_JOB_NUM_NODES"));
    }

    #[test]
    fn srun_test_detection_matches_main() {
        let cfg = LaunchConfig::new("hpc_launch", "HPC_LAUNCH", quote! {::hpc_launch}, Launcher::Srun, false);
        let t: ItemFn = syn::parse_quote! { fn my_test() {} };
        let s = expand_test(&cfg, &t).to_string();
        assert!(s.contains("SLURM_LOCALID"));
        assert!(!s.contains("SRUN_LAUNCHED"));
    }
}
