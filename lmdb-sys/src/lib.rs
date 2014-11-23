//! Provides extern declarations for `liblmdb`. Additionally, this crate provides `liblmdb` as a
//! native Cargo dependency.

#![allow(non_camel_case_types)]
extern crate libc;

use libc::{c_int, c_uint, c_void, c_char, size_t};

pub type mdb_mode_t = libc::mode_t;
pub type mdb_filehandle_t = libc::c_int;

pub type MDB_dbi = c_uint;

pub type MDB_rel_func = extern fn(*mut MDB_val, *mut c_void, *mut c_void, *mut c_void);
pub type MDB_msg_func = extern fn(*const c_char, *mut c_void) -> c_int;
pub type MDB_cmp_func = extern fn(*const MDB_val, *const MDB_val) -> c_int;

#[repr(C)]
pub struct MDB_val {
    pub mv_size: size_t,
    pub mv_data: *const c_void,
}

#[repr(C)]
pub struct MDB_env;

#[repr(C)]
pub struct MDB_txn;

#[repr(C)]
pub struct MDB_cursor;

#[repr(C)]
pub struct MDB_stat {
    ms_psize: c_uint,
    ms_depth: c_uint,
    ms_branch_pages: size_t,
    ms_leaf_pages: size_t,
    ms_overflow_pages: size_t,
    ms_entries: size_t
}

#[repr(C)]
pub struct MDB_envinfo {
    me_mapaddr: *const c_void,
    me_mapsize: size_t,
    me_last_pgno: size_t,
    me_last_txnid: size_t,
    me_maxreaders: c_uint,
    me_numreaders: c_uint
}

#[repr(C)]
pub enum MDB_cursor_op {
    MDB_FIRST,
    MDB_FIRST_DUP,
    MDB_GET_BOTH,
    MDB_GET_BOTH_RANGE,
    MDB_GET_CURRENT,
    MDB_GET_MULTIPLE,
    MDB_LAST,
    MDB_LAST_DUP,
    MDB_NEXT,
    MDB_NEXT_DUP,
    MDB_NEXT_MULTIPLE,
    MDB_NEXT_NODUP,
    MDB_PREV,
    MDB_PREV_DUP,
    MDB_PREV_NODUP,
    MDB_SET,
    MDB_SET_KEY,
    MDB_SET_RANGE
}

// Return codes
pub const MDB_SUCCESS: c_int = 0;
pub const MDB_KEYEXIST: c_int = -30799;
pub const MDB_NOTFOUND: c_int = -30798;
pub const MDB_PAGE_NOTFOUND: c_int = -30797;
pub const MDB_CORRUPTED: c_int = -30796;
pub const MDB_PANIC: c_int = -30795;
pub const MDB_VERSION_MISMATCH: c_int = -30794;
pub const MDB_INVALID: c_int = -30793;
pub const MDB_MAP_FULL: c_int = -30792;
pub const MDB_DBS_FULL: c_int = -30791;
pub const MDB_READERS_FULL: c_int = -30790;
pub const MDB_TLS_FULL: c_int = -30789;
pub const MDB_TXN_FULL: c_int = -30788;
pub const MDB_CURSOR_FULL: c_int = -30787;
pub const MDB_PAGE_FULL: c_int = -30786;
pub const MDB_MAP_RESIZED: c_int = -30785;
pub const MDB_INCOMPATIBLE: c_int = -30784;
pub const MDB_BAD_RSLOT: c_int = -30783;
pub const MDB_BAD_TXN: c_int = -30782;
pub const MDB_BAD_VALSIZE: c_int = -30781;
pub const MDB_BAD_DBI: c_int = -30780;

// Write flags
pub const MDB_NOOVERWRITE: c_uint = 0x10;
pub const MDB_NODUPDATA: c_uint = 0x20;
pub const MDB_CURRENT: c_uint = 0x40;
pub const MDB_RESERVE: c_uint = 0x10000;
pub const MDB_APPEND: c_uint = 0x20000;
pub const MDB_APPENDDUP: c_uint = 0x40000;
pub const MDB_MULTIPLE: c_uint = 0x80000;

