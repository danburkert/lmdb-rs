use libc::{c_void, size_t, c_uint};
use std::{mem, ptr, raw};
use std::kinds::marker;

use database::Database;
use error::{LmdbResult, lmdb_result, LmdbError};
use ffi;
use ffi::{MDB_cursor, mdb_cursor_open, MDB_val, WriteFlags};
use transaction::Transaction;

/// An LMDB cursor.
pub trait Cursor<'txn> {
    /// Returns a raw pointer to the underlying LMDB cursor.
    ///
    /// The caller **must** ensure that the pointer is not used after the lifetime of the cursor.
    fn cursor(&self) -> *mut MDB_cursor;
}

pub trait ReadCursor<'txn> : Cursor<'txn> {

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
}

pub trait WriteCursor<'txn> : ReadCursor<'txn> {

    /// Puts a key/data pair into the database. The cursor will be positioned at the new data item,
    /// or on failure usually near it.
    fn put(&self,
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
    /// `MDB_NODUPDATA` may be used to delete all data items for the current key, if the database
    /// was opened with `MDB_DUPSORT`.
    fn del(&self, flags: WriteFlags) -> LmdbResult<()> {
        unsafe {
            lmdb_result(ffi::mdb_cursor_del(self.cursor(), flags.bits()))
        }
    }
}

/// A read-only cursor for navigating items within a database.
pub struct RoCursor<'txn> {
    cursor: *mut MDB_cursor,
    _no_sync: marker::NoSync,
    _no_send: marker::NoSend,
    _contravariant: marker::ContravariantLifetime<'txn>,
}

impl <'txn> Cursor<'txn> for RoCursor<'txn> {
    fn cursor(&self) -> *mut MDB_cursor {
        self.cursor
    }
}

impl <'txn> ReadCursor<'txn> for RoCursor<'txn> { }

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
        let mut cursor: *mut MDB_cursor = ptr::null_mut();
        unsafe { try!(lmdb_result(mdb_cursor_open(txn.txn(), db.dbi(), &mut cursor))); }
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
    cursor: *mut MDB_cursor,
    _no_sync: marker::NoSync,
    _no_send: marker::NoSend,
    _contravariant: marker::ContravariantLifetime<'txn>,
}

impl <'txn> Cursor<'txn> for RwCursor<'txn> {
    fn cursor(&self) -> *mut MDB_cursor {
        self.cursor
    }
}

impl <'txn> ReadCursor<'txn> for RwCursor<'txn> { }
impl <'txn> WriteCursor<'txn> for RwCursor<'txn> { }

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
        let mut cursor: *mut MDB_cursor = ptr::null_mut();
        unsafe { try!(lmdb_result(mdb_cursor_open(txn.txn(), db.dbi(), &mut cursor))); }
        Ok(RwCursor {
            cursor: cursor,
            _no_sync: marker::NoSync,
            _no_send: marker::NoSend,
            _contravariant: marker::ContravariantLifetime::<'txn>,
        })
    }
}

unsafe fn slice_to_val(slice: Option<&[u8]>) -> MDB_val {
    match slice {
        Some(slice) =>
            MDB_val { mv_size: slice.len() as size_t,
                      mv_data: slice.as_ptr() as *mut c_void },
        None =>
            MDB_val { mv_size: 0,
                      mv_data: ptr::null_mut() },
    }
}

unsafe fn val_to_slice<'a>(val: MDB_val) -> &'a [u8] {
    mem::transmute(raw::Slice {
        data: val.mv_data as *const u8,
        len: val.mv_size as uint
    })
}

pub struct Items<'txn> {
    cursor: *mut MDB_cursor,
    op: c_uint,
    next_op: c_uint,
}

impl <'txn> Items<'txn> {

    /// Creates a new read-only cursor in the given database and transaction. Prefer using
    /// `WriteTransaction::open_write_cursor()`.
    pub fn new(txn: &'txn Transaction, db: Database) -> LmdbResult<Items<'txn>> {
        let mut cursor: *mut MDB_cursor = ptr::null_mut();
        unsafe {
            // Create the cursor
            try!(lmdb_result(mdb_cursor_open(txn.txn(), db.dbi(), &mut cursor)));
        }
        Ok(Items { cursor: cursor, op: ffi::MDB_FIRST, next_op: ffi::MDB_NEXT })
    }
}

impl <'txn> Iterator<(&'txn [u8], &'txn [u8])> for Items<'txn> {

