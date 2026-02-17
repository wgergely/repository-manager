//! Format handlers

pub mod html_comment;
mod json;
mod markdown;
mod plaintext;
mod toml;
mod yaml;

pub use self::json::JsonHandler;
pub use self::toml::TomlHandler;
pub use self::yaml::YamlHandler;
pub use html_comment::{
    find_blocks as find_html_blocks, insert_block as insert_html_block,
    remove_block as remove_html_block, update_block as update_html_block,
};
pub use markdown::MarkdownHandler;
pub use plaintext::PlainTextHandler;
