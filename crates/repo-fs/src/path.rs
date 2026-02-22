//! Normalized path handling for cross-platform compatibility

use std::path::{Path, PathBuf};

/// Validate that a user-supplied identifier is safe for use in file paths.
///
/// Rejects identifiers containing path separators, traversal sequences,
/// null bytes, or names starting with `-` (to prevent flag injection).
pub fn validate_path_identifier(id: &str, label: &str) -> std::result::Result<(), String> {
    if id.is_empty() {
        return Err(format!("{} must not be empty", label));
    }
    if id.contains('/') || id.contains('\\') {
        return Err(format!("{} must not contain path separators", label));
    }
    if id.contains("..") {
        return Err(format!("{} must not contain '..'", label));
    }
    if id.contains('\0') {
        return Err(format!("{} must not contain null bytes", label));
    }
    if id.starts_with('-') {
        return Err(format!("{} must not start with '-'", label));
    }
    if id.len() > 255 {
        return Err(format!(
            "{} exceeds maximum length of 255 characters",
            label
        ));
    }
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(format!(
            "{} must contain only alphanumeric characters, hyphens, underscores, or dots",
            label
        ));
    }
    Ok(())
}

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
        let cleaned = Self::clean(&normalized);

        // Reject network/UNC paths — after normalization \\server\share becomes //server/share.
        // These must not silently route to network locations.
        if Self::looks_like_network_path(&cleaned) {
            tracing::warn!(
                "Rejecting network/UNC path: {:?} — rewritten to local absolute path",
                cleaned
            );
            // Strip the leading extra slash so //server/share becomes /server/share (local absolute)
            return Self {
                inner: cleaned[1..].to_string(),
            };
        }

        Self { inner: cleaned }
    }

    /// Check if a cleaned path string looks like a network path (//host/share).
    fn looks_like_network_path(path: &str) -> bool {
        path.starts_with("//") && !path.starts_with("///")
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
        let cleaned = Self::clean(&joined);

        // Reject network/UNC paths that could result from joining
        if Self::looks_like_network_path(&cleaned) {
            return Self {
                inner: cleaned[1..].to_string(),
            };
        }

        Self { inner: cleaned }
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
        // Detect UNC-like double slash to preserve during clean; the caller (new/join)
        // is responsible for rejecting the resulting path if it is a network path.
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
    /// After normalization, backslashes are converted to forward slashes,
    /// so `\\server\share` becomes `//server/share`. This method detects
    /// all forms. Note: `NormalizedPath::new()` actively rejects UNC paths,
    /// so this should not return true for well-constructed paths.
    pub fn is_network_path(&self) -> bool {
        // After normalization, backslashes are already forward slashes,
        // so the `\\` check is unreachable — but kept for defense-in-depth.
        self.inner.starts_with("//")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_forward_slashes() {
        let path = NormalizedPath::new("foo/bar/baz");
        assert_eq!(path.as_str(), "foo/bar/baz");
    }

    #[test]
    fn test_normalize_backslashes_to_forward() {
        let path = NormalizedPath::new("foo\\bar\\baz");
        assert_eq!(path.as_str(), "foo/bar/baz");
    }

    #[test]
    fn test_normalize_mixed_slashes() {
        let path = NormalizedPath::new("foo/bar\\baz");
        assert_eq!(path.as_str(), "foo/bar/baz");
    }

    #[test]
    fn test_join_paths() {
        let base = NormalizedPath::new("foo/bar");
        let joined = base.join("baz");
        assert_eq!(joined.as_str(), "foo/bar/baz");
    }

    #[test]
    fn test_to_native_returns_pathbuf() {
        let path = NormalizedPath::new("foo/bar");
        let native = path.to_native();
        assert!(native.to_string_lossy().contains("bar"));
    }

    #[test]
    fn test_unc_path_rewritten_to_local() {
        // UNC paths should be rewritten to local paths (strip leading //)
        let path = NormalizedPath::new("//server/share/path");
        assert!(
            !path.is_network_path(),
            "UNC paths should be rejected at construction"
        );
        assert_eq!(path.as_str(), "/server/share/path");
    }

    #[test]
    fn test_backslash_unc_rewritten_to_local() {
        // Windows-style UNC \\server\share should also be rewritten
        let path = NormalizedPath::new("\\\\server\\share\\path");
        assert!(!path.is_network_path());
        assert_eq!(path.as_str(), "/server/share/path");
    }

    #[test]
    fn test_is_network_path_local() {
        let path = NormalizedPath::new("/home/user/project");
        assert!(!path.is_network_path());
    }

    #[test]
    fn test_parent() {
        let path = NormalizedPath::new("foo/bar/baz");
        let parent = path.parent().unwrap();
        assert_eq!(parent.as_str(), "foo/bar");
    }

    #[test]
    fn test_file_name() {
        let path = NormalizedPath::new("foo/bar/baz.txt");
        assert_eq!(path.file_name(), Some("baz.txt"));
    }

    #[test]
    fn test_exists_false_for_nonexistent() {
        let path = NormalizedPath::new("/nonexistent/path/that/does/not/exist");
        assert!(!path.exists());
    }
}
