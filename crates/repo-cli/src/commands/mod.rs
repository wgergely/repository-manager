//! Command implementations for repo-cli

pub mod branch;
pub mod git;
pub mod init;
pub mod list;
pub mod rule;
pub mod sync;
pub mod tool;

pub use branch::{run_branch_add, run_branch_list, run_branch_remove};
pub use git::{run_merge, run_pull, run_push};
pub use init::run_init;
pub use list::{run_list_presets, run_list_tools, run_status};
pub use rule::{run_add_rule, run_list_rules, run_remove_rule};
pub use sync::{run_check, run_fix, run_sync};
pub use tool::{run_add_preset, run_add_tool, run_remove_preset, run_remove_tool};
