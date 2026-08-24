//! Archive inspection and batch tag management for SWAL Files
//!
//! Provides `ArchiveInspector` for non-extracting inspection and entry previewing of `.zip`, `.tar`, and `.tar.gz` archives,
//! and `BatchTagManager` for querying, filtering, assigning, and persisting tags across file paths.

use chrono::{Local, TimeZone};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::{Path, PathBuf};

/// Format of supported archives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveKind {
    Zip,
    Tar,
    TarGz,
    Unknown,
}

impl ArchiveKind {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy().to_lowercase();

        if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
            ArchiveKind::TarGz
        } else if path_str.ends_with(".tar") {
            ArchiveKind::Tar
        } else if path_str.ends_with(".zip") {
            ArchiveKind::Zip
        } else {
            ArchiveKind::Unknown
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ArchiveKind::Zip => "zip",
            ArchiveKind::Tar => "tar",
            ArchiveKind::TarGz => "tar.gz",
            ArchiveKind::Unknown => "unknown",
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, ArchiveKind::Unknown)
    }
}

/// Error type for archive operations
#[derive(Debug)]
pub enum ArchiveError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    UnsupportedFormat(String),
    EntryNotFound(String),
    CorruptArchive(String),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::Io(e) => write!(f, "I/O error: {}", e),
            ArchiveError::Zip(e) => write!(f, "Zip error: {}", e),
            ArchiveError::UnsupportedFormat(fmt) => write!(f, "Unsupported archive format: {}", fmt),
            ArchiveError::EntryNotFound(entry) => write!(f, "Archive entry not found: {}", entry),
            ArchiveError::CorruptArchive(msg) => write!(f, "Corrupt archive: {}", msg),
        }
    }
}

impl std::error::Error for ArchiveError {}

impl From<std::io::Error> for ArchiveError {
    fn from(err: std::io::Error) -> Self {
        ArchiveError::Io(err)
    }
}

impl From<zip::result::ZipError> for ArchiveError {
    fn from(err: zip::result::ZipError) -> Self {
        ArchiveError::Zip(err)
    }
}

/// Representation of a single item/file within an archive
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub formatted_size: String,
    pub is_dir: bool,
    pub modified_timestamp: u64,
    pub formatted_date: String,
    pub comment: Option<String>,
    pub compression_method: Option<String>,
    pub crc32: Option<u32>,
}

impl ArchiveEntry {
    pub fn new(
        name: String,
        path: String,
        size_bytes: u64,
        is_dir: bool,
        modified_timestamp: u64,
        comment: Option<String>,
        compression_method: Option<String>,
        crc32: Option<u32>,
    ) -> Self {
        let formatted_size = format_archive_size(size_bytes, is_dir);
        let formatted_date = format_timestamp(modified_timestamp);

        Self {
            name,
            path,
            size_bytes,
            formatted_size,
            is_dir,
            modified_timestamp,
            formatted_date,
            comment,
            compression_method,
            crc32,
        }
    }
}

/// Comprehensive metadata and file list of an archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    pub archive_path: PathBuf,
    pub kind: ArchiveKind,
    pub total_entries: usize,
    pub file_count: usize,
    pub dir_count: usize,
    pub uncompressed_size_bytes: u64,
    pub formatted_uncompressed_size: String,
    pub compressed_size_bytes: u64,
    pub formatted_compressed_size: String,
    pub compression_ratio: f32,
    pub entries: Vec<ArchiveEntry>,
    pub comment: Option<String>,
}

/// Inspector for archive inspection without full disk extraction
#[derive(Debug, Default)]
pub struct ArchiveInspector;

impl ArchiveInspector {
    pub fn new() -> Self {
        Self
    }

