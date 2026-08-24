//! watcher.rs
//! File system watcher and event streaming for SWAL Files Core.

use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, RecvError, RecvTimeoutError, Sender};
use std::time::Duration;

/// Representation of a file system mutation event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsChangeEvent {
    /// A file or directory was created.
    Created(PathBuf),
    /// A file or directory was modified or written to.
    Modified(PathBuf),
    /// A file or directory was removed or deleted.
    Deleted(PathBuf),
}

impl FsChangeEvent {
    /// Returns the target path of this change event.
    pub fn path(&self) -> &Path {
        match self {
            FsChangeEvent::Created(p) => p.as_path(),
            FsChangeEvent::Modified(p) => p.as_path(),
            FsChangeEvent::Deleted(p) => p.as_path(),
        }
    }

    /// Returns true if this is a creation event.
    pub fn is_created(&self) -> bool {
        matches!(self, FsChangeEvent::Created(_))
    }

    /// Returns true if this is a modification event.
    pub fn is_modified(&self) -> bool {
        matches!(self, FsChangeEvent::Modified(_))
    }

    /// Returns true if this is a deletion event.
    pub fn is_deleted(&self) -> bool {
        matches!(self, FsChangeEvent::Deleted(_))
    }

    /// Returns the string representation of event kind.
    pub fn kind_str(&self) -> &'static str {
        match self {
            FsChangeEvent::Created(_) => "created",
            FsChangeEvent::Modified(_) => "modified",
            FsChangeEvent::Deleted(_) => "deleted",
        }
    }
}

/// Errors that can occur within the file watcher.
#[derive(Debug)]
pub enum WatcherError {
    Notify(notify::Error),
    Io(std::io::Error),
    PathNotFound(PathBuf),
    NotADirectory(PathBuf),
    WatcherNotInitialized,
}

impl fmt::Display for WatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatcherError::Notify(e) => write!(f, "Notify error: {e}"),
            WatcherError::Io(e) => write!(f, "IO error: {e}"),
            WatcherError::PathNotFound(p) => write!(f, "Path not found: {}", p.display()),
            WatcherError::NotADirectory(p) => write!(f, "Path is not a directory: {}", p.display()),
            WatcherError::WatcherNotInitialized => write!(f, "Watcher instance not initialized"),
        }
    }
}

impl std::error::Error for WatcherError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WatcherError::Notify(e) => Some(e),
            WatcherError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<notify::Error> for WatcherError {
    fn from(err: notify::Error) -> Self {
        WatcherError::Notify(err)
    }
}

impl From<std::io::Error> for WatcherError {
    fn from(err: std::io::Error) -> Self {
        WatcherError::Io(err)
    }
}

/// Directory watcher providing non-blocking file system change notifications.
pub struct DirectoryWatcher {
    path: PathBuf,
    recursive: bool,
    watcher: Option<RecommendedWatcher>,
    receiver: Receiver<FsChangeEvent>,
    sender: Sender<FsChangeEvent>,
}

