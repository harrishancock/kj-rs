// Copyright (c) 2024 Cloudflare, Inc.
// Licensed under the Apache 2.0 license found in the LICENSE file or at:
//     https://opensource.org/licenses/Apache-2.0

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
        .map(|p| {
            let p = Path::new(p);
            let p = Box::<Path>::leak(
                p.canonicalize()
                    .expect("LIBKJ_INCLUDE_PATH must be canonicalizable")
                    .into_boxed_path(),
            ) as &Path;
            p
        })
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
    let out_dir = Path::new(&out_dir);
    let out_include_dir = out_dir.join("include");
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

    // unsafe {
    //     env::set_var("CARGO_TARGET_DIR", out_dir);
    // }

    // println!(
    //     "set CARGO_TARGET_DIR = {:?}",
    //     env::var_os("CARGO_TARGET_DIR").expect("we just set it")
    // );

    let local_src_dir = Path::new("src");
    cxx_build::bridge(local_src_dir.join("lib.rs"))
        .cpp(true)
        .files(SOURCES.into_iter().map(|s| local_src_dir.join(s)))
        .cpp_set_stdlib("c++")
        .std("c++23")
        .compile("kj-rs");

    // cxxbridge leaves a symlink pointing to our CARGO_MANIFEST_DIR, which serves the same role as
    // the include directory we created above. It confuses rules_rust's symlink fixup behavior, and
    // generally seems to have been unwise:
    // - https://github.com/dtolnay/cxx/issues/754
    // - https://github.com/dtolnay/cxx/issues/1004
    let symlink_to_kill = out_dir.join("cxxbridge").join("crate").join("/kj-rs");
    assert!(symlink_to_kill.is_symlink());
    fs::remove_file(&symlink_to_kill).expect("we should be able to remove it");
}