    /// Automatically detects kind and inspects archive at the given file path
    pub fn inspect<P: AsRef<Path>>(path: P) -> Result<ArchiveMetadata, ArchiveError> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(ArchiveError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File does not exist: {}", path_ref.display()),
            )));
        }

        let compressed_size = path_ref.metadata().map(|m| m.len()).unwrap_or(0);
        let kind = ArchiveKind::from_path(path_ref);

        match kind {
            ArchiveKind::Zip => {
                let file = File::open(path_ref)?;
                let reader = BufReader::new(file);
                Self::inspect_zip(reader, Some(path_ref.to_path_buf()), compressed_size)
            }
            ArchiveKind::Tar => {
                let file = File::open(path_ref)?;
                let reader = BufReader::new(file);
                Self::inspect_tar(reader, Some(path_ref.to_path_buf()), compressed_size)
            }
            ArchiveKind::TarGz => {
                let file = File::open(path_ref)?;
                let reader = BufReader::new(file);
                Self::inspect_tar_gz(reader, Some(path_ref.to_path_buf()), compressed_size)
            }
            ArchiveKind::Unknown => {
                // Try zip first as fallback
                if let Ok(file) = File::open(path_ref) {
                    let reader = BufReader::new(file);
                    if let Ok(meta) = Self::inspect_zip(reader, Some(path_ref.to_path_buf()), compressed_size) {
                        return Ok(meta);
                    }
                }
                // Try tar.gz fallback
                if let Ok(file) = File::open(path_ref) {
                    let reader = BufReader::new(file);
                    if let Ok(meta) = Self::inspect_tar_gz(reader, Some(path_ref.to_path_buf()), compressed_size) {
                        return Ok(meta);
                    }
                }
                // Try tar fallback
                if let Ok(file) = File::open(path_ref) {
                    let reader = BufReader::new(file);
                    if let Ok(meta) = Self::inspect_tar(reader, Some(path_ref.to_path_buf()), compressed_size) {
                        return Ok(meta);
                    }
                }

                Err(ArchiveError::UnsupportedFormat(
                    path_ref.to_string_lossy().to_string(),
                ))
            }
        }
    }

    /// Inspect a ZIP archive from any Read + Seek stream
    pub fn inspect_zip<R: Read + Seek>(
        reader: R,
        archive_path: Option<PathBuf>,
        compressed_size: u64,
    ) -> Result<ArchiveMetadata, ArchiveError> {
        let mut zip = zip::ZipArchive::new(reader)?;
        let total_entries = zip.len();
        let mut entries = Vec::with_capacity(total_entries);
        let mut uncompressed_size: u64 = 0;
        let mut file_count = 0;
        let mut dir_count = 0;

        let archive_comment = if !zip.comment().is_empty() {
            Some(String::from_utf8_lossy(zip.comment()).to_string())
        } else {
            None
        };

        for i in 0..total_entries {
            let zip_file = zip.by_index(i)?;
            let raw_name = zip_file.name();
            let is_dir = zip_file.is_dir() || raw_name.ends_with('/');
            let size = zip_file.size();
            uncompressed_size = uncompressed_size.saturating_add(size);

            if is_dir {
                dir_count += 1;
            } else {
                file_count += 1;
            }

            let name = raw_name
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or(raw_name)
                .to_string();

            let dt = zip_file.last_modified();
            let year = dt.year() as i32;
            let month = dt.month() as u32;
            let day = dt.day() as u32;
            let hour = dt.hour() as u32;
            let min = dt.minute() as u32;
            let sec = dt.second() as u32;

            let mtime_secs = chrono::NaiveDate::from_ymd_opt(year, month, day)
                .and_then(|d| d.and_hms_opt(hour, min, sec))
                .and_then(|naive| Local.from_local_datetime(&naive).single())
                .map(|dt| dt.timestamp() as u64)
                .unwrap_or(0);

            let comment = if !zip_file.comment().is_empty() {
                Some(zip_file.comment().to_string())
            } else {
                None
            };

            let comp_method = format!("{:?}", zip_file.compression());

            entries.push(ArchiveEntry::new(
                name,
                raw_name.to_string(),
                size,
                is_dir,
                mtime_secs,
                comment,
                Some(comp_method),
                Some(zip_file.crc32()),
            ));
        }

        let ratio = if compressed_size > 0 {
            (uncompressed_size as f64 / compressed_size as f64) as f32
        } else {
            1.0
        };

        Ok(ArchiveMetadata {
            archive_path: archive_path.unwrap_or_else(|| PathBuf::from("archive.zip")),
            kind: ArchiveKind::Zip,
            total_entries,
            file_count,
            dir_count,
            uncompressed_size_bytes: uncompressed_size,
            formatted_uncompressed_size: format_archive_size(uncompressed_size, false),
            compressed_size_bytes: compressed_size,
            formatted_compressed_size: format_archive_size(compressed_size, false),
            compression_ratio: ratio,
            entries,
            comment: archive_comment,
        })
    }

    /// Inspect a TAR archive from any Read stream
    pub fn inspect_tar<R: Read>(
        reader: R,
        archive_path: Option<PathBuf>,
        compressed_size: u64,
    ) -> Result<ArchiveMetadata, ArchiveError> {
        let mut archive = tar::Archive::new(reader);
        let mut entries = Vec::new();
        let mut uncompressed_size: u64 = 0;
        let mut file_count = 0;
        let mut dir_count = 0;

        let tar_entries = archive.entries()?;
        for entry_res in tar_entries {
            let entry = entry_res?;
            let path_cow = entry.path()?;
            let path_str = path_cow.to_string_lossy().to_string();
            let is_dir = entry.header().entry_type().is_dir() || path_str.ends_with('/');
            let size = entry.header().size().unwrap_or(0);
            let mtime = entry.header().mtime().unwrap_or(0);

            uncompressed_size = uncompressed_size.saturating_add(size);
            if is_dir {
                dir_count += 1;
            } else {
                file_count += 1;
            }

            let name = path_str
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or(&path_str)
                .to_string();

            entries.push(ArchiveEntry::new(
                name,
                path_str,
                size,
                is_dir,
                mtime,
                None,
                Some("tar-uncompressed".to_string()),
                None,
            ));
        }

        let total_entries = entries.len();
        let ratio = if compressed_size > 0 {
            (uncompressed_size as f64 / compressed_size as f64) as f32
        } else {
            1.0
        };

        Ok(ArchiveMetadata {
            archive_path: archive_path.unwrap_or_else(|| PathBuf::from("archive.tar")),
            kind: ArchiveKind::Tar,
            total_entries,
            file_count,
            dir_count,
            uncompressed_size_bytes: uncompressed_size,
            formatted_uncompressed_size: format_archive_size(uncompressed_size, false),
            compressed_size_bytes: compressed_size,
            formatted_compressed_size: format_archive_size(compressed_size, false),
            compression_ratio: ratio,
            entries,
            comment: None,
        })
    }

    /// Inspect a TAR.GZ archive from any Read stream
    pub fn inspect_tar_gz<R: Read>(
        reader: R,
        archive_path: Option<PathBuf>,
        compressed_size: u64,
    ) -> Result<ArchiveMetadata, ArchiveError> {
        let gz = GzDecoder::new(reader);
        let mut meta = Self::inspect_tar(gz, archive_path.or_else(|| Some(PathBuf::from("archive.tar.gz"))), compressed_size)?;
        meta.kind = ArchiveKind::TarGz;
        Ok(meta)
    }

    /// Extract a single entry directly into an in-memory buffer without extracting to disk
    pub fn extract_single_entry_to_memory<P: AsRef<Path>>(
        archive_path: P,
        entry_path: &str,
    ) -> Result<Vec<u8>, ArchiveError> {
        let path_ref = archive_path.as_ref();
        let kind = ArchiveKind::from_path(path_ref);

        match kind {
            ArchiveKind::Zip => {
                let file = File::open(path_ref)?;
                let mut zip = zip::ZipArchive::new(BufReader::new(file))?;
                let mut zip_file = zip.by_name(entry_path).map_err(|_| {
                    ArchiveError::EntryNotFound(format!(
                        "Entry '{}' not found in {}",
                        entry_path,
                        path_ref.display()
                    ))
                })?;

                let mut buffer = Vec::with_capacity(zip_file.size() as usize);
                zip_file.read_to_end(&mut buffer)?;
                Ok(buffer)
            }
            ArchiveKind::Tar => {
                let file = File::open(path_ref)?;
                let mut archive = tar::Archive::new(BufReader::new(file));
                for entry_res in archive.entries()? {
                    let mut entry = entry_res?;
                    let current_path = entry.path()?.to_string_lossy().to_string();
                    if current_path == entry_path || current_path == entry_path.trim_start_matches('/') {
                        let mut buffer = Vec::new();
                        entry.read_to_end(&mut buffer)?;
                        return Ok(buffer);
                    }
                }
                Err(ArchiveError::EntryNotFound(format!(
                    "Entry '{}' not found in {}",
                    entry_path,
                    path_ref.display()
                )))
            }
            ArchiveKind::TarGz => {
                let file = File::open(path_ref)?;
                let gz = GzDecoder::new(BufReader::new(file));
                let mut archive = tar::Archive::new(gz);
                for entry_res in archive.entries()? {
                    let mut entry = entry_res?;
                    let current_path = entry.path()?.to_string_lossy().to_string();
                    if current_path == entry_path || current_path == entry_path.trim_start_matches('/') {
                        let mut buffer = Vec::new();
                        entry.read_to_end(&mut buffer)?;
                        return Ok(buffer);
                    }
                }
                Err(ArchiveError::EntryNotFound(format!(
                    "Entry '{}' not found in {}",
                    entry_path,
                    path_ref.display()
                )))
            }
            ArchiveKind::Unknown => Err(ArchiveError::UnsupportedFormat(
                path_ref.to_string_lossy().to_string(),
            )),
        }
    }

    /// Preview text content of an entry without extracting the entire archive
    pub fn preview_text_entry<P: AsRef<Path>>(
        archive_path: P,
        entry_path: &str,
        max_lines: usize,
    ) -> Result<String, ArchiveError> {
        let bytes = Self::extract_single_entry_to_memory(archive_path, entry_path)?;
        let text = String::from_utf8_lossy(&bytes);
        let lines: Vec<&str> = text.lines().take(max_lines).collect();
        Ok(lines.join("\n"))
    }

    /// Search for entries inside an archive matching a case-insensitive query string
    pub fn search_entries<P: AsRef<Path>>(
        archive_path: P,
        query: &str,
    ) -> Result<Vec<ArchiveEntry>, ArchiveError> {
        let meta = Self::inspect(archive_path)?;
        let query_lower = query.to_lowercase();
        let matches = meta
            .entries
            .into_iter()
            .filter(|e| e.name.to_lowercase().contains(&query_lower) || e.path.to_lowercase().contains(&query_lower))
            .collect();
        Ok(matches)
    }

    /// List entries residing directly under a directory prefix inside the archive
    pub fn list_directory_entries<P: AsRef<Path>>(
        archive_path: P,
        dir_prefix: &str,
    ) -> Result<Vec<ArchiveEntry>, ArchiveError> {
        let meta = Self::inspect(archive_path)?;
        let prefix = dir_prefix.trim_matches('/');
        let matches = meta
            .entries
            .into_iter()
            .filter(|e| {
                let p = e.path.trim_matches('/');
                if prefix.is_empty() {
                    !p.contains('/')
                } else if let Some(rest) = p.strip_prefix(prefix) {
                    let rest = rest.trim_start_matches('/');
                    !rest.is_empty() && !rest.contains('/')
                } else {
                    false
                }
            })
            .collect();
        Ok(matches)
    }
}