impl DirectoryWatcher {
    /// Creates and starts watching a directory non-recursively.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, WatcherError> {
        Self::create(path.as_ref(), false)
    }

    /// Creates and starts watching a directory recursively.
    pub fn new_recursive<P: AsRef<Path>>(path: P) -> Result<Self, WatcherError> {
        Self::create(path.as_ref(), true)
    }

    fn create(path: &Path, recursive: bool) -> Result<Self, WatcherError> {
        if !path.exists() {
            return Err(WatcherError::PathNotFound(path.to_path_buf()));
        }
        if !path.is_dir() {
            return Err(WatcherError::NotADirectory(path.to_path_buf()));
        }

        let canonical_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let (tx, rx) = channel();
        let tx_clone = tx.clone();

        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
            if let Ok(event) = res {
                if event.kind.is_access() {
                    return;
                }
                for p in event.paths {
                    let change_event = match event.kind {
                        EventKind::Create(_) => Some(FsChangeEvent::Created(p)),
                        EventKind::Modify(_) => Some(FsChangeEvent::Modified(p)),
                        EventKind::Remove(_) => Some(FsChangeEvent::Deleted(p)),
                        EventKind::Any | EventKind::Other => Some(FsChangeEvent::Modified(p)),
                        _ => None,
                    };
                    if let Some(evt) = change_event {
                        let _ = tx_clone.send(evt);
                    }
                }
            }
        })?;

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        watcher.watch(&canonical_path, mode)?;

        Ok(Self {
            path: canonical_path,
            recursive,
            watcher: Some(watcher),
            receiver: rx,
            sender: tx,
        })
    }

    /// Returns the currently watched directory path.
    pub fn watched_path(&self) -> &Path {
        &self.path
    }

    /// Returns whether the watcher is running in recursive mode.
    pub fn is_recursive(&self) -> bool {
        self.recursive
    }

    /// Returns true if the underlying watcher is active.
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// Switches the watched target directory.
    pub fn watch_path<P: AsRef<Path>>(&mut self, new_path: P, recursive: bool) -> Result<(), WatcherError> {
        let p = new_path.as_ref();
        if !p.exists() {
            return Err(WatcherError::PathNotFound(p.to_path_buf()));
        }
        if !p.is_dir() {
            return Err(WatcherError::NotADirectory(p.to_path_buf()));
        }

        let canonical = fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());

        if let Some(mut watcher) = self.watcher.take() {
            let _ = watcher.unwatch(&self.path);
            let mode = if recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            watcher.watch(&canonical, mode)?;
            self.watcher = Some(watcher);
            self.path = canonical;
            self.recursive = recursive;
            Ok(())
        } else {
            // Re-instantiate watcher if previously stopped
            let tx_clone = self.sender.clone();
            let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
                if let Ok(event) = res {
                    if event.kind.is_access() {
                        return;
                    }
                    for path_item in event.paths {
                        let change_event = match event.kind {
                            EventKind::Create(_) => Some(FsChangeEvent::Created(path_item)),
                            EventKind::Modify(_) => Some(FsChangeEvent::Modified(path_item)),
                            EventKind::Remove(_) => Some(FsChangeEvent::Deleted(path_item)),
                            EventKind::Any | EventKind::Other => Some(FsChangeEvent::Modified(path_item)),
                            _ => None,
                        };
                        if let Some(evt) = change_event {
                            let _ = tx_clone.send(evt);
                        }
                    }
                }
            })?;
            let mode = if recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            watcher.watch(&canonical, mode)?;
            self.watcher = Some(watcher);
            self.path = canonical;
            self.recursive = recursive;
            Ok(())
        }
    }

    /// Stops watching the current directory.
    pub fn unwatch(&mut self) -> Result<(), WatcherError> {
        if let Some(mut watcher) = self.watcher.take() {
            let _ = watcher.unwatch(&self.path);
        }
        Ok(())
    }

    /// Non-blocking check for the next file change event.
    pub fn try_recv(&self) -> Option<FsChangeEvent> {
        self.receiver.try_recv().ok()
    }

    /// Non-blocking polling that drains all currently available change events.
    pub fn poll_events(&self) -> Vec<FsChangeEvent> {
        let mut events = Vec::new();
        while let Ok(evt) = self.receiver.try_recv() {
            events.push(evt);
        }
        events
    }

    /// Blocking wait for the next event up to the specified timeout duration.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<FsChangeEvent, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    /// Blocking wait for the next event.
    pub fn recv(&self) -> Result<FsChangeEvent, RecvError> {
        self.receiver.recv()
    }

    /// Returns a reference to the underlying receiver channel.
    pub fn receiver(&self) -> &Receiver<FsChangeEvent> {
        &self.receiver
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_fs_change_event_variants_and_helpers() {
        let p = PathBuf::from("/tmp/test.txt");
        let c = FsChangeEvent::Created(p.clone());
        let m = FsChangeEvent::Modified(p.clone());
        let d = FsChangeEvent::Deleted(p.clone());

        assert!(c.is_created());
        assert!(!c.is_modified());
        assert_eq!(c.kind_str(), "created");
        assert_eq!(c.path(), p.as_path());

        assert!(m.is_modified());
        assert!(!m.is_deleted());
        assert_eq!(m.kind_str(), "modified");

        assert!(d.is_deleted());
        assert!(!d.is_created());
        assert_eq!(d.kind_str(), "deleted");

        // Serde roundtrip
        let serialized = serde_json::to_string(&c).expect("serialize");
        let deserialized: FsChangeEvent = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn test_watcher_invalid_paths() {
        let invalid_path = PathBuf::from("/nonexistent_path_404_test_swal");
        let result = DirectoryWatcher::new(&invalid_path);
        assert!(matches!(result, Err(WatcherError::PathNotFound(_))));

        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("not_a_dir.txt");
        File::create(&file_path).expect("create file");

        let result2 = DirectoryWatcher::new(&file_path);
        assert!(matches!(result2, Err(WatcherError::NotADirectory(_))));
    }

    #[test]
    fn test_watcher_lifecycle_and_non_blocking_polling() {
        let dir = tempdir().expect("tempdir");
        let watcher = DirectoryWatcher::new(dir.path()).expect("create watcher");

        assert!(watcher.is_watching());
        assert!(!watcher.is_recursive());
        assert_eq!(
            watcher.watched_path(),
            fs::canonicalize(dir.path()).unwrap().as_path()
        );

        // Initially no events
        assert_eq!(watcher.try_recv(), None);
        assert!(watcher.poll_events().is_empty());

        // Create a file
        let test_file = dir.path().join("swal_event_test.txt");
        {
            let mut file = File::create(&test_file).expect("create file");
            file.write_all(b"hello swal").expect("write file");
            file.sync_all().expect("sync");
        }

        // Wait with timeout to allow OS inotify delivery
        let mut got_event = false;
        for _ in 0..20 {
            if let Ok(_evt) = watcher.recv_timeout(Duration::from_millis(100)) {
                got_event = true;
                break;
            }
        }
        assert!(got_event, "Expected to receive file creation/modification event");

        // Drain any remaining events non-blockingly
        let _events = watcher.poll_events();

        // Modify file
        {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&test_file)
                .expect("open file");
            file.write_all(b"\nmore data").expect("append file");
            file.sync_all().expect("sync");
        }

        let mut got_modify = false;
        for _ in 0..20 {
            if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
                if evt.is_modified() {
                    got_modify = true;
                    break;
                }
            }
        }
        assert!(got_modify, "Expected modify event");

        // Delete file
        fs::remove_file(&test_file).expect("delete file");
        let mut got_delete = false;
        for _ in 0..20 {
            if let Ok(evt) = watcher.recv_timeout(Duration::from_millis(100)) {
                if evt.is_deleted() {
                    got_delete = true;
                    break;
                }
            }
        }
        assert!(got_delete, "Expected delete event");
    }

    #[test]
    fn test_watcher_unwatch_and_repath() {
        let dir_a = tempdir().expect("tempdir a");
        let dir_b = tempdir().expect("tempdir b");

        let mut watcher = DirectoryWatcher::new(dir_a.path()).expect("watcher");
        assert!(watcher.is_watching());

        watcher.unwatch().expect("unwatch");
        assert!(!watcher.is_watching());

        watcher.watch_path(dir_b.path(), true).expect("watch path b");
        assert!(watcher.is_watching());
        assert!(watcher.is_recursive());
        assert_eq!(
            watcher.watched_path(),
            fs::canonicalize(dir_b.path()).unwrap().as_path()
        );
    }
}
