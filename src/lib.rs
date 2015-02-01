//! Safe Rust bindings for the [Symas Lightning Memory-Mapped Database(LMDB)]
//! (http://symas.com/mdb/).
//!
//! Provides the minimal amount of abstraction necessary to interact with LMDB safely in Rust. In
//! general, the API is very similar to the LMDB [C-API](http://symas.com/mdb/doc/).

#![feature(collections, core, hash, io, libc, optin_builtin_traits, path, std_misc, test, unsafe_destructor)]
#![cfg_attr(test, feature(rand))]

extern crate libc;
extern crate "lmdb-sys" as ffi;

extern crate test;
extern crate collections;
#[macro_use]
extern crate bitflags;

pub use cursor::{
    Cursor,
    CursorExt,
    RoCursor,
    RwCursor
};
pub use database::Database;
pub use environment::{Environment, EnvironmentBuilder};
pub use error::{LmdbResult, LmdbError};
pub use flags::*;
pub use transaction::{
    InactiveTransaction,
    RoTransaction,
    RwTransaction,
    Transaction,
    TransactionExt,
};

macro_rules! lmdb_try {
    ($expr:expr) => ({
        match $expr {
            ::ffi::MDB_SUCCESS => (),
            err_code => return Err(::std::error::FromError::from_error(::LmdbError::from_err_code(err_code))),
        }
    })
}

macro_rules! lmdb_try_with_cleanup {
    ($expr:expr, $cleanup:expr) => ({
        match $expr {
            ::ffi::MDB_SUCCESS => (),
            err_code => {
                let _ = $cleanup;
                return Err(::std::error::FromError::from_error(::LmdbError::from_err_code(err_code)))
            },
        }
    })
}

mod flags;
mod cursor;
mod database;
mod environment;
mod error;
mod transaction;

#[cfg(test)]
mod test_utils {

    use std::old_io as io;

    use super::*;

    pub fn get_key(n: u32) -> String {
        format!("key{}", n)
    }

    pub fn get_data(n: u32) -> String {
        format!("data{}", n)
    }

    pub fn setup_bench_db<'a>(num_rows: u32) -> (io::TempDir, Environment) {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();

        {
            let db = env.open_db(None).unwrap();
            let mut txn = env.begin_rw_txn().unwrap();
            for i in range(0, num_rows) {
                txn.put(db,
                        get_key(i).as_bytes(),
                        get_data(i).as_bytes(),
                        WriteFlags::empty())
                    .unwrap();
            }
            txn.commit().unwrap();
        }
        (dir, env)
    }
}
