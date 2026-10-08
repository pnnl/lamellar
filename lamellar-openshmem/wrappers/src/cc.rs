//! The compiler wrapper (`shmemcc`, `oshcc`, `shmemcxx`, `oshcxx`).
//!
//! ```text
//! shmemcc prog.c -o prog                 compile and link against libshmem.so
//! shmemcc --shmem-static prog.c -o prog  link libshmem.a (whole-archive) instead
//! shmemcc --showme[:compile|:link|:command|:version]   print instead of running (Open MPI style); -show = --showme
//! ```
//!
//! Environment: `OSHCC_CC` / `OSHCC_CXX` (driver; then `CC` / `CXX`), `OSHCC_CFLAGS`, `OSHCC_LDFLAGS`.
//! Where libshmem lives is found in this order: `SHMEM_INCDIR` / `SHMEM_LIBDIR`, `SHMEM_PREFIX`, the install prefix of
//! this executable (`<exe>/../{lib,lib64,include}`, or a source tree's `target/release`), then `pkg-config shmem`.

use std::env;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where the header and the library are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub include: PathBuf,
    pub lib: PathBuf,
}

/// What the wrapper was asked to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Run the compiler.
    Run,
    /// `--showme` / `-show`: print the full command.
    ShowAll,
    ShowCompile,
    ShowLink,
    ShowCommand,
    ShowVersion,
}

/// The decisions made from the command line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub mode: Mode,
    pub is_static: bool,
    /// False for `-c -E -S -M -MM`: compile only, no link flags.
    pub link: bool,
    /// The user's arguments with the wrapper's own options removed.
    pub args: Vec<String>,
}

pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Plan {
    let mut plan = Plan { mode: Mode::Run, is_static: false, link: true, args: Vec::new() };
    for a in args {
        match a.as_str() {
            "--showme" | "-show" => plan.mode = Mode::ShowAll,
            "--showme:compile" => plan.mode = Mode::ShowCompile,
            "--showme:link" => plan.mode = Mode::ShowLink,
            "--showme:command" => plan.mode = Mode::ShowCommand,
            "--showme:version" => plan.mode = Mode::ShowVersion,
            "--shmem-static" => plan.is_static = true,
            "-c" | "-E" | "-S" | "-M" | "-MM" => {
                plan.link = false;
                plan.args.push(a);
            }
            _ => plan.args.push(a),
        }
    }
    plan
}

/// The compiler driver for the name this program was started under.
pub fn driver(name: &str) -> Result<String, String> {
    let get = |k: &str| env::var(k).ok().filter(|v| !v.is_empty());
    if name.contains("c++") || name.contains("cxx") {
        Ok(get("OSHCC_CXX").or_else(|| get("CXX")).unwrap_or_else(|| "c++".into()))
    } else if name.contains("fort") || name.contains("f90") || name.contains("f77") {
        Err(format!("{name}: Fortran is not supported yet"))
    } else {
        Ok(get("OSHCC_CC").or_else(|| get("CC")).unwrap_or_else(|| "cc".into()))
    }
}

fn has_lib(dir: &Path) -> bool {
    dir.join("libshmem.so").exists() || dir.join("libshmem.a").exists()
}

fn pkg_config_var(var: &str) -> Option<PathBuf> {
    let out = Command::new("pkg-config").arg(format!("--variable={var}")).arg("shmem").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then(|| PathBuf::from(s))
}

