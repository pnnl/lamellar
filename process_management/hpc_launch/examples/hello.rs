//! Smallest `#[hpc_launch::main]` program. Run with, for example:
//!
//! ```text
//! cargo run --example hello -- --pes 2
//! ```

#[hpc_launch::main]
fn main() {
    // by the time this runs we are inside a launched process
    println!("hello from PE {:?}", hpc_launch::pe_id());
}
