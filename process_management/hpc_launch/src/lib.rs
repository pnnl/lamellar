//! Support crate for `#[hpc_launch::main]` / `#[hpc_launch::test]`.
//!
//! This crate can be used standalone (without depending on the `lamellar`
//! runtime crate) to get a self-launching binary via `prterun`/`srun`. It
//! also reexports [`prrte_sys`] and provides best-effort hardware-topology
//! and tracing-init helpers used by the generated launch code. The generated
//! code itself comes from `hpc_launch_codegen`, via `hpc_launch_impl`.

pub use hpc_launch_impl::main;
pub use hpc_launch_impl::test;

#[cfg(feature = "use-prterun")]
pub use prrte_sys;

/// Initializes a `tracing-subscriber` global subscriber from the
/// `HPC_LAUNCH_LOG` environment variable, if set. Mirrors `RUST_LOG` syntax.
/// See [`init_tracing_from_env_var`] to read a differently named variable.
pub fn init_tracing_from_env() {
    init_tracing_from_env_var("HPC_LAUNCH_LOG");
}

/// Initializes a `tracing-subscriber` global subscriber from the environment
/// variable `var`, if set (for example `LAMELLAR_LOG`). Mirrors `RUST_LOG`
/// syntax. A no-op if `var` is unset or empty, or if a global subscriber is
/// already installed.
pub fn init_tracing_from_env_var(var: &str) {
    use tracing_subscriber::prelude::*;

    if let Ok(filter) = std::env::var(var) {
        if !filter.trim().is_empty() {
            std::env::set_var("RUST_LOG", &filter);
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_thread_ids(true)
                        .with_file(true)
                        .with_line_number(true)
                        .with_level(true),
                )
                .try_init();
        }
    }
}

/// Base `--help` body text (generic launch flags only, no header/footer) for
/// binaries built with `#[hpc_launch::main]`, naming the `HPC_LAUNCH_*`
/// environment variables. Callers wrap this with their own header line,
/// `--help`/`-h` line, and trailing "passed through" line, and may append
/// their own additional flag lines after it.
pub fn generic_help_text() -> String {
    generic_help_text_with_prefix("HPC_LAUNCH")
}

/// Like [`generic_help_text`], naming `{prefix}_NODES`, `{prefix}_PES`, ...
/// as the environment variables behind each flag.
pub fn generic_help_text_with_prefix(prefix: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!("  --nodes <n>            number of nodes to allocate/launch across [env: {prefix}_NODES]\n"));
    s.push_str(&format!("  --pes <n>              total number of PEs to launch [env: {prefix}_PES]\n"));
    s.push_str(&format!("  --pes-per-node <n>     PEs to launch per node [env: {prefix}_PES_PER_NODE]\n"));
    s.push_str(&format!("  --threads-per-pe <n>   worker threads per PE [env: {prefix}_THREADS]\n"));
    s.push_str(&format!("  --log <filter>         tracing filter for the launched job (RUST_LOG syntax) [env: {prefix}_LOG]\n"));
    if cfg!(feature = "with-salloc") {
        s.push_str("  --salloc-opts <opts>   extra options passed to `salloc` when requesting an allocation; takes every\n");
        s.push_str("                         following argument up to a `--` terminator, and splits quoted values on whitespace\n");
    }
    s.push_str("  --output-dir <dir>     directory to write per-PE stdout/stderr into\n");
    s.push_str(&format!("  --time                 report wall-clock application time [env: {prefix}_MAIN_TIME]\n"));
    s.push_str("  --gdb [bt]             launch each PE under `rust-gdb` (with `bt`: auto backtrace on crash)\n");
    s
}

/// Resolves `--pes`/`--pes-per-node` defaults when neither was given on the
/// command line. If `nodes` is known, splits one PE per NUMA domain across
/// all nodes; otherwise, if `default_single_pe` is set, defaults to a single
/// PE; otherwise defaults to one PE per NUMA domain on the local host.
pub fn resolve_pe_defaults(
    nodes: Option<u32>,
    numa_domains: Option<u32>,
    default_single_pe: bool,
) -> (Option<u32>, Option<u32>) {
    if let Some(n) = nodes {
        let ppn = numa_domains.unwrap_or(1).max(1);
        (Some(n * ppn), Some(ppn))
    } else if default_single_pe {
        (Some(1), Some(1))
    } else {
        let ppn = numa_domains.unwrap_or(1).max(1);
        (Some(ppn), Some(ppn))
    }
}

