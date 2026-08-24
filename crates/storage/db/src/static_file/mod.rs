//! reth's static file database table import and access

use reth_nippy_jar::{
    NippyJar, NippyJarError, CHANGESET_OFFSETS_FILE_EXTENSION, CONFIG_FILE_EXTENSION,
};
use reth_static_file_types::{
    SegmentHeader, SegmentRangeInclusive, StaticFileMap, StaticFileSegment,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod cursor;
pub use cursor::StaticFileCursor;

mod mask;
pub use mask::*;

mod masks;
pub use masks::*;

/// Alias type for a map of [`StaticFileSegment`] and sorted lists of existing static file ranges.
type SortedStaticFiles = StaticFileMap<Vec<(SegmentRangeInclusive, SegmentHeader)>>;

/// Pre-v2.4.1 segment header layout (no `expected_block_range` / `changeset_offsets_len`).
#[derive(Deserialize)]
struct LegacySegmentHeader {
    block_range: Option<SegmentRangeInclusive>,
    tx_range: Option<SegmentRangeInclusive>,
    segment: StaticFileSegment,
}

/// Change-based header with `changeset_offsets_len` **before** `segment` (intermediate v2 port).
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(Deserialize)]
struct LegacyChangeBasedSegmentHeader {
    expected_block_range: SegmentRangeInclusive,
    block_range: Option<SegmentRangeInclusive>,
    tx_range: Option<SegmentRangeInclusive>,
    changeset_offsets_len: u64,
    segment: StaticFileSegment,
}

/// Change-based header without `expected_block_range`, offsets before `segment`.
#[derive(Deserialize)]
struct LegacyChangeBasedSegmentHeaderV1 {
    block_range: Option<SegmentRangeInclusive>,
    tx_range: Option<SegmentRangeInclusive>,
    changeset_offsets_len: u64,
    segment: StaticFileSegment,
}

fn header_from_legacy_change_based(h: LegacyChangeBasedSegmentHeader) -> SegmentHeader {
    let mut header =
        SegmentHeader::new(h.expected_block_range, h.block_range, h.tx_range, h.segment);
    header.set_changeset_offsets_len(h.changeset_offsets_len);
    header
}

fn header_from_legacy_change_based_v1(h: LegacyChangeBasedSegmentHeaderV1) -> SegmentHeader {
    let mut header = SegmentHeader::from_legacy_fields(h.block_range, h.tx_range, h.segment);
    header.set_changeset_offsets_len(h.changeset_offsets_len);
    header
}

/// On-disk layout for jars that still included skipped `filter`/`phf` fields.
#[derive(Deserialize)]
struct NippyJarWithLegacyFields<H> {
    version: usize,
    user_header: H,
    columns: usize,
    rows: usize,
    compressor: Option<reth_nippy_jar::compression::Compressors>,
    filter: Option<u32>,
    phf: Option<u32>,
    max_row_size: usize,
}

/// On-disk layout without skipped `filter`/`phf` fields.
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(Deserialize)]
struct NippyJarStored<H> {
    version: usize,
    user_header: H,
    columns: usize,
    rows: usize,
    compressor: Option<reth_nippy_jar::compression::Compressors>,
    max_row_size: usize,
}

/// Removes every on-disk artifact for a static file base path (data, index, offsets, config,
/// changeset sidecar).
fn remove_static_file_bundle(data_path: &Path) -> Result<(), NippyJarError> {
    for path in [
        data_path.to_path_buf(),
        data_path.with_extension("idx"),
        data_path.with_extension("off"),
        data_path.with_extension(CONFIG_FILE_EXTENSION),
        data_path.with_extension(CHANGESET_OFFSETS_FILE_EXTENSION),
    ] {
        if path.exists() {
            reth_fs_util::remove_file(path)?;
        }
    }
    Ok(())
}

/// Returns `true` when a committed `.conf` exists for the static file base path.
pub fn static_file_config_exists(data_path: &Path) -> bool {
    data_path.with_extension(CONFIG_FILE_EXTENSION).exists()
}

/// Returns `true` if any on-disk artifact exists for this static file base path.
fn static_file_bundle_has_artifacts(data_path: &Path) -> bool {
    data_path.exists() ||
        data_path.with_extension("idx").exists() ||
        data_path.with_extension("off").exists() ||
        data_path.with_extension(CONFIG_FILE_EXTENSION).exists() ||
        data_path.with_extension(CHANGESET_OFFSETS_FILE_EXTENSION).exists()
}

