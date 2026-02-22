# Spec: Rust Project Hooks (formatter / guardrails / reviews)

Updated: 2026-02-22
SOW: .claude/workspace/planning/2026-02-22-rust-hooks/sow.md

## Functional Requirements

### Phase 1: claude-formatter-rust

| ID     | Description                               | Input                       | Output                    | Implements |
| ------ | ----------------------------------------- | --------------------------- | ------------------------- | ---------- |
| FR-001 | stdin JSON パース（Write/Edit/MultiEdit） | stdin JSON (≤10MB)          | ToolName + file_path 抽出 | AC-1       |
| FR-002 | .rs ファイル判定                          | file_path                   | bool                      | AC-1       |
| FR-003 | cargo fmt 実行                            | file_path                   | フォーマット適用          | AC-1       |
| FR-004 | 設定ファイル読み込み・マージ              | .claude-formatter-rust.json | Config 構造体             | AC-1, AC-4 |

### Phase 2: claude-guardrails-rust

| ID     | Description                        | Input                        | Output                  | Implements |
| ------ | ---------------------------------- | ---------------------------- | ----------------------- | ---------- |
| FR-101 | stdin JSON パース + content 抽出   | stdin JSON                   | file_path + content     | AC-2       |
| FR-102 | .rs ファイル判定                   | file_path                    | bool                    | AC-2       |
| FR-103 | sensitive_file ルール              | file_path                    | Vec<Violation>          | AC-2       |
| FR-104 | generated_file ルール              | file_path                    | Vec<Violation>          | AC-2       |
| FR-105 | unsafe_usage ルール                | content                      | Vec<Violation>          | AC-2       |
| FR-106 | unwrap_usage ルール（閾値: 3箇所） | content                      | Vec<Violation>          | AC-2       |
| FR-107 | todo_macro ルール（テスト外のみ）  | content + file_path          | Vec<Violation>          | AC-2       |
| FR-108 | cargo_lock ルール                  | file_path                    | Vec<Violation>          | AC-2       |
| FR-109 | cargo clippy 外部リント            | file_path                    | Vec<Violation>          | AC-2       |
| FR-110 | violation/warning レポート出力     | Vec<Violation>               | stderr フォーマット出力 | AC-2       |
| FR-111 | 設定ファイル読み込み・マージ       | .claude-guardrails-rust.json | Config 構造体           | AC-2, AC-4 |

### Phase 3: claude-reviews-rust

| ID     | Description                     | Input                     | Output          | Implements |
| ------ | ------------------------------- | ------------------------- | --------------- | ---------- |
| FR-201 | stdin JSON パース（audit 判定） | stdin JSON                | Option<()>      | AC-3       |
| FR-202 | プロジェクト情報検出            | cwd                       | ProjectInfo     | AC-3       |
| FR-203 | cargo clippy 実行               | ProjectInfo               | ToolResult      | AC-3       |
| FR-204 | cargo check 実行                | ProjectInfo               | ToolResult      | AC-3       |
| FR-205 | cargo test 実行                 | ProjectInfo               | ToolResult      | AC-3       |
| FR-206 | cargo audit 実行                | ProjectInfo               | ToolResult      | AC-3       |
| FR-207 | cargo machete 実行              | ProjectInfo               | ToolResult      | AC-3       |
| FR-208 | 並列実行マネージャ              | Vec<ToolRunner>           | Vec<ToolResult> | AC-3       |
| FR-209 | 結果 JSON 出力                  | Vec<ToolResult>           | stdout JSON     | AC-3       |
| FR-210 | 設定ファイル読み込み・マージ    | .claude-reviews-rust.json | Config 構造体   | AC-3, AC-4 |

Validation:

| FR     | Rule                                | Error                                |
| ------ | ----------------------------------- | ------------------------------------ |
| FR-001 | stdin ≤ 10MB                        | "error: input too large"             |
| FR-001 | valid JSON                          | "invalid hook input: {parse_error}"  |
| FR-001 | tool_name in {Write,Edit,MultiEdit} | silent skip (exit 0)                 |
| FR-101 | file_path 非空                      | "skipping (unsupported or empty)"    |
| FR-106 | unwrap count ≥ 3                    | warning: "Excessive .unwrap() usage" |
| FR-201 | skill == "audit"                    | silent skip (exit 0)                 |

## Data Model

### Phase 1: claude-formatter-rust

```rust
// FR-001
enum ToolName { Write, Edit, MultiEdit, Other }

struct HookInput {
    tool_name: ToolName,       // FR-001
    tool_input: ToolInput,     // FR-001
}

struct ToolInput {
    file_path: Option<String>, // FR-001
}

// FR-004
struct Config {
    enabled: bool,             // FR-004
}
```

| Model     | Fields                | Used By |
| --------- | --------------------- | ------- |
| HookInput | tool_name, tool_input | FR-001  |
| Config    | enabled               | FR-004  |

### Phase 2: claude-guardrails-rust

