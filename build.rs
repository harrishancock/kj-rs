use cxx_build::CFG;

use std::env;
use std::fs;
use std::path::Path;

const HEADERS: &[&str] = &[
    "awaiter.h",
    "executor-guarded.h",
    "future.h",
    "linked-group.h",
    "promise.h",
    "waker.h",
];

const SOURCES: &[&str] = &[
    "awaiter.c++",
    "executor-guarded.c++",
    "promise.c++",
    "waker.c++",
];

fn main() {
    // We require the builder to explicitly specify the KJ library include and static library paths.
    let libkj_include_path =
        env::var("LIBKJ_INCLUDE_PATH").expect("LIBKJ_INCLUDE_PATH must be set");
    let libkj_static_path = env::var("LIBKJ_STATIC_PATH").expect("LIBKJ_STATIC_PATH must be set");

    let libkj_include_path: Vec<&Path> = libkj_include_path
        .split(':')
        .map(|p| Path::new(p))
        .collect();
    let libkj_static_path: Vec<&Path> =
        libkj_static_path.split(':').map(|p| Path::new(p)).collect();

    println!("cargo:rerun-if-env-changed=LIBKJ_INCLUDE_PATH");
    println!("cargo:rerun-if-env-changed=LIBKJ_STATIC_PATH");

    for p in libkj_static_path {
        println!("cargo:rustc-link-search=native={}", p.display());
    }

    // Awkward: we can only list one library in our "link" value in Cargo.toml, but we need to link
    // with both kj and kj-async.
    println!("cargo:rustc-link-lib=static=kj");
    println!("cargo:rustc-link-lib=static=kj-async");

    // We publicly depend on KJ.
    CFG.exported_header_dirs.extend(libkj_include_path);

    // We export our own headers so that they are available with `#include <kj-rs/foo.h>`.
    let out_dir = env::var("OUT_DIR").expect("cargo guarantees OUT_DIR is set");
    let out_include_dir = Path::new(&out_dir).join("include");
    CFG.exported_header_dirs.push(&out_include_dir);

    let out_include_kj_rs_dir = out_include_dir.join("kj-rs");
    fs::create_dir_all(&out_include_kj_rs_dir).expect("directory should not yet exist");

    let local_include_dir = Path::new("include");
    for header in HEADERS {
        fs::copy(
            local_include_dir.join(header),
            out_include_kj_rs_dir.join(header),
        )
        .expect("header should not yet exist");
    }

    let local_src_dir = Path::new("src");
    cxx_build::bridge(local_src_dir.join("lib.rs"))
        .files(SOURCES.into_iter().map(|s| local_src_dir.join(s)))
        .cpp(true)
        .cpp_set_stdlib("c++")
        .std("c++23")
        .compile("kj-rs");
}
