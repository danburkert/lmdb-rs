//! Idiomatic and safe APIs for interacting with the
//! [Symas Lightning Memory-Mapped Database (LMDB)](http://symas.com/mdb/).

#![feature(core, libc, optin_builtin_traits, std_misc, unsafe_destructor)]
#![cfg_attr(test, feature(tempdir, test))]

extern crate libc;
extern crate "lmdb-sys" as ffi;

#[cfg(test)] extern crate rand;
#[cfg(test)] extern crate test;
#[macro_use] extern crate bitflags;

pub use cursor::{
    Cursor,
    CursorExt,
    RoCursor,
    RwCursor
};
pub use database::Database;
pub use environment::{Environment, EnvironmentBuilder};
pub use error::{Error, Result};
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
            err_code => return Err(::std::error::FromError::from_error(::Error::from_err_code(err_code))),
        }
    })
}

macro_rules! lmdb_try_with_cleanup {
    ($expr:expr, $cleanup:expr) => ({
        match $expr {
            ::ffi::MDB_SUCCESS => (),
            err_code => {
                let _ = $cleanup;
                return Err(::std::error::FromError::from_error(::Error::from_err_code(err_code)))
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

    use std::fs;

    use super::*;

    pub fn get_key(n: u32) -> String {
        format!("key{}", n)
    }

    pub fn get_data(n: u32) -> String {
        format!("data{}", n)
    }

    pub fn setup_bench_db<'a>(num_rows: u32) -> (fs::TempDir, Environment) {
        let dir = fs::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path()).unwrap();

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
