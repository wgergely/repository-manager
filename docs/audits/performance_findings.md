# Performance Audit: `repo-fs` Crate

Date: 2026-01-23

## Executive Summary

A performance benchmark suite was added to the `repo-fs` crate to measure the performance of critical I/O and filesystem detection logic. Specifically, benchmarks were created for:

1.  `io::write_atomic`: Measures the performance of atomically writing a small file.
2.  `layout::WorkspaceLayout::detect`: Measures the performance of detecting a workspace root from a nested path, for both a success case (root found) and a failure case (root not found).

Due to a persistent error in the execution environment (`@lydell/node-pty` package failure), the benchmarks could not be executed to gather performance data. However, the benchmark code has been committed and can be run by any developer with a correctly configured Rust environment.

## Benchmark Setup

The benchmarks were created using the `criterion` crate (version `0.5`).

### How to Run Benchmarks

To run the benchmarks and gather fresh performance data, execute the following command from the root of the workspace:

```sh
cargo bench -p repo-fs
```

Criterion will run the benchmarks and output a detailed report to the console. It will also save detailed results in the `target/criterion` directory.

### Benchmark Implementation

The benchmark code is located in `crates/repo-fs/benches/fs_benchmarks.rs`.

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use repo_fs::io;
use repo_fs::layout::WorkspaceLayout;
use std::fs;
use tempfile::tempdir;

fn write_atomic_benchmark(c: &mut Criterion) {
    c.bench_function("io::write_atomic", |b| {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_file.txt");
        let content = "hello world".as_bytes();

        b.iter(|| {
            io::write_atomic(black_box(&path), black_box(content)).unwrap();
        })
    });
}

fn workspace_layout_detect_benchmark(c: &mut Criterion) {
    // Benchmark for when a valid workspace is found
    c.bench_function("layout::WorkspaceLayout::detect (found)", |b| {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/refs")).unwrap();
        fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main").unwrap();
        let start_path = dir.path().join("some/nested/dir");
        fs::create_dir_all(&start_path).unwrap();

        b.iter(|| {
            let _ = WorkspaceLayout::detect(black_box(start_path.clone())).unwrap();
        })
    });

    // Benchmark for when no workspace is found (searches up to the root)
    c.bench_function("layout::WorkspaceLayout::detect (not found)", |b| {
        let dir = tempdir().unwrap();
        let start_path = dir.path().join("some/nested/dir");
        fs::create_dir_all(&start_path).unwrap();

        b.iter(|| {
            let result = WorkspaceLayout::detect(black_box(start_path.clone()));
            assert!(result.is_err());
        })
    });
}

criterion_group!(
    benches,
    write_atomic_benchmark,
    workspace_layout_detect_benchmark
);
criterion_main!(benches);
```

## Findings and Recommendations

**Actual performance data could not be gathered.**

Based on code analysis:

*   **`io::write_atomic`**: This function's performance will be heavily dependent on the underlying filesystem and operating system. It performs a write to a temporary file, a filesystem sync (`fsync`), a rename, and another sync on the parent directory. This is a robust but potentially slow operation. The benchmark will quantify this overhead.
*   **`layout::WorkspaceLayout::detect`**: This function walks up the directory tree checking for the existence of a `.git` directory. Performance will be proportional to the depth of the starting path relative to the repository root (or the filesystem root in the "not found" case). On most systems, this should be a very fast operation, dominated by metadata checks (`lstat` or similar) by the OS. The benchmark will confirm if this is a potential bottleneck in deep directory structures.

**Recommendation:** Run the benchmarks in a CI environment to establish a performance baseline and enable regression detection on future changes. The current tooling issue preventing the benchmark run should be investigated and resolved.
