## Part 1: Rust Fundamentals

This is a pure-Rust session — no Lamellar dependency here. Part 2 (`../part2_lamellar_intro/`)
picks up immediately after this and applies these same ideas (ownership, traits, generics,
atomics) across distributed memory with Lamellar.

0. Install rust
- https://www.rust-lang.org/tools/install
- on linux `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- If your IDE supports it, the Rust Analyzer Plugin (https://rust-analyzer.github.io/) is highly recommended.

1. Set up a new project
- create a new crate: `cargo new rust_tutorial`
- `cd rust_tutorial`
- now examine the directory
    - `src` directory, our main crate code will go here
    - `target` directory, build artifacts and binaries go here (by default)
    - `Cargo.toml` — the crate manifest file
    - `Cargo.lock` — contains which versions of crate dependencies were used during the build process
- take a look at Cargo.toml — no dependencies needed for this session

2. Build and execute your first rust program
- open src/main.rs
    - should see a simple "hello world" application
- enter the command `cargo run`
    - this builds and runs the application in one step
    - by default we build a debug version of the application
    - `cargo run --release` is used to build and run an optimized binary
- alternatively we can build and run in separate steps
    - `cargo build` or `cargo build --release`
    - binaries are located at:
        - `./target/debug/rust_tutorial` and `./target/release/rust_tutorial`

The rest of the tutorial steps through a number of examples highlighting various Rust
features, intentionally introducing compiler errors to see how the compiler assists in
producing correct code. Files are in the `examples/` folder; run one with:

    cargo run --example <name>

e.g. `cargo run --example 1_mutability`. Each example has an intentional bug — try to
fix it yourself first; a working reference is under `solutions/` (also runnable with
`cargo run --example ...` if you copy it into `examples/`, or just read it directly).

1. Mutability
2. Ownership
3. Borrowing
4. Closures & Iterators — chains like `.iter().map(...).for_each(...)` are exactly the
   pattern Lamellar arrays use for distributed iteration in part 2
   (`array.dist_iter_mut().enumerate().for_each(...)`)
5. Structs
6. Generics
7. Traits — implementing a trait here is the same mechanism part 2 uses to define an
   Active Message (`impl LamellarAM for YourStruct`)
8. Enums
9. Error Handling
10. Modules & Concurrency Primitives — `mod`, `Arc`, `Mutex`, `AtomicUsize`. `Arc` here is
    the single-process analog of part 2's `Darc` (distributed Arc); `AtomicUsize` here is
    the analog of part 2's `AtomicArray` element access.

### Bridge to Part 2

Part 2 takes these same ideas — ownership, borrowing, traits, generics, atomics — and
applies them across distributed memory (multiple PEs/processes) using the `lamellar` crate.
