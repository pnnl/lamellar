0. Install rust 
- https://www.rust-lang.org/tools/install
- on linux `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- If your IDE supports it, the Rust Analyzer Plugin (https://rust-analyzer.github.io/) is highly recommended.

1. Set up a new project and add a few dependencies
- create a new crate: `cargo new rust_tutorial`
- `cd rust_tutorial`
- create a nested crate: `cargo new nested_crate` 
- `cd nested_crate`
- add some dependencies `cargo add lamellar rayon rand`
- build the project (we do this step because lamellar takes a while to compile the first time) 
    - we will specify the build directory to cache the build results (this defaults to `${CWD}/target`), we will also build both debug and release binaries
    - `CARGO_TARGET_DIR=../target cargo build; CARGO_TARGET_DIR=../target cargo build --release`
- change back to top level crate directory: `cd ..`
- now examine the directory
    - `src` directory, our main crate code will go here
    - `target` directory build artifacts and binaries go here (by default)
    - `Cargo.toml` — the crate manifest file
    - `Cargo.lock’  — contains which versions of crate dependencies were used during the build process
- take a look at cargo.toml
- add the same dependencies we added to the nested crate
    - `cargo add lamellar --features runtime-warnings-panic`
    - `cargo add  rayon rand`
- take a look at cargo.toml again to see the dependencies have been added
    - you can browse crates.io for other crates!

2. Build and execute your first rust program
- open src/main.rs
    - should see a simple “hello world”  application
-  enter the command `cargo run` (you may need to wait for the build process we started in step 1 to finish)
    - this build and run application in one step
    - by default we build a debug version of the application
    - `cargo run --release` is used to build and run an optimized binary
- alternatively we can build and run in separate steps
    - `cargo build` or `cargo build --release`
    - binaries are located at:
        - `./target/debug/rust_tutorial` and ./target/release/rust_tutorial`
- regardless of the method, cargo automatically downloads, builds, and links any dependencies in Cargo.toml for you
    - Cargo + Crates.io is one of the true joys of using Rust!

The rest of the tutorial will now step through a number examples highlighting various features of Rust, as well as intentionally introducing errors to see how the compiler helps assist in producing correct code.
If following along from the GitHub repository, the relevant files are in the examples folder.

1. Mutability
2. Ownership
3. Borrowing
4. Structs
5. Generics
6. Traits
7. Enums
8. Error Handling