extern crate cc;
extern crate cmake;
extern crate path_slash;

use path_slash::PathBufExt;
use std::env;

fn main() {
    let cur_dir = std::env::current_dir().unwrap();
    let mut cfg = cmake::Config::new("titan");
    let not_win = !cfg!(target_os = "windows");
    if cfg!(feature = "portable") {
        cfg.define("PORTABLE", "ON");
    }
    if cfg!(feature = "sse") {
        cfg.define("FORCE_SSE42", "ON");
    }
    cfg.define(
        "ROCKSDB_DIR",
        cur_dir.join("..").join("rocksdb").to_slash().unwrap(),
    )
    .define("WITH_TITAN_TESTS", "OFF")
    .define("WITH_TITAN_TOOLS", "OFF");
    cfg.define("CMAKE_POLICY_DEFAULT_CMP0066", "OLD")
        .define("CMAKE_POLICY_DEFAULT_CMP0056", "OLD")
        .define("CMAKE_POLICY_DEFAULT_CMP0128", "OLD");
    if not_win {
        // RocksDB cmake script expect libz.a being under ${DEP_Z_ROOT}/lib, but libz-sys crate put it
        // under ${DEP_Z_ROOT}/build. Append the path to CMAKE_PREFIX_PATH to get around it.
        env::set_var("CMAKE_PREFIX_PATH", {
            let zlib_path = format!("{}/build", env::var("DEP_Z_ROOT").unwrap());
            if let Ok(prefix_path) = env::var("CMAKE_PREFIX_PATH") {
                format!("{};{}", prefix_path, zlib_path)
            } else {
                zlib_path
            }
        });
        cfg.register_dep("Z")
            .define("WITH_ZLIB", "ON")
            .register_dep("BZIP2")
            .define("WITH_BZ2", "ON")
            .register_dep("LZ4")
            .define("WITH_LZ4", "ON")
            .register_dep("ZSTD")
            .define("WITH_ZSTD", "ON")
            .register_dep("SNAPPY")
            .define("WITH_SNAPPY", "ON");
        // GCC 13+ false-positive -Warray-bounds on statistics_impl.h
        // tickers array under -O2+. Demote to warning only.
        cfg.cxxflag("-Wno-error=array-bounds");
    } else {
        cfg.cxxflag("/MP")
            .register_dep("LZ4")
            .define("WITH_LZ4", "ON")
            .register_dep("SNAPPY")
            .define("WITH_SNAPPY", "ON")
            .define("FAIL_ON_WARNINGS", "OFF")
            .define("WITH_RUNTIME_DEBUG", "OFF")
            .define("PORTABLE", "ON");
    }
    let dst = cfg.build_target("titan").very_verbose(true).build();
    let build_dir = format!("{}/build", dst.display());
    if cfg!(target_os = "windows") {
        let profile = match &*env::var("PROFILE").unwrap_or_else(|_| "debug".to_owned()) {
            "bench" | "release" => "Release",
            _ => "Debug",
        };
        println!("cargo:rustc-link-search=native={}/{}", build_dir, profile);
    } else {
        println!("cargo:rustc-link-search=native={}", build_dir);
    }
    println!("cargo:rustc-link-lib=static=titan");
}
