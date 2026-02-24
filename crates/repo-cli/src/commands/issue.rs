//! Issue command implementations
//!
//! Provides `repo issue assign <url>` which parses a GitHub/GitLab issue URL,
//! derives a branch name, and creates a branch or worktree.

use std::path::Path;

use colored::Colorize;

use repo_core::Mode;
use repo_fs::NormalizedPath;

use super::branch::run_branch_add;
use super::sync::detect_mode;
use crate::error::Result;

/// Parsed reference to a hosted issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueRef {
    /// Platform hosting the issue ("github" or "gitlab")
    pub platform: String,
    /// Repository owner (user or org)
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Issue number
    pub number: u64,
}

/// Parse a GitHub or GitLab issue URL into an [`IssueRef`].
///
/// Supported URL patterns:
/// - GitHub: `https://github.com/{owner}/{repo}/issues/{number}`
/// - GitLab: `https://gitlab.com/{owner}/{repo}/-/issues/{number}`
///
/// Returns `None` for any URL that does not match a supported pattern.
pub fn parse_issue_url(url: &str) -> Option<IssueRef> {
    // Strip optional trailing slash and query/fragment
    let url = url.split('?').next().unwrap_or(url);
    let url = url.split('#').next().unwrap_or(url);
    let url = url.trim_end_matches('/');

    // Require https:// or http://
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;

    // Split into host + path segments
    let (host, path) = rest.split_once('/')?;
    let segments: Vec<&str> = path.split('/').collect();

    match host {
        "github.com" => {
            // github.com/{owner}/{repo}/issues/{number}
            if segments.len() >= 4 && segments[2] == "issues" {
                let number = segments[3].parse::<u64>().ok()?;
                Some(IssueRef {
                    platform: "github".to_string(),
                    owner: segments[0].to_string(),
                    repo: segments[1].to_string(),
                    number,
                })
            } else {
                None
            }
        }
        "gitlab.com" => {
            // gitlab.com/{owner}/{repo}/-/issues/{number}
            if segments.len() >= 5 && segments[2] == "-" && segments[3] == "issues" {
                let number = segments[4].parse::<u64>().ok()?;
                Some(IssueRef {
                    platform: "gitlab".to_string(),
                    owner: segments[0].to_string(),
                    repo: segments[1].to_string(),
                    number,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Generate a sanitized branch name from an issue number and optional title slug.
///
/// Rules:
/// - Prefix with `issue-{number}`
/// - If a slug is provided, append `-{slug}` (lowercase, spaces → dashes, strip non-alphanumeric)
/// - Truncate at 50 characters
pub fn generate_branch_name(number: u64, title_slug: Option<&str>) -> String {
    let prefix = format!("issue-{}", number);

    let branch = match title_slug {
        None | Some("") => prefix,
        Some(slug) => {
            let sanitized = sanitize_slug(slug);
            if sanitized.is_empty() {
                prefix
            } else {
                format!("{}-{}", prefix, sanitized)
            }
        }
    };

    // Truncate at 50 chars, trimming any trailing dash
    if branch.len() > 50 {
        branch[..50].trim_end_matches('-').to_string()
    } else {
        branch
    }
}

/// Sanitize an arbitrary string into a branch-name-safe slug.
///
/// - Converts to lowercase
/// - Replaces spaces and underscores with dashes
/// - Removes any character that is not alphanumeric or a dash
/// - Collapses consecutive dashes
/// - Trims leading/trailing dashes
fn sanitize_slug(input: &str) -> String {
    let lowered = input.to_lowercase();
    let mut result = String::with_capacity(lowered.len());
    let mut prev_dash = false;

    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch);
            prev_dash = false;
        } else if ch == ' ' || ch == '-' || ch == '_' {
            if !prev_dash && !result.is_empty() {
                result.push('-');
                prev_dash = true;
            }
        }
        // Drop all other characters
    }

    result.trim_end_matches('-').to_string()
}

/// Handle `repo issue assign <url>`.
///
/// 1. Parses the issue URL.
/// 2. Generates a branch name.
/// 3. Creates a branch (standard mode) or worktree (worktrees mode).
/// 4. Prints the next steps.
pub fn handle_issue_assign(url: &str, path: &Path) -> Result<()> {
    let issue = parse_issue_url(url).ok_or_else(|| {
        crate::error::CliError::user(format!(
            "Could not parse issue URL: {}\n\
             Expected:\n  \
             https://github.com/{{owner}}/{{repo}}/issues/{{number}}\n  \
             https://gitlab.com/{{owner}}/{{repo}}/-/issues/{{number}}",
            url
        ))
    })?;

    let branch_name = generate_branch_name(issue.number, None);

    println!(
        "{} Assigning {} issue #{} from {}/{}",
        "=>".blue().bold(),
        issue.platform.cyan(),
        issue.number.to_string().yellow(),
        issue.owner.dimmed(),
        issue.repo.dimmed(),
    );
    println!(
        "   Branch name: {}",
        branch_name.cyan()
    );

    // Delegate to run_branch_add which handles both modes
    run_branch_add(path, &branch_name, Some("main"))?;

    // Print next steps
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;

    println!();
    println!("{} Next steps:", "=>".blue().bold());
    match mode {
        Mode::Worktrees => {
            let wt_path = root.join(&branch_name);
            println!(
                "   {} {}",
                "cd".dimmed(),
                wt_path.as_str().cyan()
            );
            println!("   {} sync", "repo".dimmed());
        }
        Mode::Standard => {
            println!(
                "   {} {}",
                "git checkout".dimmed(),
                branch_name.cyan()
            );
            println!("   {} sync", "repo".dimmed());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_issue_url ---

    #[test]
    fn test_parse_github_url() {
        let url = "https://github.com/org/repo/issues/42";
        let result = parse_issue_url(url);
        assert_eq!(
            result,
            Some(IssueRef {
                platform: "github".to_string(),
                owner: "org".to_string(),
                repo: "repo".to_string(),
                number: 42,
            })
        );
    }

    #[test]
    fn test_parse_gitlab_url() {
        let url = "https://gitlab.com/org/repo/-/issues/99";
        let result = parse_issue_url(url);
        assert_eq!(
            result,
            Some(IssueRef {
                platform: "gitlab".to_string(),
                owner: "org".to_string(),
                repo: "repo".to_string(),
                number: 99,
            })
        );
    }

    #[test]
    fn test_parse_invalid_url_returns_none() {
        assert_eq!(parse_issue_url("https://example.com/foo/bar"), None);
        assert_eq!(parse_issue_url("not-a-url"), None);
        assert_eq!(parse_issue_url("https://github.com/org/repo/pulls/1"), None);
        assert_eq!(parse_issue_url("https://github.com/org/repo/issues/abc"), None);
    }

    #[test]
    fn test_parse_github_url_trailing_slash() {
        let url = "https://github.com/my-org/my-repo/issues/123/";
        let result = parse_issue_url(url);
        assert_eq!(
            result,
            Some(IssueRef {
                platform: "github".to_string(),
                owner: "my-org".to_string(),
                repo: "my-repo".to_string(),
                number: 123,
            })
        );
    }

    // --- generate_branch_name ---

    #[test]
    fn test_branch_name_generation_no_slug() {
        assert_eq!(generate_branch_name(42, None), "issue-42");
    }

    #[test]
    fn test_branch_name_generation_with_slug() {
        assert_eq!(
            generate_branch_name(7, Some("add login feature")),
            "issue-7-add-login-feature"
        );
    }

    #[test]
    fn test_branch_name_generation_sanitize_special_chars() {
        assert_eq!(
            generate_branch_name(1, Some("Fix: Bug/Crash (v2.0)!")),
            "issue-1-fix-bugcrash-v20"
        );
    }

    #[test]
    fn test_branch_name_generation_truncation() {
        // slug long enough to exceed 50 chars
        let long_slug = "this-is-a-very-long-title-that-should-be-truncated-at-fifty-characters";
        let result = generate_branch_name(5, Some(long_slug));
        assert!(result.len() <= 50, "branch name must be <= 50 chars: {}", result);
        assert!(!result.ends_with('-'), "must not end with dash: {}", result);
    }

    #[test]
    fn test_branch_name_generation_lowercase() {
        assert_eq!(
            generate_branch_name(3, Some("UPPERCASE TITLE")),
            "issue-3-uppercase-title"
        );
    }

    #[test]
    fn test_branch_name_empty_slug() {
        assert_eq!(generate_branch_name(10, Some("")), "issue-10");
        assert_eq!(generate_branch_name(10, Some("!!!")), "issue-10");
    }

    // --- sanitize_slug ---

    #[test]
    fn test_sanitize_slug_collapses_dashes() {
        assert_eq!(sanitize_slug("foo  bar"), "foo-bar");
        assert_eq!(sanitize_slug("foo--bar"), "foo-bar");
    }
}
