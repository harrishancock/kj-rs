use std::env;

fn main() {
    let include_path = env::var("LIBKJ_INCLUDE_PATH").expect("LIBKJ_INCLUDE_PATH must be set");
    let static_path = env::var("LIBKJ_STATIC_PATH").expect("LIBKJ_STATIC_PATH must be set");

    let include_path: Vec<&str> = include_path.split(':').collect();
    let static_path: Vec<&str> = static_path.split(':').collect();

    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CXXFLAGS");
    println!("cargo:rerun-if-env-changed=LIBKJ_INCLUDE_PATH");
    println!("cargo:rerun-if-env-changed=LIBKJ_STATIC_PATH");

    for p in static_path {
        println!("cargo:rustc-link-search=native={}", p);
    }
    println!("cargo:rustc-link-lib=static=kj");

    cxx_build::bridge("lib.rs")
        .file("awaiter.c++")
        .file("executor-guarded.c++")
        .file("future-boilerplate.c++")
        .file("promise-boilerplate.c++")
        .file("promise.c++")
        .file("waker.c++")
        .includes(include_path)
        .cpp(true)
        .cpp_set_stdlib("c++")
        .std("c++23")
        .compile("kj-rs");
}
