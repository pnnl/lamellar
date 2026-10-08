//! The launcher (`oshrun`, `shmemrun`).
//!
//! ```text
//! oshrun [-np N] [-N|--nodes M] [--pes-per-node P] prog [args...]
//! ```
//!
//! `N = M * P` when both `--nodes` and `--pes-per-node` are given; an inconsistent combination is rejected.
//!
//! Where the PEs start:
//! * **Inside a Slurm allocation** (`SLURM_JOB_ID` set): `srun` for the single-node backends (`shmem`, `shmem-opt`,
//!   `local`), and `srun --mpi=pmix` or PRRTE's `prterun` for the others.
//! * **Outside one:** the single-node backends are started directly as N local processes; other backends use `prterun`
//!   if it is on `PATH`. Set `SHMEM_REQUIRE_ALLOCATION=1` to refuse to run outside an allocation instead (useful where
//!   programs must only run on compute nodes).
//!
//! Environment: `LAMELLAR_BACKEND` / `SHMEM_BACKEND` (backend), `OSHRUN_LAUNCHER` (`srun-pmix` or `prterun`),
//! `OSHRUN_CORES_PER_PE`, `OSHRUN_MAP_BY`, `OSHRUN_BIND`, `OSHRUN_CPUS`, `SHMEM_PROGRESS_THREAD`.

use std::env;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::process::Command;

const USAGE: &str = "usage: oshrun [-np N] [--nodes M] [--pes-per-node P] prog [args]";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opts {
    pub n: usize,
    pub nodes: Option<usize>,
    pub ppn: Option<usize>,
    pub prog: Vec<OsString>,
}

fn number(flag: &str, v: Option<&OsString>) -> Result<usize, String> {
    let v = v.ok_or_else(|| format!("oshrun: {flag} needs a value"))?;
    let s = v.to_string_lossy();
    match s.parse::<usize>() {
        Ok(n) if n >= 1 => Ok(n),
        _ => Err(format!("oshrun: {flag} needs a positive integer, got '{s}'")),
    }
}

/// Parse the launcher's own options, which end at the first argument that is not one of them.
pub fn parse(args: &[OsString]) -> Result<Opts, String> {
    let (mut n, mut np_set, mut nodes, mut ppn) = (1usize, false, None, None);
    let mut i = 0;
    while i < args.len() {
        match args[i].to_string_lossy().as_ref() {
            f @ ("-np" | "-n" | "--np") => {
                n = number(f, args.get(i + 1))?;
                np_set = true;
                i += 2;
            }
            f @ ("-N" | "--nodes") => {
                nodes = Some(number(f, args.get(i + 1))?);
                i += 2;
            }
            f @ ("--pes-per-node" | "-ppn") => {
                ppn = Some(number(f, args.get(i + 1))?);
                i += 2;
            }
            _ => break,
        }
    }
    let prog: Vec<OsString> = args[i..].to_vec();
    if prog.is_empty() {
        return Err(USAGE.to_string());
    }
    if let (Some(m), Some(p)) = (nodes, ppn) {
        if !np_set {
            n = m * p;
        } else if n != m * p {
            return Err(format!(
                "oshrun: inconsistent --nodes {m} / -np {n} / --pes-per-node {p} (expected np == nodes * pes-per-node)"
            ));
        }
    }
    Ok(Opts { n, nodes, ppn, prog })
}

pub fn is_single_node(backend: &str) -> bool {
    matches!(backend, "shmem" | "shmem-opt" | "local")
}

fn env_nonempty(k: &str) -> Option<String> {
    env::var(k).ok().filter(|v| !v.is_empty())
}

fn in_path(prog: &str) -> bool {
    env::var_os("PATH")
        .map(|p| env::split_paths(&p).any(|d| d.join(prog).is_file()))
        .unwrap_or(false)
}

/// An id that differs between launches (so sequential runs never share shared-memory segments).
fn unique_job_id(slurm_job_id: Option<&str>) -> u64 {
    let base = slurm_job_id.and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    (base.wrapping_mul(7).wrapping_add(std::process::id() as u64).wrapping_add(nanos) % 60000) + 1000
}

fn exec(mut cmd: Command, what: &str) -> i32 {
    let err = cmd.exec();
    eprintln!("oshrun: cannot run {what}: {err}");
    127
}

