# repo-blocks Crate Audit - 2026-01-28

## Executive Summary

The `repo-blocks` crate provides block parsing and writing functionality for managing UUID-tagged content blocks within configuration files (JSON, TOML, YAML). Overall, the crate demonstrates solid security practices with no critical vulnerabilities identified. However, several areas warrant attention:

**Risk Level: LOW to MEDIUM**

| Category | Finding Count | Severity |
|----------|--------------|----------|
| Security | 2 | Low |
| Performance | 3 | Low-Medium |
| Memory Safety | 1 | Low |
| Error Handling | 3 | Low |

Key strengths:
- No `unsafe` code blocks
- Proper use of established parsing libraries (serde_json, toml)
- Good error type design with thiserror
- Static regex compilation with `LazyLock`

Areas for improvement:
- Production code contains `expect()` and `unwrap()` calls that could panic
- Dynamic regex compilation in write/remove operations
- No input size limits for large file handling
- Inconsistent UUID validation across format handlers

## Crate Overview

### Purpose
The crate handles parsing and writing of managed blocks in structured document formats. It supports three block marker formats:

1. **HTML-style markers** (parser.rs, writer.rs): `<!-- repo:block:UUID -->` / `<!-- /repo:block:UUID -->`
2. **JSON**: Reserved `__repo_managed__` key with UUID sub-objects
3. **TOML**: Reserved `[repo_managed]` table with UUID sub-tables
4. **YAML**: Comment-based markers `# repo:block:UUID` / `# /repo:block:UUID`

### File Structure
```
src/
  lib.rs          - Public API exports
  error.rs        - Error types using thiserror
  parser.rs       - Block parsing with regex
  writer.rs       - Block writing/updating/removal
  formats/
    mod.rs        - FormatHandler trait definition
    json.rs       - JSON format handler
    toml.rs       - TOML format handler
    yaml.rs       - YAML format handler
```

### Dependencies
- `regex` - Regular expression matching
- `serde_json` - JSON parsing/serialization
- `toml` - TOML parsing/serialization
- `uuid` - UUID type and parsing
- `thiserror` - Error derive macros
- `repo-fs` - Filesystem operations (internal crate)

## Findings

### Security

#### SEC-01: Regex Patterns - No ReDoS Vulnerability (Severity: Low)

**Location**: `src/parser.rs:28-29`, `src/formats/yaml.rs:22-23`

The static regex patterns are simple and linear:

```rust
// parser.rs
r"<!-- repo:block:([a-zA-Z0-9_-]+) -->"

// yaml.rs
r"# repo:block:([0-9a-fA-F-]+)"
```

**Analysis**: These patterns contain no nested quantifiers, alternations with overlapping prefixes, or other ReDoS-vulnerable constructs. The character classes `[a-zA-Z0-9_-]` and `[0-9a-fA-F-]` are bounded and match greedily without backtracking concerns.

**Verdict**: Safe. No ReDoS vulnerability present.

#### SEC-02: Dynamic Regex with User-Controlled UUID (Severity: Low)

**Location**: `src/writer.rs:96-101`, `src/writer.rs:143-148`, `src/formats/yaml.rs:93-98`, `src/formats/yaml.rs:118-123`

Dynamic regex patterns are constructed using UUIDs:

```rust
// writer.rs:96-101
let pattern = format!(
    r"(?s)<!-- repo:block:{} -->\n.*?\n<!-- /repo:block:{} -->",
    regex::escape(uuid),
    regex::escape(uuid)
);
```

**Analysis**: The code properly uses `regex::escape()` to sanitize the UUID before embedding it in the pattern. This prevents regex injection attacks. The `.*?` quantifier with `(?s)` flag is non-greedy and bounded by the closing marker, presenting no ReDoS risk.

**Verdict**: Safe. Proper escaping is applied.

### Performance

#### PERF-01: Repeated Full Parsing in Find Operations (Severity: Low)

**Location**: `src/parser.rs:114-118`

```rust
pub fn find_block(content: &str, uuid: &str) -> Option<Block> {
    parse_blocks(content)
        .into_iter()
        .find(|block| block.uuid == uuid)
}
```

**Issue**: `find_block` parses ALL blocks even when searching for a specific UUID. For files with many blocks, this is inefficient.

**Recommendation**: Consider early-exit parsing that stops when the target UUID is found, or maintain an index for frequent lookups.

#### PERF-02: Dynamic Regex Compilation Per Operation (Severity: Medium)

**Location**: `src/writer.rs:101`, `src/writer.rs:148`, `src/formats/yaml.rs:98`, `src/formats/yaml.rs:123`

```rust
let re = Regex::new(&pattern)?;  // Compiled every call
```

**Issue**: Regex compilation occurs on every `update_block`, `remove_block`, and YAML write/remove operation. Regex compilation is relatively expensive (~microseconds).

**Recommendation**: For high-throughput scenarios, consider caching compiled regexes or using a regex pool keyed by UUID.

#### PERF-03: No Input Size Limits (Severity: Medium)

**Location**: All parsing functions

**Issue**: No limits on input content size. Processing very large files (e.g., 100MB+) will:
- Allocate significant memory for string operations
- Perform full regex scans over the entire content
- Create large intermediate vectors

**Recommendation**: Consider adding configurable size limits or streaming parsing for very large files.

### Memory Safety

#### MEM-01: String Allocations in Parsing (Severity: Low)

**Location**: `src/parser.rs:75`, `src/formats/yaml.rs:68-74`

```rust
let block_content = trimmed.strip_suffix('\n').unwrap_or(trimmed).to_string();
```

