//! Path parsing and traversal utilities
//!
//! This module provides utilities for navigating and modifying structured
//! documents using dot-separated paths with array indexing support.
//!
//! # Path Syntax
//!
//! - Dot-separated keys: `config.database.host`
//! - Array indexing: `items[0].name`
//! - Combined: `config.servers[0].host`
//!
//! # Examples
//!
//! ```
//! use repo_content::path::{parse_path, PathSegment, get_at_path};
//! use serde_json::json;
//!
//! let path = parse_path("config.database[0].host");
//! assert_eq!(path, vec![
//!     PathSegment::Key("config".to_string()),
//!     PathSegment::Key("database".to_string()),
//!     PathSegment::Index(0),
//!     PathSegment::Key("host".to_string()),
//! ]);
//!
//! let value = json!({"config": {"database": [{"host": "localhost"}]}});
//! assert_eq!(
//!     get_at_path(&value, &path),
//!     Some(json!("localhost"))
//! );
//! ```

use serde_json::Value;

/// A segment of a path - either a key or an array index
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    /// A key in an object (e.g., "database" in "config.database")
    Key(String),
    /// An index in an array (e.g., 0 in `items[0]`)
    Index(usize),
}

/// Parse a path string into segments.
///
/// Supports:
/// - Dot-separated keys: `config.database.host`
/// - Array indexing: `items[0].name`
/// - Combined: `config.servers[0].host`
///
/// # Examples
///
/// ```
/// use repo_content::path::{parse_path, PathSegment};
///
/// let path = parse_path("config.database.host");
/// assert_eq!(path, vec![
///     PathSegment::Key("config".to_string()),
///     PathSegment::Key("database".to_string()),
///     PathSegment::Key("host".to_string()),
/// ]);
///
/// let path = parse_path("items[0].name");
/// assert_eq!(path, vec![
///     PathSegment::Key("items".to_string()),
///     PathSegment::Index(0),
///     PathSegment::Key("name".to_string()),
/// ]);
/// ```
pub fn parse_path(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current_key = String::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current_key.is_empty() {
                    segments.push(PathSegment::Key(current_key.clone()));
                    current_key.clear();
                }
            }
            '[' => {
                // Push the key before the bracket if we have one
                if !current_key.is_empty() {
                    segments.push(PathSegment::Key(current_key.clone()));
                    current_key.clear();
                }
                // Parse the index
                let mut index_str = String::new();
                for ch in chars.by_ref() {
                    if ch == ']' {
                        break;
                    }
                    index_str.push(ch);
                }
                if let Ok(index) = index_str.parse::<usize>() {
                    segments.push(PathSegment::Index(index));
                }
            }
            _ => {
                current_key.push(ch);
            }
        }
    }

    // Don't forget the last key
    if !current_key.is_empty() {
        segments.push(PathSegment::Key(current_key));
    }

    segments
}

/// Get a value at the given path from a JSON value.
///
/// Returns `None` if the path doesn't exist.
///
/// # Examples
///
/// ```
/// use repo_content::path::{parse_path, get_at_path};
/// use serde_json::json;
///
/// let value = json!({"config": {"host": "localhost"}});
/// let path = parse_path("config.host");
/// assert_eq!(get_at_path(&value, &path), Some(json!("localhost")));
///
/// let path = parse_path("config.missing");
/// assert_eq!(get_at_path(&value, &path), None);
/// ```
pub fn get_at_path(value: &Value, segments: &[PathSegment]) -> Option<Value> {
    if segments.is_empty() {
        return Some(value.clone());
    }

    let (first, rest) = segments.split_first()?;

    let next_value = match first {
        PathSegment::Key(key) => value.get(key)?,
        PathSegment::Index(idx) => value.get(*idx)?,
    };

    get_at_path(next_value, rest)
}