/// Returns `true` if the jar at `data_path` is incomplete: missing `.conf` with partial
/// artifacts, or `.conf` without a data file.
pub fn static_file_bundle_is_incomplete(data_path: &Path) -> bool {
    let has_conf = data_path.with_extension(CONFIG_FILE_EXTENSION).exists();
    let has_data = data_path.exists();

    if has_conf && has_data {
        return false;
    }

    if !has_conf {
        return static_file_bundle_has_artifacts(data_path);
    }

    !has_data
}

/// Removes incomplete artifacts for a single static file bundle.
pub fn remove_incomplete_static_file_bundle(data_path: &Path) -> Result<bool, NippyJarError> {
    if !static_file_bundle_is_incomplete(data_path) {
        return Ok(false);
    }
    remove_static_file_bundle(data_path)?;
    Ok(true)
}

/// Deletes incomplete static file bundles in `dir` (crash between data write and config commit).
///
/// A complete jar always has both the base data file and `.conf`. Orphan data without `.conf`
/// (or `.conf` without data) is removed entirely — same outcome as unwinding an uncommitted
/// append rather than ignoring the files during iteration.
///
/// Returns base data paths that were removed.
pub fn remove_orphan_static_files(dir: &Path) -> Result<Vec<PathBuf>, NippyJarError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut removed = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let entries = reth_fs_util::read_dir(dir)
        .map_err(|err| NippyJarError::Custom(err.to_string()))?
        .filter_map(Result::ok);

    for entry in entries {
        if !entry.metadata().is_ok_and(|metadata| metadata.is_file()) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().into_owned();

        if !name.starts_with("static_file_") {
            continue;
        }

        let base_name = name.split('.').next().unwrap_or(&name);
        if StaticFileSegment::parse_filename(base_name).is_none() {
            continue;
        }

        let data_path = dir.join(base_name);
        if seen.contains(&data_path) {
            continue;
        }
        seen.insert(data_path.clone());

        if remove_incomplete_static_file_bundle(&data_path)? {
            removed.push(data_path);
        }
    }

    Ok(removed)
}

/// Loads a segment jar, or returns `None` when the bundle is missing or was incomplete
/// (removed on disk).
pub fn load_segment_nippy_jar_or_remove_incomplete(
    path: &Path,
) -> Result<Option<NippyJar<SegmentHeader>>, NippyJarError> {
    if remove_incomplete_static_file_bundle(path)? {
        return Ok(None);
    }
    let config_path = path.with_extension(CONFIG_FILE_EXTENSION);
    if !config_path.exists() {
        return Ok(None);
    }
    load_segment_nippy_jar(path).map(Some)
}