    fn next(&mut self) -> Option<(&'txn [u8], &'txn [u8])> {
        let mut key = MDB_val { mv_size: 0, mv_data: ptr::null_mut() };
        let mut data = MDB_val { mv_size: 0, mv_data: ptr::null_mut() };

        unsafe {
            let err_code = ffi::mdb_cursor_get(self.cursor, &mut key, &mut data, self.op);
            if err_code == ffi::MDB_NOTFOUND {
                None
            } else {
                // The documentation and a quick reading of mdb_cursor_get say that mdb_cursor_get
                // may only fail with MDB_NOTFOUND and MDB_EINVAL (and we shouldn't be passing in
                // invalid parameters).
                debug_assert!(err_code == 0, "Unexpected error {}.", LmdbError::from_err_code(err_code));

                // Seek to the next item
                self.op = self.next_op;

                Some((val_to_slice(key), val_to_slice(data)))
            }
        }
    }
}

#[cfg(test)]
mod test {

    use libc::{c_void, size_t, c_uint};
    use std::{io, ptr};
    use test::{Bencher, black_box};
    use collections::BTreeMap;

    use transaction::*;
    use environment::*;
    use error::{LmdbResult, lmdb_result};
    use ffi::*;
    use super::*;

    #[test]
    fn test_items() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
        let db = env.open_db(None).unwrap();

        {
            let mut txn = env.begin_write_txn().unwrap();
            txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
            txn.put(db, b"key2", b"val2", WriteFlags::empty()).unwrap();
            txn.put(db, b"key3", b"val3", WriteFlags::empty()).unwrap();
            txn.commit().unwrap();
        }

        let txn = env.begin_read_txn().unwrap();
        let iter = txn.iter(db).unwrap();

        let items: Vec<(&[u8], &[u8])> = iter.collect();
        assert_eq!(vec!((b"key1", b"val1"),
                        (b"key2", b"val2"),
                        (b"key3", b"val3")),
                    items);
    }