/// Set a value at the given path in a JSON value.
///
/// Returns `true` if the path existed and was set, `false` otherwise.
/// This function will create intermediate objects/arrays if they don't exist
/// for object keys, but will fail for array indices that don't exist.
///
/// # Examples
///
/// ```
/// use repo_content::path::{parse_path, set_at_path, get_at_path};
/// use serde_json::json;
///
/// let mut value = json!({"config": {"host": "old"}});
/// let path = parse_path("config.host");
/// assert!(set_at_path(&mut value, &path, json!("new")));
/// assert_eq!(get_at_path(&value, &path), Some(json!("new")));
/// ```
pub fn set_at_path(value: &mut Value, segments: &[PathSegment], new_value: Value) -> bool {
    if segments.is_empty() {
        *value = new_value;
        return true;
    }

    if segments.len() == 1 {
        // We're at the parent, set the child
        return match &segments[0] {
            PathSegment::Key(key) => {
                if let Value::Object(map) = value {
                    map.insert(key.clone(), new_value);
                    true
                } else {
                    false
                }
            }
            PathSegment::Index(idx) => {
                if let Value::Array(arr) = value {
                    if *idx < arr.len() {
                        arr[*idx] = new_value;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };
    }

    let (first, rest) = segments.split_first().unwrap();

    let next_value = match first {
        PathSegment::Key(key) => {
            if let Value::Object(map) = value {
                map.get_mut(key)
            } else {
                None
            }
        }
        PathSegment::Index(idx) => {
            if let Value::Array(arr) = value {
                arr.get_mut(*idx)
            } else {
                None
            }
        }
    };

    match next_value {
        Some(v) => set_at_path(v, rest, new_value),
        None => false,
    }
}

/// Remove a value at the given path from a JSON value.
///
/// Returns the removed value if the path existed, `None` otherwise.
///
/// # Examples
///
/// ```
/// use repo_content::path::{parse_path, remove_at_path, get_at_path};
/// use serde_json::json;
///
/// let mut value = json!({"name": "test", "version": "1.0"});
/// let path = parse_path("version");
/// let removed = remove_at_path(&mut value, &path);
/// assert_eq!(removed, Some(json!("1.0")));
/// assert_eq!(get_at_path(&value, &path), None);
/// ```
pub fn remove_at_path(value: &mut Value, segments: &[PathSegment]) -> Option<Value> {
    if segments.is_empty() {
        return None;
    }

    if segments.len() == 1 {
        // We're at the parent, remove the child
        return match &segments[0] {
            PathSegment::Key(key) => {
                if let Value::Object(map) = value {
                    map.remove(key)
                } else {
                    None
                }
            }
            PathSegment::Index(idx) => {
                if let Value::Array(arr) = value {
                    if *idx < arr.len() {
                        Some(arr.remove(*idx))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
    }

    let (first, rest) = segments.split_first().unwrap();

    let next_value = match first {
        PathSegment::Key(key) => {
            if let Value::Object(map) = value {
                map.get_mut(key)
            } else {
                None
            }
        }
        PathSegment::Index(idx) => {
            if let Value::Array(arr) = value {
                arr.get_mut(*idx)
            } else {
                None
            }
        }
    };

    match next_value {
        Some(v) => remove_at_path(v, rest),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_path_simple() {
        let path = parse_path("name");
        assert_eq!(path, vec![PathSegment::Key("name".to_string())]);
    }

    #[test]
    fn test_parse_path_dotted() {
        let path = parse_path("config.database.host");
        assert_eq!(
            path,
            vec![
                PathSegment::Key("config".to_string()),
                PathSegment::Key("database".to_string()),
                PathSegment::Key("host".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_path_array_index() {
        let path = parse_path("items[0]");
        assert_eq!(
            path,
            vec![PathSegment::Key("items".to_string()), PathSegment::Index(0),]
        );
    }

    #[test]
    fn test_parse_path_mixed() {
        let path = parse_path("items[0].name");
        assert_eq!(
            path,
            vec![
                PathSegment::Key("items".to_string()),
                PathSegment::Index(0),
                PathSegment::Key("name".to_string()),
            ]
        );
    }

    #[test]
    fn test_get_at_path_simple() {
        let value = json!({"name": "test"});
        let path = parse_path("name");
        assert_eq!(get_at_path(&value, &path), Some(json!("test")));
    }

    #[test]
    fn test_get_at_path_nested() {
        let value = json!({"config": {"database": {"host": "localhost"}}});
        let path = parse_path("config.database.host");
        assert_eq!(get_at_path(&value, &path), Some(json!("localhost")));
    }

    #[test]
    fn test_get_at_path_array() {
        let value = json!({"items": [{"name": "first"}, {"name": "second"}]});
        let path = parse_path("items[0].name");
        assert_eq!(get_at_path(&value, &path), Some(json!("first")));
    }

    #[test]
    fn test_get_at_path_missing() {
        let value = json!({"name": "test"});
        let path = parse_path("missing");
        assert_eq!(get_at_path(&value, &path), None);
    }

    #[test]
    fn test_set_at_path() {
        let mut value = json!({"name": "old"});
        let path = parse_path("name");
        assert!(set_at_path(&mut value, &path, json!("new")));
        assert_eq!(value, json!({"name": "new"}));
    }

    #[test]
    fn test_remove_at_path() {
        let mut value = json!({"name": "test", "version": "1.0"});
        let path = parse_path("version");
        let removed = remove_at_path(&mut value, &path);
        assert_eq!(removed, Some(json!("1.0")));
        assert_eq!(value, json!({"name": "test"}));
    }
}
