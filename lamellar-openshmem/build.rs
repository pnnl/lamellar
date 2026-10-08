fn main() {
    // libshmem.so finds a libfabric.so installed next to it (make install copies it there).
    println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,$ORIGIN");
}