/// Start `n` copies of the program directly, one environment per PE.
fn spawn_local(o: &Opts, backend: &str) -> i32 {
    let job = unique_job_id(None);
    let mut children = Vec::new();
    for pe in 0..o.n {
        let mut c = Command::new(&o.prog[0]);
        c.args(&o.prog[1..])
            .env("LAMELLAR_BACKEND", backend)
            .env("LAMELLAR_NUM_PES", o.n.to_string())
            .env("LAMELLAR_JOB_ID", job.to_string())
            .env("LAMELLAR_PE_ID", pe.to_string());
        match c.spawn() {
            Ok(ch) => children.push(ch),
            Err(e) => {
                eprintln!("oshrun: cannot run {}: {e}", o.prog[0].to_string_lossy());
                for mut ch in children {
                    let _ = ch.kill();
                }
                return 127;
            }
        }
    }
    let mut rc = 0;
    for mut ch in children {
        match ch.wait() {
            Ok(st) if !st.success() => rc = st.code().unwrap_or(1),
            Err(_) => rc = 1,
            _ => {}
        }
    }
    rc
}

fn prterun_cores_per_pe() -> String {
    env_nonempty("OSHRUN_CORES_PER_PE").unwrap_or_else(|| {
        // one core for the application thread, plus one for the optional progress thread
        match env::var("SHMEM_PROGRESS_THREAD").as_deref() {
            Ok("0") | Ok("") | Err(_) => "1".into(),
            Ok(_) => "2".into(),
        }
    })
}

