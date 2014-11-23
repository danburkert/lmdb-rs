extern crate "pkg-config" as pkg_config;

use std::io::process::Command;
use std::os;

/// Run a command and ensure a successful result.
fn run_cmd(cmd: &Command) {
    assert!(cmd.status().unwrap().success(),
            format!("Failed to execute command \"{}\"", cmd))
}

fn main() {
    if pkg_config::find_library("liblmdb").is_ok() { return }

    let base_dir = Path::new(os::getenv("CARGO_MANIFEST_DIR").unwrap());
    let lmdb_dir = base_dir.join_many(&["mdb", "libraries", "liblmdb"]);
    let dest_dir = Path::new(os::getenv("OUT_DIR").unwrap());

    let mut cflags = os::getenv("CFLAGS").unwrap_or(String::new());
    let target = os::getenv("TARGET").unwrap();

    if target.contains("i686") {
        cflags.push_str(" -m32");
    } else if target.contains("x86_64") {
        cflags.push_str(" -m64");
    }
    if !target.contains("i686") {
        cflags.push_str(" -fPIC");
    }

    let mut make = Command::new("make");
    make.arg("-C").arg(lmdb_dir.clone());

    let mut make_build = make.clone();
    make_build.arg("liblmdb.a")
              .arg(format!("XCFLAGS={}", cflags));

    let mut make_clean = make.clone();
    make_clean.arg("clean");

    run_cmd(&make_clean);
    run_cmd(&make_build);
    run_cmd(Command::new("cp")
                    .arg(lmdb_dir.join("liblmdb.a"))
                    .arg(dest_dir.clone()));
    run_cmd(&make_clean);

    println!("cargo:rustc-flags=-L {} -l lmdb:static", dest_dir.display());
}
