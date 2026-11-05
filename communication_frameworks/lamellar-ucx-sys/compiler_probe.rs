//! Falls back from a broken compiler wrapper to the compiler it wraps, for the vendored autotools builds.
//!
//! Compiler wrappers (Cray PE `cc`/`CC`, MPI `mpicc`, ...) add their own link flags, and sometimes a static-link
//! default, to every link. That breaks `./configure` link probes such as UCC's `-lucp -luct -lucs -lucm` check
//! ("UCX support is requested but UCX packages cannot be found"). This module tests the compiler cargo would use
//! with a tiny link, and if that fails, looks for the compiler underneath and uses it when it links.
//!
//! Set `LAMELLAR_NO_COMPILER_FALLBACK=1` to turn it off (the build then uses `CC`/`CXX` exactly as given).
//! `LAMELLAR_FALLBACK_CC` / `LAMELLAR_FALLBACK_CXX` name the fallback compilers explicitly.
//!
//! std only, so the tests run without cargo: `rustc --edition 2021 --test compiler_probe.rs -o /tmp/cp && /tmp/cp`.
//! This file is kept identical in `lamellar-ucx-sys` and `lamellar-ucc-sys`; edit both.
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub const DISABLE_VAR: &str = "LAMELLAR_NO_COMPILER_FALLBACK";
pub const FALLBACK_CC_VAR: &str = "LAMELLAR_FALLBACK_CC";
pub const FALLBACK_CXX_VAR: &str = "LAMELLAR_FALLBACK_CXX";

/// Environment variables that change the result; build scripts register them with `rerun-if-env-changed`.
pub const ENV_VARS: &[&str] = &[
    DISABLE_VAR,
    FALLBACK_CC_VAR,
    FALLBACK_CXX_VAR,
    "CC",
    "CXX",
    "PE_ENV",
];

/// What the test link should pull in: the same libraries the real configure probe needs.
#[derive(Default, Clone)]
pub struct ProbeLink {
    pub lib_dirs: Vec<PathBuf>,
    pub link_args: Vec<String>,
}

/// A working compiler found underneath a wrapper that failed the test link.
#[derive(Debug, PartialEq)]
pub struct Fallback {
    pub wrapper: String,
    pub cc: String,
    /// `None` when no C++ counterpart could be determined; the caller then keeps its `CXX`.
    pub cxx: Option<String>,
}

impl Fallback {
    pub fn message(&self) -> String {
        format!(
            "the compiler `{}` failed a test link (compiler wrappers such as Cray `cc` add flags that break the vendored \
             UCX/UCC configure checks); using `{}`{} instead. Set {}=1 to keep `{}`, or set {}/{} to choose the compiler.",
            self.wrapper,
            self.cc,
            self.cxx
                .as_ref()
                .map(|c| format!(" and `{c}`"))
                .unwrap_or_default(),
            DISABLE_VAR,
            self.wrapper,
            FALLBACK_CC_VAR,
            FALLBACK_CXX_VAR,
        )
    }
}

pub fn resolve(cc: &Path, cxx: &Path, link: &ProbeLink, scratch: &Path) -> Option<Fallback> {
    resolve_with(&|k| std::env::var(k).ok(), cc, cxx, link, scratch)
}

/// `getenv` is injected so tests do not touch the process environment.
pub fn resolve_with(
    getenv: &dyn Fn(&str) -> Option<String>,
    cc: &Path,
    _cxx: &Path,
    link: &ProbeLink,
    scratch: &Path,
) -> Option<Fallback> {
    if matches!(getenv(DISABLE_VAR).as_deref(), Some(v) if !v.is_empty() && v != "0") {
        return None;
    }
    let wrapper = cc.to_string_lossy().into_owned();
    if links(&wrapper, link, scratch) {
        return None;
    }
    for cand in candidates(&wrapper, getenv) {
        if links(&cand, link, scratch) {
            let cxx = cxx_for(&cand, getenv).filter(|c| runs(c));
            return Some(Fallback { wrapper, cc: cand, cxx });
        }
    }
    None
}

fn base(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

fn runs(compiler: &str) -> bool {
    Command::new(compiler)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Compile and link an empty `main` with `compiler` plus the probe libraries.
fn links(compiler: &str, link: &ProbeLink, scratch: &Path) -> bool {
    let _ = fs::create_dir_all(scratch);
    let src = scratch.join("lamellar_cc_probe.c");
    let out = scratch.join("lamellar_cc_probe.out");
    if fs::write(&src, "int main(void) { return 0; }\n").is_err() {
        return true; // cannot probe: do not change anything
    }
    let mut cmd = Command::new(compiler);
    cmd.arg(&src).arg("-o").arg(&out);
    for d in &link.lib_dirs {
        cmd.arg(format!("-L{}", d.display()));
    }
    cmd.args(&link.link_args);
    let ok = cmd.output().map(|o| o.status.success()).unwrap_or(false);
    let _ = fs::remove_file(&src);
    let _ = fs::remove_file(&out);
    ok
}

/// First word of the first non-empty line of `text`.
fn first_word(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .and_then(|l| l.split_whitespace().next())
        .map(str::to_owned)
}

/// Compilers to try in place of `wrapper`, most specific first, without the wrapper itself or duplicates.
fn candidates(wrapper: &str, getenv: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    if let Some(c) = getenv(FALLBACK_CC_VAR).filter(|c| !c.is_empty()) {
        found.push(c);
    }
    // Cray PE wrappers print the command they would run.
    if let Ok(o) = Command::new(wrapper)
        .args(["-craype-verbose", "-E", "-x", "c", "/dev/null", "-o", "/dev/null"])
        .output()
    {
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&o.stderr),
            String::from_utf8_lossy(&o.stdout)
        );
        found.extend(first_word(&text));
    }
    // MPICH-style and Open MPI-style wrappers.
    for flag in ["-show", "-showme"] {
        if let Ok(o) = Command::new(wrapper).arg(flag).output() {
            if o.status.success() {
                found.extend(first_word(&String::from_utf8_lossy(&o.stdout)));
            }
        }
    }
    if getenv("PE_ENV").as_deref() == Some("GNU") {
        found.push("gcc".to_owned());
    }
    found.push("gcc".to_owned());

    let mut out: Vec<String> = Vec::new();
    for c in found {
        if base(&c) == base(wrapper) || out.contains(&c) {
            continue;
        }
        out.push(c);
    }
    out
}

