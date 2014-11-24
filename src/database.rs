use std::kinds::marker;
use std::ptr;

use error::{LmdbResult, lmdb_result};
use ffi::{MDB_dbi, mdb_dbi_open};
use flags::DatabaseFlags;
use transaction::Transaction;

/// A handle to an individual database in an environment.
///
/// A database handle denotes the name and parameters of a database in an environment. The database
/// may not exist in the environment (for instance, if the database is opened during a transaction
/// that has not yet committed).
pub struct Database<'env> {
    dbi: MDB_dbi,
    _marker: marker::ContravariantLifetime<'env>,
}

impl <'env> Copy for Database<'env> { }

impl <'env> Database<'env> {

    /// Opens a database in the given transaction. Prefer using `Transaction::open_db`.
    #[doc(hidden)]
    pub fn new(txn: &Transaction<'env>,
               name: Option<&str>,
               flags: DatabaseFlags)
               -> LmdbResult<Database<'env>> {
        let c_name = name.map(|n| n.to_c_str());
        let name_ptr = if let Some(ref c_name) = c_name { c_name.as_ptr() } else { ptr::null() };
        let mut dbi: MDB_dbi = 0;
        unsafe {
            try!(lmdb_result(mdb_dbi_open(txn.txn(), name_ptr, flags.bits(), &mut dbi)));
        }
        Ok(Database { dbi: dbi, _marker: marker::ContravariantLifetime::<'env> })
    }

    /// Returns the underlying LMDB database handle.
    ///
    /// The caller **must** ensure that the handle is not used after the lifetime of the
    /// environment, or after the database handle has been closed.
    pub fn dbi(&self) -> MDB_dbi {
        self.dbi
    }
}
