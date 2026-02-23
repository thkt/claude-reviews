# IDR: Rust ツール追加と audit findings 14 件修正

> 2026-02-23

## Summary

`/audit` で検出された 14 件の findings に対応し、Rust 静的解析ツール群（clippy, cargo check, cargo test, cargo audit, machete）を追加。`define_tools!` マクロでツール定義を一元化し、`run_with_timeout` で全ツール実行に 60s タイムアウトと 100KB 出力制限を導入。JS ツールランナーを `run_js_command` 共通化で ~120 行削減。

## Changes

### [src/config.rs](file:////Users/thkt/GitHub/claude-reviews/src/config.rs)

```diff
@@ -1,20 +1,56 @@
+macro_rules! define_tools {
+    ($($field:ident $(=> $rename:literal)?),+ $(,)?) => {
+        #[derive(Debug, Clone)]
+        pub struct ToolsConfig { $(pub $field: bool,)+ }
+        impl Default for ToolsConfig { ... $($field: true,)+ }
+        #[derive(Debug, Deserialize)]
+        struct ProjectToolsConfig { $( $(#[serde(rename = $rename)])? $field: Option<bool>, )+ }
+        impl ToolsConfig { fn apply(&mut self, overrides: &ProjectToolsConfig) { ... } }
+    };
+}
+define_tools! { knip, oxlint, tsgo, react_doctor, clippy, cargo_check => "cargoCheck", ... }
@@ -63,7 +77,11 @@
     fn find_config(start: &Path) -> Option<PathBuf> {
+        let mut depth = 0;
         while let Some(d) = dir {
+            if depth >= MAX_TRAVERSAL_DEPTH { break; }
+            depth += 1;
```

> [!NOTE]
>
> - `define_tools!` マクロで ToolsConfig / ProjectToolsConfig / Default / apply を単一定義から生成
> - `merge()` の field-by-field ロジックを `tools.apply(tools)` 1 行に置換
> - `find_config` に `MAX_TRAVERSAL_DEPTH` (20) 追加
> - テストヘルパー `make_temp_dir` を `test_utils.rs` に移動

> [!TIP]
>
> - **define_tools! マクロ**: ツール追加が 1 箇所で完結する。新ツール追加時の 4+ 箇所同時修正を排除
> - **Not adopted**: HashMap\<String, bool\> — 型安全性を失う。コンパイル時フィールド検証が効かなくなる
> - **Not adopted**: Tool trait + 自己登録 — 現時点では YAGNI。マクロで十分

---

### [src/tools/mod.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/mod.rs)

```diff
@@ -1,10 +1,21 @@
+const TOOL_TIMEOUT: Duration = Duration::from_secs(60);
+const MAX_OUTPUT_SIZE: usize = 102_400; // 100KB
@@ -26,10 +37,83 @@
+fn run_with_timeout(name: &'static str, mut cmd: Command) -> ToolResult {
+    let (tx, rx) = mpsc::channel();
+    std::thread::spawn(move || { let _ = tx.send(cmd.output()); });
+    match rx.recv_timeout(TOOL_TIMEOUT) { ... }
+}
+pub(crate) fn run_cargo_command(...) -> ToolResult { run_with_timeout(name, cmd) }
+pub(crate) fn run_js_command(...) -> ToolResult { run_with_timeout(name, cmd) }
+pub(crate) fn is_command_available(command: &str) -> bool {
+    Command::new("which").arg(command)...
+}
```

> [!NOTE]
>
> - `run_with_timeout`: mpsc channel + `recv_timeout(60s)` でタイムアウト実装
> - `truncate_output`: 100KB 超の出力を切り詰め
> - `combine_output`: stdout 空時の先頭改行修正、中間変数排除で直接 sanitize 呼び出し
> - `is_command_available`: `sh -c "command -v"` → `which` 直接実行で shell injection 排除
> - `run_js_command`: JS/TS ツール共通ランナー追加

> [!TIP]
>
> - **mpsc channel 方式**: 外部 crate 不要で std のみでタイムアウト実現
> - **Not adopted**: `wait-timeout` crate — 依存追加を避けた。タイムアウト時の child kill は v2 で対応
> - **Not adopted**: CommandRunner trait — テスタビリティ向上するが現時点では YAGNI

---

### [src/main.rs](file:////Users/thkt/GitHub/claude-reviews/src/main.rs)

