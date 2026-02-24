//! Version constraint parsing and checking.
//!
//! Supports two constraint formats:
//!
//! - **Semver ranges** for extension versions (e.g., `>=1.0.0,<2.0.0`)
//! - **PEP 440-style** constraints for Python versions (e.g., `>=3.12`, `>=3.10,<3.13`)
//!
//! PEP 440 constraints are mapped to semver by treating the Python version
//! as `major.minor.patch` (where patch defaults to 0 if omitted).
//!
//! # Examples
//!
//! ```
//! use repo_extensions::version::VersionConstraint;
//!
//! // Python-style constraint
//! let constraint = VersionConstraint::parse(">=3.12").unwrap();
//! assert!(constraint.satisfies("3.12.0"));
//! assert!(constraint.satisfies("3.13.1"));
//! assert!(!constraint.satisfies("3.11.5"));
//!
//! // Compound constraint
//! let constraint = VersionConstraint::parse(">=3.10,<3.13").unwrap();
//! assert!(constraint.satisfies("3.12.0"));
//! assert!(!constraint.satisfies("3.13.0"));
//! ```

use crate::error::{Error, Result};

/// A single version comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareOp {
    /// `>=`
    Gte,
    /// `>`
    Gt,
    /// `<=`
    Lte,
    /// `<`
    Lt,
    /// `==`
    Eq,
    /// `!=`
    Ne,
}

/// A single version specifier: an operator paired with a version.
#[derive(Debug, Clone)]
struct Specifier {
    op: CompareOp,
    version: semver::Version,
}

impl Specifier {
    fn matches(&self, candidate: &semver::Version) -> bool {
        match self.op {
            CompareOp::Gte => candidate >= &self.version,
            CompareOp::Gt => candidate > &self.version,
            CompareOp::Lte => candidate <= &self.version,
            CompareOp::Lt => candidate < &self.version,
            CompareOp::Eq => candidate == &self.version,
            CompareOp::Ne => candidate != &self.version,
        }
    }
}

/// A parsed version constraint that can be checked against concrete versions.
///
/// Supports comma-separated compound constraints (all must match).
#[derive(Debug, Clone)]
pub struct VersionConstraint {
    specifiers: Vec<Specifier>,
    /// The original constraint string for display.
    raw: String,
}

impl VersionConstraint {
    /// Parse a version constraint string.
    ///
    /// Supports PEP 440-style syntax:
    /// - `>=3.12`
    /// - `>=3.10,<3.13`
    /// - `==3.12.0`
    /// - `!=3.11`
    ///
    /// Version components can be `major.minor` (patch defaults to 0) or
    /// `major.minor.patch`.
    pub fn parse(constraint: &str) -> Result<Self> {
        let raw = constraint.to_string();
        let parts: Vec<&str> = constraint.split(',').map(|s| s.trim()).collect();
        let mut specifiers = Vec::with_capacity(parts.len());

        for part in parts {
            if part.is_empty() {
                continue;
            }
            specifiers.push(parse_specifier(part)?);
        }

        if specifiers.is_empty() {
            return Err(Error::VersionConstraintParse {
                constraint: raw,
                reason: "empty constraint".to_string(),
            });
        }

        Ok(Self { specifiers, raw })
    }

    /// Check if a version string satisfies this constraint.
    ///
    /// The version string can be `major.minor` or `major.minor.patch`.
    /// Returns `false` if the version string cannot be parsed.
    pub fn satisfies(&self, version: &str) -> bool {
        let parsed = match normalize_version(version) {
            Ok(v) => v,
            Err(_) => return false,
        };

        self.specifiers.iter().all(|spec| spec.matches(&parsed))
    }

    /// Check if a `semver::Version` satisfies this constraint.
    pub fn satisfies_version(&self, version: &semver::Version) -> bool {
        self.specifiers.iter().all(|spec| spec.matches(version))
    }

    /// Return the original constraint string.
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw)
    }
}

/// Parse a single specifier like `>=3.12` or `<3.13.0`.
fn parse_specifier(s: &str) -> Result<Specifier> {
    let (op, version_str) = if let Some(rest) = s.strip_prefix(">=") {
        (CompareOp::Gte, rest)
    } else if let Some(rest) = s.strip_prefix("<=") {
        (CompareOp::Lte, rest)
    } else if let Some(rest) = s.strip_prefix("!=") {
        (CompareOp::Ne, rest)
    } else if let Some(rest) = s.strip_prefix("==") {
        (CompareOp::Eq, rest)
    } else if let Some(rest) = s.strip_prefix('>') {
        (CompareOp::Gt, rest)
    } else if let Some(rest) = s.strip_prefix('<') {
        (CompareOp::Lt, rest)
    } else {
        // Bare version implies ==
        (CompareOp::Eq, s)
    };

    let version_str = version_str.trim();
    let version = normalize_version(version_str).map_err(|_| Error::VersionConstraintParse {
        constraint: s.to_string(),
        reason: format!("invalid version: {version_str}"),
    })?;

    Ok(Specifier { op, version })
}

