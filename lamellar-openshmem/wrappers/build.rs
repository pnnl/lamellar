// Nothing to do here. A package only compiles its build-dependencies when it has a build script, and with the bundled-prrte
// feature the prrte-sys build dependency is what builds PRRTE; tools/build-static.sh then copies that install tree.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
