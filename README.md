# claude-reviews

Pre-flight static analysis hook for Claude Code's `/audit` command. Runs JS/TS tools (knip, oxlint, tsgo, react-doctor) and Rust tools (clippy, cargo check, cargo test, cargo-audit, cargo-machete) in parallel and injects results as `additionalContext`.

## Features

- **Parallel execution**: All tools run simultaneously via OS threads
- **Fail-open design**: Errors never block the parent `/audit` command
- **Auto-detection**: Only runs tools relevant to the project (package.json, tsconfig.json, React, Cargo.toml)
- **Binary resolution**: Finds JS/TS tools in `node_modules/.bin` with `.git` boundary

## Installation

### Homebrew (Recommended)

```bash
brew install thkt/tap/reviews
```

### From Release

Download the latest binary from [Releases](https://github.com/thkt/claude-reviews/releases):

```bash
# macOS (Apple Silicon)
curl -L https://github.com/thkt/claude-reviews/releases/latest/download/reviews-aarch64-apple-darwin.tar.gz | tar xz
mv reviews ~/.local/bin/
```

### From Source

```bash
cd /tmp
git clone https://github.com/thkt/claude-reviews.git
cd claude-reviews
cargo build --release
cp target/release/reviews ~/.local/bin/
cd .. && rm -rf claude-reviews
```

## Usage

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Skill",
        "hooks": [
          {
            "type": "command",
            "command": "reviews",
            "timeout": 45000
          }
        ]
      }
    ]
  }
}
```

When `/audit` is invoked, the hook:

1. Reads the Skill tool input from stdin
2. Checks if the skill is `audit` (exits silently for other skills)
3. Detects project type and runs applicable tools in parallel
4. Outputs JSON with `additionalContext` containing tool results

## Tools

### JS/TS

| Tool                                                           | Condition              | Arguments                        |
| -------------------------------------------------------------- | ---------------------- | -------------------------------- |
| [knip](https://knip.dev)                                       | `package.json` exists  | `--reporter json --no-exit-code` |
| [oxlint](https://oxc.rs)                                       | `package.json` exists  | `--format json .`                |
| [tsgo](https://github.com/nicolo-ribaudo/tsgo)                 | `tsconfig.json` exists | `--noEmit`                       |
| [react-doctor](https://github.com/nicolo-ribaudo/react-doctor) | React in dependencies  | `. --verbose`                    |

JS/TS tools are resolved from `node_modules/.bin` first, falling back to `$PATH`.

### Rust

| Tool                                                     | Condition                       | Arguments                                         |
| -------------------------------------------------------- | ------------------------------- | ------------------------------------------------- |
| [clippy](https://doc.rust-lang.org/clippy/)              | `Cargo.toml` exists             | `clippy --message-format=short -- -W clippy::all` |
| cargo check                                              | `Cargo.toml` exists             | `check --message-format=short`                    |
| cargo test                                               | `Cargo.toml` exists             | `test --no-fail-fast`                             |
| [cargo-audit](https://rustsec.org)                       | `Cargo.toml` exists + installed | `audit`                                           |
| [cargo-machete](https://github.com/bnjbvr/cargo-machete) | `Cargo.toml` exists + installed | `machete`                                         |

cargo-audit and cargo-machete require separate installation (`cargo install cargo-audit cargo-machete`).

If a tool is not installed, it is silently skipped.

## Configuration

Place `.claude-reviews.json` at your project root (next to `.git/`). All fields are optional — only specify what you want to override.

**Defaults** (no config file needed): all tools enabled.

```json
{
  "enabled": true,
  "tools": {
    "knip": true,
    "oxlint": true,
    "tsgo": true,
    "react_doctor": true,
    "clippy": true,
    "cargoCheck": true,
    "cargoTest": true,
    "audit": true,
    "machete": true
  }
}
```

### Examples

**Disable a specific tool:**

```json
{
  "tools": {
    "tsgo": false
  }
}
```

**Disable reviews for a project:**

```json
{
  "enabled": false
}
```

### Config Resolution

The config file is found by walking up from `$CWD` to the nearest `.git` directory.

## License

MIT
