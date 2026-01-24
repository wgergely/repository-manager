//! Basic usage example for repo-content

use repo_content::{BlockLocation, Document, Format};
use uuid::Uuid;

fn main() -> repo_content::Result<()> {
    // Parse a TOML config file
    let source = r#"[package]
name = "my-app"
version = "1.0.0"

[dependencies]
serde = "1.0"
"#;

    // Use parse_as with explicit format when content starts with [section]
    // Auto-detection works best when key = value pairs appear before sections
    let mut doc = Document::parse_as(source, Format::Toml)?;
    println!("Format: {:?}", doc.format());

    // Insert a managed block
    let uuid = Uuid::new_v4();
    let edit = doc.insert_block(
        uuid,
        "[managed.settings]\nenabled = true",
        BlockLocation::End,
    )?;
    println!("Inserted block with UUID: {}", uuid);
    println!("Edit: {:?}", edit.kind);

    // Find all blocks
    println!("\nManaged blocks:");
    for block in doc.find_blocks() {
        println!("  - {}: {} bytes", block.uuid, block.content.len());
        println!("    Checksum: {}", block.checksum());
    }

    // Update the block
    let edit = doc.update_block(uuid, "[managed.settings]\nenabled = false")?;
    println!("\nUpdated block");

    // Render output
    println!("\nFinal document:\n{}", doc.render());

    // Rollback the update
    let rollback = edit.inverse();
    println!("\nRollback edit: {:?}", rollback.kind);

    Ok(())
}
