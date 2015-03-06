#![feature(path)]

extern crate "pkg-config" as pkg_config;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {

    let mut lmdb: PathBuf = PathBuf::new(&env::var("CARGO_MANIFEST_DIR").unwrap());
    lmdb.push("mdb");
    lmdb.push("libraries");
    lmdb.push("liblmdb");

    let mut mdb: PathBuf = lmdb.clone();
    let mut midl: PathBuf = lmdb.clone();

    mdb.push("mdb.c");
    midl.push("midl.c");

    if !pkg_config::find_library("liblmdb").is_ok() {
        gcc::compile_library("liblmdb.a",
                             &[(*mdb).to_str().unwrap(),
                               (*midl).to_str().unwrap()]);
    }
}
