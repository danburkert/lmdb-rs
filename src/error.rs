use libc::c_int;
use std::error::Error;
use std::str;

use ffi;

#[derive(Show, Eq, PartialEq, Copy, Clone)]
pub enum LmdbError {
    /// key/data pair already exists.
    KeyExist,
    /// key/data pair not found (EOF).
    NotFound,
    /// Requested page not found - this usually indicates corruption.
    PageNotFound,
    /// Located page was wrong type.
    Corrupted,
    /// Update of meta page failed or environment had fatal error.
    Panic,
    /// Environment version mismatch.
    VersionMismatch,
    /// File is not a valid LMDB file.
    Invalid,
    /// Environment mapsize reached.
    MapFull,
    /// Environment maxdbs reached.
    DbsFull,
    /// Environment maxreaders reached.
    ReadersFull,
    /// Too many TLS keys in use - Windows only.
    TlsFull,
    /// Txn has too many dirty pages.
    TxnFull,
    /// Cursor stack too deep - internal error.
    CursorFull,
    /// Page has not enough space - internal error.
    PageFull,
    /// Database contents grew beyond environment mapsize.
    MapResized,
    /// MDB_Incompatible: Operation and DB incompatible, or DB flags changed.
    Incompatible,
    /// Invalid reuse of reader locktable slot.
    BadRslot,
    /// Transaction cannot recover - it must be aborted.
    BadTxn,
    /// Unsupported size of key/DB name/data, or wrong DUP_FIXED size.
    BadValSize,
    /// The specified DBI was changed unexpectedly.
    BadDbi,
    /// Other error.
    Other(c_int),
}

impl LmdbError {
    pub fn from_err_code(err_code: c_int) -> LmdbError {
        match err_code {
            ffi::MDB_KEYEXIST         => LmdbError::KeyExist,
            ffi::MDB_NOTFOUND         => LmdbError::NotFound,
            ffi::MDB_PAGE_NOTFOUND    => LmdbError::PageNotFound,
            ffi::MDB_CORRUPTED        => LmdbError::Corrupted,
            ffi::MDB_PANIC            => LmdbError::Panic,
            ffi::MDB_VERSION_MISMATCH => LmdbError::VersionMismatch,
            ffi::MDB_INVALID          => LmdbError::Invalid,
            ffi::MDB_MAP_FULL         => LmdbError::MapFull,
            ffi::MDB_DBS_FULL         => LmdbError::DbsFull,
            ffi::MDB_READERS_FULL     => LmdbError::ReadersFull,
            ffi::MDB_TLS_FULL         => LmdbError::TlsFull,
            ffi::MDB_TXN_FULL         => LmdbError::TxnFull,
            ffi::MDB_CURSOR_FULL      => LmdbError::CursorFull,
            ffi::MDB_PAGE_FULL        => LmdbError::PageFull,
            ffi::MDB_MAP_RESIZED      => LmdbError::MapResized,
            ffi::MDB_INCOMPATIBLE     => LmdbError::Incompatible,
            ffi::MDB_BAD_RSLOT        => LmdbError::BadRslot,
            ffi::MDB_BAD_TXN          => LmdbError::BadTxn,
            ffi::MDB_BAD_VALSIZE      => LmdbError::BadValSize,
            ffi::MDB_BAD_DBI          => LmdbError::BadDbi,
            other                     => LmdbError::Other(other),
        }
    }

    pub fn to_err_code(&self) -> c_int {
        match *self {
            LmdbError::KeyExist        => ffi::MDB_KEYEXIST,
            LmdbError::NotFound        => ffi::MDB_NOTFOUND,
            LmdbError::PageNotFound    => ffi::MDB_PAGE_NOTFOUND,
            LmdbError::Corrupted       => ffi::MDB_CORRUPTED,
            LmdbError::Panic           => ffi::MDB_PANIC,
            LmdbError::VersionMismatch => ffi::MDB_VERSION_MISMATCH,
            LmdbError::Invalid         => ffi::MDB_INVALID,
            LmdbError::MapFull         => ffi::MDB_MAP_FULL,
            LmdbError::DbsFull         => ffi::MDB_DBS_FULL,
            LmdbError::ReadersFull     => ffi::MDB_READERS_FULL,
            LmdbError::TlsFull         => ffi::MDB_TLS_FULL,
            LmdbError::TxnFull         => ffi::MDB_TXN_FULL,
            LmdbError::CursorFull      => ffi::MDB_CURSOR_FULL,
            LmdbError::PageFull        => ffi::MDB_PAGE_FULL,
            LmdbError::MapResized      => ffi::MDB_MAP_RESIZED,
            LmdbError::Incompatible    => ffi::MDB_INCOMPATIBLE,
            LmdbError::BadRslot        => ffi::MDB_BAD_RSLOT,
            LmdbError::BadTxn          => ffi::MDB_BAD_TXN,
            LmdbError::BadValSize      => ffi::MDB_BAD_VALSIZE,
            LmdbError::BadDbi          => ffi::MDB_BAD_DBI,
            LmdbError::Other(err_code) => err_code,
        }
    }
}

impl Error for LmdbError {
    fn description(&self) -> &str {
        unsafe { str::from_c_str(ffi::mdb_strerror(self.to_err_code()) as *const _) }
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

#[cfg(test)]
mod test {

    use std::error::Error;

    use super::*;

    #[test]
    fn test_description() {
        assert_eq!("Permission denied",
                   LmdbError::from_err_code(13).description());
        assert_eq!("MDB_NOTFOUND: No matching key/data pair found",
                   LmdbError::NotFound.description());
    }

}