```rust
// FR-101
struct ToolInput {
    tool_name: String,             // FR-101
    tool_input: ToolInputData,     // FR-101
}

struct ToolInputData {
    file_path: Option<String>,     // FR-101
    content: Option<String>,       // FR-101 (Write)
    new_string: Option<String>,    // FR-101 (Edit)
    edits: Option<Vec<EditItem>>,  // FR-101 (MultiEdit)
}

// FR-103-FR-108
enum Severity { Critical, High, Medium, Low } // FR-110

struct Violation {
    rule: String,          // FR-103
    severity: Severity,    // FR-103
    failure: String,       // FR-103
    file: String,          // FR-103
    line: Option<u32>,     // FR-103
}

struct Rule {
    file_pattern: Regex,       // FR-103
    checker: Box<dyn Fn>,      // FR-103
}

// FR-111
struct Config {
    enabled: bool,             // FR-111
    rules: RulesConfig,        // FR-111
    severity: SeverityConfig,  // FR-111
}

struct RulesConfig {
    sensitive_file: bool,   // FR-103
    generated_file: bool,   // FR-104
    unsafe_usage: bool,     // FR-105
    unwrap_usage: bool,     // FR-106
    todo_macro: bool,       // FR-107
    cargo_lock: bool,       // FR-108
    clippy: bool,           // FR-109
}

struct SeverityConfig {
    block_on: Vec<Severity>, // FR-110
}
```

| Model     | Fields                              | Used By       |
| --------- | ----------------------------------- | ------------- |
| ToolInput | tool_name, tool_input               | FR-101        |
| Violation | rule, severity, failure, file, line | FR-103~FR-110 |
| Rule      | file_pattern, checker               | FR-103~FR-108 |
| Config    | enabled, rules, severity            | FR-111        |

### Phase 3: claude-reviews-rust

```rust
// FR-201
struct HookInput {
    tool_input: SkillInput,   // FR-201
}

struct SkillInput {
    skill: Option<String>,    // FR-201
}

// FR-202
struct ProjectInfo {
    root: PathBuf,            // FR-202
    has_cargo_toml: bool,     // FR-202
}

// FR-203-FR-207
struct ToolResult {
    name: &'static str,       // FR-203
    output: String,            // FR-203
    success: bool,             // FR-203
}

// FR-210
struct Config {
    enabled: bool,             // FR-210
    tools: ToolsConfig,        // FR-210
}

struct ToolsConfig {
    clippy: bool,              // FR-203
    check: bool,               // FR-204
    test: bool,                // FR-205
    audit: bool,               // FR-206
    machete: bool,             // FR-207
}
```

| Model       | Fields                | Used By       |
| ----------- | --------------------- | ------------- |
| HookInput   | tool_input            | FR-201        |
| ProjectInfo | root, has_cargo_toml  | FR-202~FR-207 |
| ToolResult  | name, output, success | FR-203~FR-209 |
| Config      | enabled, tools        | FR-210        |

## Implementation

| Phase | FRs           | Files                                        |
| ----- | ------------- | -------------------------------------------- |
| 1     | FR-001~FR-004 | main.rs, config.rs                           |
| 2     | FR-101~FR-111 | main.rs, config.rs, rules/\*.rs, reporter.rs |
| 3     | FR-201~FR-210 | main.rs, config.rs, project.rs, tools/\*.rs  |

## Test Scenarios

### Phase 1: claude-formatter-rust

| ID    | Type | FR     | Given                            | When            | Then                       |
| ----- | ---- | ------ | -------------------------------- | --------------- | -------------------------- |
| T-001 | unit | FR-001 | Write + .rs file_path            | パース実行      | ToolName::Write, path 取得 |
| T-002 | unit | FR-001 | Read ツール                      | パース実行      | ToolName::Other            |
| T-003 | unit | FR-001 | 不正 JSON                        | パース実行      | エラー出力, exit 0         |
| T-004 | unit | FR-002 | "src/main.rs"                    | is_rs_file 判定 | true                       |
| T-005 | unit | FR-002 | "src/app.ts"                     | is_rs_file 判定 | false                      |
| T-006 | unit | FR-004 | .claude-formatter-rust.json 存在 | Config::load    | 設定値反映                 |
| T-007 | unit | FR-004 | 設定ファイルなし                 | Config::load    | デフォルト値               |
| T-008 | unit | FR-004 | enabled: false                   | Config::load    | enabled == false           |
| T-009 | unit | FR-004 | 不正 JSON 設定                   | Config::load    | デフォルトにフォールバック |

### Phase 2: claude-guardrails-rust

