#![allow(
    dead_code,
    missing_copy_implementations,
    non_camel_case_types,
    non_snake_case,
    raw_pointer_derive,
   )]

#![feature(libc, plugin)]

extern crate libc;

pub use constants::*;
pub use ffi::*;

mod ffi;
mod constants;
