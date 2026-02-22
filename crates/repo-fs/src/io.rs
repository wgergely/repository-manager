//! Atomic I/O operations with file locking

use crate::{Error, NormalizedPath, Result};
use fs2::FileExt;
use std::fs::{self, OpenOptions};
use std::io::Write;

use backoff::ExponentialBackoff;
use std::time::Duration;

/// Configuration for filesystem robustness and performance trade-offs.
#[derive(Debug, Clone, Copy)]
pub struct RobustnessConfig {
    /// Whether to perform `fs::File::sync_all` guarantees.
    ///
    /// Disable this on high-latency network drives if performance is critical
    /// and data loss on power failure is an acceptable risk.
    pub enable_fsync: bool,

    /// Maximum duration to wait for a file lock before failing.
    pub lock_timeout: Duration,
}

impl Default for RobustnessConfig {
    fn default() -> Self {
        Self {
            enable_fsync: true,
            // default 10s timeout for locks
            lock_timeout: Duration::from_secs(10),
        }
    }
}

/// Check if any component in the path (or its ancestors) is a symlink.
///
/// This prevents symlink-based attacks where writes could escape intended directories.
fn contains_symlink(path: &std::path::Path) -> std::io::Result<bool> {
    use std::path::PathBuf;

    let mut current = PathBuf::from(path);

    // Walk up the path checking each component
    loop {
        if current.exists() {
            let metadata = std::fs::symlink_metadata(&current)?;
            if metadata.file_type().is_symlink() {
                return Ok(true);
            }
        }

        match current.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => {
                current = parent.to_path_buf();
            }
            _ => break,
        }
    }

    Ok(false)
}

/// Write content atomically to a file with locking and retry logic.
///
/// Uses write-to-temp-then-rename strategy to prevent partial writes.
/// Acquires an advisory lock to prevent concurrent access.
///
/// # Retry Logic
/// Uses exponential backoff for:
/// - Acquiring locks (simulating timeout via try_lock loop)
/// - Transient I/O errors (e.g. network blips)
pub fn write_atomic(path: &NormalizedPath, content: &[u8], config: RobustnessConfig) -> Result<()> {
    tracing::debug!(path = %path.as_str(), content_len = content.len(), "Starting atomic write");
    let native_path = path.to_native();

    // Security: Reject paths containing symlinks to prevent escape attacks
    if contains_symlink(&native_path).unwrap_or(true) {
        return Err(Error::SymlinkInPath {
            path: native_path.clone(),
        });
    }

    // Ensure parent directory exists
    if let Some(parent) = native_path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }

    // 1. Acquire coordination lock on a separate lock file
    let lock_path = format!("{}.lock", native_path.to_string_lossy());
    let lock_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false) // Don't truncate lock file, just open it
        .open(&lock_path)
        .map_err(|e| Error::io(&lock_path, e))?;

    // Compute temp file path up front so we can clean up on final failure
    let temp_name = format!(
        ".{}.{}.tmp",
        native_path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        std::process::id()
    );
    let temp_path = native_path.with_file_name(&temp_name);

    // Define the operation to perform with retry support
    // We wrap the whole locking + write + rename sequence
    let op = || -> std::result::Result<(), backoff::Error<Error>> {
        // Try to acquire exclusive lock on the lock file
        // This coordinates between processes
        lock_file.try_lock_exclusive().map_err(|_| {
            backoff::Error::transient(Error::LockFailed {
                path: native_path.clone(),
            })
        })?;

        // Guard to ensure we unlock even if we panic (though try_lock_exclusive releases on close)
        // Explicit unlock is not strictly needed if we drop the file, but good for clarity.
        // We will hold this lock until the end of the closure.

        let mut temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .map_err(|e| backoff::Error::transient(Error::io(&temp_path, e)))?;

        // Write content
        temp_file
            .write_all(content)
            .map_err(|e| backoff::Error::transient(Error::io(&temp_path, e)))?;

        // Flush to disk if enabled
        if config.enable_fsync {
            temp_file
                .sync_all()
                .map_err(|e| backoff::Error::transient(Error::io(&temp_path, e)))?;
        }

        // Close temp file explicitly before rename (improves Windows reliability)
        drop(temp_file);

        // 3. Atomic rename
        // Replaces target if exists
        fs::rename(&temp_path, &native_path)
            .map_err(|e| backoff::Error::transient(Error::io(&native_path, e)))?;

        // 4. Release lock (advisory locks are also released on fd close,
        // so an explicit unlock failure is non-critical but worth logging)
        if let Err(e) = lock_file.unlock() {
            tracing::warn!(
                "Failed to release lock for {}: {}",
                native_path.display(),
                e
            );
        }

        Ok(())
    };

    // Configure retry backoff
    let backoff_policy = ExponentialBackoff {
        max_elapsed_time: Some(config.lock_timeout),
        ..ExponentialBackoff::default()
    };

    // Run the operation; clean up temp file on final failure
    let result = backoff::retry(backoff_policy, op).map_err(|e| match e {
        backoff::Error::Permanent(err) | backoff::Error::Transient { err, .. } => err,
    });

    if result.is_err() {
        // Best-effort cleanup of orphaned temp file
        let _ = fs::remove_file(&temp_path);
    }

    // Best-effort cleanup of lock file after successful write
    // Lock files are no longer needed once the atomic rename completes
    let _ = fs::remove_file(&lock_path);

    result
}

/// Read text content from a file.
pub fn read_text(path: &NormalizedPath) -> Result<String> {
    let native_path = path.to_native();
    fs::read_to_string(&native_path).map_err(|e| Error::io(&native_path, e))
}

/// Write text content to a file atomically.
pub fn write_text(path: &NormalizedPath, content: &str) -> Result<()> {
    write_atomic(path, content.as_bytes(), RobustnessConfig::default())
}
