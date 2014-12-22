use libc::{c_void, size_t, c_uint};
use std::{mem, ptr, raw};
use std::kinds::marker;

use ffi;

use database::Database;
use error::{LmdbResult, lmdb_result, LmdbError};
use flags::WriteFlags;
use transaction::Transaction;

/// An LMDB cursor.
pub trait Cursor<'txn> {
    /// Returns a raw pointer to the underlying LMDB cursor.
    ///
    /// The caller **must** ensure that the pointer is not used after the lifetime of the cursor.
    fn cursor(&self) -> *mut ffi::MDB_cursor;
}

/// Cursor extension methods.
pub trait CursorExt<'txn> : Cursor<'txn> {

    /// Retrieves a key/data pair from the cursor. Depending on the cursor op, the current key is
    /// returned.
    fn get(&self,
           key: Option<&[u8]>,
           data: Option<&[u8]>,
           op: c_uint)
           -> LmdbResult<(Option<&'txn [u8]>, &'txn [u8])> {
        unsafe {
            let mut key_val = slice_to_val(key);
            let mut data_val = slice_to_val(data);
            let key_ptr = key_val.mv_data;
            try!(lmdb_result(ffi::mdb_cursor_get(self.cursor(),
                                                 &mut key_val,
                                                 &mut data_val,
                                                 op)));
            let key_out = if key_ptr != key_val.mv_data { Some(val_to_slice(key_val)) } else { None };
            let data_out = val_to_slice(data_val);
            Ok((key_out, data_out))
        }
    }

    /// Open a new read-only cursor on the given database.
    fn iter(&mut self) -> Items<'txn> {
        Items::new(self)
    }
}

impl<'txn, T> CursorExt<'txn> for T where T: Cursor<'txn> {}

/// A read-only cursor for navigating items within a database.
pub struct RoCursor<'txn> {
    cursor: *mut ffi::MDB_cursor,
    _no_sync: marker::NoSync,
    _no_send: marker::NoSend,
    _contravariant: marker::ContravariantLifetime<'txn>,
}

impl <'txn> Cursor<'txn> for RoCursor<'txn> {
    fn cursor(&self) -> *mut ffi::MDB_cursor {
        self.cursor
    }
}

#[unsafe_destructor]
impl <'txn> Drop for RoCursor<'txn> {
    fn drop(&mut self) {
        unsafe { ffi::mdb_cursor_close(self.cursor) }
    }
}

impl <'txn> RoCursor<'txn> {

    /// Creates a new read-only cursor in the given database and transaction. Prefer using
    /// `Transaction::open_cursor()`.
    #[doc(hidden)]
    pub fn new(txn: &'txn Transaction, db: Database) -> LmdbResult<RoCursor<'txn>> {
        let mut cursor: *mut ffi::MDB_cursor = ptr::null_mut();
        unsafe { try!(lmdb_result(ffi::mdb_cursor_open(txn.txn(), db.dbi(), &mut cursor))); }
        Ok(RoCursor {
            cursor: cursor,
            _no_sync: marker::NoSync,
            _no_send: marker::NoSend,
            _contravariant: marker::ContravariantLifetime::<'txn>,
        })
    }
}

/// A read-only cursor for navigating items within a database.
pub struct RwCursor<'txn> {
    cursor: *mut ffi::MDB_cursor,
    _no_sync: marker::NoSync,
    _no_send: marker::NoSend,
    _contravariant: marker::ContravariantLifetime<'txn>,
}

impl <'txn> Cursor<'txn> for RwCursor<'txn> {
    fn cursor(&self) -> *mut ffi::MDB_cursor {
        self.cursor
    }
}

#[unsafe_destructor]
impl <'txn> Drop for RwCursor<'txn> {
    fn drop(&mut self) {
        unsafe { ffi::mdb_cursor_close(self.cursor) }
    }
}

impl <'txn> RwCursor<'txn> {

