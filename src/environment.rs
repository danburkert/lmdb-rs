use libc::{c_uint, size_t, mode_t};
use std::ffi::{AsOsStr, CString};
use std::os::unix::OsStrExt;
use std::old_io::FilePermission;
use std::path::Path;
use std::ptr;
use std::sync::Mutex;

use ffi;

use error::{LmdbResult, lmdb_result};
use database::Database;
use transaction::{RoTransaction, RwTransaction, Transaction, TransactionExt};
use flags::{DatabaseFlags, EnvironmentFlags};

/// An LMDB environment.
///
/// An environment supports multiple databases, all residing in the same shared-memory map.
pub struct Environment {
    env: *mut ffi::MDB_env,
    dbi_open_mutex: Mutex<()>,
}

impl Environment {

    /// Creates a new builder for specifying options for opening an LMDB environment.
    pub fn new() -> EnvironmentBuilder {
        EnvironmentBuilder {
            flags: EnvironmentFlags::empty(),
            max_readers: None,
            max_dbs: None,
            map_size: None
        }
    }

    /// Returns a raw pointer to the underlying LMDB environment.
    ///
    /// The caller **must** ensure that the pointer is not dereferenced after the lifetime of the
    /// environment.
    pub fn env(&self) -> *mut ffi::MDB_env {
        self.env
    }

    /// Opens a handle to an LMDB database.
    ///
    /// If `name` is `None`, then the returned handle will be for the default database.
    ///
    /// If `name` is not `None`, then the returned handle will be for a named database. In this
    /// case the environment must be configured to allow named databases through
    /// `EnvironmentBuilder::set_max_dbs`.
    ///
    /// The returned database handle may be shared among any transaction in the environment.
    ///
    /// This function will fail with `LmdbError::BadRslot` if called by a thread which has an ongoing
    /// transaction.
    ///
    /// The database name may not contain the null character.
    pub fn open_db<'env>(&'env self, name: Option<&str>) -> LmdbResult<Database> {
        let mutex = self.dbi_open_mutex.lock();
        let txn = try!(self.begin_ro_txn());
        let db = unsafe { try!(txn.open_db(name)) };
        try!(txn.commit());
        drop(mutex);
        Ok(db)
    }

    /// Opens a handle to an LMDB database, creating the database if necessary.
    ///
    /// If the database is already created, the given option flags will be added to it.
    ///
    /// If `name` is `None`, then the returned handle will be for the default database.
    ///
    /// If `name` is not `None`, then the returned handle will be for a named database. In this
    /// case the environment must be configured to allow named databases through
    /// `EnvironmentBuilder::set_max_dbs`.
    ///
    /// The returned database handle may be shared among any transaction in the environment.
    ///
    /// This function will fail with `LmdbError::BadRslot` if called by a thread with an open
    /// transaction.
    pub fn create_db<'env>(&'env self,
                           name: Option<&str>,
                           flags: DatabaseFlags)
                           -> LmdbResult<Database> {
        let mutex = self.dbi_open_mutex.lock();
        let txn = try!(self.begin_rw_txn());
        let db = unsafe { try!(txn.create_db(name, flags)) };
        try!(txn.commit());
        drop(mutex);
        Ok(db)
    }

    pub fn get_db_flags<'env>(&'env self, db: Database) -> LmdbResult<DatabaseFlags> {
        let txn = try!(self.begin_ro_txn());
        let mut flags: c_uint = 0;
        unsafe {
            try!(lmdb_result(ffi::mdb_dbi_flags(txn.txn(), db.dbi(), &mut flags)));
        }
        Ok(DatabaseFlags::from_bits(flags).unwrap())
    }

    /// Create a read-only transaction for use with the environment.
    pub fn begin_ro_txn<'env>(&'env self) -> LmdbResult<RoTransaction<'env>> {
        RoTransaction::new(self)
    }

    /// Create a read-write transaction for use with the environment. This method will block while
    /// there are any other read-write transactions open on the environment.
    pub fn begin_rw_txn<'env>(&'env self) -> LmdbResult<RwTransaction<'env>> {
        RwTransaction::new(self)
    }

    /// Flush data buffers to disk.
    ///
    /// Data is always written to disk when `Transaction::commit` is called, but the operating
    /// system may keep it buffered. LMDB always flushes the OS buffers upon commit as well, unless
    /// the environment was opened with `MDB_NOSYNC` or in part `MDB_NOMETASYNC`.
    pub fn sync(&self, force: bool) -> LmdbResult<()> {
        unsafe {
            lmdb_result(ffi::mdb_env_sync(self.env(), if force { 1 } else { 0 }))
        }
    }

    /// Closes the database handle. Normally unnecessary.
    ///
    /// Closing a database handle is not necessary, but lets `Transaction::open_database` reuse the
    /// handle value. Usually it's better to set a bigger `EnvironmentBuilder::set_max_dbs`, unless
    /// that value would be large.
    ///
    /// ## Safety
    ///
    /// This call is not mutex protected. Databases should only be closed by a single thread, and
    /// only if no other threads are going to reference the database handle or one of its cursors
    /// any further. Do not close a handle if an existing transaction has modified its database.
    /// Doing so can cause misbehavior from database corruption to errors like
    /// `LmdbError::BadValSize` (since the DB name is gone).
    pub unsafe fn close_db(&mut self, db: Database) {
        ffi::mdb_dbi_close(self.env, db.dbi());
    }
}

