use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Records compile-time information by capturing into into environment variables.
/// This function should call all of the functions needed to record build time information.
/// This is likely everything that is made with the define_env_vars! macro.
pub fn record_build_time_info(manifest_path: String) {
    println!("cargo:rerun-if-changed={manifest_path}");
    println!("cargo:rerun-if-env-changed=PROFILE");
    record_cargo_env_vars();
    record_git_info();
    record_manifest_info(manifest_path);
}

#[macro_export]
macro_rules! embed_build_time_info {
    ($bench:ident) => {
        benchmark_record::embed_compile_info!($bench);
        benchmark_record::embed_git_info!($bench);
        benchmark_record::embed_manifest_info!($bench);
        benchmark_record::embed_package_info!($bench);
    };
}

/// Attempts to read the rust edition from Cargo.toml in the current directory or CARGO_MANIFEST_DIR
/// Record the edition and specific package version information.
fn record_manifest_info(manifest_path: String) {
    let manifest_content = fs::read_to_string(manifest_path);
    let mut the_edition = String::new();
    match manifest_content {
        Ok(contents) => {
            for line in contents.lines() {
                if line.trim_start().starts_with("edition = ")
                    && let Some(edition) = line.split('=').nth(1)
                {
                    the_edition = edition.trim().trim_matches('"').to_string();
                }
            }
        }
        Err(_) => the_edition = "Unknown".to_string(),
    }
    println!("cargo:rustc-env=BUILD_RUST_EDITION={}", the_edition);

    let mut package_info = HashMap::new();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let lock_path = find_workspace_lockfile(Path::new(&manifest_dir));

    if let Some(lock_path) = lock_path
        && let Ok(contents) = fs::read_to_string(&lock_path)
    {
        println!("cargo:rerun-if-changed={}", lock_path.display());
        let mut lines = contents.lines();
        while let Some(line) = lines.next() {
            if line.trim_start().starts_with("name = ")
                && let Some(name) = line.split('=').nth(1)
            {
                let name = name.trim().trim_matches('"').to_string();
                if !CHECK_PACKAGES.contains(&name.as_str()) {
                    continue;
                }

                let mut version = String::new();
                let mut source = String::new();

                // Look ahead for version line
                if let Some(version_line) = lines.next()
                    && version_line.trim_start().starts_with("version = ")
                    && let Some(v) = version_line.split('=').nth(1)
                {
                    version = v.trim().trim_matches('"').to_string();
                }

                // Look ahead for source line
                if let Some(source_line) = lines.next()
                    && source_line.trim_start().starts_with("source = ")
                    && let Some(s) = source_line.split('=').nth(1)
                {
                    source = s.trim().trim_matches('"').to_string();
                }

                // Store as "version/source" format
                if !version.is_empty() || !source.is_empty() {
                    let combined = format!("{}/{}", version, source);
                    package_info.insert(name, combined);
                }
            }
        }
    }

    for (key, value) in package_info.iter() {
        println!("cargo:rustc-env={}={}", key, value);
        println!("cargo:warning={}={}", key, value);
    }
}

/// Finds the nearest workspace lockfile, which may be above an individual
/// package's `CARGO_MANIFEST_DIR` in a virtual workspace.
fn find_workspace_lockfile(manifest_dir: &Path) -> Option<PathBuf> {
    manifest_dir
        .ancestors()
        .map(|directory| directory.join("Cargo.lock"))
        .find(|path| path.is_file())
}

#[macro_export]
macro_rules! embed_manifest_info {
    ($bench:ident) => {
        $bench.rust_edition = option_env!("BUILD_RUST_EDITION")
            .unwrap_or(concat!("BUILD_RUST_EDITION variable not set"))
            .to_string();
    };
}

/// Macro to define both a const array and a corresponding compile-time capture macro.
///
/// This macro generates:
/// 1. A public const array containing the variable names (for use in build.rs)
/// 2. A macro that captures those variables at compile time using option_env!
///
/// # Example
/// ```
/// define_env_vars!(
///     MY_VARS,           // Name of the const array
///     embed_my_vars,   // Name of the capture macro
///     with_compile_info, // Name of the BenchmarkInformation method to call
///     [
///         "CARGO_PKG_NAME",
///         "PROFILE",
///     ]
/// );
///
/// // Usage in build.rs:
/// capture_my_vars();
///
/// // Usage in main code:
/// let mut bench = BenchmarkInformation::new();
/// embed_my_vars!(bench);
/// ```
macro_rules! define_env_vars {
    ($const_name:ident, $macro_name:ident, $record_fn:ident, [$($var_name:literal),* $(,)?]) => {
        /// A set of environment variable names to record
        pub const $const_name: &[&str] = &[
            $($var_name,)*
        ];

        /// Macro to capture the compile-time values of the defined environment variables
        #[macro_export]
        macro_rules! $macro_name {
            ($bench:ident) => {
                $(
                    $bench.$record_fn(
                        $var_name,
                        option_env!($var_name)
                            .unwrap_or(concat!($var_name, " variable not set"))
                            .to_string()
                    );
                )*
            };
        }
    };
}

