# SOW: Rust Project Hooks (formatter / guardrails / reviews)

## Status

draft

## Overview

| Field     | Value                                                                     |
| --------- | ------------------------------------------------------------------------- |
| Purpose   | Rust プロジェクト用の Claude Code フックツール3種を新規作成               |
| Target    | 3つの独立 Rust バイナリ（formatter-rust, guardrails-rust, reviews-rust）  |
| Approach  | 既存フロントエンド版の設計パターンを踏襲、Rust エコシステムのツールに置換 |
| Reference | claude-formatter, claude-guardrails, claude-reviews（JS/TS版）            |

## Background

現在 Claude Code のフックツール（formatter, guardrails, reviews）は JavaScript/TypeScript プロジェクト専用。Rust プロジェクトでも同等の自動品質チェックを行いたい。

既存3ツールの共通設計パターン:

- stdin JSON 入力（10MB上限）
- `.git` 境界での設定ファイル探索
- exit code による成功/失敗通知
- プロジェクトローカル設定のマージ

## Scope

### In Scope

| Target                 | Change                                            | Files |
| ---------------------- | ------------------------------------------------- | ----- |
| claude-formatter-rust  | `cargo fmt` ラッパー（PostToolUse）               | ~5    |
| claude-guardrails-rust | `cargo clippy` + カスタムルール6種（PreToolCall） | ~15   |
| claude-reviews-rust    | 5ツール並列実行（Skill /audit）                   | ~12   |

### Out of Scope

- 既存フロントエンド版ツールの変更
- Rust 以外の言語サポート追加
- IDE/エディタ統合
- CI/CD パイプライン統合

### YAGNI Checklist

- [ ] Complex permission management
- [ ] Analytics/monitoring dashboards
- [ ] Caching layers
- [ ] Multi-tenant / API versioning
- [ ] Real-time notifications
- [ ] Batch processing / scheduled jobs

## Acceptance Criteria

### AC-1: claude-formatter-rust

- [ ] Write/Edit/MultiEdit で `.rs` ファイル変更時に `cargo fmt` が自動実行される
- [ ] `.rs` 以外のファイルはスキップされる
- [ ] `.claude-formatter-rust.json` で enabled/disabled を切り替えられる
- [ ] フォーマッタ失敗時も exit 0 で終了する（ブロックしない）
- [ ] `cargo fmt` が PATH に見つからない場合はサイレントスキップ

### AC-2: claude-guardrails-rust

- [ ] Write/Edit/MultiEdit で `.rs` ファイル変更時にカスタムルールが実行される
- [ ] 以下のカスタムルールが動作する:
  - `sensitive_file`: .env, .pem, .key 等を検出しブロック
  - `generated_file`: .generated.rs, /generated/ 等を検出し警告
  - `unsafe_usage`: `unsafe` ブロックを検出し警告
  - `unwrap_usage`: `.unwrap()` の過度な使用（3箇所以上）を検出し警告
  - `todo_macro`: `todo!()`, `unimplemented!()` をテスト外で検出し警告
  - `cargo_lock`: Cargo.lock の直接編集を検出しブロック
- [ ] `.claude-guardrails-rust.json` で各ルールの有効/無効を設定できる
- [ ] severity 設定で block_on を設定できる（デフォルト: critical, high）
- [ ] `cargo clippy` が利用可能な場合は外部リントも実行される

### AC-3: claude-reviews-rust

- [ ] `/audit` Skill 実行時に5ツールが並列実行される:
  - `cargo clippy` — リント
  - `cargo check` — コンパイルチェック
  - `cargo test` — テスト実行
  - `cargo audit` — 依存脆弱性スキャン（インストール時のみ）
  - `cargo machete` — 未使用依存検出（インストール時のみ）
- [ ] 各ツールは Cargo.toml 存在時のみ実行される
- [ ] ツール未インストール時はスキップ（エラーにしない）
- [ ] 結果は JSON で stdout に出力される
- [ ] `.claude-reviews-rust.json` で各ツールの有効/無効を設定できる

### AC-4: 共通仕様

- [ ] 3ツールとも release ビルドで最適化（LTO, strip）
- [ ] 各ツールに単体テストが存在する
- [ ] `cargo test` が全ツールで pass する
- [ ] `cargo clippy` が全ツールで warning なし

## Implementation Plan

### Phase 1: claude-formatter-rust

1. プロジェクト初期化（Cargo.toml, .gitignore）
2. stdin JSON パース（Write/Edit/MultiEdit 対応）
3. `.rs` ファイル判定
4. `cargo fmt` 実行
5. config モジュール（`.claude-formatter-rust.json` 探索・マージ）
6. テスト追加

### Phase 2: claude-guardrails-rust

1. プロジェクト初期化
2. stdin JSON パース + file_path/content 抽出
3. Rule trait / Violation 構造体設計
4. カスタムルール6種実装
5. `cargo clippy` 外部リンター統合
6. reporter モジュール（violation/warning 出力整形）
7. config モジュール
8. テスト追加

### Phase 3: claude-reviews-rust

1. プロジェクト初期化
2. stdin JSON パース（audit skill フィルタ）
3. プロジェクト情報検出（Cargo.toml 有無）
4. 5ツールの実行モジュール
5. 並列実行（std::thread）
6. 結果 JSON 出力
7. config モジュール
8. テスト追加

## Test Plan

| Test | AC   | Target                     | Verification                 |
| ---- | ---- | -------------------------- | ---------------------------- |
| T-1  | AC-1 | formatter: rs ファイル     | cargo fmt が実行される       |
| T-2  | AC-1 | formatter: 非 rs ファイル  | スキップされる               |
| T-3  | AC-1 | formatter: config          | enabled: false でスキップ    |
| T-4  | AC-2 | guardrails: sensitive_file | .env 検出でブロック          |
| T-5  | AC-2 | guardrails: unsafe_usage   | unsafe ブロック検出で警告    |
| T-6  | AC-2 | guardrails: unwrap_usage   | 3箇所以上で警告              |
| T-7  | AC-2 | guardrails: todo_macro     | テスト外の todo!() で警告    |
| T-8  | AC-2 | guardrails: cargo_lock     | Cargo.lock 編集でブロック    |
| T-9  | AC-2 | guardrails: config         | ルール個別無効化が動作       |
| T-10 | AC-3 | reviews: 並列実行          | 5ツール並列で結果収集        |
| T-11 | AC-3 | reviews: ツール未検出      | スキップしてエラーにならない |
| T-12 | AC-3 | reviews: JSON 出力         | 正しい JSON 構造             |
| T-13 | AC-4 | 全ツール: cargo test       | テスト全 pass                |
| T-14 | AC-4 | 全ツール: cargo clippy     | warning なし                 |

## Risks

| Risk                                   | Impact | Mitigation                             |
| -------------------------------------- | ------ | -------------------------------------- |
| cargo audit/machete 未インストール環境 | LOW    | which で検出、未インストール時スキップ |
| cargo fmt が workspace で動作差異      | MED    | --manifest-path で明示的にパス指定     |
| clippy の JSON 出力フォーマット変更    | LOW    | --message-format=json は安定 API       |
| 3リポの config コード重複              | LOW    | 各80行程度、許容範囲                   |