    /// Creates a new read-only cursor in the given database and transaction. Prefer using
    /// `WriteTransaction::open_write_cursor()`.
    #[doc(hidden)]
    pub fn new(txn: &'txn Transaction, db: Database) -> LmdbResult<RwCursor<'txn>> {
        let mut cursor: *mut ffi::MDB_cursor = ptr::null_mut();
        unsafe { try!(lmdb_result(ffi::mdb_cursor_open(txn.txn(), db.dbi(), &mut cursor))); }
        Ok(RwCursor {
            cursor: cursor,
            _no_sync: marker::NoSync,
            _no_send: marker::NoSend,
            _contravariant: marker::ContravariantLifetime::<'txn>,
        })
    }

    /// Puts a key/data pair into the database. The cursor will be positioned at the new data item,
    /// or on failure usually near it.
    pub fn put(&self,
           key: &[u8],
           data: &[u8],
           flags: WriteFlags)
           -> LmdbResult<()> {
        let mut key_val: ffi::MDB_val = ffi::MDB_val { mv_size: key.len() as size_t,
                                                       mv_data: key.as_ptr() as *mut c_void };
        let mut data_val: ffi::MDB_val = ffi::MDB_val { mv_size: data.len() as size_t,
                                                        mv_data: data.as_ptr() as *mut c_void };
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
    /// `WriteFlags::NO_DUP_DATA` may be used to delete all data items for the current key, if the
    /// database was opened with `DatabaseFlags::DUP_SORT`.
    pub fn del(&self, flags: WriteFlags) -> LmdbResult<()> {
        unsafe {
            lmdb_result(ffi::mdb_cursor_del(self.cursor(), flags.bits()))
        }
    }
}

unsafe fn slice_to_val(slice: Option<&[u8]>) -> ffi::MDB_val {
    match slice {
        Some(slice) =>
            ffi::MDB_val { mv_size: slice.len() as size_t,
                           mv_data: slice.as_ptr() as *mut c_void },
        None =>
            ffi::MDB_val { mv_size: 0,
                           mv_data: ptr::null_mut() },
    }
}

unsafe fn val_to_slice<'a>(val: ffi::MDB_val) -> &'a [u8] {
    mem::transmute(raw::Slice {
        data: val.mv_data as *const u8,
        len: val.mv_size as uint
    })
}

pub struct Items<'txn> {
    cursor: *mut ffi::MDB_cursor,
    op: c_uint,
    next_op: c_uint,
}

impl <'txn> Items<'txn> {

    /// Creates a new iterator backed by the given cursor.
    fn new<'t>(cursor: &Cursor<'t>) -> Items<'t> {
        Items { cursor: cursor.cursor(), op: ffi::MDB_FIRST, next_op: ffi::MDB_NEXT }
    }
}

impl <'txn> Iterator<(&'txn [u8], &'txn [u8])> for Items<'txn> {