/// Normalize a version string to semver by appending `.0` for missing patch.
///
/// - `"3.12"` -> `"3.12.0"`
/// - `"3.12.1"` -> `"3.12.1"`
/// - `"3"` -> error
fn normalize_version(s: &str) -> std::result::Result<semver::Version, String> {
    let s = s.trim();

    // Try direct parse first
    if let Ok(v) = semver::Version::parse(s) {
        return Ok(v);
    }

    // Try appending .0 for major.minor format
    let with_patch = format!("{s}.0");
    semver::Version::parse(&with_patch).map_err(|e| format!("invalid version '{s}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- VersionConstraint::parse ---

    #[test]
    fn test_parse_gte() {
        let c = VersionConstraint::parse(">=3.12").unwrap();
        assert_eq!(c.specifiers.len(), 1);
        assert_eq!(c.as_str(), ">=3.12");
    }

    #[test]
    fn test_parse_compound() {
        let c = VersionConstraint::parse(">=3.10,<3.13").unwrap();
        assert_eq!(c.specifiers.len(), 2);
    }

    #[test]
    fn test_parse_eq() {
        let c = VersionConstraint::parse("==3.12.0").unwrap();
        assert_eq!(c.specifiers.len(), 1);
    }

    #[test]
    fn test_parse_bare_version() {
        let c = VersionConstraint::parse("3.12.0").unwrap();
        assert_eq!(c.specifiers.len(), 1);
    }

    #[test]
    fn test_parse_empty_rejected() {
        assert!(VersionConstraint::parse("").is_err());
    }

    #[test]
    fn test_parse_garbage_rejected() {
        assert!(VersionConstraint::parse(">=abc").is_err());
    }

    // --- satisfies ---

    #[test]
    fn test_satisfies_gte() {
        let c = VersionConstraint::parse(">=3.12").unwrap();
        assert!(c.satisfies("3.12.0"));
        assert!(c.satisfies("3.13.0"));
        assert!(c.satisfies("4.0.0"));
        assert!(!c.satisfies("3.11.9"));
    }

    #[test]
    fn test_satisfies_lt() {
        let c = VersionConstraint::parse("<3.13").unwrap();
        assert!(c.satisfies("3.12.9"));
        assert!(!c.satisfies("3.13.0"));
        assert!(!c.satisfies("3.14.0"));
    }

    #[test]
    fn test_satisfies_compound() {
        let c = VersionConstraint::parse(">=3.10,<3.13").unwrap();
        assert!(c.satisfies("3.10.0"));
        assert!(c.satisfies("3.12.5"));
        assert!(!c.satisfies("3.9.0"));
        assert!(!c.satisfies("3.13.0"));
    }

    #[test]
    fn test_satisfies_eq() {
        let c = VersionConstraint::parse("==3.12.0").unwrap();
        assert!(c.satisfies("3.12.0"));
        assert!(!c.satisfies("3.12.1"));
    }

    #[test]
    fn test_satisfies_ne() {
        let c = VersionConstraint::parse("!=3.11.0").unwrap();
        assert!(c.satisfies("3.12.0"));
        assert!(!c.satisfies("3.11.0"));
    }

    #[test]
    fn test_satisfies_two_part_version() {
        let c = VersionConstraint::parse(">=3.12").unwrap();
        // "3.13" -> "3.13.0"
        assert!(c.satisfies("3.13"));
    }

    #[test]
    fn test_satisfies_invalid_version_returns_false() {
        let c = VersionConstraint::parse(">=3.12").unwrap();
        assert!(!c.satisfies("not-a-version"));
    }

    // --- normalize_version ---

    #[test]
    fn test_normalize_three_part() {
        let v = normalize_version("3.12.1").unwrap();
        assert_eq!(v, semver::Version::new(3, 12, 1));
    }

    #[test]
    fn test_normalize_two_part() {
        let v = normalize_version("3.12").unwrap();
        assert_eq!(v, semver::Version::new(3, 12, 0));
    }

    #[test]
    fn test_normalize_whitespace() {
        let v = normalize_version("  3.12.0  ").unwrap();
        assert_eq!(v, semver::Version::new(3, 12, 0));
    }

    // --- Display ---

    #[test]
    fn test_display() {
        let c = VersionConstraint::parse(">=3.10,<3.13").unwrap();
        assert_eq!(format!("{c}"), ">=3.10,<3.13");
    }

    // --- Real-world Python constraint scenarios ---

    #[test]
    fn test_python_312_satisfies_gte_313() {
        // This is the exact gap: Python 3.12 should NOT satisfy >=3.13
        let c = VersionConstraint::parse(">=3.13").unwrap();
        assert!(!c.satisfies("3.12.0"), "Python 3.12 must not satisfy >=3.13");
        assert!(c.satisfies("3.13.0"));
        assert!(c.satisfies("3.14.0"));
    }

    #[test]
    fn test_rust_semver_constraint() {
        let c = VersionConstraint::parse(">=1.75.0,<2.0.0").unwrap();
        assert!(c.satisfies("1.80.0"));
        assert!(!c.satisfies("1.74.0"));
        assert!(!c.satisfies("2.0.0"));
    }
}