/// Loads a static file [`NippyJar`] with legacy on-disk header compatibility.
pub fn load_segment_nippy_jar(path: &Path) -> Result<NippyJar<SegmentHeader>, NippyJarError> {
    if remove_incomplete_static_file_bundle(path)? {
        tracing::warn!(
            target: "reth::static_file",
            path = %path.display(),
            "Removed incomplete static file bundle before load"
        );
    }

    let config_path = path.with_extension(CONFIG_FILE_EXTENSION);
    if !config_path.exists() {
        return Err(NippyJarError::Custom(format!("static file not found at {}", path.display())));
    }

    let bytes = std::fs::read(&config_path).map_err(|err| {
        NippyJarError::Custom(format!(
            "failed to read static file config {}: {err}",
            config_path.display()
        ))
    })?;

    if let Ok(jar) = NippyJar::<SegmentHeader>::load_from_bytes(&bytes) {
        return Ok(jar.with_data_path(path.to_path_buf()));
    }

    if let Ok(stored) = bincode::deserialize::<NippyJarWithLegacyFields<SegmentHeader>>(&bytes) {
        return Ok(NippyJar::from_on_disk_parts(
            stored.version,
            stored.user_header,
            stored.columns,
            stored.rows,
            stored.compressor,
            stored.max_row_size,
        )
        .with_data_path(path.to_path_buf()));
    }

    if let Ok(stored) =
        bincode::deserialize::<NippyJarWithLegacyFields<LegacySegmentHeader>>(&bytes)
    {
        let user_header = SegmentHeader::from_legacy_fields(
            stored.user_header.block_range,
            stored.user_header.tx_range,
            stored.user_header.segment,
        );
        return Ok(NippyJar::from_on_disk_parts(
            stored.version,
            user_header,
            stored.columns,
            stored.rows,
            stored.compressor,
            stored.max_row_size,
        )
        .with_data_path(path.to_path_buf()));
    }

    if let Ok(stored) = bincode::deserialize::<NippyJarStored<LegacySegmentHeader>>(&bytes) {
        let user_header = SegmentHeader::from_legacy_fields(
            stored.user_header.block_range,
            stored.user_header.tx_range,
            stored.user_header.segment,
        );
        return Ok(NippyJar::from_on_disk_parts(
            stored.version,
            user_header,
            stored.columns,
            stored.rows,
            stored.compressor,
            stored.max_row_size,
        )
        .with_data_path(path.to_path_buf()));
    }

    if let Ok(stored) =
        bincode::deserialize::<NippyJarStored<LegacyChangeBasedSegmentHeader>>(&bytes)
    {
        let user_header = header_from_legacy_change_based(stored.user_header);
        return Ok(NippyJar::from_on_disk_parts(
            stored.version,
            user_header,
            stored.columns,
            stored.rows,
            stored.compressor,
            stored.max_row_size,
        )
        .with_data_path(path.to_path_buf()));
    }

    if let Ok(stored) =
        bincode::deserialize::<NippyJarWithLegacyFields<LegacyChangeBasedSegmentHeader>>(&bytes)
    {
        let user_header = header_from_legacy_change_based(stored.user_header);
        return Ok(NippyJar::from_on_disk_parts(
            stored.version,
            user_header,
            stored.columns,
            stored.rows,
            stored.compressor,
            stored.max_row_size,
        )
        .with_data_path(path.to_path_buf()));
    }

    if let Ok(stored) =
        bincode::deserialize::<NippyJarStored<LegacyChangeBasedSegmentHeaderV1>>(&bytes)
    {
        let user_header = header_from_legacy_change_based_v1(stored.user_header);
        return Ok(NippyJar::from_on_disk_parts(
            stored.version,
            user_header,
            stored.columns,
            stored.rows,
            stored.compressor,
            stored.max_row_size,
        )
        .with_data_path(path.to_path_buf()));
    }

    if let Ok(stored) =
        bincode::deserialize::<NippyJarWithLegacyFields<LegacyChangeBasedSegmentHeaderV1>>(&bytes)
    {
        let user_header = header_from_legacy_change_based_v1(stored.user_header);
        return Ok(NippyJar::from_on_disk_parts(
            stored.version,
            user_header,
            stored.columns,
            stored.rows,
            stored.compressor,
            stored.max_row_size,
        )
        .with_data_path(path.to_path_buf()));
    }

    Err(NippyJarError::Custom(format!(
        "failed to load static file jar at {}: {}",
        path.display(),
        NippyJar::<SegmentHeader>::load_from_bytes(&bytes).unwrap_err()
    )))
}

