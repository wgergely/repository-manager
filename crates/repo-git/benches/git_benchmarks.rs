use criterion::{Criterion, criterion_group, criterion_main};
use git2::Repository;
use repo_fs::NormalizedPath;
use repo_git::{ContainerLayout, LayoutProvider, NamingStrategy};
use std::fs;
use tempfile::tempdir;

fn benchmark_container_layout_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("container_layout");

    group.bench_function("create_feature", |b| {
        b.iter_with_setup(
            || {
                // Setup: Create a new temp repo for each iteration
                let dir = tempdir().unwrap();
                let root = NormalizedPath::new(dir.path());
                let git_dir = root.join(".gt");

                // Initialize bare repo
                let repo = Repository::init_bare(git_dir.to_native()).unwrap();

                // Create initial commit
                let tree_id = {
                    let mut index = repo.index().unwrap();
                    index.write_tree().unwrap()
                };
                {
                    let tree = repo.find_tree(tree_id).unwrap();
                    let sig = repo.signature().unwrap();
                    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                        .unwrap();
                }

                // Initialize layout
                let layout = ContainerLayout::new(root.clone(), NamingStrategy::Slug).unwrap();

                // Create 'main' worktree manually as baseline
                let main_path = root.join("main");
                fs::create_dir_all(main_path.to_native()).unwrap();

                (layout, repo, dir)
            },
            |(layout, _repo, _dir)| {
                // Benchmark creation of a feature
                layout.create_feature("new-feature", None).unwrap();
            },
        );
    });

    group.finish();
}

criterion_group!(benches, benchmark_container_layout_operations);
criterion_main!(benches);
