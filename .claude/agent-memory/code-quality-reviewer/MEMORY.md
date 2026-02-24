# Code Quality Reviewer Memory

## Project: claude-reviews

- Rust CLI tool, binary name `reviews`, runs as Claude Code hook
- Entry: `src/main.rs`, reads JSON from stdin, runs static analysis tools in parallel
- Config: `.claude-reviews.json` found by walking up to `.git` boundary
- Tools: 9 total (4 JS/TS + 5 Rust/Cargo), each in `src/tools/<name>.rs`
- Key patterns: directory traversal to `.git`, tool result collection, output sanitization
- Macro: `define_tools!` in `config.rs` generates 3 structs from a field list
- Test helper: `test_utils::make_temp_dir` used across config/project/resolve tests

## Known Issues (from CQ review 2026-02-23)

- Triple-duplicated ancestor traversal in config.rs, project.rs, resolve.rs
- 5x duplicated test body (`skips_without_cargo_toml`) across cargo tool modules
- `run_with_timeout` leaks thread+child on timeout (low impact for short-lived CLI)
- `which` command is Unix-only (no Windows support)
- Manual `fs::remove_dir_all` cleanup in tests is panic-unsafe

## Review Patterns

- Rust tool modules follow: guard check -> `run_cargo_command(name, args, info)`
- JS tool modules follow: guard check -> `resolve_bin` -> `run_js_command`
- All tool `run` functions have signature `fn run(&ProjectInfo) -> ToolResult`
