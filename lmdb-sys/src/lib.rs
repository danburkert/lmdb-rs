#![allow(dead_code, uppercase_variables, non_camel_case_types)]
#![feature(plugin)]

#[plugin]
extern crate bindgen;
extern crate libc;

use libc::{size_t, mode_t};
pub use constants::*;

mod constants;

bindgen!("../mdb/libraries/liblmdb/lmdb.h", match="lmdb.h", link="lmdb");