/// Whether this process was already started by a launcher, so the generated `main` should run the program body rather
/// than re-execute itself under `prterun`/`srun`.
///
/// True when `own_env_var` (the variable the launcher this crate would use sets on its children: `PRTE_LAUNCHED` or
/// `SLURM_LOCALID`) is set, or when a per-task rank variable of another launcher is: `FLUX_TASK_RANK` (`flux run`),
/// `PMIX_RANK` (any PMIx launcher: `prterun`, Open MPI `mpirun`, `srun --mpi=pmix`, `flux run -o pmi=pmix`), `PMI_RANK`
/// (PMI-1 and PMI-2: `srun`, `mpiexec`, Flux's default) or `OMPI_COMM_WORLD_RANK`. Those launchers already chose the
/// number of processes and where they run, and provide the process-management service, so launching again from inside
/// one would start a second, conflicting job.
pub fn already_launched(own_env_var: &str) -> bool {
    const OTHER_LAUNCHERS: [&str; 4] = ["FLUX_TASK_RANK", "PMIX_RANK", "PMI_RANK", "OMPI_COMM_WORLD_RANK"];
    std::env::var_os(own_env_var).is_some() || OTHER_LAUNCHERS.iter().any(|v| std::env::var_os(v).is_some())
}

/// Returns the current process's PE (rank) id, if known, without requiring
/// any `LamellarWorld`/team object to have been created first.
///
/// Reads `PMIX_RANK` (set on every launched child by PRRTE's vendored PMIx
/// server via `PMIx_server_setup_fork`, regardless of whether the process
/// goes on to call into PMIx itself), falling back to `PMI_RANK` (PMI1/PMI2
/// convention, set by launchers using those older protocols instead of
/// PMIx) and then `SLURM_PROCID` (`srun`-launched jobs). Returns `None` if
/// launched under none of these (e.g. running standalone, pre-launch, or
/// under an unrecognized launcher).
pub fn pe_id() -> Option<u32> {
    std::env::var("PMIX_RANK")
        .ok()
        .or_else(|| std::env::var("PMI_RANK").ok())
        .or_else(|| std::env::var("SLURM_PROCID").ok())
        .and_then(|s| s.parse().ok())
}

/// Best-effort count of NUMA domains on the current host, via a `/sys` scan.
/// Returns `None` if the count can't be determined.
pub fn numa_domain_count() -> Option<u32> {
    numa_domain_count_fs()
}

fn numa_domain_count_fs() -> Option<u32> {
    let mut count = 0u32;
    for entry in std::fs::read_dir("/sys/devices/system/node")
        .ok()?
        .flatten()
    {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let rest = match name.strip_prefix("node") {
            Some(r) => r,
            None => continue,
        };
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            count += 1;
        }
    }
    if count == 0 {
        None
    } else {
        Some(count)
    }
}

/// Best-effort count of physical packages (sockets) on the current host,
/// via a `/sys` scan.
pub fn package_count() -> Option<u32> {
    package_count_fs()
}

fn package_count_fs() -> Option<u32> {
    use std::collections::HashSet;
    let mut ids = HashSet::new();
    for entry in std::fs::read_dir("/sys/devices/system/cpu").ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let rest = match name.strip_prefix("cpu") {
            Some(r) => r,
            None => continue,
        };
        if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let path = entry.path().join("topology/physical_package_id");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(id) = content.trim().parse::<u32>() {
                ids.insert(id);
            }
        }
    }
    if ids.is_empty() {
        None
    } else {
        Some(ids.len() as u32)
    }
}

/// Best-effort count of physical cores on the current host (i.e. excluding
/// hyperthreads/SMT siblings), via a `/sys` scan.
pub fn core_count() -> Option<u32> {
    core_count_fs()
}

