//! Concurrent access tests for write_atomic locking
//!
//! Verifies that the fs2-based locking in write_atomic prevents
//! data corruption under concurrent access.

use repo_fs::{NormalizedPath, RobustnessConfig, io};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_concurrent_writes_no_corruption() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("concurrent.txt");
    let path = Arc::new(NormalizedPath::new(&file_path));

    let num_threads = 10;
    let writes_per_thread = 20;
    let barrier = Arc::new(Barrier::new(num_threads));

    let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let failure_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let path = Arc::clone(&path);
            let barrier = Arc::clone(&barrier);
            let successes = Arc::clone(&success_count);
            let failures = Arc::clone(&failure_count);

            thread::spawn(move || {
                // Synchronize all threads to start simultaneously
                barrier.wait();

                for i in 0..writes_per_thread {
                    let content = format!("thread{}:write{}\n", thread_id, i);
                    match io::write_text(&path, &content) {
                        Ok(_) => {
                            successes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                        Err(_) => {
                            failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    let total_successes = success_count.load(std::sync::atomic::Ordering::Relaxed);
    let total_failures = failure_count.load(std::sync::atomic::Ordering::Relaxed);
    let total_attempts = num_threads * writes_per_thread;

    assert_eq!(
        total_successes + total_failures,
        total_attempts,
        "All attempts must be accounted for"
    );

    // At least some writes must succeed - a total failure indicates a bug
    assert!(
        total_successes > 0,
        "At least some concurrent writes must succeed (got {total_successes}/{total_attempts})"
    );

    // Verify file exists and contains valid content (not corrupted/interleaved)
    let content = std::fs::read_to_string(&file_path).unwrap();

    // Content should be a complete write from one thread, not corrupted
    assert!(
        content.starts_with("thread"),
        "Content should start with 'thread', got: {}",
        &content[..content.len().min(50)]
    );
    assert!(
        content.contains(":write"),
        "Content should contain ':write'"
    );
    // Should be a single line (one complete write), not interleaved
    assert!(
        content.matches("thread").count() == 1,
        "Content should have exactly one 'thread' (no interleaving)"
    );
}

#[test]
fn test_concurrent_writes_to_different_files_all_succeed() {
    let dir = tempdir().unwrap();
    let num_threads = 5;
    let barrier = Arc::new(Barrier::new(num_threads));
    let results = Arc::new(std::sync::Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let dir_path = dir.path().to_path_buf();
            let barrier = Arc::clone(&barrier);
            let results = Arc::clone(&results);

            thread::spawn(move || {
                barrier.wait();

                let file_path = dir_path.join(format!("file_{}.txt", thread_id));
                let path = NormalizedPath::new(&file_path);
                let result = io::write_text(&path, &format!("content_{}", thread_id));

                results.lock().unwrap().push((thread_id, result.is_ok()));
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // All writes to different files should succeed
    let results = results.lock().unwrap();
    for (thread_id, success) in results.iter() {
        assert!(*success, "Write from thread {} should succeed", thread_id);
    }
}

#[test]
fn test_lock_timeout_is_respected() {
    use fs2::FileExt;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("locked.txt");
    let lock_path = format!("{}.lock", file_path.display());

    // Create and hold the lock file externally
    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false) // Lock files should not be truncated
        .open(&lock_path)
        .unwrap();
    lock_file.lock_exclusive().unwrap();

    let path = NormalizedPath::new(&file_path);
    let config = RobustnessConfig {
        lock_timeout: Duration::from_millis(500),
        enable_fsync: false,
    };

    let result = io::write_atomic(&path, b"content", config);

    // Release lock
    drop(lock_file);

    // Should have failed due to lock being held
    // Note: On some platforms the lock may fail immediately rather than timeout,
    // but the important behavior is that it fails when the lock is held.
    assert!(result.is_err(), "Write should fail when lock is held");
}