unsafe impl Send for Environment {}
unsafe impl Sync for Environment {}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe { ffi::mdb_env_close(self.env) }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
//// Environment Builder
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Options for opening or creating an environment.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct EnvironmentBuilder {
    flags: EnvironmentFlags,
    max_readers: Option<c_uint>,
    max_dbs: Option<c_uint>,
    map_size: Option<size_t>,
}

impl EnvironmentBuilder {

    /// Open an environment.
    ///
    /// The path may not contain the null character.
    pub fn open(&self, path: &Path, mode: FilePermission) -> LmdbResult<Environment> {
        let mut env: *mut ffi::MDB_env = ptr::null_mut();
        unsafe {
            lmdb_try!(ffi::mdb_env_create(&mut env));
            if let Some(max_readers) = self.max_readers {
                lmdb_try_with_cleanup!(ffi::mdb_env_set_maxreaders(env, max_readers),
                                       ffi::mdb_env_close(env))
            }
            if let Some(max_dbs) = self.max_dbs {
                lmdb_try_with_cleanup!(ffi::mdb_env_set_maxdbs(env, max_dbs),
                                       ffi::mdb_env_close(env))
            }
            if let Some(map_size) = self.map_size {
                lmdb_try_with_cleanup!(ffi::mdb_env_set_mapsize(env, map_size),
                                       ffi::mdb_env_close(env))
            }
            lmdb_try_with_cleanup!(ffi::mdb_env_open(env,
                                                     CString::new(path.as_os_str().as_bytes()).unwrap().as_ptr(),
                                                     self.flags.bits(),
                                                     mode.bits() as mode_t),
                                   ffi::mdb_env_close(env));
        }
        Ok(Environment { env: env,
                         dbi_open_mutex: Mutex::new(()) })
    }

    pub fn set_flags(&mut self, flags: EnvironmentFlags) -> &mut EnvironmentBuilder {
        self.flags = flags;
        self
    }

    /// Sets the maximum number of threads or reader slots for the environment.
    ///
    /// This defines the number of slots in the lock table that is used to track readers in the
    /// the environment. The default is 126. Starting a read-only transaction normally ties a lock
    /// table slot to the current thread until the environment closes or the thread exits. If
    /// `MDB_NOTLS` is in use, `Environment::open_txn` instead ties the slot to the `Transaction`
    /// object until it or the `Environment` object is destroyed.
    pub fn set_max_readers(&mut self, max_readers: c_uint) -> &mut EnvironmentBuilder {
        self.max_readers = Some(max_readers);
        self
    }

