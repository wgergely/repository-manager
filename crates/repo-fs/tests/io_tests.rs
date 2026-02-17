use repo_fs::{NormalizedPath, io};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_write_atomic_creates_file() {
    let temp = TempDir::new().unwrap();
    let path = NormalizedPath::new(temp.path().join("test.txt"));

    io::write_atomic(&path, b"hello world", io::RobustnessConfig::default()).unwrap();

    let content = fs::read_to_string(path.to_native()).unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn test_write_atomic_overwrites_existing() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "original").unwrap();

    let path = NormalizedPath::new(&file_path);
    io::write_atomic(&path, b"updated", io::RobustnessConfig::default()).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "updated");
}

#[test]
fn test_write_atomic_replaces_content_completely() {
    // Verify that write_atomic fully replaces the file content via rename,
    // so old content never leaks through even if the new content is shorter.
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");

    // Write long original content
    let original = "A".repeat(10_000);
    fs::write(&file_path, &original).unwrap();

    // Overwrite with shorter content
    let path = NormalizedPath::new(&file_path);
    let short_content = "short";
    io::write_atomic(&path, short_content.as_bytes(), io::RobustnessConfig::default()).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(
        content, short_content,
        "Content must be exactly the new short content, not a mix of old and new"
    );
    assert_eq!(
        content.len(),
        short_content.len(),
        "File size must match new content exactly (no leftover bytes from old content)"
    );
}

#[test]
fn test_write_atomic_concurrent_reader_sees_complete_content() {
    // Simulate a reader that checks the file while a writer is active.
    // Because write_atomic uses rename, the reader should see either the
    // old complete content or the new complete content, never partial.
    use std::sync::{Arc, Barrier};
    use std::thread;

    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("atomic.txt");

    let original = "ORIGINAL_CONTENT_AAAA";
    fs::write(&file_path, original).unwrap();

    let new_content = "NEW_CONTENT_BBBBB";
    let barrier = Arc::new(Barrier::new(2));
    let reader_path = file_path.clone();
    let b1 = barrier.clone();

    // Reader thread: reads the file content after synchronization
    let reader = thread::spawn(move || {
        b1.wait();
        // Read multiple times to increase chance of catching partial state
        let mut contents = Vec::new();
        for _ in 0..50 {
            if let Ok(c) = fs::read_to_string(&reader_path) {
                contents.push(c);
            }
        }
        contents
    });

    // Writer thread: writes new content after synchronization
    let writer_path = file_path.clone();
    let b2 = barrier.clone();
    let writer = thread::spawn(move || {
        b2.wait();
        let path = NormalizedPath::new(&writer_path);
        io::write_atomic(&path, new_content.as_bytes(), io::RobustnessConfig::default()).unwrap();
    });

    writer.join().unwrap();
    let observed = reader.join().unwrap();

    // Guard against vacuous truth: if all reads failed, the for-all assertion
    // below would pass trivially over an empty iterator.
    assert!(!observed.is_empty(), "Reader must observe at least one successful read");

    // Every observed content must be one of the two complete versions
    for content in &observed {
        assert!(
            content == original || content == new_content,
            "Reader saw partial/corrupted content: {:?} (expected {:?} or {:?})",
            content,
            original,
            new_content
        );
    }
}

#[test]
fn test_read_text_existing_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "hello").unwrap();

    let path = NormalizedPath::new(&file_path);
    let content = io::read_text(&path).unwrap();
    assert_eq!(content, "hello");
}

#[test]
fn test_read_text_nonexistent_file() {
    let path = NormalizedPath::new("/nonexistent/file.txt");
    let result = io::read_text(&path);
    assert!(result.is_err());
}

#[test]
fn test_write_text_creates_file() {
    let temp = TempDir::new().unwrap();
    let path = NormalizedPath::new(temp.path().join("test.txt"));

    io::write_text(&path, "hello world").unwrap();

    let content = fs::read_to_string(path.to_native()).unwrap();
    assert_eq!(content, "hello world");
}
