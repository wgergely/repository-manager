//! Writes projections to the filesystem
//!
//! Uses symlink-safe write operations to prevent path traversal attacks.

use crate::ledger::{Projection, ProjectionKind};
use crate::{Error, Result};
use repo_fs::NormalizedPath;
use sha2::{Digest, Sha256};
use std::fs;
use uuid::Uuid;

/// Write content to a file safely (with symlink protection)
fn safe_write(path: &NormalizedPath, content: &str) -> Result<()> {
    repo_fs::io::write_text(path, content).map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
}

/// Writes projections to filesystem
pub struct ProjectionWriter {
    root: NormalizedPath,
    dry_run: bool,
}

impl ProjectionWriter {
    pub fn new(root: NormalizedPath, dry_run: bool) -> Self {
        Self { root, dry_run }
    }

    /// Apply a projection to the filesystem
    pub fn apply(&self, projection: &Projection, content: &str) -> Result<String> {
        let file_path = self.root.join(projection.file.to_string_lossy().as_ref());

        match &projection.kind {
            ProjectionKind::FileManaged { .. } => self.write_managed_file(&file_path, content),
            ProjectionKind::TextBlock { marker, .. } => {
                self.write_text_block(&file_path, *marker, content)
            }
            ProjectionKind::JsonKey { path, .. } => self.write_json_key(&file_path, path, content),
        }
    }

    /// Remove a projection from the filesystem
    pub fn remove(&self, projection: &Projection) -> Result<String> {
        let file_path = self.root.join(projection.file.to_string_lossy().as_ref());

        match &projection.kind {
            ProjectionKind::FileManaged { .. } => self.remove_managed_file(&file_path),
            ProjectionKind::TextBlock { marker, .. } => {
                self.remove_text_block(&file_path, *marker)
            }
            ProjectionKind::JsonKey { path, .. } => self.remove_json_key(&file_path, path),
        }
    }

    fn write_managed_file(&self, path: &NormalizedPath, content: &str) -> Result<String> {
        if self.dry_run {
            return Ok(format!("[dry-run] Would create {}", path));
        }

        safe_write(path, content)?;
        Ok(format!("Created {}", path))
    }

