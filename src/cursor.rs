use libc::{c_void, size_t};
use std::{mem, ptr, raw};
use std::kinds::marker;

use database::Database;
use error::{LmdbResult, lmdb_result};
use ffi;
use ffi::{MDB_cursor, mdb_cursor_open, MDB_cursor_op, MDB_val};
use flags::WriteFlags;
use transaction::Transaction;

/// A cursor for navigating within a database.
pub struct Cursor<'txn> {
    cursor: *mut MDB_cursor,
    _no_sync: marker::NoSync,
    _no_send: marker::NoSend,
    _contravariant: marker::ContravariantLifetime<'txn>,
}

#[unsafe_destructor]
impl <'txn> Drop for Cursor<'txn> {
    fn drop(&mut self) {
        unsafe { ffi::mdb_cursor_close(self.cursor) }
    }
}

impl <'txn> Cursor<'txn> {

    /// Creates a new cursor into the given database in the given transaction. Prefer using
    /// `Transaction::open_cursor()`.
    #[doc(hidden)]
    pub fn new(txn: &'txn Transaction, db: Database) -> LmdbResult<Cursor<'txn>> {
        let mut cursor: *mut MDB_cursor = ptr::null_mut();
        unsafe { try!(lmdb_result(mdb_cursor_open(txn.txn(), db.dbi(), &mut cursor))); }
        Ok(Cursor {
            cursor: cursor,
            _no_sync: marker::NoSync,
            _no_send: marker::NoSend,
            _contravariant: marker::ContravariantLifetime::<'txn>,
        })
    }

    pub fn cursor(&self) -> *mut MDB_cursor {
        self.cursor
    }

    /// Retrieves a key/data pair from the cursor. Depending on the cursor op, the current key is
    /// returned.
    pub fn get(&self,
               key: Option<&[u8]>,
               data: Option<&[u8]>,
               op: MDB_cursor_op)
               -> LmdbResult<(Option<&'txn [u8]>, &'txn [u8])> {
        unsafe {
            let mut key_val = slice_to_val(key);
            let mut data_val = slice_to_val(data);
            let key_ptr = key_val.mv_data;
            try!(lmdb_result(ffi::mdb_cursor_get(self.cursor,
                                                 &mut key_val,
                                                 &mut data_val,
                                                 op)));
            let key_out = if key_ptr != key_val.mv_data { Some(val_to_slice(key_val)) } else { None };
            let data_out = val_to_slice(data_val);
            Ok((key_out, data_out))
        }
    }

    /// Puts a key/data pair into the database. The cursor will be positioned at the new data item,
    /// or on failure usually near it.
    pub fn put(&self,
               key: &[u8],
               data: &[u8],
               flags: WriteFlags)
               -> LmdbResult<()> {

        let mut key_val: ffi::MDB_val = ffi::MDB_val { mv_size: key.len() as size_t,
                                                       mv_data: key.as_ptr() as *const c_void };
        let mut data_val: ffi::MDB_val = ffi::MDB_val { mv_size: data.len() as size_t,
                                                        mv_data: data.as_ptr() as *const c_void };

        unsafe {
            lmdb_result(ffi::mdb_cursor_put(self.cursor(),
                                            &mut key_val,
                                            &mut data_val,
                                            flags.bits()))
        }
    }

    /// Deletes the current key/data pair.
    ///
    /// ### Flags
    ///
    /// `MDB_NODUPDATA` may be used to delete all data items for the current key, if the database
    /// was opened with `MDB_DUPSORT`.
    pub fn del(&self, flags: WriteFlags) -> LmdbResult<()> {
        unsafe {
            lmdb_result(ffi::mdb_cursor_del(self.cursor(), flags.bits()))
        }
    }
}

unsafe fn slice_to_val(slice: Option<&[u8]>) -> MDB_val {
    match slice {
        Some(slice) =>
            MDB_val { mv_size: slice.len() as size_t,
                      mv_data: slice.as_ptr() as *const c_void },
        None =>
            MDB_val { mv_size: 0,
                      mv_data: ptr::null() },
    }
}

unsafe fn val_to_slice<'a>(val: MDB_val) -> &'a [u8] {
    mem::transmute(raw::Slice {
        data: val.mv_data as *const u8,
        len: val.mv_size as uint
    })
}

#[cfg(test)]
mod test {

    use libc::{c_void, size_t};
    use std::{io, ptr};

    use environment::*;
    use error::{LmdbResult, lmdb_result};
    use flags::*;
    use ffi;
    use ffi::{MDB_cursor_op, MDB_val};
    use super::*;