/// Given the `static_files` directory path, it returns a list over the existing `static_files`
/// organized by [`StaticFileSegment`]. Each segment has a sorted list of block ranges and
/// segment headers as presented in the file configuration.
pub fn iter_static_files(path: &Path) -> Result<SortedStaticFiles, NippyJarError> {
    if !path.exists() {
        reth_fs_util::create_dir_all(path).map_err(|err| NippyJarError::Custom(err.to_string()))?;
    }

    let mut static_files = SortedStaticFiles::default();
    let entries = reth_fs_util::read_dir(path)
        .map_err(|err| NippyJarError::Custom(err.to_string()))?
        .filter_map(Result::ok);
    for entry in entries {
        if entry.metadata().is_ok_and(|metadata| metadata.is_file()) &&
            let Some((segment, _)) =
                StaticFileSegment::parse_filename(&entry.file_name().to_string_lossy())
        {
            let path = entry.path();
            if static_file_bundle_is_incomplete(&path) {
                return Err(NippyJarError::Custom(format!(
                    "incomplete static file at {}: data and .conf are not both present; \
                     remove orphan artifacts or open the datadir with write access so startup \
                     can delete uncommitted static file writes",
                    path.display()
                )));
            }
            let jar = load_segment_nippy_jar(&path)?;

            if let Some(block_range) = jar.user_header().block_range() {
                let block_range = *block_range;
                static_files
                    .entry(segment)
                    .and_modify(|headers| headers.push((block_range, jar.user_header().clone())))
                    .or_insert_with(|| vec![(block_range, jar.user_header().clone())]);
            }
        }
    }

    // Sort by block end range.
    for range_list in static_files.values_mut() {
        range_list.sort_unstable_by_key(|(block_range, _)| block_range.end());
    }

    Ok(static_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_orphan_static_file_sidecar_only() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("static_file_account-change-sets_117500000_117999999");
        std::fs::write(data_path.with_extension("off"), b"off").unwrap();
        std::fs::write(data_path.with_extension(CHANGESET_OFFSETS_FILE_EXTENSION), b"csoff")
            .unwrap();

        let removed = remove_orphan_static_files(dir.path()).unwrap();
        assert_eq!(removed, vec![data_path.clone()]);
        assert!(!data_path.with_extension("off").exists());
        assert!(!data_path.with_extension(CHANGESET_OFFSETS_FILE_EXTENSION).exists());
    }

    #[test]
    fn remove_orphan_static_file_without_conf() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("static_file_account-change-sets_117500000_117999999");
        std::fs::write(&data_path, b"orphan").unwrap();
        std::fs::write(data_path.with_extension("off"), b"off").unwrap();

        let removed = remove_orphan_static_files(dir.path()).unwrap();
        assert_eq!(removed, vec![data_path.clone()]);
        assert!(!data_path.exists());
        assert!(!data_path.with_extension("off").exists());
    }

    #[test]
    fn remove_orphan_static_file_conf_without_data() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("static_file_account-change-sets_117500000_117999999");
        std::fs::write(data_path.with_extension(CONFIG_FILE_EXTENSION), b"conf").unwrap();

        let removed = remove_orphan_static_files(dir.path()).unwrap();
        assert_eq!(removed, vec![data_path.clone()]);
        assert!(!data_path.with_extension(CONFIG_FILE_EXTENSION).exists());
    }

    #[test]
    fn complete_static_file_not_removed_by_orphan_cleanup() {
        let legacy_header = LegacyChangeBasedSegmentHeader {
            expected_block_range: SegmentRangeInclusive::new(117_470_000, 117_479_999),
            block_range: Some(SegmentRangeInclusive::new(117_470_000, 117_479_999)),
            tx_range: None,
            changeset_offsets_len: 247,
            segment: StaticFileSegment::AccountChangeSets,
        };
        let stored = NippyJarStored {
            version: 1,
            user_header: legacy_header,
            columns: 1,
            rows: 247,
            compressor: None,
            max_row_size: 0,
        };
        let bytes = bincode::serialize(&stored).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("static_file_account-change-sets_117470000_117479999");
        std::fs::write(&data_path, b"data").unwrap();
        std::fs::write(data_path.with_extension(CONFIG_FILE_EXTENSION), bytes).unwrap();

        let removed = remove_orphan_static_files(dir.path()).unwrap();
        assert!(removed.is_empty());
        assert!(data_path.exists());
        assert!(data_path.with_extension(CONFIG_FILE_EXTENSION).exists());
    }

    #[test]
    fn load_segment_nippy_jar_or_remove_incomplete_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("static_file_account-change-sets_117500000_117999999");
        assert!(load_segment_nippy_jar_or_remove_incomplete(&data_path).unwrap().is_none());
    }

    #[test]
    fn load_legacy_change_based_header_offsets_before_segment() {
        let legacy_header = LegacyChangeBasedSegmentHeader {
            expected_block_range: SegmentRangeInclusive::new(117_470_000, 117_479_999),
            block_range: Some(SegmentRangeInclusive::new(117_470_000, 117_479_999)),
            tx_range: None,
            changeset_offsets_len: 247,
            segment: StaticFileSegment::AccountChangeSets,
        };
        let stored = NippyJarStored {
            version: 1,
            user_header: legacy_header,
            columns: 1,
            rows: 247,
            compressor: None,
            max_row_size: 0,
        };
        let bytes = bincode::serialize(&stored).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let data_path = dir.path().join("static_file_account-change-sets_117470000_117479999");
        std::fs::write(&data_path, []).unwrap();
        std::fs::write(data_path.with_extension(CONFIG_FILE_EXTENSION), bytes).unwrap();
        let jar = load_segment_nippy_jar(&data_path).unwrap();
        assert_eq!(jar.user_header().segment(), StaticFileSegment::AccountChangeSets);
        assert_eq!(jar.user_header().changeset_offsets_len(), 247);
        assert_eq!(jar.rows(), 247);
    }
}
