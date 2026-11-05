use benchmark_record::build_utils;
use std::env;

fn main() {
    let manifest = env::var("CARGO_MANIFEST_PATH").unwrap();
    build_utils::record_build_time_info(manifest);
}
