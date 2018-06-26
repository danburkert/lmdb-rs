extern crate pkg_config;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let mut lmdb: PathBuf = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    lmdb.push("lmdb");
    lmdb.push("libraries");
    lmdb.push("liblmdb");

    if !pkg_config::find_library("liblmdb").is_ok() {
        let target = env::var("TARGET").expect("No TARGET found");
        let mut build = cc::Build::new();
        if target.contains("android") {
            build.define("ANDROID", "1");
        }
        build
            .file(lmdb.join("mdb.c"))
            .file(lmdb.join("midl.c"))
            // https://github.com/LMDB/lmdb/blob/LMDB_0.9.21/libraries/liblmdb/Makefile#L25
            .opt_level(2)
            .compile("liblmdb.a")
    }
}
