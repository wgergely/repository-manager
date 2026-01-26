use criterion::{Criterion, black_box, criterion_group, criterion_main};
use repo_fs::io::{self, RobustnessConfig};
use repo_fs::layout::WorkspaceLayout;
use repo_fs::NormalizedPath;
use std::fs;
use tempfile::tempdir;

fn write_atomic_benchmark(c: &mut Criterion) {
    c.bench_function("io::write_atomic", |b| {
        let dir = tempdir().unwrap();
        let path = NormalizedPath::new(dir.path().join("test_file.txt"));
        let content = "hello world".as_bytes();
        let config = RobustnessConfig::default();

        b.iter(|| {
            io::write_atomic(black_box(&path), black_box(content), config).unwrap();
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