**Issue**: Each block's content is cloned into a new `String`. For content with many large blocks, this creates significant memory pressure.

**Analysis**: This is standard Rust practice and safe. The owned strings ensure the `Block` struct is self-contained and doesn't require lifetime annotations. The trade-off is acceptable for the typical use case (configuration files).

**Verdict**: Acceptable. No memory safety issue, just a performance consideration.

### Error Handling

#### ERR-01: Production Code Panics via expect() (Severity: Medium)

**Location**: `src/parser.rs:29`, `src/formats/yaml.rs:23`

```rust
static OPEN_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<!-- repo:block:([a-zA-Z0-9_-]+) -->").expect("Invalid open marker regex")
});
```

**Analysis**: These are static regex compilations at startup. The patterns are compile-time constants and have been tested. A panic here indicates a programming error in the regex pattern itself, which is acceptable since it's caught during development.

**Verdict**: Acceptable for static initialization.

#### ERR-02: Production Code Panic via expect() in JSON Handler (Severity: Medium)

**Location**: `src/formats/json.rs:73`

```rust
let obj = json.as_object_mut().expect("Root must be object");
```

**Issue**: This `expect()` can panic in production code if the JSON root is not an object. The preceding code creates a fallback `Value::Object(Map::new())` if parsing fails, but if existing content parses successfully to a non-object (e.g., an array `[]`), this will panic.

**Attack Vector**: Providing a JSON file with an array root `[1, 2, 3]` would cause a panic.

**Recommendation**: Return an error or handle non-object roots gracefully:
```rust
let Some(obj) = json.as_object_mut() else {
    return content.to_string(); // or return an error
};
```

#### ERR-03: unwrap() on Regex Match Groups (Severity: Low)

**Location**: `src/parser.rs:58-59`, `src/formats/yaml.rs:52-57`

```rust
let uuid = open_caps.get(1).unwrap().as_str();
let open_match = open_caps.get(0).unwrap();
```

**Analysis**: These unwraps are safe because:
1. `get(0)` always succeeds on a successful capture
2. `get(1)` succeeds because the regex defines a capture group at position 1

**Verdict**: Safe. The unwraps are logically guaranteed by the regex structure.

#### ERR-04: Inconsistent Error Handling in Format Handlers (Severity: Low)

**Location**: `src/formats/json.rs`, `src/formats/toml.rs`, `src/formats/yaml.rs`

**Issue**: The `FormatHandler` trait methods return `String` directly, not `Result<String, Error>`. This means errors (like invalid input) silently fall back to returning the original content unchanged.

```rust
// json.rs:65
serde_json::from_str(content).unwrap_or(Value::Object(Map::new()))

// toml.rs:67
content.parse().unwrap_or_default()
```

**Impact**: Users have no way to know if an operation actually succeeded or silently failed.

**Recommendation**: Consider changing the `FormatHandler` trait to return `Result` types for write/remove operations.

### Additional Observations

#### OBS-01: UUID Validation Inconsistency

The parser.rs accepts any alphanumeric string with hyphens/underscores as a UUID:
```rust
r"<!-- repo:block:([a-zA-Z0-9_-]+) -->"
```

But yaml.rs only accepts hex characters and hyphens:
```rust
r"# repo:block:([0-9a-fA-F-]+)"
```

And the JSON/TOML handlers use the strict `uuid` crate for parsing:
```rust
Uuid::parse_str(key).ok()?
```

**Impact**: A block with UUID `my-custom-id` would work with HTML markers but not with YAML format, and would fail validation in JSON/TOML handlers.

#### OBS-02: No Concurrent Access Protection

The crate operates on string content and does not handle file locking or concurrent modifications. This is expected for a parsing library, but consumers should be aware.

#### OBS-03: Well-Structured Error Types

The error.rs file demonstrates good practices:
- Uses `thiserror` for derive macros
- Provides specific error variants for different failure modes
- Includes context (path, uuid) in error messages

## Recommendations

### High Priority

1. **Fix ERR-02**: Replace `expect("Root must be object")` with graceful error handling to prevent panics on malformed JSON input.

2. **Consider input size limits**: Add optional size limits to prevent resource exhaustion on extremely large inputs.

### Medium Priority

3. **Standardize UUID validation**: Use consistent UUID format validation across all format handlers.

4. **Add Result return types to FormatHandler**: Allow callers to distinguish between successful operations and silent failures.

### Low Priority

5. **Cache dynamic regexes**: For performance-critical paths, consider caching compiled regexes.

6. **Document concurrent access limitations**: Clarify in documentation that consumers must handle file locking externally.

## Test Coverage Assessment

The crate has good test coverage including:
- Empty content handling
- Single and multiple block parsing
- Block insertion, update, removal, and upsert
- Content preservation around blocks
- Special characters in content
- Line position tracking

Missing test cases:
- Extremely large input handling
- Malformed/partial block markers
- Non-object JSON root handling (would reveal ERR-02)
- Unicode in UUIDs and content
- Deeply nested block content

## Conclusion

The `repo-blocks` crate is well-designed with appropriate use of Rust's safety features. No critical security vulnerabilities were identified. The main concerns are:

1. One potential panic path in the JSON handler (ERR-02)
2. Performance considerations for high-throughput or large-file scenarios
3. Inconsistent UUID validation across formats

The crate is suitable for production use with the recommendation to address ERR-02 before deployment in untrusted-input scenarios.

---

**Auditor**: Claude Opus 4.5
**Date**: 2026-01-28
**Crate Version**: Workspace version (see Cargo.toml)
**Audit Scope**: Security, Performance, Memory Safety, Error Handling
