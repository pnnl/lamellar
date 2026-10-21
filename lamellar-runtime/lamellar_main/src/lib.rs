//! Lamellar-specific launcher wrapper around [`hpc_launch`], providing
//! `#[lamellar_main::main]` / `#[lamellar_main::test]`. Adds
//! `--lamellae`/`--cmd-queue`/`--batcher` launch flags, lamellar's backend
//! list in `--help`, backend-aware PE defaulting, and `enable-prof` support
//! on top of the generic launch support `hpc_launch` provides standalone.
//!
//! Most users get this for free via `lamellar`'s `enable-lamellar-main`
//! feature, which reexports [`main`]/[`test`] as `#[lamellar::main]`/
//! `#[lamellar::test]`.

pub use lamellar_main_impl::{main, test};

pub use hpc_launch::{
    core_count, cores_per_package, explicit_cores_per_pe, numa_domain_count, package_count, pe_id,
    pu_count, resolve_pe_defaults,
};

/// Base `--help` body text for the generic launch flags, naming the
/// `LAMELLAR_*` environment variables behind them.
pub fn generic_help_text() -> String {
    hpc_launch::generic_help_text_with_prefix("LAMELLAR")
}

/// Initializes a `tracing-subscriber` global subscriber from the
/// `LAMELLAR_LOG` environment variable, if set. Mirrors `RUST_LOG` syntax.
pub fn init_tracing_from_env() {
    hpc_launch::init_tracing_from_env_var("LAMELLAR_LOG");
}

#[cfg(feature = "use-prterun")]
pub use hpc_launch::prrte_sys;
