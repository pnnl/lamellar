#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Environment variable holding the full path of the `prterun` executable.
/// Takes precedence over every other lookup.
pub const PRTERUN_ENV: &str = "LAMELLAR_PRTERUN";

/// Environment variable holding a PRRTE `bin` directory (the directory that
/// contains `prterun`), checked after [`PRTERUN_ENV`].
pub const PRTE_BIN_DIR_ENV: &str = "PRRTE_BIN_DIR";

/// Where this crate's build script put `prterun`. This is an absolute path
/// into the build tree, so it is only a default: it is wrong once the binary
/// is deployed to another machine, or after `cargo clean`.
const BUILD_TIME_PRTERUN: &str = concat!(env!("OUT_DIR"), "/bin/prterun");

fn is_file(p: &Path) -> bool {
    p.is_file()
}

/// Candidate locations for `prterun`, in the order they are tried, each with
/// a description of where it came from.
fn candidates() -> Vec<(PathBuf, String)> {
    let mut c: Vec<(PathBuf, String)> = Vec::new();
    if let Some(v) = std::env::var_os(PRTERUN_ENV).filter(|v| !v.is_empty()) {
        c.push((PathBuf::from(v), format!("{PRTERUN_ENV} environment variable")));
    }
    if let Some(v) = std::env::var_os(PRTE_BIN_DIR_ENV).filter(|v| !v.is_empty()) {
        c.push((
            PathBuf::from(v).join("prterun"),
            format!("{PRTE_BIN_DIR_ENV} environment variable"),
        ));
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    {
        c.push((dir.join("prterun"), "next to the running executable".to_string()));
        c.push((
            dir.join("../bin/prterun"),
            "<exe dir>/../bin of the running executable".to_string(),
        ));
    }
    c.push((
        PathBuf::from(BUILD_TIME_PRTERUN),
        "build-time location (OUT_DIR)".to_string(),
    ));
    c
}

/// Locate `prterun` at run time.
///
/// Tried in order: `$LAMELLAR_PRTERUN` (full path), `$PRRTE_BIN_DIR/prterun`,
/// next to / one level above the running executable, the location baked in
/// at build time, then a `PATH` search. On failure the error lists every
/// location that was checked.
pub fn find_prterun() -> Result<PathBuf, String> {
    let cands = candidates();
    if let Some((p, _)) = cands.iter().find(|(p, _)| is_file(p)) {
        return Ok(p.clone());
    }
    if let Some(path) = std::env::var_os("PATH") {
        if let Some(p) = std::env::split_paths(&path)
            .map(|d| d.join("prterun"))
            .find(|p| is_file(p))
        {
            return Ok(p);
        }
    }
    let mut msg = String::from("could not find `prterun`; looked in:\n");
    for (p, why) in &cands {
        msg.push_str(&format!("  - {} ({why})\n", p.display()));
    }
    msg.push_str("  - $PATH\n");
    msg.push_str(&format!(
        "Set {PRTERUN_ENV} to the full path of prterun, or {PRTE_BIN_DIR_ENV} to its directory."
    ));
    Err(msg)
}

/// Path to `prterun`, for use as a `Command` program.
///
/// Resolved at run time (see [`find_prterun`]). If nothing is found, a
/// diagnostic listing the locations tried is printed to stderr and the bare
/// name `prterun` is returned so the eventual spawn error still names it.
pub fn prterun_path() -> PathBuf {
    match find_prterun() {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("prrte-sys: {msg}");
            PathBuf::from("prterun")
        }
    }
}

/// Install prefix of the PRRTE tree that contains `prterun`, i.e. the
/// directory above its `bin/`. Symlinks are resolved first (`prterun` is a
/// link to `prte`). `None` if `prterun` isn't found or isn't in a `bin/`.
pub fn prterun_prefix() -> Option<PathBuf> {
    let exe = std::fs::canonicalize(find_prterun().ok()?).ok()?;
    let bin = exe.parent()?;
    if bin.file_name()? != "bin" {
        return None;
    }
    bin.parent().map(Path::to_path_buf)
}

/// Point a launched `prterun` at the install tree it was found in.
///
/// A vendored PRRTE has its build-tree prefix compiled in, so a copied or
/// deployed tree would otherwise keep reading its `etc/` and `share/` files
/// from the original location. This sets `PRTE_PREFIX` to [`prterun_prefix`],
/// and `PMIX_PREFIX` too when that prefix also holds PMIx's files (PMIx is
/// often installed separately; then its own prefix is left alone). Variables
/// already set in the environment are never overridden.
pub fn apply_prefix_env(cmd: &mut std::process::Command) {
    let Some(prefix) = prterun_prefix() else {
        return;
    };
    if std::env::var_os("PRTE_PREFIX").is_none() {
        cmd.env("PRTE_PREFIX", &prefix);
    }
    let holds_pmix = prefix.join("etc/pmix-mca-params.conf").is_file()
        || prefix.join("share/pmix").is_dir();
    if holds_pmix && std::env::var_os("PMIX_PREFIX").is_none() {
        cmd.env("PMIX_PREFIX", &prefix);
    }
}
