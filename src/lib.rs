//! Safe Rust bindings for the [Symas Lightning Memory-Mapped Database(LMDB)]
//! (http://symas.com/mdb/).
//!
//! Provides the minimal amount of abstraction necessary to interact with LMDB safely in Rust. In
//! general, the API is very similar to the LMDB [C-API](http://symas.com/mdb/doc/).

#![feature(phase, globs, macro_rules, unsafe_destructor, if_let)]

#[phase(plugin, link)] extern crate log;
extern crate libc;
extern crate sync;
extern crate "lmdb-sys" as ffi;

pub use environment::{Environment, EnvironmentBuilder};
pub use error::{LmdbResult, LmdbError};
pub use transaction::{Database, Transaction};

macro_rules! lmdb_try {
    ($expr:expr) => ({
        match $expr {
            ffi::MDB_SUCCESS => (),
            err_code => return Err(::std::error::FromError::from_error(LmdbError::from_err_code(err_code))),
        }
    })
}

macro_rules! lmdb_try_with_cleanup {
    ($expr:expr, $cleanup:expr) => ({
        match $expr {
            ffi::MDB_SUCCESS => (),
            err_code => {
                let _ = $cleanup;
                return Err(::std::error::FromError::from_error(LmdbError::from_err_code(err_code)))
            },
        }
    })
}

mod environment;
mod error;
mod transaction;
pub mod flags;
