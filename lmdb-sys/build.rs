extern crate pkg_config;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {

    let mut lmdb: PathBuf = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    lmdb.push("lmdb");
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
