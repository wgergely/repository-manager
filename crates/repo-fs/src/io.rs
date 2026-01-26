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

        // 2. Write to temp file
        // Generate temp file path in same directory (ensures same filesystem)
        // We keep the PID to avoid collisions within the locked region if different threads used different temp files?
        // Actually, since we hold the .lock, we are the only one writing TO THIS DESTINATION.
        // But to be extra safe against stale temp files, we use a random/pid name.
        let temp_name = format!(
            ".{}.{}.tmp",
            native_path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default(),
            std::process::id()
        );
        let temp_path = native_path.with_file_name(&temp_name);

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

        // 4. Release lock
        let _ = lock_file.unlock();

        Ok(())
    };

    // Configure retry backoff
    let backoff_policy = ExponentialBackoff {
        max_elapsed_time: Some(config.lock_timeout),
        ..ExponentialBackoff::default()
    };

    // Run the operation
    backoff::retry(backoff_policy, op).map_err(|e| match e {
        backoff::Error::Permanent(err) | backoff::Error::Transient { err, .. } => err,
    })
}

/// Read text content from a file.
///
/// TODO: PLACEHOLDER - replace with ManagedBlockEditor
pub fn read_text(path: &NormalizedPath) -> Result<String> {
    let native_path = path.to_native();
    fs::read_to_string(&native_path).map_err(|e| Error::io(&native_path, e))
}

/// Write text content to a file atomically.
///
/// TODO: PLACEHOLDER - replace with ManagedBlockEditor
pub fn write_text(path: &NormalizedPath, content: &str) -> Result<()> {
    write_atomic(path, content.as_bytes(), RobustnessConfig::default())
}