    fn bench_items(b: &mut Bencher, n: uint) {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
        let db = env.open_db(None).unwrap();

        {
            let mut txn = env.begin_write_txn().unwrap();
            for i in range(0, n) {
                txn.put(db, format!("key{}", i).as_bytes(), format!("val{}", i).as_bytes(), WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        }

        let txn = env.begin_read_txn().unwrap();
        b.iter(|| {
            for item in txn.iter(db).unwrap() {
                black_box(item);
            }
        });
    }

    fn bench_cursor(b: &mut Bencher, n: uint) {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();
        let db = env.open_db(None).unwrap();

        {
            let mut txn = env.begin_write_txn().unwrap();
            for i in range(0, n) {
                txn.put(db, format!("key{}", i).as_bytes(), format!("val{}", i).as_bytes(), WriteFlags::empty()).unwrap();
            }
            txn.commit().unwrap();
        }

        let txn = env.begin_read_txn().unwrap();
        b.iter(|| {
            for item in txn.iter(db).unwrap() {
                black_box(item);
            }
        });
    }

    fn bench_btree(b: &mut Bencher, n: uint) {
        let mut btree = BTreeMap::new();

        {
            for i in range(0, n) {
                btree.insert(format!("key{}", i), format!("val{}", i));
            }
        }

        b.iter(|| {
            for item in btree.iter() {
                black_box(item);
            }
        });
    }

    #[bench]
    fn bench_items_100(b: &mut Bencher) {
        bench_items(b, 100);
    }

    #[bench]
    fn bench_items_500(b: &mut Bencher) {
        bench_items(b, 500);
    }

    #[bench]
    fn bench_items_1000(b: &mut Bencher) {
        bench_items(b, 1000);
    }

    #[bench]
    fn bench_btree_100(b: &mut Bencher) {
        bench_btree(b, 100);
    }

    #[bench]
    fn bench_btree_500(b: &mut Bencher) {
        bench_btree(b, 500);
    }

    #[bench]
    fn bench_btree_1000(b: &mut Bencher) {
        bench_btree(b, 1000);
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
        let db = env.create_db(None, MDB_DUPSORT).unwrap();

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
        let db = env.create_db(None, MDB_DUPSORT | MDB_DUPFIXED).unwrap();

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

    /// Checks assumptions about which get operations return keys.
    #[test]
    fn test_get_keys() {
        let dir = io::TempDir::new("test").unwrap();
        let env = Environment::new().open(dir.path(), io::USER_RWX).unwrap();

        unsafe fn slice_to_val(slice: Option<&[u8]>) -> MDB_val {
            match slice {
                Some(slice) =>
                    MDB_val { mv_size: slice.len() as size_t,
                              mv_data: slice.as_ptr() as *mut c_void },
                None =>
                    MDB_val { mv_size: 0,
                              mv_data: ptr::null_mut() },
            }
        }

        /// Returns true if the cursor get sets the key.
        fn sets_key(cursor: &Cursor,
                    key: Option<&[u8]>,
                    data: Option<&[u8]>,
                    op: c_uint)
                    -> LmdbResult<bool> {
            unsafe {
                let mut key_val = slice_to_val(key);
                let mut data_val = slice_to_val(data);
                let key_ptr = key_val.mv_data;
                try!(lmdb_result(mdb_cursor_get(cursor.cursor(),
                                                &mut key_val,
                                                &mut data_val,
                                                op)));
                Ok(key_ptr != key_val.mv_data)
            }
        }
        let db = env.create_db(None, MDB_DUPSORT | MDB_DUPFIXED).unwrap();

        let mut txn = env.begin_write_txn().unwrap();
        txn.put(db, b"key1", b"val1", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val2", WriteFlags::empty()).unwrap();
        txn.put(db, b"key1", b"val3", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val4", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val5", WriteFlags::empty()).unwrap();
        txn.put(db, b"key2", b"val6", WriteFlags::empty()).unwrap();
        txn.put(db, b"key3", b"val7", WriteFlags::empty()).unwrap();
        txn.put(db, b"key3", b"val8", WriteFlags::empty()).unwrap();
        txn.put(db, b"key3", b"val9", WriteFlags::empty()).unwrap();

        let cursor = txn.open_read_cursor(db).unwrap();
        assert!(sets_key(&cursor, None, None, MDB_FIRST).unwrap());
        assert!(!sets_key(&cursor, None, None, MDB_FIRST_DUP).unwrap());
        assert!(!sets_key(&cursor, Some(b"key2"), Some(b"val5"), MDB_GET_BOTH).unwrap());
        assert!(!sets_key(&cursor, Some(b"key2"), Some(b"val"), MDB_GET_BOTH_RANGE).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_GET_CURRENT).unwrap());
        assert!(!sets_key(&cursor, None, None, MDB_GET_MULTIPLE).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_LAST).unwrap());
        assert!(!sets_key(&cursor, None, None, MDB_LAST_DUP).unwrap());
        sets_key(&cursor, None, None, MDB_FIRST).unwrap();
        assert!(sets_key(&cursor, None, None, MDB_NEXT).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_NEXT_DUP).unwrap());
        sets_key(&cursor, None, None, MDB_FIRST).unwrap();
        assert!(sets_key(&cursor, None, None, MDB_NEXT_MULTIPLE).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_NEXT_NODUP).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_PREV).unwrap());
        sets_key(&cursor, None, None, MDB_LAST).unwrap();
        assert!(sets_key(&cursor, None, None, MDB_PREV_DUP).unwrap());
        assert!(sets_key(&cursor, None, None, MDB_PREV_NODUP).unwrap());
        assert!(!sets_key(&cursor, Some(b"key2"), None, MDB_SET).unwrap());
        assert!(sets_key(&cursor, Some(b"key2"), None, MDB_SET_KEY).unwrap());
        assert!(sets_key(&cursor, Some(b"key2"), None, MDB_SET_RANGE).unwrap());
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
}