// Define a set of package information variables to capture
define_env_vars!(
    CHECK_PACKAGES,
    embed_package_info,
    with_package_info,
    ["lamellar", "rofisys", "lamellar-impl"]
);

// Define the default set of environment variables to capture
define_env_vars!(
    DEFAULT_ENV_VARS,
    embed_compile_info,
    with_compile_info,
    [
        "CARGO_CFG_FEATURE",
        "CARGO_CFG_TARGET_ARCH",
        "CARGO_CFG_TARGET_OS",
        "CARGO_CFG_TARGET_FEATURE",
        "CARGO_CFG_TARGET_HAS_ATOMIC",
        "CARGO_CFG_TARGET_POINTER_WIDTH",
        "CARGO_PKG_NAME",
        "CARGO_PKG_VERSION",
        "PROFILE",
        "OPT_LEVEL",
    ]
);

/// For each variable name given in `var_names`, if the environment variable is set,
/// passes it through to rustc via cargo:rustc-env so it can be accessed with option_env!
pub fn record_cargo_env_vars() {
    for &var_name in DEFAULT_ENV_VARS {
        if let Ok(var_value) = env::var(var_name) {
            println!("cargo:rustc-env={}={}", var_name, var_value);
        }
    }
}

define_env_vars!(
    GIT_ENV_VARS,
    embed_git_info,
    with_git_info,
    [
        "commit_hash",
        "short_hash",
        "commit_date",
        "commit_message",
        "status",
    ]
);

/// Records git information as environment variables (which may then be visible at compile-time)
pub fn record_git_info() -> HashMap<String, String> {
    let mut git_info = HashMap::new();

    // Get long hash
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        && output.status.success()
    {
        let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        git_info.insert("commit_hash".to_string(), commit_hash);
    }

    // Get short hash
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        && output.status.success()
    {
        let short_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        git_info.insert("short_hash".to_string(), short_hash);
    }

    // Get commit date
    if let Ok(output) = std::process::Command::new("git")
        .args(["log", "-1", "--format=%cd", "--date=iso"])
        .output()
        && output.status.success()
    {
        let commit_date = String::from_utf8_lossy(&output.stdout).trim().to_string();
        git_info.insert("commit_date".to_string(), commit_date);
    }

    // Get commit message
    if let Ok(output) = std::process::Command::new("git")
        .args(["log", "-1", "--format=%s"])
        .output()
        && output.status.success()
    {
        let commit_message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        git_info.insert("commit_message".to_string(), commit_message);
    }

    // Get git status in short format
    if let Ok(output) = std::process::Command::new("git")
        .args(["status", "-s"])
        .output()
        && output.status.success()
    {
        let git_status_output = String::from_utf8_lossy(&output.stdout);
        let git_status = git_status_output
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join(", ");
        git_info.insert("status".to_string(), git_status);
    } else {
        // If git command fails, store empty string
        git_info.insert("status".to_string(), "command failed".to_string());
    }

    for &entry in GIT_ENV_VARS {
        git_info
            .entry(entry.to_string())
            .or_insert_with(|| "unavailable".to_string());
    }

    for (key, value) in &git_info {
        println!("cargo:rustc-env={}={}", key, value);
    }

    git_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_info() {
        let git_info = record_git_info();

        println!("Git info: {:?}", git_info);

        // Check that all expected git fields are present
        assert!(
            git_info.contains_key("commit_hash"),
            "commit_hash should be present"
        );
        assert!(
            git_info.contains_key("short_hash"),
            "short_hash should be present"
        );
        assert!(
            git_info.contains_key("commit_date"),
            "commit_date should be present"
        );
        assert!(
            git_info.contains_key("commit_message"),
            "commit_message should be present"
        );

        // Check that values are not empty (assuming we're in a git repository)
        if let Some(commit_hash) = git_info.get("commit_hash") {
            assert!(!commit_hash.is_empty(), "commit_hash should not be empty");
            assert!(
                commit_hash.len() >= 40,
                "commit_hash should be at least 40 characters"
            );
        }

        if let Some(short_hash) = git_info.get("short_hash") {
            assert!(!short_hash.is_empty(), "short_hash should not be empty");
            assert!(
                short_hash.len() >= 7,
                "short_hash should be at least 7 characters"
            );
        }

        if let Some(commit_date) = git_info.get("commit_date") {
            assert!(!commit_date.is_empty(), "commit_date should not be empty");
        }

        if let Some(commit_message) = git_info.get("commit_message") {
            assert!(
                !commit_message.is_empty(),
                "commit_message should not be empty"
            );
        }
    }
}
