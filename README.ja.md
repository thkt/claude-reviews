# claude-reviews

[English](README.md) | 日本語

Claude Code の `/audit` コマンド実行前に静的解析ツールを走らせ、結果を監査エージェントにコンテキストとして渡す [Claude Code hook](https://docs.anthropic.com/en/docs/claude-code/hooks)。エージェントが手動でコードを読む代わりに、リンター出力・型エラー・テスト結果を事前に取得できる。

## 仕組み

```
/audit → PreToolUse hook 発火 → reviews バイナリ実行
  ├─ プロジェクト種別を検出（package.json, Cargo.toml など）
  ├─ 該当ツールを OS スレッドで並列実行
  └─ ツール出力を additionalContext として JSON 返却
        → 監査エージェントが実際の静的解析結果を参照
```

hook は**アドバイザリー専用**：常にツール呼び出しを承認し、`/audit` をブロックしない。ツールの失敗や未インストールは静かにスキップされる。

## 特徴

- **並列実行**: 有効な全ツールを OS スレッドで同時実行
- **フェイルオープン設計**: エラーが `/audit` をブロックしない
- **自動検出**: プロジェクトに該当するツールのみ実行（package.json, tsconfig.json, React, Cargo.toml）
- **バイナリ解決**: JS/TS ツールを `node_modules/.bin` から `.git` 境界まで探索

## インストール

### Homebrew（推奨）

```bash
brew install thkt/tap/reviews
```

### リリースバイナリ

[Releases](https://github.com/thkt/claude-reviews/releases) から最新バイナリをダウンロード：

```bash
# macOS (Apple Silicon)
curl -L https://github.com/thkt/claude-reviews/releases/latest/download/reviews-aarch64-apple-darwin.tar.gz | tar xz
mv reviews ~/.local/bin/
```

### ソースから

```bash
cd /tmp
git clone https://github.com/thkt/claude-reviews.git
cd claude-reviews
cargo build --release
cp target/release/reviews ~/.local/bin/
cd .. && rm -rf claude-reviews
```

## 使い方

`~/.claude/settings.json` に追加：

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

`/audit` が呼ばれると、hook は以下を実行する：

1. stdin から Skill ツール入力を読み取り
2. skill が `audit` か確認（それ以外は無出力で終了）
3. プロジェクト種別を検出し、該当ツールを並列実行
4. ツール結果を `additionalContext` として JSON 出力

## ツール

### JS/TS

| ツール                                                         | 条件                   | 引数                             |
| -------------------------------------------------------------- | ---------------------- | -------------------------------- |
| [knip](https://knip.dev)                                       | `package.json` あり    | `--reporter json --no-exit-code` |
| [oxlint](https://oxc.rs)                                       | `package.json` あり    | `--format json .`                |
| [tsgo](https://github.com/nicolo-ribaudo/tsgo)                 | `tsconfig.json` あり   | `--noEmit`                       |
| [react-doctor](https://github.com/nicolo-ribaudo/react-doctor) | React が依存関係に存在 | `. --verbose`                    |

JS/TS ツールはまず `node_modules/.bin` から解決し、見つからなければ `$PATH` にフォールバック。

### Rust

| ツール                                                   | 条件                               | 引数                                              |
| -------------------------------------------------------- | ---------------------------------- | ------------------------------------------------- |
| [clippy](https://doc.rust-lang.org/clippy/)              | `Cargo.toml` あり                  | `clippy --message-format=short -- -W clippy::all` |
| cargo check                                              | `Cargo.toml` あり                  | `check --message-format=short`                    |
| cargo test                                               | `Cargo.toml` あり                  | `test --no-fail-fast`                             |
| [cargo-audit](https://rustsec.org)                       | `Cargo.toml` あり + インストール済 | `audit`                                           |
| [cargo-machete](https://github.com/bnjbvr/cargo-machete) | `Cargo.toml` あり + インストール済 | `machete`                                         |

cargo-audit と cargo-machete は別途インストールが必要（`cargo install cargo-audit cargo-machete`）。

未インストールのツールは静かにスキップされる。

## 設定

プロジェクトルート（`.git/` の隣）に `.claude-reviews.json` を配置。全フィールド省略可 — 上書きしたい項目のみ指定。

**デフォルト**（設定ファイル不要）: 全ツール有効。

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

### 例

**特定ツールを無効化：**

```json
{
  "tools": {
    "tsgo": false
  }
}
```

**プロジェクト単位で無効化：**

```json
{
  "enabled": false
}
```

### 設定ファイルの解決

設定ファイルは `$CWD` から最も近い `.git` ディレクトリまで上方向に探索される。

## ライセンス

MIT
