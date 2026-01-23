//! Tests for Edit types

use repo_content::edit::{Edit, EditKind};
use uuid::Uuid;

#[test]
fn test_edit_inverse_insert() {
    let edit = Edit {
        kind: EditKind::Insert,
        span: 10..10, // Zero-width insertion point
        old_content: String::new(),
        new_content: "inserted".to_string(),
    };

    let inverse = edit.inverse();
    assert!(matches!(inverse.kind, EditKind::Delete));
    assert_eq!(inverse.span, 10..18); // Spans the inserted content
    assert_eq!(inverse.old_content, "inserted");
    assert_eq!(inverse.new_content, "");
}

#[test]
fn test_edit_inverse_delete() {
    let edit = Edit {
        kind: EditKind::Delete,
        span: 10..20,
        old_content: "deleted".to_string(),
        new_content: String::new(),
    };

    let inverse = edit.inverse();
    assert!(matches!(inverse.kind, EditKind::Insert));
    assert_eq!(inverse.old_content, "");
    assert_eq!(inverse.new_content, "deleted");
    assert_eq!(inverse.span, 10..10);
}

#[test]
fn test_edit_inverse_replace() {
    let edit = Edit {
        kind: EditKind::Replace,
        span: 10..20,
        old_content: "old".to_string(),
        new_content: "new".to_string(),
    };

    let inverse = edit.inverse();
    assert!(matches!(inverse.kind, EditKind::Replace));
    assert_eq!(inverse.span, 10..13); // Spans the new content length (3 chars)
    assert_eq!(inverse.old_content, "new");
    assert_eq!(inverse.new_content, "old");
}

#[test]
fn test_edit_inverse_block_insert() {
    let uuid = Uuid::new_v4();
    let edit = Edit {
        kind: EditKind::BlockInsert { uuid },
        span: 0..0, // Zero-width insertion point
        old_content: String::new(),
        new_content: "block content".to_string(),
    };

    let inverse = edit.inverse();
    assert!(matches!(inverse.kind, EditKind::BlockRemove { uuid: u } if u == uuid));
    assert_eq!(inverse.span, 0..13); // Spans the inserted content
    assert_eq!(inverse.old_content, "block content");
    assert_eq!(inverse.new_content, "");
}

#[test]
fn test_edit_inverse_block_update() {
    let uuid = Uuid::new_v4();
    let edit = Edit {
        kind: EditKind::BlockUpdate { uuid },
        span: 0..50,
        old_content: "old block".to_string(),
        new_content: "new block".to_string(),
    };

    let inverse = edit.inverse();
    assert!(matches!(inverse.kind, EditKind::BlockUpdate { uuid: u } if u == uuid));
    assert_eq!(inverse.span, 0..9); // Spans the new content length (9 chars)
    assert_eq!(inverse.old_content, "new block");
    assert_eq!(inverse.new_content, "old block");
}

#[test]
fn test_edit_inverse_block_remove() {
    let uuid = Uuid::new_v4();
    let edit = Edit {
        kind: EditKind::BlockRemove { uuid },
        span: 0..50,
        old_content: "removed block".to_string(),
        new_content: String::new(),
    };

    let inverse = edit.inverse();
    assert!(matches!(inverse.kind, EditKind::BlockInsert { uuid: u } if u == uuid));
    assert_eq!(inverse.old_content, "");
    assert_eq!(inverse.new_content, "removed block");
    assert_eq!(inverse.span, 0..0);
}

#[test]
fn test_edit_inverse_path_set() {
    let edit = Edit {
        kind: EditKind::PathSet {
            path: "config.key".to_string(),
        },
        span: 10..20,
        old_content: "old_value".to_string(),
        new_content: "new_value".to_string(),
    };

    let inverse = edit.inverse();
    assert!(
        matches!(inverse.kind, EditKind::PathSet { ref path } if path == "config.key")
    );
    assert_eq!(inverse.span, 10..19); // Spans the new content length (9 chars)
    assert_eq!(inverse.old_content, "new_value");
    assert_eq!(inverse.new_content, "old_value");
}

#[test]
fn test_edit_inverse_path_remove() {
    let edit = Edit {
        kind: EditKind::PathRemove {
            path: "config.key".to_string(),
        },
        span: 10..30,
        old_content: "removed_value".to_string(),
        new_content: String::new(),
    };

    let inverse = edit.inverse();
    assert!(
        matches!(inverse.kind, EditKind::PathSet { ref path } if path == "config.key")
    );
    assert_eq!(inverse.span, 10..10); // Zero-width insertion point
    assert_eq!(inverse.old_content, "");
    assert_eq!(inverse.new_content, "removed_value");
}

