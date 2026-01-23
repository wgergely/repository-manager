//! Normalized path handling for cross-platform compatibility

use std::path::{Path, PathBuf};

/// A path normalized to use forward slashes internally.
///
/// Provides consistent path handling across platforms by normalizing
/// all paths to forward slashes internally and converting to
/// platform-native format only at I/O boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPath {
    /// Internal representation always uses forward slashes
    inner: String,
}

impl NormalizedPath {
    /// Create a new NormalizedPath from any path-like input.
    ///
    /// Converts backslashes to forward slashes for internal storage.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path_str = path.as_ref().to_string_lossy();
        let normalized = path_str.replace('\\', "/");
        Self { inner: normalized }
    }

    /// Get the internal normalized string representation.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to a platform-native PathBuf for I/O operations.
    pub fn to_native(&self) -> PathBuf {
        PathBuf::from(&self.inner)
    }

    /// Join this path with a segment.
    pub fn join(&self, segment: &str) -> Self {
        let segment_normalized = segment.replace('\\', "/");
        let joined = if self.inner.ends_with('/') {
            format!("{}{}", self.inner, segment_normalized)
        } else {
            format!("{}/{}", self.inner, segment_normalized)
        };
        Self { inner: joined }
    }

    /// Get the parent directory.
    pub fn parent(&self) -> Option<Self> {
        let trimmed = self.inner.trim_end_matches('/');
        match trimmed.rfind('/') {
            Some(idx) if idx > 0 => Some(Self {
                inner: trimmed[..idx].to_string(),
            }),
            Some(0) => Some(Self {
                inner: "/".to_string(),
            }),
            _ => None,
        }
    }

    /// Get the file name component.
    pub fn file_name(&self) -> Option<&str> {
        let trimmed = self.inner.trim_end_matches('/');
        trimmed.rsplit('/').next()
    }

    /// Check if this path exists on the filesystem.
    pub fn exists(&self) -> bool {
        self.to_native().exists()
    }

    /// Check if this is a directory.
    pub fn is_dir(&self) -> bool {
        self.to_native().is_dir()
    }

    /// Check if this is a file.
    pub fn is_file(&self) -> bool {
        self.to_native().is_file()
    }

    /// Check if this appears to be a network path.
    ///
    /// Detects UNC paths (//server/share or \\server\share)
    /// and warns but allows operation.
    pub fn is_network_path(&self) -> bool {
        self.inner.starts_with("//")
            || self.inner.starts_with("\\\\")
            || self.inner.starts_with("smb://")
            || self.inner.starts_with("nfs://")
    }

    /// Get the extension if present.
    pub fn extension(&self) -> Option<&str> {
        self.file_name().and_then(|name| {
            let idx = name.rfind('.')?;
            if idx == 0 {
                None
            } else {
                Some(&name[idx + 1..])
            }
        })
    }
}

impl AsRef<Path> for NormalizedPath {
    fn as_ref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl std::fmt::Display for NormalizedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<&str> for NormalizedPath {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for NormalizedPath {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<PathBuf> for NormalizedPath {
    fn from(p: PathBuf) -> Self {
        Self::new(p)
    }
}

impl From<&Path> for NormalizedPath {
    fn from(p: &Path) -> Self {
        Self::new(p)
    }
}