// Database flags
pub const MDB_REVERSEKEY: c_uint = 0x02;
pub const MDB_DUPSORT: c_uint = 0x04;
pub const MDB_INTEGERKEY: c_uint = 0x08;
pub const MDB_DUPFIXED: c_uint = 0x10;
pub const MDB_INTEGERDUP: c_uint = 0x20;
pub const MDB_REVERSEDUP: c_uint =  0x40;
pub const MDB_CREATE: c_uint = 0x40000;

// Environment flags
pub const MDB_FIXEDMAP: c_uint =  0x01;
pub const MDB_NOSUBDIR: c_uint = 0x4000;
pub const MDB_NOSYNC: c_uint = 0x10000;
pub const MDB_RDONLY: c_uint = 0x20000;
pub const MDB_NOMETASYNC: c_uint = 0x40000;
pub const MDB_WRITEMAP: c_uint = 0x80000;
pub const MDB_MAPASYNC: c_uint = 0x100000;
pub const MDB_NOTLS: c_uint = 0x200000;
pub const MDB_NOLOCK: c_uint =  0x400000;
pub const MDB_NORDAHEAD: c_uint = 0x800000;
pub const MDB_NOMEMINIT: c_uint =  0x1000000;

extern {
    pub fn mdb_version(major: *mut c_int, minor: *mut c_int, patch: *mut c_int) -> *mut c_char;
    pub fn mdb_strerror(err: c_int) -> *mut c_char;
    pub fn mdb_env_create(env: *mut *mut MDB_env) -> c_int;
    pub fn mdb_env_open(env: *mut MDB_env, path: *const c_char, flags: c_uint, mode: mdb_mode_t) -> c_int;
    pub fn mdb_env_copy(env: *mut MDB_env, path: *const c_char) -> c_int;
    pub fn mdb_env_copyfd(env: *mut MDB_env, fd: mdb_filehandle_t) -> c_int;
    pub fn mdb_env_copy2(env: *mut MDB_env, path: *const c_char, flags: c_uint) -> c_int;
    pub fn mdb_env_copyfd2(env: *mut MDB_env, fd: mdb_filehandle_t, flags: c_uint) -> c_int;
    pub fn mdb_env_stat(env: *mut MDB_env, stat: *mut MDB_stat) -> c_int;
    pub fn mdb_env_info(env: *mut MDB_env, stat: *mut MDB_envinfo) -> c_int;
    pub fn mdb_env_sync(env: *mut MDB_env, force: c_int) -> c_int;
    pub fn mdb_env_close(env: *mut MDB_env);
    pub fn mdb_env_set_flags(env: *mut MDB_env, flags: c_uint, onoff: c_int) -> c_int;
    pub fn mdb_env_get_flags(env: *mut MDB_env, flags: *mut c_uint) -> c_int;
    pub fn mdb_env_get_path(env: *mut MDB_env, path: *const *const c_char) -> c_int;
    pub fn mdb_env_get_fd(env: *mut MDB_env, fd: *mut mdb_filehandle_t) -> c_int;
    pub fn mdb_env_set_mapsize(env: *mut MDB_env, size: size_t) -> c_int;
    pub fn mdb_env_set_maxreaders(env: *mut MDB_env, readers: c_uint) -> c_int;
    pub fn mdb_env_get_maxreaders(env: *mut MDB_env, readers: *mut c_uint) -> c_int;
    pub fn mdb_env_set_maxdbs(env: *mut MDB_env, dbs: MDB_dbi) -> c_int;
    pub fn mdb_env_get_maxkeysize(env: *mut MDB_env) -> c_int;
    pub fn mdb_txn_begin(env: *mut MDB_env, parent: *mut MDB_txn, flags: c_uint, txn: *mut *mut MDB_txn) -> c_int;
    pub fn mdb_txn_env(txn: *mut MDB_txn) -> *mut MDB_env;
    pub fn mdb_txn_commit(txn: *mut MDB_txn) -> c_int;
    pub fn mdb_txn_abort(txn: *mut MDB_txn);
    pub fn mdb_txn_reset(txn: *mut MDB_txn);
    pub fn mdb_txn_renew(txn: *mut MDB_txn) -> c_int;
    pub fn mdb_dbi_open(txn: *mut MDB_txn, name: *const c_char, flags: c_uint, dbi: *mut MDB_dbi) -> c_int;
    pub fn mdb_stat(txn: *mut MDB_txn, dbi: MDB_dbi, stat: *mut MDB_stat) -> c_int;
    pub fn mdb_dbi_flags(txn: *mut MDB_txn, dbi: MDB_dbi, flags: *mut c_uint) -> c_int;
    pub fn mdb_dbi_close(txn: *mut MDB_txn, dbi: MDB_dbi);
    pub fn mdb_drop(txn: *mut MDB_txn, dbi: MDB_dbi, del: c_int) -> c_int;
    pub fn mdb_set_compare(txn: *mut MDB_txn, dbi: MDB_dbi, cmp: *mut MDB_cmp_func) -> c_int;
    pub fn mdb_set_dupsort(txn: *mut MDB_txn, dbi: MDB_dbi, cmp: *mut MDB_cmp_func) -> c_int;
    pub fn mdb_set_relfunc(txn: *mut MDB_txn, dbi: MDB_dbi, rel: *mut MDB_rel_func) -> c_int;
    pub fn mdb_set_relctx(txn: *mut MDB_txn, dbi: MDB_dbi, ctx: *mut c_void) -> c_int;
    pub fn mdb_get(txn: *mut MDB_txn, dbi: MDB_dbi, key: *mut MDB_val, data: *mut MDB_val) -> c_int;
    pub fn mdb_put(txn: *mut MDB_txn, dbi: MDB_dbi, key: *mut MDB_val, data: *mut MDB_val, flags: c_uint) -> c_int;
    pub fn mdb_del(txn: *mut MDB_txn, dbi: MDB_dbi, key: *mut MDB_val, data: *mut MDB_val) -> c_int;
    pub fn mdb_cursor_open(txn: *mut MDB_txn, dbi: MDB_dbi, cursor: *mut *mut MDB_cursor) -> c_int;
    pub fn mdb_cursor_close(cursor: *mut MDB_cursor);
    pub fn mdb_cursor_renew(txn: *mut MDB_txn, cursor: *mut MDB_cursor) -> c_int;
    pub fn mdb_cursor_txn(cursor: *mut MDB_cursor) -> *mut MDB_txn;
    pub fn mdb_cursor_dbi(cursor: *mut MDB_cursor) -> MDB_dbi;
    pub fn mdb_cursor_get(cursor: *mut MDB_cursor, key: *mut MDB_val, data: *mut MDB_val, op: MDB_cursor_op) -> c_int;
    pub fn mdb_cursor_put(cursor: *mut MDB_cursor, key: *mut MDB_val, data: *mut MDB_val, flags: c_uint) -> c_int;
    pub fn mdb_cursor_del(cursor: *mut MDB_cursor, flags: c_uint) -> c_int;
    pub fn mdb_cursor_count(cursor: *mut MDB_cursor, countp: *mut size_t) -> c_int;
    pub fn mdb_cmp(txn: *mut MDB_txn, dbi: MDB_dbi, a: *const MDB_val, b: *const MDB_val) -> c_int;
    pub fn mdb_dcmp(txn: *mut MDB_txn, dbi: MDB_dbi, a: *const MDB_val, b: *const MDB_val) -> c_int;
    pub fn mdb_reader_list(env: *mut MDB_env, func: *mut MDB_msg_func, ctx: *mut c_void) -> c_int;
    pub fn mdb_reader_check(env: *mut MDB_env, dead: *mut c_int) -> c_int;
}