#[test]
fn test_edit_apply() {
    let source = "Hello World";
    let edit = Edit::replace(6..11, "World", "Rust");
    let result = edit.apply(source);
    assert_eq!(result, "Hello Rust");
}

#[test]
fn test_edit_apply_insert() {
    let source = "Hello World";
    let edit = Edit::insert(5, " Beautiful");
    let result = edit.apply(source);
    assert_eq!(result, "Hello Beautiful World");
}

#[test]
fn test_edit_apply_delete() {
    let source = "Hello Beautiful World";
    let edit = Edit::delete(5..15, " Beautiful");
    let result = edit.apply(source);
    assert_eq!(result, "Hello World");
}

#[test]
fn test_edit_insert_helper() {
    let edit = Edit::insert(5, "test");
    assert!(matches!(edit.kind, EditKind::Insert));
    assert_eq!(edit.span, 5..5); // Zero-width range at insertion point
    assert_eq!(edit.new_content, "test");
    assert_eq!(edit.old_content, "");
}

#[test]
fn test_edit_delete_helper() {
    let edit = Edit::delete(5..10, "hello");
    assert!(matches!(edit.kind, EditKind::Delete));
    assert_eq!(edit.span, 5..10);
    assert_eq!(edit.old_content, "hello");
    assert_eq!(edit.new_content, "");
}

#[test]
fn test_edit_replace_helper() {
    let edit = Edit::replace(5..10, "hello", "world");
    assert!(matches!(edit.kind, EditKind::Replace));
    assert_eq!(edit.span, 5..10);
    assert_eq!(edit.old_content, "hello");
    assert_eq!(edit.new_content, "world");
}

#[test]
fn test_edit_block_insert_helper() {
    let uuid = Uuid::new_v4();
    let edit = Edit::block_insert(uuid, 10, "block content");
    assert!(matches!(edit.kind, EditKind::BlockInsert { uuid: u } if u == uuid));
    assert_eq!(edit.span, 10..10); // Zero-width range at insertion point
    assert_eq!(edit.new_content, "block content");
    assert_eq!(edit.old_content, "");
}

#[test]
fn test_edit_block_update_helper() {
    let uuid = Uuid::new_v4();
    let edit = Edit::block_update(uuid, 10..30, "old", "new");
    assert!(matches!(edit.kind, EditKind::BlockUpdate { uuid: u } if u == uuid));
    assert_eq!(edit.span, 10..30);
    assert_eq!(edit.old_content, "old");
    assert_eq!(edit.new_content, "new");
}

#[test]
fn test_edit_block_remove_helper() {
    let uuid = Uuid::new_v4();
    let edit = Edit::block_remove(uuid, 10..30, "removed");
    assert!(matches!(edit.kind, EditKind::BlockRemove { uuid: u } if u == uuid));
    assert_eq!(edit.span, 10..30);
    assert_eq!(edit.old_content, "removed");
    assert_eq!(edit.new_content, "");
}

#[test]
fn test_edit_path_set_helper() {
    let edit = Edit::path_set("config.key", 10..20, "old", "new");
    assert!(matches!(edit.kind, EditKind::PathSet { ref path } if path == "config.key"));
    assert_eq!(edit.span, 10..20);
    assert_eq!(edit.old_content, "old");
    assert_eq!(edit.new_content, "new");
}

#[test]
fn test_edit_path_remove_helper() {
    let edit = Edit::path_remove("config.key", 10..20, "removed");
    assert!(matches!(edit.kind, EditKind::PathRemove { ref path } if path == "config.key"));
    assert_eq!(edit.span, 10..20);
    assert_eq!(edit.old_content, "removed");
    assert_eq!(edit.new_content, "");
}

#[test]
fn test_edit_roundtrip() {
    // Verify that applying an edit and then its inverse returns to original
    let source = "Hello World";
    let edit = Edit::replace(6..11, "World", "Rust");

    let modified = edit.apply(source);
    assert_eq!(modified, "Hello Rust");

    let inverse = edit.inverse();
    let restored = inverse.apply(&modified);
    assert_eq!(restored, source);
}

#[test]
fn test_edit_insert_roundtrip() {
    let source = "Hello World";
    let edit = Edit::insert(5, " Beautiful");

    let modified = edit.apply(source);
    assert_eq!(modified, "Hello Beautiful World");

    let inverse = edit.inverse();
    let restored = inverse.apply(&modified);
    assert_eq!(restored, source);
}

#[test]
fn test_edit_delete_roundtrip() {
    let source = "Hello Beautiful World";
    let edit = Edit::delete(5..15, " Beautiful");

    let modified = edit.apply(source);
    assert_eq!(modified, "Hello World");

    let inverse = edit.inverse();
    let restored = inverse.apply(&modified);
    assert_eq!(restored, source);
}
