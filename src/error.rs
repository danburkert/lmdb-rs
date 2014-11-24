use libc::c_int;
use std::error::Error;
use std::io::IoError;
use std::str;

use ffi;

#[deriving(Show, Eq, PartialEq)]
pub enum LmdbError {
    KeyExist,
    NotFound,
    PageNotFound,
    Corrupted,
    Panic,
    VersionMismatch,
    Invalid,
    MapFull,
    DbsFull,
    ReadersFull,
    TlsFull,
    TxnFull,
    CursorFull,
    PageFull,
    MapResized,
    Incompatible,
    BadRslot,
    BadTxn,
    BadValSize,
    BadDbi,
    Unknown(c_int),
    Io(IoError),
}

impl Error for LmdbError {
    fn description(&self) -> &str {
        let err_code = match *self {
            LmdbError::KeyExist => ffi::MDB_KEYEXIST,
            LmdbError::NotFound => ffi::MDB_NOTFOUND,
            LmdbError::PageNotFound => ffi::MDB_PAGE_NOTFOUND,
            LmdbError::Corrupted => ffi::MDB_CORRUPTED,
            LmdbError::Panic => ffi::MDB_PANIC,
            LmdbError::VersionMismatch => ffi::MDB_VERSION_MISMATCH,
            LmdbError::Invalid => ffi::MDB_INVALID,
            LmdbError::MapFull => ffi::MDB_MAP_FULL,
            LmdbError::DbsFull => ffi::MDB_DBS_FULL,
            LmdbError::ReadersFull => ffi::MDB_READERS_FULL,
            LmdbError::TlsFull => ffi::MDB_TLS_FULL,
            LmdbError::TxnFull => ffi::MDB_TXN_FULL,
            LmdbError::CursorFull => ffi::MDB_CURSOR_FULL,
            LmdbError::PageFull => ffi::MDB_PAGE_FULL,
            LmdbError::MapResized => ffi::MDB_MAP_RESIZED,
            LmdbError::Incompatible => ffi::MDB_INCOMPATIBLE,
            LmdbError::BadRslot => ffi::MDB_BAD_RSLOT,
            LmdbError::BadTxn => ffi::MDB_BAD_TXN,
            LmdbError::BadValSize => ffi::MDB_BAD_VALSIZE,
            LmdbError::BadDbi => ffi::MDB_BAD_DBI,
            LmdbError::Unknown(i) => i,
            LmdbError::Io(ref io_error) => return io_error.description(),
        };
        unsafe { str::from_c_str(ffi::mdb_strerror(err_code) as *const _) }
    }
}

impl LmdbError {

    pub fn from_err_code(err_code: c_int) -> LmdbError {
        match err_code {
            i if i > 0 => LmdbError::Io(IoError::from_errno(err_code as uint, true)),
            ffi::MDB_KEYEXIST => LmdbError::KeyExist,
            ffi::MDB_NOTFOUND => LmdbError::NotFound,
            ffi::MDB_PAGE_NOTFOUND => LmdbError::PageNotFound,
            ffi::MDB_CORRUPTED => LmdbError::Corrupted,
            ffi::MDB_PANIC => LmdbError::Panic,
            ffi::MDB_VERSION_MISMATCH => LmdbError::VersionMismatch,
            ffi::MDB_INVALID => LmdbError::Invalid,
            ffi::MDB_MAP_FULL => LmdbError::MapFull,
            ffi::MDB_DBS_FULL => LmdbError::DbsFull,
            ffi::MDB_READERS_FULL => LmdbError::ReadersFull,
            ffi::MDB_TLS_FULL => LmdbError::TlsFull,
            ffi::MDB_TXN_FULL => LmdbError::TxnFull,
            ffi::MDB_CURSOR_FULL => LmdbError::CursorFull,
            ffi::MDB_PAGE_FULL => LmdbError::PageFull,
            ffi::MDB_MAP_RESIZED => LmdbError::MapResized,
            ffi::MDB_INCOMPATIBLE => LmdbError::Incompatible,
            ffi::MDB_BAD_RSLOT => LmdbError::BadRslot,
            ffi::MDB_BAD_TXN => LmdbError::BadTxn,
            ffi::MDB_BAD_VALSIZE => LmdbError::BadValSize,
            i => LmdbError::Unknown(i),
        }
    }
}

pub type LmdbResult<T> = Result<T, LmdbError>;

pub fn lmdb_result(err_code: c_int) -> LmdbResult<()> {
    if err_code == ffi::MDB_SUCCESS {
        Ok(())
    } else {
        Err(LmdbError::from_err_code(err_code))
    }
}
