use libc::c_int;
use std::error::Error;
use std::str;

use ffi;

#[deriving(Show, Eq, PartialEq, Copy, Clone)]
pub struct LmdbError {
    err_code: c_int,
}

impl Error for LmdbError {
    fn description(&self) -> &str {
        unsafe { str::from_c_str(ffi::mdb_strerror(self.err_code) as *const _) }
    }
}

impl LmdbError {
    pub fn from_err_code(err_code: c_int) -> LmdbError {
        LmdbError { err_code: err_code}
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
    use ffi;

    #[test]
    fn test_description() {
        assert_eq!("Permission denied",
                   LmdbError::from_err_code(13).description());
        assert_eq!("MDB_NOTFOUND: No matching key/data pair found",
                   LmdbError::from_err_code(ffi::MDB_NOTFOUND).description());
    }

}