/// The C++ compiler that goes with C compiler `cc`.
fn cxx_for(cc: &str, getenv: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    if let Some(c) = getenv(FALLBACK_CXX_VAR).filter(|c| !c.is_empty()) {
        return Some(c);
    }
    for (c, cxx) in [("gcc", "g++"), ("clang", "clang++"), ("icx", "icpx"), ("icc", "icpc")] {
        if let Some(i) = cc.rfind(c) {
            return Some(format!("{}{}{}", &cc[..i], cxx, &cc[i + c.len()..]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        os::unix::fs::PermissionsExt,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static N: AtomicUsize = AtomicUsize::new(0);

    fn tmp() -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "lamellar_cp_test_{}_{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    /// A wrapper that links fine only without the extra library it appends, like the Cray `cc`.
    fn broken_wrapper(dir: &Path, extra: &str) -> PathBuf {
        script(dir, "fakewrap", &format!("{extra}\nexec gcc \"$@\" -lfake_missing_wrapper_lib"))
    }

    fn have_gcc() -> bool {
        runs("gcc")
    }

    fn env_none(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn falls_back_via_craype_verbose() {
        if !have_gcc() {
            return;
        }
        let d = tmp();
        let w = broken_wrapper(
            &d,
            "case \"$*\" in *-craype-verbose*) echo 'gcc -O2 -E /dev/null' >&2; exit 0;; esac",
        );
        let fb = resolve_with(&env_none, &w, &w, &ProbeLink::default(), &d).expect("fallback");
        assert_eq!(fb.cc, "gcc");
        assert_eq!(fb.cxx.as_deref(), Some("g++"));
        assert!(fb.message().contains("failed a test link"));
    }

    #[test]
    fn falls_back_via_show() {
        if !have_gcc() {
            return;
        }
        let d = tmp();
        let w = broken_wrapper(&d, "case \"$*\" in *-show*) echo 'gcc -I/x -L/y -lmpi'; exit 0;; esac");
        let fb = resolve_with(&env_none, &w, &w, &ProbeLink::default(), &d).expect("fallback");
        assert_eq!(fb.cc, "gcc");
    }

    #[test]
    fn healthy_compiler_is_left_alone() {
        if !have_gcc() {
            return;
        }
        let d = tmp();
        assert_eq!(resolve_with(&env_none, Path::new("gcc"), Path::new("g++"), &ProbeLink::default(), &d), None);
    }

    #[test]
    fn disable_variable_keeps_the_broken_compiler() {
        if !have_gcc() {
            return;
        }
        let d = tmp();
        let w = broken_wrapper(&d, "");
        let off = |k: &str| (k == DISABLE_VAR).then(|| "1".to_owned());
        assert_eq!(resolve_with(&off, &w, &w, &ProbeLink::default(), &d), None);
        // "0" and "" mean not disabled.
        let zero = |k: &str| (k == DISABLE_VAR).then(|| "0".to_owned());
        assert!(resolve_with(&zero, &w, &w, &ProbeLink::default(), &d).is_some());
    }

    #[test]
    fn explicit_override_is_tried_first() {
        if !have_gcc() {
            return;
        }
        let d = tmp();
        let w = broken_wrapper(&d, "");
        let env = |k: &str| match k {
            FALLBACK_CC_VAR => Some("gcc".to_owned()),
            FALLBACK_CXX_VAR => Some("g++".to_owned()),
            _ => None,
        };
        let fb = resolve_with(&env, &w, &w, &ProbeLink::default(), &d).expect("fallback");
        assert_eq!((fb.cc.as_str(), fb.cxx.as_deref()), ("gcc", Some("g++")));
    }

    #[test]
    fn no_fallback_when_nothing_links() {
        if !have_gcc() {
            return;
        }
        let d = tmp();
        let w = broken_wrapper(&d, "");
        let link = ProbeLink { lib_dirs: vec![], link_args: vec!["-lnonexistent_library_xyz".to_owned()] };
        assert_eq!(resolve_with(&env_none, &w, &w, &link, &d), None);
    }

    #[test]
    fn probe_uses_the_given_libraries() {
        if !have_gcc() {
            return;
        }
        let d = tmp();
        let ok = ProbeLink { lib_dirs: vec![], link_args: vec!["-lm".to_owned()] };
        assert!(links("gcc", &ok, &d));
        let bad = ProbeLink { lib_dirs: vec![], link_args: vec!["-lnonexistent_library_xyz".to_owned()] };
        assert!(!links("gcc", &bad, &d));
    }

    #[test]
    fn cxx_names() {
        assert_eq!(cxx_for("gcc", &env_none).as_deref(), Some("g++"));
        assert_eq!(cxx_for("/usr/bin/gcc-12", &env_none).as_deref(), Some("/usr/bin/g++-12"));
        assert_eq!(cxx_for("clang", &env_none).as_deref(), Some("clang++"));
        assert_eq!(cxx_for("mycompiler", &env_none), None);
    }
}
