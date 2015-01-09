extern crate "pkg-config" as pkg_config;
extern crate bindgen;
extern crate gcc;

use bindgen::{Bindings, BindgenOptions, LinkType};
use std::default::Default;
use std::io::fs;
use std::os;

fn main() {

    let mdb = Path::new(os::getenv("CARGO_MANIFEST_DIR").unwrap())
                   .join_many(&["mdb", "libraries", "liblmdb"]);

    if !pkg_config::find_library("liblmdb").is_ok() {
        gcc::compile_library("liblmdb.a",
                             &Default::default(),
                             &[mdb.join("mdb.c").as_str().unwrap(),
                               mdb.join("midl.c").as_str().unwrap()]);
    }

    let mut bindgen_opts: BindgenOptions = Default::default();
    bindgen_opts.clang_args.push(mdb.join("lmdb.h").as_str().unwrap().to_string());
    bindgen_opts.links.push(("lmdb".to_string(), LinkType::Default));
    bindgen_opts.builtins = true;

    let bindings: Bindings = Bindings::generate(&bindgen_opts, None, None).unwrap();
    let mut dst = fs::File::create(&Path::new(os::getenv("OUT_DIR").unwrap())
                                         .join("lmdb.rs")).unwrap();
    bindings.write(&mut dst).unwrap()
}
