# Benchmark record utility

## Setup
To capture runtime-values:
- Depend on this library in the `dependencies` secion of `Cargo.toml`.

To accurately capture compile-time values (like feature flags):
- Depend on this library in the `build-dependencies` section of `Cargo.toml`.
- Each benchmark should have a `build.rs` in the same directory as `Cargo.toml`.
- Each benchmark should call `embed_build_time_info` to embed the build-time information captured by `build.rs`. (See useage example)


### Example build.rs file
```rust
use benchmark_record::build_utils;

fn main() {
    build_utils::record_build_time_info(std::env::var("CARGO_MANIFEST_PATH").unwrap());
}
```


## Example (basic) use:
- Create a record at the start of the program: `let mut result_record = benchmark_record::BenchmarkInformation::new();`
- Ensure build-time info is property recorded `embed_build_time_info!(result_record );`
- Record results as they become available: `result_record.with_output("result_name", "value as a string");`
- Save to disk: `result_record.write(&result_record.default_output_path("benchmarking"))?;`
- Print to screen: `result_record.display(Some(3));`

The `result_record` gathers the contextual information at construction time.
The record acts as an accumulator of result information using the `.with_output` method.
If you write to same key twice, the first value with be lost!

The `default_output_path` function will return a name based on the current executable and slurm context (if available).
It writes in a JSON lines format and appends values (so you can write multiple times from the same program).
The `display` function prints to the screen.  The optional integer value controls the indenentation (`None` prints on one line).
