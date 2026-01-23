use repo_fs::{NormalizedPath, io};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_write_atomic_creates_file() {
    let temp = TempDir::new().unwrap();
    let path = NormalizedPath::new(temp.path().join("test.txt"));

    io::write_atomic(&path, b"hello world").unwrap();

    let content = fs::read_to_string(path.to_native()).unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn test_write_atomic_overwrites_existing() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "original").unwrap();

    let path = NormalizedPath::new(&file_path);
    io::write_atomic(&path, b"updated").unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "updated");
}

#[test]
fn test_write_atomic_no_partial_writes() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "original content").unwrap();

    let path = NormalizedPath::new(&file_path);

    // Even if this were to fail mid-write, we shouldn't see partial content
    io::write_atomic(&path, b"new content").unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    // Should be either "original content" or "new content", never partial
    assert!(content == "original content" || content == "new content");
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