fn core_count_fs() -> Option<u32> {
    use std::collections::HashSet;
    let mut ids = HashSet::new();
    for entry in std::fs::read_dir("/sys/devices/system/cpu").ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let rest = match name.strip_prefix("cpu") {
            Some(r) => r,
            None => continue,
        };
        if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let path = entry.path();
        let package_id = std::fs::read_to_string(path.join("topology/physical_package_id"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());
        let core_id = std::fs::read_to_string(path.join("topology/core_id"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());
        if let (Some(pkg), Some(core)) = (package_id, core_id) {
            ids.insert((pkg, core));
        }
    }
    if ids.is_empty() {
        None
    } else {
        Some(ids.len() as u32)
    }
}

/// Best-effort core count within a single physical package (socket) on the
/// current host, assuming packages have a uniform core count. Returns `None`
/// if either `core_count` or `package_count` can't be determined.
pub fn cores_per_package() -> Option<u32> {
    let cores = core_count()?;
    let packages = package_count()?;
    if packages == 0 {
        return None;
    }
    Some((cores / packages).max(1))
}

/// Threads per PE (and, under prterun, the cores `PE=` binds each PE to) when the user gave no count: the node's `cores` shared
/// by the `pes_per_node` PEs on it. A bound PE must not straddle two packages (PRRTE refuses: "bound to CPUs in more than one
/// package"), and prterun hands the cores out in order, so when the plain division leaves a PE across a package boundary (3 PEs
/// on two 32-core packages: 21 each puts the second PE on cores 21-41) the count is lowered until every package holds a whole
/// number of PEs and the packages together hold all of them (16 each: cores 0-15 and 16-31 in the first package, 32-47 in the
/// second). `packages` is the number of packages (`None` when unknown, which keeps the plain division). A PE bigger than a
/// package is returned unchanged: the caller already leaves such a PE unbound.
pub fn default_threads_per_pe(cores: u32, pes_per_node: u32, packages: Option<u32>) -> u32 {
    let pes = pes_per_node.max(1);
    let mut per_pe = (cores / pes).max(1);
    let packages = match packages {
        Some(p) if p > 1 => p,
        _ => return per_pe,
    };
    let per_package = cores / packages;
    if per_pe > per_package {
        return per_pe;
    }
    while per_pe > 1 && packages * (per_package / per_pe) < pes {
        per_pe -= 1;
    }
    per_pe
}

#[cfg(test)]
mod default_threads_per_pe_tests {
    use super::default_threads_per_pe;

    #[test]
    fn divides_evenly_when_that_fits_the_packages() {
        // 64 cores in 2 packages of 32
        assert_eq!(default_threads_per_pe(64, 2, Some(2)), 32);
        assert_eq!(default_threads_per_pe(64, 4, Some(2)), 16);
        assert_eq!(default_threads_per_pe(64, 8, Some(2)), 8);
        assert_eq!(default_threads_per_pe(64, 64, Some(2)), 1);
    }

    #[test]
    fn does_not_straddle_a_package_boundary() {
        // 64 / 3 = 21 would put the second PE on cores 21-41, across the boundary at 32
        assert_eq!(default_threads_per_pe(64, 3, Some(2)), 16);
        assert_eq!(default_threads_per_pe(64, 5, Some(2)), 10);
        assert_eq!(default_threads_per_pe(64, 7, Some(2)), 8);
        // four packages of 16: 5 PEs need 2 per package
        assert_eq!(default_threads_per_pe(64, 5, Some(4)), 8);
    }

    #[test]
    fn every_pe_fits_in_one_package() {
        for packages in [2u32, 4] {
            for cores in [32u32, 48, 64, 96, 128] {
                if cores % packages != 0 {
                    continue;
                }
                for pes in 1..=cores {
                    let per_pe = default_threads_per_pe(cores, pes, Some(packages));
                    let per_package = cores / packages;
                    if cores / pes > per_package {
                        continue; // a PE wider than a package is left to the caller
                    }
                    assert!(per_pe >= 1 && per_pe <= per_package, "{cores} cores, {packages} packages, {pes} PEs -> {per_pe}");
                    assert!(
                        packages * (per_package / per_pe) >= pes,
                        "{cores} cores, {packages} packages, {pes} PEs -> {per_pe} leaves PEs across a boundary"
                    );
                }
            }
        }
    }

    #[test]
    fn plain_division_without_package_information() {
        assert_eq!(default_threads_per_pe(64, 3, None), 21);
        assert_eq!(default_threads_per_pe(64, 3, Some(1)), 21);
        assert_eq!(default_threads_per_pe(64, 0, Some(2)), 64);
    }

    #[test]
    fn a_pe_wider_than_a_package_is_unchanged() {
        assert_eq!(default_threads_per_pe(64, 1, Some(2)), 64);
    }
}

/// Cores per PE requested by an explicit binding among `args`, the raw launcher
/// arguments (headed for prterun/srun), if there is one: the `PE=<n>` modifier of a
/// `--map-by` value (`--map-by node:PE=4` or `--map-by=node:PE=4`), or
/// `--cpus-per-task <n>` / `--cpus-per-task=<n>`. `-c` is not recognised since
/// prterun/mpirun read it as `-np`.
///
/// The launcher sizes a PE's worker threads from this so the thread count matches the
/// cores the PE is bound to; otherwise an unbound default (all the node's cores) runs
/// many busy-polling threads on a few cores.
pub fn explicit_cores_per_pe(args: &[String]) -> Option<u32> {
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let map_by = if arg == "--map-by" {
            it.next().map(String::as_str)
        } else {
            arg.strip_prefix("--map-by=")
        };
        if let Some(value) = map_by {
            let cores = value
                .split(|c| c == ':' || c == ',')
                .find_map(|part| part.strip_prefix("PE="))
                .and_then(|n| n.parse::<u32>().ok())
                .filter(|&n| n > 0);
            if cores.is_some() {
                return cores;
            }
            continue;
        }
        let cpus_per_task = if arg == "--cpus-per-task" {
            it.next().map(String::as_str)
        } else {
            arg.strip_prefix("--cpus-per-task=")
        };
        if let Some(n) = cpus_per_task.and_then(|n| n.parse::<u32>().ok()).filter(|&n| n > 0) {
            return Some(n);
        }
    }
    None
}

