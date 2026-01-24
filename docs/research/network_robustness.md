# Research: Network Robustness Patterns

**Date**: 2026-01-23

## Problem Statement

Network drives (SMB/NFS) present unique challenges:

1. **Latency**: Operations like `fsync` are orders of magnitude slower.
2. **Unreliability**: Locks may stall; connections may drop temporarily.
3. **Concurrency**: Distributed locking is brittle.

## Proposed Patterns & Mitigations

### 1. Lock Timeouts (Simulation)

The `fs2` crate's `lock_exclusive` blocks indefinitely. On a network drive, if a client dies holding a lock, other clients hang forever.
**Pattern**: Use `try_lock_exclusive` in a loop with exponential backoff until a deadline (e.g., 10 seconds).

### 2. Configurable Durability

Strict durability (`File::sync_all`) is expensive on network drives (round-trip required).
**Pattern**: specific `RobustnessConfig` struct.

- `sync_mode`: `Strict` (default) vs `Fast` (skip `sync_all`).
- `lock_timeout`: Duration.

### 3. Transient Error Retry

Network operations can fail with transient errors (`Target resource unavailable`, `Network busy`).
**Pattern**: Wrap critical I/O in a retry loop using the `backoff` crate. logic.

## Selected Stack

- **`backoff`**: For exponential backoff on retries.
- **`fs2`**: Continue handling cross-platform locking primitives.
- **`io::retry`**: Custom wrapper for retryable IO operations.
