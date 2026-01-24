# Architecture & Network Robustness Assessment

**Date**: 2026-01-23

## 1. Architecture Agnosticism

**Status**: **High**

* **Path Handling**: `NormalizedPath` is the MVP here. It correctly abstracts Windows (`\`) vs Unix (`/`) separators, normalizing everything to forward slashes internally. This ensures logic written on Windows runs correctly on Linux/macOS and vice versa.
* **Git Compatibility**: The `dunce` crate is used for canonicalization, which handles Windows UNC path quirks effectively (stripping `\\?\` prefixes that confuse some tools).
* **IO Primitives**: Uses `std::fs` and `std::path`, which are standard cross-platform abstractions.

## 2. Network Drive Robustness

**Status**: **Medium / Risky**

Network drives (SMB/CIFS/NFS) introduce specific failure modes that `repo-fs` partially addresses but leaves some risks:

### A. UNC Paths (Windows)

* **Good**: `NormalizedPath` correctly identifies and preserves UNC paths (`//server/share`).
* **Risk**: `std::fs` operations usually work fine, but some older tools or specific Git versions might struggle depending on how strictly they parse paths. `repo-fs` itself handles them well.

### B. File Locking (`fs2`)

* **Risk (High)**: `src/io.rs` uses `lock_exclusive()`.
  * **NFS**: Advisory locking on NFS is historically flaky. If the lock daemon (`lockd`) is not running or network partitions occur, this call can hang indefinitely or fail mysteriously.
  * **SMB**: Usually works (Oplocks), but can be slow.
* **Mitigation needed**: We likely need a timeout mechanism for locking, rather than blocking indefinitely (which `fs2::lock_exclusive` often does on Unix).

### C. Performance & Durability

* **Risk**: `write_atomic` calls `temp_file.sync_all()`.
  * **Impact**: On network drives, `fsync` forces a round-trip to the server to flush to disk. This is *extremely* slow (10ms-100ms+ per write). For a repository manager doing many small writes, this will be a bottleneck.
* **Mitigation needed**: Consider a configuration to disable strict syncing (`sync_all`) for network paths if performance is critical, accepting slightly higher data loss risk on power failure.

### D. Atomic Rename

* **Risk**: `write_atomic` relies on `fs::rename`.
  * **Issue**: `rename` ensures atomicity. However, if the "temp" file and "target" file end up on different mount points (unlikely for `write_atomic` as it uses the same parent dir), it would fail.
  * **NFS**: `rename` is atomic, but caching can lead to "Stale file handle" errors or TOCTOU issues on read-after-write from other clients.

## 3. Complex Error Handling

**Status**: **Basic**

* **Current State**: Errors are wrapped in `Error::Io`.
* **Gap**: No distinction between **Transient Errors** (network glitch, locked file) and **Permanent Errors** (permission denied, disk full).
* **Recommendation**:
  * Implement **Retry Logic** for transient errors (e.g., `Interrupted`, `TimedOut`, `WouldBlock`).
  * Add **Timeout** support for IO operations to preventing hanging on disconnected network shares.

## 4. Recommendations

1. **Implement Retry Logic**: Add a helper, potentially using the `backoff` crate, to retry operations like `lock_exclusive` or `rename` on transient failures.
2. **Add Lock Timeouts**: Wrap locking calls in a timeout (e.g., attempt lock for 5s, then fail).
3. **Configurable Robustness**: Add a `RobustnessConfig` struct passed to IO methods:
    * `enable_fsync: bool` (disable for speed on network drives?)
    * `lock_timeout_ms: u64`