pub fn main() -> i32 {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let o = match parse(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let backend = env_nonempty("LAMELLAR_BACKEND").or_else(|| env_nonempty("SHMEM_BACKEND")).unwrap_or_default();
    let single = is_single_node(&backend);
    if backend == "local" && o.n != 1 {
        eprintln!("oshrun: backend local supports one PE");
        return 2;
    }
    if single && o.nodes.map_or(false, |m| m != 1) {
        eprintln!(
            "oshrun: backend '{backend}' is single-node; use libfabric/ucx/rofi for --nodes {}",
            o.nodes.unwrap_or(1)
        );
        return 2;
    }

    let slurm_job = env_nonempty("SLURM_JOB_ID");
    let Some(slurm_job) = slurm_job else {
        if env::var("SHMEM_REQUIRE_ALLOCATION").as_deref() == Ok("1") {
            let host = Command::new("hostname")
                .output()
                .ok()
                .map(|r| String::from_utf8_lossy(&r.stdout).trim().to_string())
                .unwrap_or_default();
            eprintln!(
                "oshrun: refusing to run on {host}: not inside a Slurm allocation (SHMEM_REQUIRE_ALLOCATION=1).\n  \
                 For example: salloc -N1 -t 30 oshrun -np 4 ./a.out"
            );
            return 2;
        }
        if single {
            return spawn_local(&o, &backend);
        }
        if !in_path("prterun") {
            eprintln!("oshrun: no launcher found (backend '{backend}' needs prterun or a Slurm allocation)");
            return 1;
        }
        let mut c = Command::new("prterun");
        c.args(["-n", &o.n.to_string(), "--oversubscribe"]).args(&o.prog);
        return exec(c, "prterun");
    };

    // Inside an allocation: srun places the tasks on the allocated nodes.
    if !in_path("srun") {
        eprintln!("oshrun: SLURM_JOB_ID is set but srun was not found");
        return 1;
    }
    if single {
        let mut c = Command::new("srun");
        c.args(["--nodes=1", &format!("--ntasks={}", o.n), "--kill-on-bad-exit=1"])
            .args(["bash", "-c", "LAMELLAR_PE_ID=$SLURM_PROCID exec \"$@\"", "oshrun-task"])
            .args(&o.prog)
            .env("LAMELLAR_BACKEND", &backend)
            .env("LAMELLAR_NUM_PES", o.n.to_string())
            .env("LAMELLAR_JOB_ID", unique_job_id(Some(&slurm_job)).to_string());
        return exec(c, "srun");
    }

    let launcher = env_nonempty("OSHRUN_LAUNCHER").unwrap_or_else(|| {
        let pmix = Command::new("srun")
            .arg("--mpi=list")
            .output()
            .map(|r| {
                let text = format!("{}{}", String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr));
                text.split(|c: char| !c.is_alphanumeric() && c != '_').any(|w| w == "pmix")
            })
            .unwrap_or(false);
        if pmix {
            "srun-pmix".into()
        } else if in_path("prterun") {
            "prterun".into()
        } else {
            String::new()
        }
    });
    match launcher.as_str() {
        "srun-pmix" => {
            let mut c = Command::new("srun");
            c.arg("--mpi=pmix");
            if let Some(m) = o.nodes {
                c.arg(format!("--nodes={m}"));
            }
            if let Some(p) = o.ppn {
                c.arg(format!("--ntasks-per-node={p}"));
            }
            c.arg(format!("--ntasks={}", o.n)).arg("--kill-on-bad-exit=1").args(&o.prog);
            exec(c, "srun")
        }
        "prterun" => {
            let have_nodelist = env_nonempty("SLURM_JOB_NODELIST").is_some() || env_nonempty("SLURM_NODELIST").is_some();
            if have_nodelist {
                // Give every PE its own cores: prterun's default binds per NUMA domain, so co-located PEs would
                // share a small set of cores. PE=<k> reserves k cores per PE.
                let cpp = prterun_cores_per_pe();
                let mapby = env_nonempty("OSHRUN_MAP_BY").unwrap_or_else(|| match o.ppn {
                    Some(p) => format!("ppr:{p}:node:PE={cpp}"),
                    None => format!("slot:PE={cpp}"),
                });
                let bind = env_nonempty("OSHRUN_BIND").unwrap_or_else(|| "core".into());
                let mut c = Command::new("prterun");
                c.args(["-n", &o.n.to_string(), "--map-by", &mapby, "--bind-to", &bind]).args(&o.prog);
                return exec(c, "prterun");
            }
            // Only a job id is known (no node list): run prterun as one srun task on a compute node, which forks
            // the ranks locally. Single node only.
            if o.nodes.map_or(false, |m| m != 1) {
                eprintln!(
                    "oshrun: multi-node needs the allocation environment; run under salloc: salloc -N{} oshrun -np {} ...",
                    o.nodes.unwrap_or(1),
                    o.n
                );
                return 2;
            }
            let cpus = env_nonempty("OSHRUN_CPUS").or_else(|| env_nonempty("SLURM_CPUS_ON_NODE")).unwrap_or_else(|| "1".into());
            let mut c = Command::new("srun");
            c.args(["--nodes=1", "--ntasks=1", &format!("--cpus-per-task={cpus}")])
                .args(["prterun", "-n", &o.n.to_string(), "--oversubscribe"])
                .args(&o.prog);
            exec(c, "srun")
        }
        "" => {
            eprintln!("oshrun: no PMI(x) launcher available for backend '{backend}'");
            1
        }
        other => {
            eprintln!("oshrun: unknown OSHRUN_LAUNCHER '{other}'");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(a: &[&str]) -> Vec<OsString> {
        a.iter().map(OsString::from).collect()
    }

    #[test]
    fn plain_program() {
        let o = parse(&v(&["./a.out", "x"])).unwrap();
        assert_eq!((o.n, o.nodes, o.ppn), (1, None, None));
        assert_eq!(o.prog, v(&["./a.out", "x"]));
    }

    #[test]
    fn np_flags_and_options_end_at_the_program() {
        let o = parse(&v(&["-np", "4", "./a.out", "-np", "9"])).unwrap();
        assert_eq!(o.n, 4);
        assert_eq!(o.prog, v(&["./a.out", "-np", "9"]));
    }

    #[test]
    fn nodes_times_pes_per_node_sets_np() {
        let o = parse(&v(&["-N", "2", "--pes-per-node", "8", "p"])).unwrap();
        assert_eq!((o.n, o.nodes, o.ppn), (16, Some(2), Some(8)));
    }

    #[test]
    fn inconsistent_counts_are_rejected() {
        let e = parse(&v(&["-np", "5", "-N", "2", "-ppn", "8", "p"])).unwrap_err();
        assert!(e.contains("inconsistent"), "{e}");
        assert!(parse(&v(&["-np", "16", "-N", "2", "-ppn", "8", "p"])).is_ok());
    }

    #[test]
    fn missing_program_or_value_is_an_error() {
        assert_eq!(parse(&v(&[])).unwrap_err(), USAGE);
        assert_eq!(parse(&v(&["-np", "4"])).unwrap_err(), USAGE);
        assert!(parse(&v(&["-np"])).unwrap_err().contains("needs a value"));
        assert!(parse(&v(&["-np", "x", "p"])).unwrap_err().contains("positive integer"));
        assert!(parse(&v(&["-np", "0", "p"])).unwrap_err().contains("positive integer"));
    }

    #[test]
    fn single_node_backends() {
        for b in ["shmem", "shmem-opt", "local"] {
            assert!(is_single_node(b), "{b}");
        }
        for b in ["", "ucx", "ucx-opt", "libfabric", "libfabric-sys-opt", "rofi"] {
            assert!(!is_single_node(b), "{b}");
        }
    }

    #[test]
    fn job_id_range() {
        let id = unique_job_id(Some("174883"));
        assert!((1000..61000).contains(&id));
    }
}
