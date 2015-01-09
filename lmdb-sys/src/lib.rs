#![allow(
    dead_code,
    missing_copy_implementations,
    non_camel_case_types,
    non_snake_case,
    raw_pointer_derive,
   )]
#![feature(plugin)]

extern crate libc;

pub use constants::*;

mod constants;

include!(concat!(env!("OUT_DIR"), "/lmdb.rs"));
