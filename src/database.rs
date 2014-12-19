use std::kinds::marker;
use std::ptr;

use error::{LmdbResult, lmdb_result};
use ffi::*;
use transaction::{Transaction, ReadTransaction, WriteTransaction};

/// A handle to an individual database in an environment.
///
/// A database handle denotes the name and parameters of a database in an environment. The database
/// may not exist in the environment (for instance, if the database is opened during a transaction
/// that has not yet committed).
#[deriving(Clone, Copy)]
pub struct Database<'env> {
    dbi: MDB_dbi,
    _marker: marker::ContravariantLifetime<'env>,
}

impl <'env> Database<'env> {

    pub unsafe fn open<'e>(txn: &ReadTransaction<'e>,
                             name: Option<&str>)
                             -> LmdbResult<Database<'e>> {
        let c_name = name.map(|n| n.to_c_str());
        let name_ptr = if let Some(ref c_name) = c_name { c_name.as_ptr() } else { ptr::null() };
        let mut dbi: MDB_dbi = 0;
        try!(lmdb_result(mdb_dbi_open(txn.txn(), name_ptr, 0, &mut dbi)));
        Ok(Database { dbi: dbi,
                      _marker: marker::ContravariantLifetime::<'e> })
    }

    pub unsafe fn create<'e>(txn: &WriteTransaction<'e>,
                               name: Option<&str>,
                               flags: DatabaseFlags)
                               -> LmdbResult<Database<'e>> {
        let c_name = name.map(|n| n.to_c_str());
        let name_ptr = if let Some(ref c_name) = c_name { c_name.as_ptr() } else { ptr::null() };
        let mut dbi: MDB_dbi = 0;
        try!(lmdb_result(mdb_dbi_open(txn.txn(), name_ptr, flags.bits() | MDB_CREATE, &mut dbi)));
        Ok(Database { dbi: dbi,
                      _marker: marker::ContravariantLifetime::<'e> })
    }

    /// Returns the underlying LMDB database handle.
    ///
    /// The caller **must** ensure that the handle is not used after the lifetime of the
    /// environment, or after the database handle has been closed.
    pub fn dbi(&self) -> MDB_dbi {
        self.dbi
    }
}