/// Find the header and library directories (see the module documentation for the order).
pub fn locate(exe: &Path) -> Result<Layout, String> {
    let nonempty = |k: &str| env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let mut layout: Option<Layout> = None;

    if let Some(prefix) = nonempty("SHMEM_PREFIX") {
        for lib in [prefix.join("lib"), prefix.join("lib64")] {
            if has_lib(&lib) {
                layout = Some(Layout { include: prefix.join("include"), lib });
                break;
            }
        }
    }
    if layout.is_none() {
        if let Some(root) = exe.canonicalize().ok().as_deref().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            for lib in [root.join("lib"), root.join("lib64"), root.join("target").join("release")] {
                if has_lib(&lib) {
                    layout = Some(Layout { include: root.join("include"), lib });
                    break;
                }
            }
        }
    }
    if layout.is_none() {
        if let (Some(include), Some(lib)) = (pkg_config_var("includedir"), pkg_config_var("libdir")) {
            if has_lib(&lib) {
                layout = Some(Layout { include, lib });
            }
        }
    }
    let mut layout = layout.ok_or_else(|| {
        "cannot find libshmem: set SHMEM_PREFIX (or SHMEM_INCDIR and SHMEM_LIBDIR), install it next to this program, \
         or put shmem.pc on PKG_CONFIG_PATH"
            .to_string()
    })?;
    if let Some(i) = nonempty("SHMEM_INCDIR") {
        layout.include = i;
    }
    if let Some(l) = nonempty("SHMEM_LIBDIR") {
        layout.lib = l;
    }
    Ok(layout)
}

fn words(var: &str) -> Vec<String> {
    env::var(var).map(|v| v.split_whitespace().map(String::from).collect()).unwrap_or_default()
}

/// The libraries a static link of libshmem.a needs after it: `lib/shmem.libs` if present, then pkg-config's private
/// libraries, then a default.
fn static_extra(lib: &Path) -> Vec<String> {
    if let Ok(s) = std::fs::read_to_string(lib.join("shmem.libs")) {
        return s.split_whitespace().map(String::from).collect();
    }
    if let Ok(out) = Command::new("pkg-config").args(["--static", "--libs-only-l", "shmem"]).output() {
        if out.status.success() {
            let v: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .filter(|w| *w != "-lshmem")
                .map(String::from)
                .collect();
            if !v.is_empty() {
                return v;
            }
        }
    }
    ["-lutil", "-lrt", "-lpthread", "-lm", "-ldl"].iter().map(|s| s.to_string()).collect()
}

/// Compile flags: the include path and `OSHCC_CFLAGS`.
pub fn cflags(layout: &Layout, extra: &[String]) -> Vec<String> {
    let mut v = vec![format!("-I{}", layout.include.display())];
    v.extend(extra.iter().cloned());
    v
}

/// Link flags. Static linking uses whole-archive: it keeps the active-message registrations (static initializers) that
/// the linker would otherwise drop.
pub fn lflags(layout: &Layout, is_static: bool, static_libs: &[String], extra: &[String]) -> Vec<String> {
    let lib = layout.lib.display();
    let mut v = Vec::new();
    if is_static {
        v.push("-Wl,--whole-archive".to_string());
        v.push(format!("{lib}/libshmem.a"));
        v.push("-Wl,--no-whole-archive".to_string());
        v.push(format!("-L{lib}"));
        v.push(format!("-Wl,-rpath,{lib}"));
        v.extend(static_libs.iter().cloned());
    } else {
        v.push(format!("-L{lib}"));
        v.push("-lshmem".to_string());
        v.push(format!("-Wl,-rpath,{lib}"));
    }
    v.extend(extra.iter().cloned());
    v
}

/// The full compiler command line (driver first).
pub fn command_line(driver: &str, plan: &Plan, cflags: &[String], lflags: &[String]) -> Vec<String> {
    let mut cmd = vec![driver.to_string()];
    cmd.extend(cflags.iter().cloned());
    cmd.extend(plan.args.iter().cloned());
    if plan.link {
        cmd.extend(lflags.iter().cloned());
    }
    cmd
}