    /// Sets the maximum number of named databases for the environment.
    ///
    /// This function is only needed if multiple databases will be used in the
    /// environment. Simpler applications that use the environment as a single
    /// unnamed database can ignore this option.
    ///
    /// Currently a moderate number of slots are cheap but a huge number gets
    /// expensive: 7-120 words per transaction, and every `Transaction::open_db`
    /// does a linear search of the opened slots.
    pub fn set_max_dbs(&mut self, max_readers: c_uint) -> &mut EnvironmentBuilder {
        self.max_dbs = Some(max_readers);
        self
    }

    /// Sets the size of the memory map to use for the environment.
    ///
    /// The size should be a multiple of the OS page size. The default is
    /// 10485760 bytes. The size of the memory map is also the maximum size
    /// of the database. The value should be chosen as large as possible,
    /// to accommodate future growth of the database. It may be increased at
    /// later times.
    ///
    /// Any attempt to set a size smaller than the space already consumed
    /// by the environment will be silently changed to the current size of the used space.
    pub fn set_map_size(&mut self, map_size: size_t) -> &mut EnvironmentBuilder {
        self.map_size = Some(map_size);
        self
    }
}

#[cfg(test)]
mod test {

    use std::old_io as io;

    use flags::*;
    use tempdir;

    use super::*;

    #[test]
    fn test_open() {
        let dir = tempdir::TempDir::new("test").unwrap();

        // opening non-existent env with read-only should fail
        assert!(Environment::new().set_flags(READ_ONLY)
                                  .open(dir.path(), io::USER_RWX)
                                  .is_err());

        // opening non-existent env should succeed
        assert!(Environment::new().open(dir.path(), io::USER_RWX).is_ok());

        // opening env with read-only should succeed
        assert!(Environment::new().set_flags(READ_ONLY)
                                  .open(dir.path(), io::USER_RWX)
                                  .is_ok());
    }

    #[test]
    fn test_begin_txn() {
        let dir = tempdir::TempDir::new("test").unwrap();

        { // writable environment
            let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();

            assert!(env.begin_rw_txn().is_ok());
            assert!(env.begin_ro_txn().is_ok());
        }

        { // read-only environment
            let env = Environment::new().set_flags(READ_ONLY)
                                        .open(dir.path(), io::USER_RWX)
                                        .unwrap();

            assert!(env.begin_rw_txn().is_err());
            assert!(env.begin_ro_txn().is_ok());
        }
    }

    #[test]
    fn test_open_db() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let env = Environment::new().set_max_dbs(1)
                                    .open(dir.path(), io::USER_RWX)
                                    .unwrap();

        assert!(env.open_db(None).is_ok());
        assert!(env.open_db(Some("testdb")).is_err());
    }

    #[test]
    fn test_create_db() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let env = Environment::new().set_max_dbs(11)
                                    .open(dir.path(), io::USER_RWX)
                                    .unwrap();
        assert!(env.open_db(Some("testdb")).is_err());
        assert!(env.create_db(Some("testdb"), DatabaseFlags::empty()).is_ok());
        assert!(env.open_db(Some("testdb")).is_ok())
    }

    #[test]
    fn test_close_database() {
        let dir = tempdir::TempDir::new("test").unwrap();
        let mut env = Environment::new().set_max_dbs(10)
                                        .open(dir.path(), io::USER_RWX)
                                        .unwrap();

        let db = env.create_db(Some("db"), DatabaseFlags::empty()).unwrap();
        unsafe { env.close_db(db); }
        assert!(env.open_db(Some("db")).is_ok());
    }

    #[test]
    fn test_sync() {
        let dir = tempdir::TempDir::new("test").unwrap();
        {
            let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
            assert!(env.sync(true).is_ok());
        } {
            let env = Environment::new().set_flags(READ_ONLY)
                                        .open(dir.path(), io::USER_RWX)
                                        .unwrap();
            assert!(env.sync(true).is_err());
        }
    }
}
