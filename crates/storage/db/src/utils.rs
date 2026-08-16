//! Utils crate for `db`.

use std::path::Path;

/// Default MDBX page size floor for newly created databases (16 KiB).
///
/// Historically [`default_page_size`] followed the OS page size alone, which is typically
/// 4 KiB on x86_64 Linux. There is no libmdbx requirement to stay at 4 KiB — 4096 was only
/// documented as a practical *minimum* for value size. Prefer 16 KiB on modern hardware
/// (and align with the RocksDB block size used elsewhere); with storage v2, MDBX holds less
/// of the overall dataset. Page size is fixed at database creation (`--db.page-size`).
pub(crate) const DEFAULT_PAGE_SIZE: usize = 16 * 1024;

/// Returns the default page size for a new MDBX environment.
///
/// Uses at least [`DEFAULT_PAGE_SIZE`] (16 KiB). If the OS page size is larger (e.g. 64 KiB
/// on some ARM platforms), that value is used instead, clamped to libmdbx's maximum (64 KiB).
pub(crate) fn default_page_size() -> usize {
    let os_page_size = page_size::get();

    // source: https://gitflic.ru/project/erthink/libmdbx/blob?file=mdbx.h#line-num-821
    let libmdbx_max_page_size = 0x10000;

    os_page_size.clamp(DEFAULT_PAGE_SIZE, libmdbx_max_page_size)
}

/// Check if a db is empty. It does not provide any information on the
/// validity of the data in it. We consider a database as non empty when it's a non empty directory.
pub fn is_database_empty<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    if !path.exists() {
        true
    } else if path.is_file() {
        false
    } else if let Ok(mut dir) = path.read_dir() {
        // Check if directory has any entries without counting all of them
        dir.next().is_none()
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_database_empty_false_if_db_path_is_a_file() {
        let db_file = tempfile::NamedTempFile::new().unwrap();

        let result = is_database_empty(&db_file);

        assert!(!result);
    }

    #[test]
    fn default_page_size_is_at_least_16kib() {
        let size = default_page_size();
        assert!(size >= DEFAULT_PAGE_SIZE, "got {size}");
        assert!(size.is_power_of_two());
        assert!(size <= 0x10000);
    }
}
