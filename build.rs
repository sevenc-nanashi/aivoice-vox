extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=src/cpp/bridge.cpp");
    cc::Build::new()
        .cpp(true)
        .warnings(true)
        .file("src/cpp/bridge.cpp")
        .compile("bridge");
    println!("cargo:rustc-link-lib=static=bridge");
    println!("cargo:rustc-link-search=./src/cpp");
}
