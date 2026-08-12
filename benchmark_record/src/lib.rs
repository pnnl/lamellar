use json::JsonValue;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub mod build_utils;

pub struct BenchmarkInformation {
    pub benchmark_name: String,
    executable: PathBuf,
    parameters: Vec<String>,
    run_date: String,
    output: HashMap<String, String>,
    package_info: HashMap<String, String>,
    compile_info: HashMap<String, String>,
    git: HashMap<String, String>,
    slurm_params: HashMap<String, String>,
    system: HashMap<String, String>,
    environment_vars: HashMap<String, String>,
    pub rust_edition: String,
    rust_compiler: String,
}

impl BenchmarkInformation {
    /// Create a new BenchmarkInformation instance with default benchmark name.
    /// This is the suggested way to construct a benchmark information record.
    pub fn new() -> Self {
        let benchmark_name = default_benchmark_name();
        Self::with_name(&benchmark_name)
    }

    /// Create a new BenchmarkInformation instance with the given benchmark name
    pub fn with_name(benchark_name: &str) -> Self {
        let executable_name = executable();
        Self {
            benchmark_name: benchark_name.to_string(),
            executable: executable_name,
            parameters: env::args().skip(1).collect(),
            run_date: BenchmarkInformation::get_run_date(),
            output: HashMap::new(),
            package_info: HashMap::new(),
            compile_info: HashMap::new(),
            git: HashMap::new(),
            slurm_params: BenchmarkInformation::collect_env_vars("SLURM"),
            system: BenchmarkInformation::get_system_info(),
            environment_vars: BenchmarkInformation::collect_env_vars("LAMELLAR"),
            rust_edition: "Unknown".to_string(),
            rust_compiler: BenchmarkInformation::get_rust_compiler(),
        }
    }

    /// Add a key/value pair to the compiler info section of the benchmark information.
    pub fn with_compile_info(&mut self, key: &str, value: String) {
        self.compile_info.insert(key.to_string(), value);
    }

    /// Add a key/value pair to the git info section of the benchmark information.
    pub fn with_git_info(&mut self, key: &str, value: String) {
        self.git.insert(key.to_string(), value);
    }

    /// Add a key/value pair to the package info of the benchmark information.
    pub fn with_package_info(&mut self, key: &str, value: String) {
        self.package_info.insert(key.to_string(), value);
    }

    /// Add a key/value pair to the output section of the benchmark information.
    pub fn with_output(&mut self, key: &str, value: String) {
        self.output.insert(key.to_string(), value);
    }

    /// Convert the captured information into a JsonValue object.
    /// This is not intended as a stable API, but may be useful for some cases...use with caution.
    pub fn as_json(&self) -> JsonValue {
        json::object! {
            "benchmark_name" => self.benchmark_name.clone(),
            "executable" => self.executable.to_string_lossy().to_string(),
            "parameters" => self.parameters.clone(),
            "run_date" => self.run_date.clone(),
            "output" => self.output.clone(),
            "dependencies" => self.package_info.clone(),
            "compile_info" => self.compile_info.clone(),
            "git" => self.git.clone(),
            "system" => self.system.clone(),
            "environment" => self.environment_vars.clone(),
            "slurm_params" => self.slurm_params.clone(),
            "rust_edition" => self.rust_edition.clone(),
            "rust_compiler" => self.rust_compiler.clone(),
        }
    }

    /// Display the captured information in JSON format to stdout.
    /// If `indent` is `Some(u16)`, pretty-prints with the given indentation level.
    /// If `indent` is `None`, prints in compact form as a single line.
    pub fn display(&self, indent: Option<u16>) {
        let json_obj = self.as_json();
        match indent {
            None => println!("{}", json::stringify(json_obj)),
            Some(indent) => println!("{}", json::stringify_pretty(json_obj, indent)),
        }
    }

    /// Write the captured information to a specified file in JSON format.
    /// Assumes the file is in JSON-lines format and appends to the end.
    /// Will create the file if it does not exist, appends to it if it does.
    ///
    pub fn write(&self, file: &PathBuf) -> std::io::Result<()> {
        let json_obj = self.as_json();

        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
            .and_then(|mut f| {
                writeln!(f, "{}", json::stringify(json_obj))?;
                f.sync_data()
            })
    }

    /// Generate a default output file name based on the benchmark name and slurm ID or current time.
    pub fn default_output_path(&self, root: &str) -> PathBuf {
        let stem = self.benchmark_name.clone();
        let time = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let id = self.slurm_params.get("SLURM_JOB_ID").unwrap_or(&time);
        PathBuf::from(format!("{root}/{stem}_{id}_result.jsonl"))
    }

