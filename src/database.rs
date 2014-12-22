use std::ptr;

use ffi;

use error::{LmdbResult, lmdb_result};
use flags::DatabaseFlags;
use transaction::{RwTransaction, Transaction};

/// A handle to an individual database in an environment.
///
/// A database handle denotes the name and parameters of a database in an environment.
#[deriving(Clone, Copy)]
pub struct Database {
    dbi: ffi::MDB_dbi,
}

impl Database {

    /// Opens a database in the provided transaction.
    ///
    /// If `name` is `None`, then the default database will be opened, otherwise a named database
    /// will be opened. The database handle will be private to the transaction until the transaction
    /// is successfully committed. If the transaction is aborted the returned database handle
    /// should no longer be used.
    ///
    /// ## Unsafety
    ///
    /// * This function (as well as `Environment::open_db`, `Environment::create_db`, and
    /// `Database::create`) **must not** be called from multiple concurrent transactions in the same
    /// environment. A transaction which uses this function must finish (either commit or abort)
    /// before any other transaction may use this function.
    pub unsafe fn open(txn: &Transaction,
                       name: Option<&str>)
                       -> LmdbResult<Database> {
        let c_name = name.map(|n| n.to_c_str());
        let name_ptr = if let Some(ref c_name) = c_name { c_name.as_ptr() } else { ptr::null() };
        let mut dbi: ffi::MDB_dbi = 0;
        try!(lmdb_result(ffi::mdb_dbi_open(txn.txn(), name_ptr, 0, &mut dbi)));
        Ok(Database { dbi: dbi })
    }

    /// Opens a handle in the provided transaction, creating the database if necessary.
    ///
    /// If `name` is `None`, then the default database will be opened, otherwise a named database
    /// will be opened. The database handle will be private to the transaction until the transaction
    /// is successfully committed. If the transaction is aborted the returned database handle
    /// should no longer be used.
    ///
    /// ## Unsafety
    ///
    /// * This function (as well as `Environment::open_db`, `Environment::create_db`, and
    /// `Database::open`) **must not** be called from multiple concurrent transactions in the same
    /// environment. A transaction which uses this function must finish (either commit or abort)
    /// before any other transaction may use this function.
    pub unsafe fn create(txn: &RwTransaction,
                         name: Option<&str>,
                         flags: DatabaseFlags)
                         -> LmdbResult<Database> {
        let c_name = name.map(|n| n.to_c_str());
        let name_ptr = if let Some(ref c_name) = c_name { c_name.as_ptr() } else { ptr::null() };
        let mut dbi: ffi::MDB_dbi = 0;
        try!(lmdb_result(ffi::mdb_dbi_open(txn.txn(), name_ptr, flags.bits() | ffi::MDB_CREATE, &mut dbi)));
        Ok(Database { dbi: dbi })
    }

    /// Returns the underlying LMDB database handle.
    ///
    /// The caller **must** ensure that the handle is not used after the lifetime of the
    /// environment, or after the database handle has been closed.
    pub fn dbi(&self) -> ffi::MDB_dbi {
        self.dbi
    }
}
