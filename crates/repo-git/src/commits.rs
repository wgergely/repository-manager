//! Recent commit history extraction from git repositories.

use chrono::{DateTime, TimeZone, Utc};
use git2::Repository;

use crate::Result;

/// Information about a single commit.
pub struct CommitInfo {
    /// Short commit hash (7 characters)
    pub hash: String,

    /// First line of the commit message
    pub message: String,

    /// Commit author name
    pub author: String,

    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
}

/// Extract the last `max_count` commits from a specific branch.
///
/// Performs a time-sorted revwalk starting from the tip of `branch`.
/// Returns commits in reverse-chronological order (most recent first).
pub fn list_recent_commits(
    repo: &Repository,
    branch: &str,
    max_count: usize,
) -> Result<Vec<CommitInfo>> {
    let reference = match repo.find_reference(&format!("refs/heads/{branch}")) {
        Ok(r) => r,
        Err(_) => repo.find_reference(branch)?,
    };

    let commit = reference.peel_to_commit()?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push(commit.id())?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::with_capacity(max_count);

    for oid_result in revwalk.take(max_count) {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        let timestamp = commit.time();
        let dt: DateTime<Utc> = Utc
            .timestamp_opt(timestamp.seconds(), 0)
            .single()
            .unwrap_or_default();

        let message = commit
            .message()
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();

        let short_hash = format!("{:.7}", oid);

        commits.push(CommitInfo {
            hash: short_hash,
            message,
            author: author_name,
            timestamp: dt,
        });
    }

    Ok(commits)
}
