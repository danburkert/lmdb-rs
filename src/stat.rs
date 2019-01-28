use ffi;
use std::mem;

/// Environment statistics.
///
/// Contains information about the size and layout of an LMDB environment.
pub struct Stat(ffi::MDB_stat);

impl Stat {
    /// Create new zero'd LMDB statistics.
    pub fn new() -> Stat {
        unsafe {
            Stat(mem::zeroed())
        }
    }

    /// Returns a raw pointer to the underlying LMDB statistics.
    ///
    /// The caller **must** ensure that the pointer is not dereferenced after the lifetime of the
    /// stat.
    pub fn stat(&mut self) -> *mut ffi::MDB_stat {
        &mut self.0
    }

    /// Size of a database page. This is the same for all databases in the environment.
    #[inline]
    pub fn page_size(&self) -> u32 {
        self.0.ms_psize
    }

    /// Depth (height) of the B-tree.
    #[inline]
    pub fn depth(&self) -> u32 {
        self.0.ms_depth
    }

    /// Number of internal (non-leaf) pages.
    #[inline]
    pub fn branch_pages(&self) -> usize {
        self.0.ms_branch_pages
    }

    /// Number of leaf pages.
    #[inline]
    pub fn leaf_pages(&self) -> usize {
        self.0.ms_leaf_pages
    }

    /// Number of overflow pages.
    #[inline]
    pub fn overflow_pages(&self) -> usize {
        self.0.ms_overflow_pages
    }

    /// Number of data items.
    #[inline]
    pub fn entries(&self) -> usize {
        self.0.ms_entries
    }
}

