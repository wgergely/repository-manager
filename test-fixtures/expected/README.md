# Golden Test Fixtures

These files represent **expected output** for tool config generation.

## Rules

1. **Never regenerate from code.** Golden files are authored from tool
   documentation, not from running the code under test.
2. **Update manually** when tool format requirements change.
3. **Each file must have a provenance header** documenting the format source.
4. **If a golden file test fails**, investigate whether the code change
   broke the output format or whether the golden file needs updating.
   Do not blindly update the golden file to match new output.

## Provenance Headers

Every golden file must start with provenance comments:

- **Golden file:** What tool and file format it represents
- **Format source:** Where the format requirements were documented
- **Last validated:** Date when the file was last checked against tool docs
- **WARNING:** Reminder not to regenerate from code

These headers are stripped by `strip_provenance_header()` in
`fixture_tests.rs` before comparison against generated output.

## Directory Layout

```
expected/
├── aider/.aider.conf.yml    # Aider YAML config
├── claude/CLAUDE.md          # Claude Code Markdown
├── cursor/.cursorrules       # Cursor plain Markdown
└── README.md                 # This file
```