/// Best-effort count of processing units (logical CPUs, including SMT
/// siblings) on the current host, via a `/sys` scan.
pub fn pu_count() -> Option<u32> {
    pu_count_fs()
}

fn pu_count_fs() -> Option<u32> {
    let mut count = 0u32;
    for entry in std::fs::read_dir("/sys/devices/system/cpu").ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let rest = match name.strip_prefix("cpu") {
            Some(r) => r,
            None => continue,
        };
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            count += 1;
        }
    }
    if count == 0 {
        None
    } else {
        Some(count)
    }
}

#[cfg(test)]
mod explicit_cores_per_pe_tests {
    use super::explicit_cores_per_pe;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn map_by_pe_modifier() {
        assert_eq!(explicit_cores_per_pe(&args(&["--map-by", "node:PE=4", "-np", "2"])), Some(4));
        assert_eq!(explicit_cores_per_pe(&args(&["--map-by=node:PE=8"])), Some(8));
        assert_eq!(explicit_cores_per_pe(&args(&["--map-by", "ppr:2:node:PE=16:HWTCPUS"])), Some(16));
    }

    #[test]
    fn cpus_per_task() {
        assert_eq!(explicit_cores_per_pe(&args(&["--cpus-per-task", "6"])), Some(6));
        assert_eq!(explicit_cores_per_pe(&args(&["--cpus-per-task=12"])), Some(12));
    }

    #[test]
    fn no_binding_or_unusable_values() {
        assert_eq!(explicit_cores_per_pe(&args(&[])), None);
        assert_eq!(explicit_cores_per_pe(&args(&["--map-by", "node", "--bind-to", "none"])), None);
        assert_eq!(explicit_cores_per_pe(&args(&["--map-by", "node:PE=0"])), None);
        assert_eq!(explicit_cores_per_pe(&args(&["--map-by", "node:PE=x"])), None);
        // `-c` is `-np` for prterun/mpirun, not a core count
        assert_eq!(explicit_cores_per_pe(&args(&["-c", "4"])), None);
    }

    #[test]
    fn map_by_without_pe_does_not_hide_a_later_binding() {
        assert_eq!(
            explicit_cores_per_pe(&args(&["--map-by", "node", "--cpus-per-task", "3"])),
            Some(3)
        );
    }
}

#[cfg(test)]
mod already_launched_tests {
    use super::already_launched;

    // one test, because it changes the process environment
    #[test]
    fn recognizes_launchers() {
        let vars = ["HPC_LAUNCH_TEST_OWN", "FLUX_TASK_RANK", "PMIX_RANK", "PMI_RANK", "OMPI_COMM_WORLD_RANK"];
        for v in vars {
            std::env::remove_var(v);
        }
        assert!(!already_launched("HPC_LAUNCH_TEST_OWN"));
        for v in vars {
            std::env::set_var(v, "0");
            assert!(already_launched("HPC_LAUNCH_TEST_OWN"), "{v} should count as launched");
            std::env::remove_var(v);
        }
        assert!(!already_launched("HPC_LAUNCH_TEST_OWN"));
    }
}