    fn next(&mut self) -> Option<(&'txn [u8], &'txn [u8])> {
        let mut key = ffi::MDB_val { mv_size: 0, mv_data: ptr::null_mut() };
        let mut data = ffi::MDB_val { mv_size: 0, mv_data: ptr::null_mut() };

        unsafe {
            let err_code = ffi::mdb_cursor_get(self.cursor, &mut key, &mut data, self.op);
            // Set the operation for the next get
            self.op = self.next_op;
            if err_code == ffi::MDB_SUCCESS {
                Some((val_to_slice(key), val_to_slice(data)))
            } else {
                // The documentation for mdb_cursor_get specifies that it may fail with MDB_NOTFOUND
                // and MDB_EINVAL (and we shouldn't be passing in invalid parameters).
                // TODO: validate that these are the only failures possible.
                debug_assert!(err_code == ffi::MDB_NOTFOUND,
                              "Unexpected LMDB error {}.", LmdbError::from_err_code(err_code));
                None
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::{io, ptr};
    use test::{Bencher, black_box};

    use ffi::*;

    use environment::*;
    use flags::*;
    use super::*;
    use test_utils::*;
    use transaction::*;

    #[test]
    fn test_iter() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
        let db = env.open_db(None).unwrap();

        let items = vec!((b"key1", b"val1"),
                         (b"key2", b"val2"),
                         (b"key3", b"val3"));

        {
            let mut txn = env.begin_write_txn().unwrap();
            for &(key, data) in items.iter() {
                txn.put(db, key, data, WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        }

        let txn = env.begin_read_txn().unwrap();
        let mut cursor = txn.open_read_cursor(db).unwrap();
        assert_eq!(items, cursor.iter().collect::<Vec<(&[u8], &[u8])>>());
    }

    #[test]
    fn test_get() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
        let db = env.open_db(None).unwrap();

        let mut txn = env.begin_write_txn().unwrap();
        txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key3", b"val3", WriteFlags::empty()).unwrap();

        let cursor = txn.open_read_cursor(db).unwrap();
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_FIRST).unwrap());
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_GET_CURRENT).unwrap());
        assert_eq!((Some(b"key2"), b"val2"),
                   cursor.get(None, None, MDB_NEXT).unwrap());
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_PREV).unwrap());
        assert_eq!((Some(b"key3"), b"val3"),
                   cursor.get(None, None, MDB_LAST).unwrap());
        assert_eq!((None, b"val2"),
                   cursor.get(Some(b"key2"), None, MDB_SET).unwrap());
        assert_eq!((Some(b"key3"), b"val3"),
                   cursor.get(Some(b"key3"), None, MDB_SET_KEY).unwrap());
        assert_eq!((Some(b"key3"), b"val3"),
                   cursor.get(Some(b"key2\0"), None, MDB_SET_RANGE).unwrap());
    }

    #[test]
    fn test_get_dup() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
        let db = env.create_db(None, DUP_SORT).unwrap();

        let mut txn = env.begin_write_txn().unwrap();
        txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val3", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val3", WriteFlags::empty()).unwrap();

        let cursor = txn.open_read_cursor(db).unwrap();
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_FIRST).unwrap());
        assert_eq!((None, b"val1"),
                   cursor.get(None, None, MDB_FIRST_DUP).unwrap());
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_GET_CURRENT).unwrap());
        assert_eq!((Some(b"key2"), b"val1"),
                   cursor.get(None, None, MDB_NEXT_NODUP).unwrap());
        assert_eq!((Some(b"key2"), b"val2"),
                   cursor.get(None, None, MDB_NEXT_DUP).unwrap());
        assert_eq!((Some(b"key2"), b"val3"),
                   cursor.get(None, None, MDB_NEXT_DUP).unwrap());
        assert!(cursor.get(None, None, MDB_NEXT_DUP).is_err());
        assert_eq!((Some(b"key2"), b"val2"),
                   cursor.get(None, None, MDB_PREV_DUP).unwrap());
        assert_eq!((None, b"val3"),
                   cursor.get(None, None, MDB_LAST_DUP).unwrap());
        assert_eq!((Some(b"key1"), b"val3"),
                   cursor.get(None, None, MDB_PREV_NODUP).unwrap());
        assert_eq!((None, b"val1"),
                   cursor.get(Some(b"key1"), None, MDB_SET).unwrap());
        assert_eq!((Some(b"key2"), b"val1"),
                   cursor.get(Some(b"key2"), None, MDB_SET_KEY).unwrap());
        assert_eq!((Some(b"key2"), b"val1"),
                   cursor.get(Some(b"key1\0"), None, MDB_SET_RANGE).unwrap());
        assert_eq!((None, b"val3"),
                   cursor.get(Some(b"key1"), Some(b"val3"), MDB_GET_BOTH).unwrap());
        assert_eq!((None, b"val1"),
                   cursor.get(Some(b"key2"), Some(b"val"), MDB_GET_BOTH_RANGE).unwrap());
    }

    #[test]
    fn test_get_dupfixed() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
        let db = env.create_db(None, DUP_SORT | DUP_FIXED).unwrap();

        let mut txn = env.begin_write_txn().unwrap();
        txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val3", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val4", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val5", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val6", WriteFlags::empty()).unwrap();

        let cursor = txn.open_read_cursor(db).unwrap();
        assert_eq!((Some(b"key1"), b"val1"),
                   cursor.get(None, None, MDB_FIRST).unwrap());
        assert_eq!((None, b"val1val2val3"),
                   cursor.get(None, None, MDB_GET_MULTIPLE).unwrap());
        assert!(cursor.get(None, None, MDB_NEXT_MULTIPLE).is_err());
    }

    #[test]
    fn test_put_del() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
        let db = env.open_db(None).unwrap();

        let mut txn = env.begin_write_txn().unwrap();
        let cursor = txn.open_write_cursor(db).unwrap();

        cursor.put(b"key1", b"val1", WriteFlags::empty()).unwrap();
        cursor.put(b"key2", b"val2", WriteFlags::empty()).unwrap();
        cursor.put(b"key3", b"val3", WriteFlags::empty()).unwrap();

        assert_eq!((Some(b"key3"), b"val3"),
                   cursor.get(None, None, MDB_GET_CURRENT).unwrap());

        cursor.del(WriteFlags::empty()).unwrap();
        assert_eq!((Some(b"key2"), b"val2"),
                   cursor.get(None, None, MDB_LAST).unwrap());
    }

    /// Benchmark of iterator sequential read performance.
    #[bench]
    fn bench_get_seq_iter(b: &mut Bencher) {
        let n = 100;
        let (_dir, env) = setup_bench_db(n);
        let db = env.open_db(None).unwrap();
        let txn = env.begin_read_txn().unwrap();

        b.iter(|| {
            let mut cursor = txn.open_read_cursor(db).unwrap();
            let mut i = 0;
            let mut count = 0u32;

            for (key, data) in cursor.iter() {
                i = i + key.len() + data.len();
                count = count + 1;
            }

            black_box(i);
            assert_eq!(count, n);
        });
    }

    /// Benchmark of cursor sequential read performance.
    #[bench]
    fn bench_get_seq_cursor(b: &mut Bencher) {
        let n = 100;
        let (_dir, env) = setup_bench_db(n);
        let db = env.open_db(None).unwrap();
        let txn = env.begin_read_txn().unwrap();

        b.iter(|| {
            let cursor = txn.open_read_cursor(db).unwrap();
            let mut i = 0;
            let mut count = 0u32;

            while let Ok((key_opt, val)) = cursor.get(None, None, MDB_NEXT) {
                i += key_opt.map(|key| key.len()).unwrap_or(0) + val.len();
                count += 1;
            }

            black_box(i);
            assert_eq!(count, n);
        });
    }

    /// Benchmark of raw LMDB sequential read performance (control).
    #[bench]
    fn bench_get_seq_raw(b: &mut Bencher) {
        let n = 100;
        let (_dir, env) = setup_bench_db(n);
        let db = env.open_db(None).unwrap();

        let dbi: MDB_dbi = db.dbi();
        let _txn = env.begin_read_txn().unwrap();
        let txn = _txn.txn();

        let mut key = MDB_val { mv_size: 0, mv_data: ptr::null_mut() };
        let mut data = MDB_val { mv_size: 0, mv_data: ptr::null_mut() };
        let mut cursor: *mut MDB_cursor = ptr::null_mut();

        b.iter(|| unsafe {
            mdb_cursor_open(txn, dbi, &mut cursor);
            let mut i = 0;
            let mut count = 0u32;

            while mdb_cursor_get(cursor, &mut key, &mut data, MDB_NEXT) == 0 {
                i += key.mv_size + data.mv_size;
                count += 1;
            };

            black_box(i);
            assert_eq!(count, n);
            mdb_cursor_close(cursor);
        });
    }
}
