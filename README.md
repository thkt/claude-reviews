**English** | [日本語](README.ja.md)

# claude-reviews

A [Claude Code hook](https://docs.anthropic.com/en/docs/claude-code/hooks) that runs static analysis tools before configured skills (default: `/review`) and feeds the results to the agent as context. Instead of the agent scanning code manually, it gets real linter output and type errors upfront.

## How it works

```text
/review → PreToolUse hook fires → reviews binary runs
  ├─ Detects project type (package.json, tsconfig.json, React)
  ├─ Runs applicable tools in parallel (OS threads)
  └─ Returns JSON with tool output as additionalContext
        → Audit agent sees real static analysis results
```

The hook is **advisory-only**: it always approves the tool call and never blocks the skill. Tool failures or missing tools are silently skipped.

## Features

- **Parallel execution**: All enabled tools run simultaneously via OS threads
- **Fail-open design**: Errors never block the parent skill command
- **Auto-detection**: Only runs tools relevant to the project (package.json, tsconfig.json, React)
- **Binary resolution**: Finds tools in `node_modules/.bin` with `.git` boundary

## Requirements

Install the tools you want to use:

| Tool                                                      | Install                                     |
| --------------------------------------------------------- | ------------------------------------------- |
| [oxlint](https://oxc.rs)                                  | `npm i -g oxlint`                           |
| [knip](https://knip.dev)                                  | `npm i -D knip` (project-local recommended) |
| [tsgo](https://github.com/microsoft/typescript-go)        | `npm i -g @typescript/native-preview`       |
| [react-doctor](https://github.com/millionco/react-doctor) | `npm i -g react-doctor`                     |

If a tool is not installed, it is silently skipped.

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

> **Note**: Do not clone into your project directory. The cloned repository will remain as a nested git repo and may interfere with your project's git operations.

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
        "hooks": [
          {
            "type": "command",
            "command": "reviews",
            "timeout": 45000
          }
        ],
        "matcher": "Skill"
      }
    ]
  }
}
```

When a configured skill is invoked (default: `/review`), the hook:

1. Reads the Skill tool input from stdin
2. Checks if the skill name matches the `skills` list (exits silently for non-matching skills)
3. Detects project type and runs applicable tools in parallel
4. Outputs JSON with `additionalContext` containing tool results

## Tools

| Tool                                                      | Condition              | Arguments                        |
| --------------------------------------------------------- | ---------------------- | -------------------------------- |
| [knip](https://knip.dev)                                  | `package.json` exists  | `--reporter json --no-exit-code` |
| [oxlint](https://oxc.rs)                                  | `package.json` exists  | `--format json .`                |
| [tsgo](https://github.com/microsoft/typescript-go)        | `tsconfig.json` exists | `--noEmit`                       |
| [react-doctor](https://github.com/millionco/react-doctor) | React in dependencies  | `. --verbose`                    |

Tools are resolved from `node_modules/.bin` first, falling back to `$PATH`.

## Configuration

Place `.claude-reviews.json` at your project root (next to `.git/`). All fields are optional — only specify what you want to override.

**Defaults** (no config file needed): all tools enabled, activates on `/review`.

```json
{
  "enabled": true,
  "skills": ["review"],
  "tools": {
    "knip": true,
    "oxlint": true,
    "tsgo": true,
    "react_doctor": true
  }
}
```

### Examples

**Activate on `/audit` instead of `/review`:**

```json
{
  "skills": ["audit"]
}
```

**Activate on multiple skills:**

```json
{
  "skills": ["review", "audit"]
}
```

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

The config file is found by walking up from `$CWD` to the nearest `.git` directory. If `.claude-reviews.json` exists there, it is loaded and merged with defaults.

## Using with Existing Linters

If you already run oxlint via lefthook, husky, or lint-staged on commit, reviews' checks may overlap. The two serve different purposes:

| Tool             | When                | Purpose                                      |
| ---------------- | ------------------- | -------------------------------------------- |
| reviews (hook)   | On configured skill | Provide static analysis context to the agent |
| lefthook / husky | On commit           | Final gate before code enters history        |

To disable overlapping tools in reviews and rely on your commit hook instead:

```json
{
  "tools": {
    "oxlint": false
  }
}
```

## License

MIT