    #[test]
    fn test_get() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();

        let mut txn = env.begin_txn(EnvironmentFlags::empty()).unwrap();
        let db = txn.open_db(None, DatabaseFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key3", b"val3", WriteFlags::empty()).unwrap();

        let cursor = txn.open_cursor(db).unwrap();
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_cursor_op::MDB_FIRST).unwrap());
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_cursor_op::MDB_GET_CURRENT).unwrap());
        assert_eq!((Some(b"key2"), b"val2"),
                   cursor.get(None, None, MDB_cursor_op::MDB_NEXT).unwrap());
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_cursor_op::MDB_PREV).unwrap());
        assert_eq!((Some(b"key3"), b"val3"),
                   cursor.get(None, None, MDB_cursor_op::MDB_LAST).unwrap());
        assert_eq!((None, b"val2"),
                   cursor.get(Some(b"key2"), None, MDB_cursor_op::MDB_SET).unwrap());
        assert_eq!((Some(b"key3"), b"val3"),
                   cursor.get(Some(b"key3"), None, MDB_cursor_op::MDB_SET_KEY).unwrap());
        assert_eq!((Some(b"key3"), b"val3"),
                   cursor.get(Some(b"key2\0"), None, MDB_cursor_op::MDB_SET_RANGE).unwrap());
    }

    #[test]
    fn test_get_dup() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();

        let mut txn = env.begin_txn(EnvironmentFlags::empty()).unwrap();
        let db = txn.open_db(None, MDB_DUPSORT).unwrap();
        txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val3", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val3", WriteFlags::empty()).unwrap();

        let cursor = txn.open_cursor(db).unwrap();
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_cursor_op::MDB_FIRST).unwrap());
        assert_eq!((None, b"val1"),
                   cursor.get(None, None, MDB_cursor_op::MDB_FIRST_DUP).unwrap());
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_cursor_op::MDB_GET_CURRENT).unwrap());
        assert_eq!((Some(b"key2"), b"val1"),
                   cursor.get(None, None, MDB_cursor_op::MDB_NEXT_NODUP).unwrap());
        assert_eq!((Some(b"key2"), b"val2"),
                   cursor.get(None, None, MDB_cursor_op::MDB_NEXT_DUP).unwrap());
        assert_eq!((Some(b"key2"), b"val3"),
                   cursor.get(None, None, MDB_cursor_op::MDB_NEXT_DUP).unwrap());
        assert!(cursor.get(None, None, MDB_cursor_op::MDB_NEXT_DUP).is_err());
        assert_eq!((Some(b"key2"), b"val2"),
                   cursor.get(None, None, MDB_cursor_op::MDB_PREV_DUP).unwrap());
        assert_eq!((None, b"val3"),
                   cursor.get(None, None, MDB_cursor_op::MDB_LAST_DUP).unwrap());
        assert_eq!((Some(b"key1"), b"val3"),
                   cursor.get(None, None, MDB_cursor_op::MDB_PREV_NODUP).unwrap());
        assert_eq!((None, b"val1"),
                   cursor.get(Some(b"key1"), None, MDB_cursor_op::MDB_SET).unwrap());
        assert_eq!((Some(b"key2"), b"val1"),
                   cursor.get(Some(b"key2"), None, MDB_cursor_op::MDB_SET_KEY).unwrap());
        assert_eq!((Some(b"key2"), b"val1"),
                   cursor.get(Some(b"key1\0"), None, MDB_cursor_op::MDB_SET_RANGE).unwrap());
        assert_eq!((None, b"val3"),
                   cursor.get(Some(b"key1"), Some(b"val3"), MDB_cursor_op::MDB_GET_BOTH).unwrap());
        assert_eq!((None, b"val1"),
                   cursor.get(Some(b"key2"), Some(b"val"), MDB_cursor_op::MDB_GET_BOTH_RANGE).unwrap());
    }

    #[test]
    fn test_get_dupfixed() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();

        let mut txn = env.begin_txn(EnvironmentFlags::empty()).unwrap();
        let db = txn.open_db(None, MDB_DUPSORT | MDB_DUPFIXED).unwrap();
        txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val3", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val4", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val5", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val6", WriteFlags::empty()).unwrap();

        let cursor = txn.open_cursor(db).unwrap();
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_cursor_op::MDB_FIRST).unwrap());
        assert_eq!((None, b"val1val2val3"),
                   cursor.get(None, None, MDB_cursor_op::MDB_GET_MULTIPLE).unwrap());
        assert!(cursor.get(None, None, MDB_cursor_op::MDB_NEXT_MULTIPLE).is_err());
    }

    /// Checks assumptions about which get operations return keys.
    #[test]
    fn test_get_keys() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();

        unsafe fn slice_to_val(slice: Option<&[u8]>) -> MDB_val {
            match slice {
                Some(slice) =>
                    MDB_val { mv_size: slice.len() as size_t,
                              mv_data: slice.as_ptr() as *const c_void },
                None =>
                    MDB_val { mv_size: 0,
                              mv_data: ptr::null() },
            }
        }

        /// Returns true if the cursor get sets the key.
        fn sets_key(cursor: &Cursor,
                    key: Option<&[u8]>,
                    data: Option<&[u8]>,
                    op: MDB_cursor_op)
                    -> LmdbResult<bool> {
            unsafe {
                let mut key_val = slice_to_val(key);
                let mut data_val = slice_to_val(data);
                let key_ptr = key_val.mv_data;
                try!(lmdb_result(ffi::mdb_cursor_get(cursor.cursor(),
                                                     &mut key_val,
                                                     &mut data_val,
                                                     op)));
                Ok(key_ptr != key_val.mv_data)
            }
        }

        let mut txn = env.begin_txn(EnvironmentFlags::empty()).unwrap();
        let db = txn.open_db(None, MDB_DUPSORT | MDB_DUPFIXED).unwrap();
        txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val3", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val4", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val5", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val6", WriteFlags::empty()).unwrap();
        txn.put(db, b"key3", b"val7", WriteFlags::empty()).unwrap();
        txn.put(db, b"key3", b"val8", WriteFlags::empty()).unwrap();
        txn.put(db, b"key3", b"val9", WriteFlags::empty()).unwrap();

        let cursor = txn.open_cursor(db).unwrap();
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_FIRST).unwrap());
        assert!(!sets_key(&cursor, None, None, MDB_cursor_op::MDB_FIRST_DUP).unwrap());
        assert!(!sets_key(&cursor, Some(b"key2"), Some(b"val5"), MDB_cursor_op::MDB_GET_BOTH).unwrap());
        assert!(!sets_key(&cursor, Some(b"key2"), Some(b"val"), MDB_cursor_op::MDB_GET_BOTH_RANGE).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_GET_CURRENT).unwrap());
        assert!(!sets_key(&cursor, None, None, MDB_cursor_op::MDB_GET_MULTIPLE).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_LAST).unwrap());
        assert!(!sets_key(&cursor, None, None, MDB_cursor_op::MDB_LAST_DUP).unwrap());
        sets_key(&cursor, None, None, MDB_cursor_op::MDB_FIRST).unwrap();
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_NEXT).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_NEXT_DUP).unwrap());
        sets_key(&cursor, None, None, MDB_cursor_op::MDB_FIRST).unwrap();
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_NEXT_MULTIPLE).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_NEXT_NODUP).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_PREV).unwrap());
        sets_key(&cursor, None, None, MDB_cursor_op::MDB_LAST).unwrap();
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_PREV_DUP).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_cursor_op::MDB_PREV_NODUP).unwrap());
        assert!(!sets_key(&cursor, Some(b"key2"), None, MDB_cursor_op::MDB_SET).unwrap());
        assert!(sets_key(&cursor, Some(b"key2"), None, MDB_cursor_op::MDB_SET_KEY).unwrap());
        assert!(sets_key(&cursor, Some(b"key2"), None, MDB_cursor_op::MDB_SET_RANGE).unwrap());
    }

    #[test]
    fn test_put_del() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();

        let mut txn = env.begin_txn(EnvironmentFlags::empty()).unwrap();
        let db = txn.open_db(None, DatabaseFlags::empty()).unwrap();
        let cursor = txn.open_cursor(db).unwrap();

        cursor.put(b"key1", b"val1", WriteFlags::empty()).unwrap();
        cursor.put(b"key2", b"val2", WriteFlags::empty()).unwrap();
        cursor.put(b"key3", b"val3", WriteFlags::empty()).unwrap();

        assert_eq!((Some(b"key3"), b"val3"),
                   cursor.get(None, None, MDB_cursor_op::MDB_GET_CURRENT).unwrap());

        cursor.del(WriteFlags::empty()).unwrap();
        assert_eq!((Some(b"key2"), b"val2"),
                   cursor.get(None, None, MDB_cursor_op::MDB_LAST).unwrap());
    }
}
