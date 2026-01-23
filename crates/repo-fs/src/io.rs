//! Atomic I/O operations with file locking

use std::fs::{self, OpenOptions};
use std::io::Write;
use fs2::FileExt;
use crate::{Error, NormalizedPath, Result};

/// Write content atomically to a file with locking.
///
/// Uses write-to-temp-then-rename strategy to prevent partial writes.
/// Acquires an advisory lock to prevent concurrent access.
pub fn write_atomic(path: &NormalizedPath, content: &[u8]) -> Result<()> {
    let native_path = path.to_native();

    // Ensure parent directory exists
    if let Some(parent) = native_path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }

    // Generate temp file path in same directory (ensures same filesystem)
    let temp_name = format!(
        ".{}.{}.tmp",
        native_path.file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        std::process::id()
    );
    let temp_path = native_path.with_file_name(&temp_name);

    // Write to temp file
    let mut temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp_path)
        .map_err(|e| Error::io(&temp_path, e))?;

    // Acquire exclusive lock
    temp_file.lock_exclusive()
        .map_err(|_| Error::LockFailed { path: native_path.clone() })?;

    // Write content
    temp_file.write_all(content)
        .map_err(|e| Error::io(&temp_path, e))?;

    // Flush to disk
    temp_file.sync_all()
        .map_err(|e| Error::io(&temp_path, e))?;

    // Release lock (implicit on drop, but be explicit)
    temp_file.unlock()
        .map_err(|_| Error::LockFailed { path: native_path.clone() })?;

    // Atomic rename
    fs::rename(&temp_path, &native_path)
        .map_err(|e| Error::io(&native_path, e))?;

    Ok(())
}

/// Read text content from a file.
///
/// TODO: PLACEHOLDER - replace with ManagedBlockEditor
pub fn read_text(path: &NormalizedPath) -> Result<String> {
    let native_path = path.to_native();
    fs::read_to_string(&native_path)
        .map_err(|e| Error::io(&native_path, e))
}

/// Write text content to a file atomically.
///
/// TODO: PLACEHOLDER - replace with ManagedBlockEditor
pub fn write_text(path: &NormalizedPath, content: &str) -> Result<()> {
    write_atomic(path, content.as_bytes())
}