```diff
@@ -19,36 +21,38 @@
-fn parse_audit_skill(input: &str) -> Option<()> {
+fn is_audit_skill(input: &str) -> bool {
@@ -57,23 +54,23 @@
-    .take(MAX_INPUT_SIZE as u64)
+    .take((MAX_INPUT_SIZE + 1) as u64)
-    if bytes_read == MAX_INPUT_SIZE {
+    if bytes_read > MAX_INPUT_SIZE {
-        return;
+        std::process::exit(1);
@@ -34,9 +35,6 @@
-    if successful.is_empty() { return None; }
+    // results.is_empty() のみ None を返す（全ツール失敗時も JSON 出力）
```

> [!NOTE]
>
> - `parse_audit_skill` → `is_audit_skill`: `Option<()>` を `bool` に変更（意図が明確に）
> - Off-by-one 修正: `.take(N+1)` + `> N` で境界値を正しく処理
> - エラー時 `std::process::exit(1)` で hook consumer にエラーを伝達
> - `build_output`: 全ツール失敗時も JSON 返却（`0/N tools reported`）

> [!TIP]
>
> - **exit(1) 方式**: hook consumer がエラー検出可能に。非 audit スキルと disabled は従来通り exit(0)
> - **build_output always Some**: advisory-only モデルを維持しつつ、全失敗時の visibility を確保

---

### [src/project.rs](file:////Users/thkt/GitHub/claude-reviews/src/project.rs)

```diff
@@ -1,5 +1,7 @@
+const MAX_TRAVERSAL_DEPTH: usize = 20;
@@ -4,6 +6,7 @@
+    pub has_cargo_toml: bool,
@@ -29,8 +34,14 @@
     fn find_root(start: &Path) -> PathBuf {
+        let mut depth = 0;
+        if depth >= MAX_TRAVERSAL_DEPTH { break; }
+        depth += 1;
```

> [!NOTE]
>
> - `has_cargo_toml` フィールド追加で Rust プロジェクト検出
> - `find_root` に `MAX_TRAVERSAL_DEPTH` 追加（resolve.rs と一貫性）
> - テストヘルパーを `test_utils.rs` に移動

---

### [src/resolve.rs](file:////Users/thkt/GitHub/claude-reviews/src/resolve.rs)

```diff
@@ -13,6 +14,7 @@
         if candidate.exists() {
+            eprintln!("reviews: resolved {} -> {}", name, candidate.display());
             return candidate;
```

> [!NOTE]
>
> - バイナリパス解決時にログ出力追加（監査可能性向上）
> - テストヘルパーを `test_utils.rs` に移動

---

### [src/test_utils.rs](file:////Users/thkt/GitHub/claude-reviews/src/test_utils.rs)

> [!NOTE]
>
> - 新規ファイル: 3 モジュールで重複していた `make_temp_dir` を共通化

---

### [src/tools/knip.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/knip.rs), [oxlint.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/oxlint.rs), [tsgo.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/tsgo.rs), [react_doctor.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/react_doctor.rs)

> [!NOTE]
>
> - 4 ファイルとも `Command::new` 直接実行 → `super::run_js_command` 呼び出しに統一
> - ボイラープレート（match Command + ToolResult 構築）を排除

---

### [src/tools/audit.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/audit.rs), [cargo_check.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/cargo_check.rs), [cargo_test.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/cargo_test.rs), [clippy.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/clippy.rs), [machete.rs](file:////Users/thkt/GitHub/claude-reviews/src/tools/machete.rs)

> [!NOTE]
>
> - 新規ファイル: Rust 静的解析ツール 5 種
> - `audit`, `machete` は `is_command_available` で存在チェック後に実行
> - 全て `run_cargo_command` 共通ランナー使用

---

### git diff --stat

```
 src/config.rs             | 111 ++++++++++++++++++++++++----------------------
 src/main.rs               |  84 ++++++++++++++++++++++-------------
 src/project.rs            |  49 +++++++++++++++-----
 src/resolve.rs            |  13 +-----
 src/test_utils.rs         |  12 +++++
 src/tools/audit.rs        |  33 ++++++++++++++
 src/tools/cargo_check.rs  |  29 ++++++++++++
 src/tools/cargo_test.rs   |  29 ++++++++++++
 src/tools/clippy.rs       |  39 ++++++++++++++++
 src/tools/knip.rs         |  27 +++--------
 src/tools/machete.rs      |  33 ++++++++++++++
 src/tools/mod.rs          |  94 ++++++++++++++++++++++++++++++++++++---
 src/tools/oxlint.rs       |  22 +--------
 src/tools/react_doctor.rs |  22 +--------
 src/tools/tsgo.rs         |  22 +--------
 15 files changed, 427 insertions(+), 192 deletions(-)
```