/// Batch Tag Manager for labeling, querying, and persisting file path tags
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchTagManager {
    #[serde(skip)]
    pub store_path: Option<PathBuf>,
    pub tags_by_path: HashMap<PathBuf, HashSet<String>>,
}

impl BatchTagManager {
    pub fn new() -> Self {
        Self {
            store_path: None,
            tags_by_path: HashMap::new(),
        }
    }

    pub fn with_store_path<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            store_path: Some(path.into()),
            tags_by_path: HashMap::new(),
        }
    }

    /// Normalize a path for consistent dictionary lookup
    fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
        let p = path.as_ref();
        if let Ok(can) = p.canonicalize() {
            can
        } else {
            p.to_path_buf()
        }
    }

    /// Add a single tag to a file path. Returns true if tag was newly inserted.
    pub fn add_tag<P: AsRef<Path>>(&mut self, path: P, tag: &str) -> bool {
        let clean_tag = tag.trim().to_lowercase();
        if clean_tag.is_empty() {
            return false;
        }
        let norm = Self::normalize_path(path);
        self.tags_by_path
            .entry(norm)
            .or_default()
            .insert(clean_tag)
    }

    /// Add a batch of tags across multiple file paths. Returns total tags newly applied.
    pub fn add_tags_batch<P: AsRef<Path>, S: AsRef<str>>(
        &mut self,
        paths: &[P],
        tags: &[S],
    ) -> usize {
        let mut count = 0;
        let clean_tags: Vec<String> = tags
            .iter()
            .map(|t| t.as_ref().trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();

        if clean_tags.is_empty() {
            return 0;
        }

        for path in paths {
            let norm = Self::normalize_path(path);
            let entry = self.tags_by_path.entry(norm).or_default();
            for tag in &clean_tags {
                if entry.insert(tag.clone()) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Remove a single tag from a file path. Returns true if tag was present.
    pub fn remove_tag<P: AsRef<Path>>(&mut self, path: P, tag: &str) -> bool {
        let clean_tag = tag.trim().to_lowercase();
        let norm = Self::normalize_path(path);
        if let Some(tags) = self.tags_by_path.get_mut(&norm) {
            let removed = tags.remove(&clean_tag);
            if tags.is_empty() {
                self.tags_by_path.remove(&norm);
            }
            return removed;
        }
        false
    }

    /// Remove a batch of tags across multiple file paths. Returns count of tags removed.
    pub fn remove_tags_batch<P: AsRef<Path>, S: AsRef<str>>(
        &mut self,
        paths: &[P],
        tags: &[S],
    ) -> usize {
        let mut count = 0;
        let clean_tags: Vec<String> = tags
            .iter()
            .map(|t| t.as_ref().trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();

        for path in paths {
            let norm = Self::normalize_path(path);
            let mut empty = false;
            if let Some(set) = self.tags_by_path.get_mut(&norm) {
                for tag in &clean_tags {
                    if set.remove(tag) {
                        count += 1;
                    }
                }
                empty = set.is_empty();
            }
            if empty {
                self.tags_by_path.remove(&norm);
            }
        }
        count
    }

    /// Clear all tags for a specific path
    pub fn clear_tags<P: AsRef<Path>>(&mut self, path: P) -> bool {
        let norm = Self::normalize_path(path);
        self.tags_by_path.remove(&norm).is_some()
    }

    /// Clear all stored tags
    pub fn clear_all(&mut self) {
        self.tags_by_path.clear();
    }

    /// Get sorted list of tags for a path
    pub fn get_tags<P: AsRef<Path>>(&self, path: P) -> Vec<String> {
        let norm = Self::normalize_path(path);
        if let Some(set) = self.tags_by_path.get(&norm) {
            let mut tags: Vec<String> = set.iter().cloned().collect();
            tags.sort();
            tags
        } else {
            Vec::new()
        }
    }

    /// Set full replacement list of tags for a path
    pub fn set_tags<P: AsRef<Path>, S: AsRef<str>>(&mut self, path: P, tags: &[S]) {
        let norm = Self::normalize_path(path);
        let clean_set: HashSet<String> = tags
            .iter()
            .map(|t| t.as_ref().trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();

        if clean_set.is_empty() {
            self.tags_by_path.remove(&norm);
        } else {
            self.tags_by_path.insert(norm, clean_set);
        }
    }

    /// Check if path has a given tag
    pub fn has_tag<P: AsRef<Path>>(&self, path: P, tag: &str) -> bool {
        let clean_tag = tag.trim().to_lowercase();
        let norm = Self::normalize_path(path);
        self.tags_by_path
            .get(&norm)
            .map(|set| set.contains(&clean_tag))
            .unwrap_or(false)
    }

    /// Find all paths tagged with a specific tag
    pub fn find_paths_by_tag(&self, tag: &str) -> Vec<PathBuf> {
        let clean_tag = tag.trim().to_lowercase();
        let mut paths: Vec<PathBuf> = self
            .tags_by_path
            .iter()
            .filter(|(_, tags)| tags.contains(&clean_tag))
            .map(|(p, _)| p.clone())
            .collect();
        paths.sort();
        paths
    }

    /// Find paths that have ALL specified tags (AND condition)
    pub fn find_paths_by_all_tags(&self, tags: &[&str]) -> Vec<PathBuf> {
        let clean_tags: Vec<String> = tags
            .iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();

        if clean_tags.is_empty() {
            return Vec::new();
        }

        let mut paths: Vec<PathBuf> = self
            .tags_by_path
            .iter()
            .filter(|(_, set)| clean_tags.iter().all(|t| set.contains(t)))
            .map(|(p, _)| p.clone())
            .collect();
        paths.sort();
        paths
    }

    /// Find paths that have ANY of the specified tags (OR condition)
    pub fn find_paths_by_any_tag(&self, tags: &[&str]) -> Vec<PathBuf> {
        let clean_tags: Vec<String> = tags
            .iter()
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();

        if clean_tags.is_empty() {
            return Vec::new();
        }

        let mut paths: Vec<PathBuf> = self
            .tags_by_path
            .iter()
            .filter(|(_, set)| clean_tags.iter().any(|t| set.contains(t)))
            .map(|(p, _)| p.clone())
            .collect();
        paths.sort();
        paths
    }

    /// Retrieve all distinct tags in the manager
    pub fn all_tags(&self) -> Vec<String> {
        let mut unique = HashSet::new();
        for set in self.tags_by_path.values() {
            for tag in set {
                unique.insert(tag.clone());
            }
        }
        let mut tags: Vec<String> = unique.into_iter().collect();
        tags.sort();
        tags
    }

    /// Count how many files are associated with each tag
    pub fn tag_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for set in self.tags_by_path.values() {
            for tag in set {
                *counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Rename an existing tag across all files. Returns number of files affected.
    pub fn rename_tag(&mut self, old_tag: &str, new_tag: &str) -> usize {
        let old_clean = old_tag.trim().to_lowercase();
        let new_clean = new_tag.trim().to_lowercase();

        if old_clean == new_clean || old_clean.is_empty() || new_clean.is_empty() {
            return 0;
        }

        let mut affected = 0;
        for set in self.tags_by_path.values_mut() {
            if set.remove(&old_clean) {
                set.insert(new_clean.clone());
                affected += 1;
            }
        }
        affected
    }

    /// Delete a tag from all paths. Returns number of files affected.
    pub fn delete_tag(&mut self, tag: &str) -> usize {
        let clean_tag = tag.trim().to_lowercase();
        let mut affected = 0;
        let mut empty_paths = Vec::new();

        for (path, set) in self.tags_by_path.iter_mut() {
            if set.remove(&clean_tag) {
                affected += 1;
            }
            if set.is_empty() {
                empty_paths.push(path.clone());
            }
        }

        for path in empty_paths {
            self.tags_by_path.remove(&path);
        }

        affected
    }

    /// Total number of uniquely tagged paths
    pub fn total_tagged_paths(&self) -> usize {
        self.tags_by_path.len()
    }

    /// Total number of unique tags
    pub fn total_unique_tags(&self) -> usize {
        self.all_tags().len()
    }

    /// Save tags mapping to configured store path
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(store) = &self.store_path {
            if let Some(parent) = store.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            std::fs::write(store, json)?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No store_path configured for BatchTagManager",
            ))
        }
    }

    /// Load tags mapping from configured store path
    pub fn load(&mut self) -> Result<(), std::io::Error> {
        if let Some(store) = &self.store_path {
            if store.exists() {
                let content = std::fs::read_to_string(store)?;
                let loaded: BatchTagManager = serde_json::from_str(&content)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                self.tags_by_path = loaded.tags_by_path;
            }
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No store_path configured for BatchTagManager",
            ))
        }
    }

    /// Export tag database as JSON string
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import tag database from JSON string
    pub fn import_json(&mut self, json_str: &str) -> Result<(), serde_json::Error> {
        let imported: BatchTagManager = serde_json::from_str(json_str)?;
        for (path, tags) in imported.tags_by_path {
            self.tags_by_path.entry(path).or_default().extend(tags);
        }
        Ok(())
    }
}

/// Helper function to format archive bytes
fn format_archive_size(bytes: u64, is_dir: bool) -> String {
    if is_dir {
        return "--".to_string();
    }
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Helper function to format epoch timestamp into local date string
fn format_timestamp(secs: u64) -> String {
    if secs == 0 {
        return "--".to_string();
    }
    match Local.timestamp_opt(secs as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => "--".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    #[test]
    fn test_archive_kind_detection() {
        assert_eq!(ArchiveKind::from_path("test.zip"), ArchiveKind::Zip);
        assert_eq!(ArchiveKind::from_path("archive.tar"), ArchiveKind::Tar);
        assert_eq!(ArchiveKind::from_path("package.tar.gz"), ArchiveKind::TarGz);
        assert_eq!(ArchiveKind::from_path("package.tgz"), ArchiveKind::TarGz);
        assert_eq!(ArchiveKind::from_path("file.txt"), ArchiveKind::Unknown);
        assert!(ArchiveKind::Zip.is_supported());
        assert!(!ArchiveKind::Unknown.is_supported());
    }

    #[test]
    fn test_zip_inspection_and_in_memory_preview() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test_sample.zip");

        // Create test zip archive
        {
            let file = File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            zip.add_directory("docs/", options).unwrap();
            zip.start_file("docs/readme.txt", options).unwrap();
            zip.write_all(b"Hello SWAL Desktop Archive Inspector!\nLine 2\nLine 3\n").unwrap();

            zip.start_file("config.json", options).unwrap();
            zip.write_all(b"{\"name\": \"swal-files\"}").unwrap();

            zip.finish().unwrap();
        }

        // Inspect metadata
        let meta = ArchiveInspector::inspect(&zip_path).unwrap();
        assert_eq!(meta.kind, ArchiveKind::Zip);
        assert_eq!(meta.total_entries, 3);
        assert_eq!(meta.file_count, 2);
        assert_eq!(meta.dir_count, 1);
        assert!(meta.uncompressed_size_bytes > 0);

        // Search entries
        let search_results = ArchiveInspector::search_entries(&zip_path, "readme").unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].name, "readme.txt");

        // In-memory extraction without disk extraction
        let buffer = ArchiveInspector::extract_single_entry_to_memory(&zip_path, "docs/readme.txt").unwrap();
        let text = String::from_utf8(buffer).unwrap();
        assert!(text.contains("Hello SWAL Desktop Archive Inspector!"));

        // Text preview
        let preview = ArchiveInspector::preview_text_entry(&zip_path, "docs/readme.txt", 2).unwrap();
        assert_eq!(preview, "Hello SWAL Desktop Archive Inspector!\nLine 2");
    }

    #[test]
    fn test_tar_and_tar_gz_inspection() {
        let dir = tempdir().unwrap();
        let tar_gz_path = dir.path().join("test_archive.tar.gz");

        // Create test tar.gz archive
        {
            let file = File::create(&tar_gz_path).unwrap();
            let gz = GzEncoder::new(file, Compression::default());
            let mut tar_builder = tar::Builder::new(gz);

            let data = b"Rust TarGz Stream Inspection Test";
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();

            tar_builder.append_data(&mut header, "src/main.rs", &data[..]).unwrap();
            tar_builder.finish().unwrap();
        }

        // Inspect TAR.GZ
        let meta = ArchiveInspector::inspect(&tar_gz_path).unwrap();
        assert_eq!(meta.kind, ArchiveKind::TarGz);
        assert_eq!(meta.total_entries, 1);
        assert_eq!(meta.file_count, 1);
        assert_eq!(meta.entries[0].name, "main.rs");

        // Memory extraction
        let buf = ArchiveInspector::extract_single_entry_to_memory(&tar_gz_path, "src/main.rs").unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "Rust TarGz Stream Inspection Test");
    }

    #[test]
    fn test_batch_tag_manager_crud_and_queries() {
        let mut manager = BatchTagManager::new();

        let path1 = PathBuf::from("/tmp/doc1.txt");
        let path2 = PathBuf::from("/tmp/doc2.txt");
        let path3 = PathBuf::from("/tmp/code.rs");

        // Add single tag
        assert!(manager.add_tag(&path1, "work"));
        assert!(manager.add_tag(&path1, "urgent"));
        assert!(!manager.add_tag(&path1, "work")); // Duplicate should return false

        // Add batch tags
        let applied = manager.add_tags_batch(&[path2.clone(), path3.clone()], &["work", "project-x"]);
        assert_eq!(applied, 4);

        // Verify tags for path1
        let tags1 = manager.get_tags(&path1);
        assert_eq!(tags1, vec!["urgent".to_string(), "work".to_string()]);

        // Queries
        let work_paths = manager.find_paths_by_tag("work");
        assert_eq!(work_paths.len(), 3);

        let urgent_work = manager.find_paths_by_all_tags(&["work", "urgent"]);
        assert_eq!(urgent_work, vec![path1.clone()]);

        let any_project = manager.find_paths_by_any_tag(&["urgent", "project-x"]);
        assert_eq!(any_project.len(), 3);

        // Tag counts
        let counts = manager.tag_counts();
        assert_eq!(counts.get("work"), Some(&3));
        assert_eq!(counts.get("urgent"), Some(&1));
        assert_eq!(counts.get("project-x"), Some(&2));

        // Rename tag
        let renamed = manager.rename_tag("project-x", "swal-project");
        assert_eq!(renamed, 2);
        assert!(manager.has_tag(&path2, "swal-project"));
        assert!(!manager.has_tag(&path2, "project-x"));

        // Remove tag batch
        let removed = manager.remove_tags_batch(&[path2.clone()], &["work"]);
        assert_eq!(removed, 1);
        assert!(!manager.has_tag(&path2, "work"));

        // Delete tag completely
        let deleted = manager.delete_tag("swal-project");
        assert_eq!(deleted, 2);
        assert_eq!(manager.find_paths_by_tag("swal-project").len(), 0);
    }

    #[test]
    fn test_batch_tag_manager_persistence_and_json() {
        let dir = tempdir().unwrap();
        let store_file = dir.path().join("tags_store.json");

        let mut manager = BatchTagManager::with_store_path(&store_file);
        let path = PathBuf::from("/tmp/file.pdf");
        manager.add_tags_batch(&[path.clone()], &["finance", "receipts", "2026"]);

        // Save
        manager.save().unwrap();
        assert!(store_file.exists());

        // Load into new instance
        let mut loaded = BatchTagManager::with_store_path(&store_file);
        loaded.load().unwrap();
        assert_eq!(loaded.get_tags(&path), vec!["2026", "finance", "receipts"]);

        // JSON roundtrip
        let json = manager.export_json().unwrap();
        let mut imported = BatchTagManager::new();
        imported.import_json(&json).unwrap();
        assert_eq!(imported.get_tags(&path), vec!["2026", "finance", "receipts"]);
    }
}