/// Entry point of the wrapper binaries. Returns the exit status (it only returns if nothing was executed).
pub fn main() -> i32 {
    let argv: Vec<OsString> = env::args_os().collect();
    let exe = PathBuf::from(argv.first().cloned().unwrap_or_default());
    let name = exe.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let args = argv.iter().skip(1).map(|a| a.to_string_lossy().into_owned());
    let plan = parse_args(args);

    let drv = match driver(&name) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    if plan.mode == Mode::ShowCommand {
        println!("{drv}");
        return 0;
    }
    if plan.mode == Mode::ShowVersion {
        println!("Lamellar OpenSHMEM 1.4 (driver: {drv})");
        return 0;
    }
    let exe_path = env::current_exe().unwrap_or(exe);
    let layout = match locate(&exe_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{name}: {e}");
            return 1;
        }
    };
    let cf = cflags(&layout, &words("OSHCC_CFLAGS"));
    let statics = if plan.is_static { static_extra(&layout.lib) } else { Vec::new() };
    let lf = lflags(&layout, plan.is_static, &statics, &words("OSHCC_LDFLAGS"));
    match plan.mode {
        Mode::ShowCompile => {
            println!("{}", cf.join(" "));
            return 0;
        }
        Mode::ShowLink => {
            println!("{}", lf.join(" "));
            return 0;
        }
        _ => {}
    }
    let cmd = command_line(&drv, &plan, &cf, &lf);
    if plan.mode == Mode::ShowAll {
        println!("{}", cmd.join(" "));
        return 0;
    }
    let err = Command::new(&cmd[0]).args(&cmd[1..]).exec();
    eprintln!("{name}: cannot run {}: {err}", cmd[0]);
    127
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout { include: "/opt/s/include".into(), lib: "/opt/s/lib".into() }
    }

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parses_wrapper_options_and_keeps_the_rest() {
        let p = parse_args(s(&["--shmem-static", "-O2", "a.c", "-o", "a", "--showme:link"]));
        assert_eq!(p.mode, Mode::ShowLink);
        assert!(p.is_static && p.link);
        assert_eq!(p.args, s(&["-O2", "a.c", "-o", "a"]));
    }

    #[test]
    fn compile_only_flags_drop_the_link_flags() {
        for f in ["-c", "-E", "-S", "-M", "-MM"] {
            let p = parse_args(s(&[f, "a.c"]));
            assert!(!p.link, "{f}");
            let cmd = command_line("cc", &p, &cflags(&layout(), &[]), &lflags(&layout(), false, &[], &[]));
            assert_eq!(cmd, s(&["cc", "-I/opt/s/include", f, "a.c"]));
        }
    }

    #[test]
    fn showme_aliases() {
        assert_eq!(parse_args(s(&["-show"])).mode, Mode::ShowAll);
        assert_eq!(parse_args(s(&["--showme"])).mode, Mode::ShowAll);
        assert_eq!(parse_args(s(&["--showme:command"])).mode, Mode::ShowCommand);
        assert_eq!(parse_args(s(&["--showme:version"])).mode, Mode::ShowVersion);
    }

    #[test]
    fn shared_and_static_link_lines() {
        let l = layout();
        assert_eq!(lflags(&l, false, &[], &s(&["-lx"])), s(&["-L/opt/s/lib", "-lshmem", "-Wl,-rpath,/opt/s/lib", "-lx"]));
        assert_eq!(
            lflags(&l, true, &s(&["-lm", "-ldl"]), &[]),
            s(&[
                "-Wl,--whole-archive",
                "/opt/s/lib/libshmem.a",
                "-Wl,--no-whole-archive",
                "-L/opt/s/lib",
                "-Wl,-rpath,/opt/s/lib",
                "-lm",
                "-ldl"
            ])
        );
    }

    #[test]
    fn full_command_order_is_driver_cflags_args_lflags() {
        let p = parse_args(s(&["a.c", "-o", "a"]));
        let cmd = command_line("cc", &p, &cflags(&layout(), &s(&["-g"])), &lflags(&layout(), false, &[], &[]));
        assert_eq!(cmd, s(&["cc", "-I/opt/s/include", "-g", "a.c", "-o", "a", "-L/opt/s/lib", "-lshmem", "-Wl,-rpath,/opt/s/lib"]));
    }

    #[test]
    fn driver_follows_the_program_name() {
        // environment-independent checks only: Fortran is rejected, and C++ names pick a C++ default
        assert!(driver("oshfort").is_err());
        assert!(driver("shmemcxx").is_ok());
        assert!(driver("shmemcc").is_ok());
    }
}
