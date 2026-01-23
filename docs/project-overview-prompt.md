This is the design propsal origin - without knowledge and incorporation of the findings saved in the docs/research.

# Design aim

Manage context and tooling between many different modern coding tools like agentic claude code and gemini cli, ides to ensure they all work with the same set of rules, skills and workflows.

# Details

This project implements RUST based command line tool for managing repository configurations and agentic tooling support. In 2026 agentic tools implement various ways of managing context and coding styles and standards. Implement schema that leverages simple commands to manage plugin, skill, workflow and tooling for various ides and agentic tools via add/remove/modify calls.

Goal: initialize reposiutories with a .repository metaschema. This schema will speifcy the kind of repository we're working with (standard, worktrees, and based on research findings any other).
Registers "installed" plugins (plugin might not be the right word, the a given IDE or agent system that uses its own schema for setting reading tools, rules, and other configuration options. Examples are cluade code, gemini cli, cursor, etc.).

The cli will provide the tools for managin the .repository schema, regustrations and repository branches, plugin files, git ignore management, docker ignore etc to abstract away tedious and repetitive worktree, feature, main, remote push management.

The cli will implement high level add/remove/modify commands for plugins, skills, workflows and tooling. The exact matrix of commands and supported should be deductible from the research findings (but complimentary research is likely needed to find and identify missing knowledge gaps).

# Technical suggestions

Use rust using highly idependent cli based crates. Must perform 2026 standards for modern rust cli practices. I'm keen to keep feature groups as independent as possible so must think and research WHAT exact features are required for mission siuccess and what would be the logical thresholds for feature groups. Research should yeild clear tehcnical feature, plugin support matrixes, required tooling for each plugin requirement and what each plugin requires for proper functioning to avoid functionality drift.