    /// Collects all environment variables that start with the given prefix into a HashMap
    fn collect_env_vars(prefix: &str) -> HashMap<String, String> {
        env::vars()
            .filter(|(key, _)| key.starts_with(prefix))
            .collect()
    }

    fn get_run_date() -> String {
        let datetime = chrono::Local::now();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Gathers selected system information using the sysinfo crate.
    fn get_system_info() -> HashMap<String, String> {
        let mut system_info = HashMap::new();
        let sys = sysinfo::System::new_all();

        if let Some(os_name) = sysinfo::System::name() {
            system_info.insert("os_name".to_string(), os_name);
        }
        if let Some(kernel_version) = sysinfo::System::kernel_version() {
            system_info.insert("kernel_version".to_string(), kernel_version);
        }
        if let Some(os_version) = sysinfo::System::long_os_version() {
            system_info.insert("os_version".to_string(), os_version);
        }
        if let Some(hostname) = sysinfo::System::host_name() {
            system_info.insert("hostname".to_string(), hostname);
        }

        system_info.insert("cpu_cores".to_string(), sys.cpus().len().to_string());
        system_info.insert(
            "physical_cpu_cores".to_string(),
            sysinfo::System::physical_core_count()
                .unwrap_or(0)
                .to_string(),
        );
        if let Some(cpu) = sys.cpus().first() {
            system_info.insert("cpu_frequency_mhz".to_string(), cpu.frequency().to_string());
            system_info.insert("cpu_vendor_id".to_string(), cpu.vendor_id().to_string());
            system_info.insert("cpu_brand".to_string(), cpu.brand().to_string());
        }
        system_info.insert("ram_bytes".to_string(), sys.total_memory().to_string());
        system_info.insert("swap_bytes".to_string(), sys.total_swap().to_string());

        system_info
    }

    /// Inspects the 'strings' portion of the binary to find the rustc version used to compile it.
    fn get_rust_compiler() -> String {
        let executable = executable();

        if let Ok(output) = std::process::Command::new("strings")
            .args(["-a", executable.to_str().unwrap_or("")])
            .output()
            && output.status.success()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("rustc version") {
                    return line.to_string();
                }
            }
        }
        "<unknown>".to_string()
    }
}

/// Get the current executable path
fn executable() -> PathBuf {
    env::current_exe().unwrap_or(PathBuf::from("__unknown__"))
}

/// Generate a default benchmark name based on the executable file name
pub fn default_benchmark_name() -> String {
    executable()
        .file_stem()
        .unwrap_or(OsStr::new("__unknown__"))
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_updates() {
        let mut benchmark_info = BenchmarkInformation::new();
        assert_eq!(benchmark_info.output.len(), 0);

        benchmark_info.with_output("test_key", "test_value".into());
        assert_eq!(benchmark_info.output.len(), 1);
        assert_eq!(benchmark_info.output["test_key"], "test_value");
    }

    #[test]
    fn test_named() {
        let benchmark_info = BenchmarkInformation::with_name("MyBenchmark");
        assert_eq!(benchmark_info.benchmark_name, "MyBenchmark");
    }

    #[test]
    fn test_executable() {
        let exe_path = executable().to_string_lossy().to_string();
        assert!(exe_path.contains("benchmark_record"));
        println!("Executable: {exe_path}");
    }

    #[test]
    fn test_default_output_path() {
        let benchmark_info = BenchmarkInformation::new();
        let output_path = benchmark_info.default_output_path(".");
        let output_path_str = output_path.to_string_lossy().to_string();
        println!("Default output file name: {output_path_str}");

        assert!(output_path_str.ends_with(".jsonl"));
        assert!(output_path_str.contains(default_benchmark_name().as_str()));
    }

    #[test]
    fn test_env_capture() {
        const PREFIX: &str = "BENCHMARK_RECORD_TEST_";
        unsafe {
            env::set_var("BENCHMARK_RECORD_TEST_A", "A");
            env::set_var("BENCHMARK_RECORD_TEST_B", "B");
        }

        let captured = BenchmarkInformation::collect_env_vars(PREFIX);
        assert_eq!(
            captured.get("BENCHMARK_RECORD_TEST_A"),
            Some(&"A".to_string())
        );
        assert_eq!(
            captured.get("BENCHMARK_RECORD_TEST_B"),
            Some(&"B".to_string())
        );

        unsafe {
            env::remove_var("BENCHMARK_RECORD_TEST_A");
            env::remove_var("BENCHMARK_RECORD_TEST_B");
        }
    }
}
