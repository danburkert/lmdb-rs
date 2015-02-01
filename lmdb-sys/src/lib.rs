#![allow(
    dead_code,
    missing_copy_implementations,
    non_camel_case_types,
    non_snake_case,
    raw_pointer_derive,
   )]

#![feature(libc, plugin)]

extern crate libc;

#[macro_use]
extern crate bitflags;

pub use constants::*;

mod constants;

include!(concat!(env!("OUT_DIR"), "/lmdb.rs"));