| ID    | Type | FR     | Given                            | When              | Then                          |
| ----- | ---- | ------ | -------------------------------- | ----------------- | ----------------------------- |
| T-101 | unit | FR-101 | Write + .rs + content            | パース実行        | file_path + content 取得      |
| T-102 | unit | FR-101 | Edit + .rs + new_string          | パース実行        | file_path + new_string 取得   |
| T-103 | unit | FR-103 | file_path = "/proj/.env"         | sensitive_file    | 1 violation (Critical)        |
| T-104 | unit | FR-103 | file_path = "/proj/src/main.rs"  | sensitive_file    | 0 violations                  |
| T-105 | unit | FR-104 | file_path = "types.generated.rs" | generated_file    | 1 violation                   |
| T-106 | unit | FR-105 | content に unsafe ブロック       | unsafe_usage      | 1 violation (Medium)          |
| T-107 | unit | FR-105 | content に unsafe なし           | unsafe_usage      | 0 violations                  |
| T-108 | unit | FR-106 | content に unwrap() 4箇所        | unwrap_usage      | 1 violation (Medium)          |
| T-109 | unit | FR-106 | content に unwrap() 2箇所        | unwrap_usage      | 0 violations                  |
| T-110 | unit | FR-107 | テスト外に todo!()               | todo_macro        | 1 violation (Medium)          |
| T-111 | unit | FR-107 | #[test] fn 内に todo!()          | todo_macro        | 0 violations                  |
| T-112 | unit | FR-108 | file_path = "Cargo.lock"         | cargo_lock        | 1 violation (Critical)        |
| T-113 | unit | FR-108 | file_path = "Cargo.toml"         | cargo_lock        | 0 violations                  |
| T-114 | unit | FR-110 | 2 blocking violations            | format_violations | "2 issues blocked" メッセージ |
| T-115 | unit | FR-111 | ルール個別無効化                 | Config merge      | 無効ルールはスキップ          |

### Phase 3: claude-reviews-rust

| ID    | Type | FR     | Given              | When                | Then                      |
| ----- | ---- | ------ | ------------------ | ------------------- | ------------------------- |
| T-201 | unit | FR-201 | skill: "audit"     | parse_audit_skill   | Some(())                  |
| T-202 | unit | FR-201 | skill: "commit"    | parse_audit_skill   | None                      |
| T-203 | unit | FR-202 | Cargo.toml 存在    | ProjectInfo::detect | has_cargo_toml == true    |
| T-204 | unit | FR-202 | Cargo.toml なし    | ProjectInfo::detect | has_cargo_toml == false   |
| T-205 | unit | FR-209 | 3/5 ツール成功     | build_output        | "3/5 tools reported" JSON |
| T-206 | unit | FR-209 | 全ツール失敗       | build_output        | None                      |
| T-207 | unit | FR-209 | 全ツール成功       | build_output        | "5/5 tools reported" JSON |
| T-208 | unit | FR-210 | 設定で clippy 無効 | Config merge        | tools.clippy == false     |
| T-209 | unit | FR-210 | 設定ファイルなし   | Config::load        | 全ツール有効              |

## Non-Functional Requirements

| ID      | Category    | Requirement                        | Target                | Validates |
| ------- | ----------- | ---------------------------------- | --------------------- | --------- |
| NFR-001 | performance | バイナリサイズ最小化               | release: LTO + strip  | AC-4      |
| NFR-002 | performance | stdin 読み込み上限                 | 10MB                  | AC-4      |
| NFR-003 | reliability | 外部ツール未検出時のグレースフル   | skip + stderr warning | AC-3      |
| NFR-004 | reliability | 設定ファイル不正時のフォールバック | デフォルト設定に戻る  | AC-4      |

## Dependencies

| Type     | Name          | Purpose             | Used By        |
| -------- | ------------- | ------------------- | -------------- |
| external | serde         | JSON シリアライズ   | FR-001~FR-210  |
| external | serde_json    | JSON パース         | FR-001~FR-210  |
| external | regex         | パターンマッチ      | FR-103~FR-108  |
| external | once_cell     | Lazy 正規表現初期化 | FR-103~FR-108  |
| runtime  | cargo fmt     | Rust フォーマッタ   | FR-003         |
| runtime  | cargo clippy  | Rust リンター       | FR-109, FR-203 |
| runtime  | cargo check   | コンパイルチェック  | FR-204         |
| runtime  | cargo test    | テスト実行          | FR-205         |
| runtime  | cargo audit   | 脆弱性スキャン      | FR-206         |
| runtime  | cargo machete | 未使用依存検出      | FR-207         |

## Implementation Checklist

- [ ] Phase 1: claude-formatter-rust (FR-001~FR-004)
- [ ] Phase 2: claude-guardrails-rust (FR-101~FR-111)
- [ ] Phase 3: claude-reviews-rust (FR-201~FR-210)

## Traceability Matrix

| AC   | FR             | Test         | NFR                       |
| ---- | -------------- | ------------ | ------------------------- |
| AC-1 | FR-001~FR-004  | T-001~T-009  | NFR-001, NFR-002          |
| AC-2 | FR-101~FR-111  | T-101~T-115  | NFR-001, NFR-002, NFR-004 |
| AC-3 | FR-201~FR-210  | T-201~T-209  | NFR-001~NFR-004           |
| AC-4 | FR-004,111,210 | T-013, T-014 | NFR-001                   |
