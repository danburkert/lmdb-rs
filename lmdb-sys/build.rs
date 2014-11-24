extern crate "pkg-config" as pkg_config;
extern crate gcc;

use std::default::Default;
use std::os;

fn main() {
    if !pkg_config::find_library("liblmdb").is_ok() {

        let mdb = Path::new(os::getenv("CARGO_MANIFEST_DIR").unwrap())
                       .join_many(&["mdb", "libraries", "liblmdb"]);

        gcc::compile_library("liblmdb.a",
                             &Default::default(),
                             &[mdb.join("mdb.c").as_str().unwrap(),
                               mdb.join("midl.c").as_str().unwrap()])
    }
}
