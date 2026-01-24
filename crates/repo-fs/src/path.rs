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
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path_str = path.as_ref().to_string_lossy();

        // Optimization: Fast path for already-clean paths
        // Check for backslashes (Windows) or . / .. / empty components (Cleaning)
        let mut needs_work = false;
        if path_str.contains('\\') {
            needs_work = true;
        } else {
            for component in path_str.split('/') {
                if component.is_empty() || component == "." || component == ".." {
                    needs_work = true;
                    break;
                }
            }
        }

        if !needs_work {
            return Self {
                inner: path_str.into_owned(),
            };
        }

        let normalized = path_str.replace('\\', "/");
        Self {
            inner: Self::clean(&normalized),
        }
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
        // We still need to clean joined result because "a/b" + "../c" -> Needs resolution
        Self {
            inner: Self::clean(&joined),
        }
    }

    /// Clean the path by resolving . and .. components
    fn clean(path: &str) -> String {
        // Optimization: check if we actually need to do anything
        // calling split matches internal logic
        let mut needs_cleaning = false;
        for component in path.split('/') {
            if component.is_empty() || component == "." || component == ".." {
                needs_cleaning = true;
                break;
            }
        }

        if !needs_cleaning {
            return path.to_owned();
        }

        let mut out = Vec::new();
        // Check for UNC-like double slash (but not triple)
        let is_network = path.starts_with("//") && !path.starts_with("///");
        let is_absolute = path.starts_with('/') || is_network;

        for component in path.split('/') {
            match component {
                "" | "." => continue,
                ".." => {
                    if !out.is_empty() {
                        out.pop();
                    } else if !is_absolute {
                        // If relative, we drop leading .. (sandbox behavior)
                    }
                }
                c => out.push(c),
            }
        }

        // Re-construct
        let mut res = String::with_capacity(path.len());
        if is_network {
            res.push_str("//");
        } else if is_absolute {
            res.push('/');
        }

        for (i, component) in out.iter().enumerate() {
            if i > 0 {
                res.push('/');
            }
            res.push_str(component);
        }

        if res.is_empty() && !is_absolute {
            res.push('.');
        }

        res
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
