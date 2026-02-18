//! Error types for repo-blocks

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Block not found: {uuid} in {path}")]
    BlockNotFound { uuid: String, path: PathBuf },
}