    fn write_text_block(
        &self,
        path: &NormalizedPath,
        marker: Uuid,
        content: &str,
    ) -> Result<String> {
        let existing = if path.exists() {
            fs::read_to_string(path.as_ref())?
        } else {
            String::new()
        };

        let marker_start = format!("<!-- repo:block:{} -->", marker);
        let marker_end = format!("<!-- /repo:block:{} -->", marker);

        let block_content = format!("{}\n{}\n{}", marker_start, content, marker_end);

        let new_content = if existing.contains(&marker_start) {
            // Replace existing block
            let start_idx = existing.find(&marker_start)
                .ok_or_else(|| Error::InternalError {
                    message: format!("marker_start not found despite contains() check: {}", marker_start),
                })?;
            let end_idx = existing
                .find(&marker_end)
                .map(|i| i + marker_end.len())
                .ok_or_else(|| Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Malformed text block: start marker found but end marker missing for {}", marker),
                )))?;
            format!(
                "{}{}{}",
                &existing[..start_idx],
                block_content,
                &existing[end_idx..]
            )
        } else {
            // Append new block
            if existing.is_empty() {
                block_content
            } else {
                format!("{}\n\n{}", existing.trim_end(), block_content)
            }
        };

        if self.dry_run {
            return Ok(format!("[dry-run] Would update block {} in {}", marker, path));
        }

        safe_write(path, &new_content)?;
        Ok(format!("Updated block {} in {}", marker, path))
    }

    fn write_json_key(
        &self,
        path: &NormalizedPath,
        key_path: &str,
        value: &str,
    ) -> Result<String> {
        let existing = if path.exists() {
            fs::read_to_string(path.as_ref())?
        } else {
            "{}".to_string()
        };

        let mut json: serde_json::Value = serde_json::from_str(&existing).map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;

        let value: serde_json::Value = serde_json::from_str(value).map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;

        set_json_path(&mut json, key_path, value);

        if self.dry_run {
            return Ok(format!("[dry-run] Would set {} in {}", key_path, path));
        }

        let output = serde_json::to_string_pretty(&json)?;
        safe_write(path, &output)?;
        Ok(format!("Set {} in {}", key_path, path))
    }

    fn remove_managed_file(&self, path: &NormalizedPath) -> Result<String> {
        if self.dry_run {
            return Ok(format!("[dry-run] Would delete {}", path));
        }

        if path.exists() {
            fs::remove_file(path.as_ref())?;
            Ok(format!("Deleted {}", path))
        } else {
            Ok(format!("File already missing: {}", path))
        }
    }

    fn remove_text_block(&self, path: &NormalizedPath, marker: Uuid) -> Result<String> {
        if !path.exists() {
            return Ok(format!("File already missing: {}", path));
        }

        let existing = fs::read_to_string(path.as_ref())?;
        let marker_start = format!("<!-- repo:block:{} -->", marker);
        let marker_end = format!("<!-- /repo:block:{} -->", marker);

        if !existing.contains(&marker_start) {
            return Ok(format!("Block {} not found in {}", marker, path));
        }

        let start_idx = existing.find(&marker_start)
            .ok_or_else(|| Error::InternalError {
                message: format!("marker_start not found despite contains() check: {}", marker_start),
            })?;
        let end_idx = existing
            .find(&marker_end)
            .map(|i| i + marker_end.len())
            .ok_or_else(|| Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Malformed text block: start marker found but end marker missing for {}", marker),
            )))?;

        let new_content = format!("{}{}", &existing[..start_idx], &existing[end_idx..])
            .trim()
            .to_string();

        if self.dry_run {
            return Ok(format!(
                "[dry-run] Would remove block {} from {}",
                marker, path
            ));
        }

        safe_write(path, &new_content)?;
        Ok(format!("Removed block {} from {}", marker, path))
    }

    fn remove_json_key(&self, path: &NormalizedPath, key_path: &str) -> Result<String> {
        if !path.exists() {
            return Ok(format!("File already missing: {}", path));
        }

        let existing = fs::read_to_string(path.as_ref())?;
        let mut json: serde_json::Value = serde_json::from_str(&existing).map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;

        remove_json_path(&mut json, key_path);

        if self.dry_run {
            return Ok(format!("[dry-run] Would remove {} from {}", key_path, path));
        }

        let output = serde_json::to_string_pretty(&json)?;
        safe_write(path, &output)?;
        Ok(format!("Removed {} from {}", key_path, path))
    }
}

/// Set a value at a JSON path (e.g., "editor.fontSize")
fn set_json_path(json: &mut serde_json::Value, path: &str, value: serde_json::Value) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let serde_json::Value::Object(map) = current {
                map.insert(part.to_string(), value);
            }
            return;
        }

        if let serde_json::Value::Object(map) = current {
            current = map
                .entry(part.to_string())
                .or_insert(serde_json::Value::Object(serde_json::Map::new()));
        }
    }
}

/// Remove a key at a JSON path
fn remove_json_path(json: &mut serde_json::Value, path: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return;
    }

    let mut current = json;
    for part in &parts[..parts.len() - 1] {
        if let serde_json::Value::Object(map) = current {
            if let Some(next) = map.get_mut(*part) {
                current = next;
            } else {
                return;
            }
        } else {
            return;
        }
    }

    if let serde_json::Value::Object(map) = current
        && let Some(last_part) = parts.last()
    {
        map.remove(*last_part);
    }
}

/// Compute checksum of content
pub fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_checksum() {
        let checksum = compute_checksum("hello world");
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_set_json_path() {
        let mut json = serde_json::json!({});
        set_json_path(&mut json, "editor.fontSize", serde_json::json!(14));
        assert_eq!(json["editor"]["fontSize"], 14);
    }

    #[test]
    fn test_remove_json_path() {
        let mut json = serde_json::json!({"editor": {"fontSize": 14, "tabSize": 2}});
        remove_json_path(&mut json, "editor.fontSize");
        assert!(json["editor"]["fontSize"].is_null());
        assert_eq!(json["editor"]["tabSize"], 2);
    }
}
