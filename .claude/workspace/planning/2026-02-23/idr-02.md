# IDR: build_output の単一使用変数インライン化

> 2026-02-23

## Summary

`build_output` 内の `total` と `count` 変数を削除し、`results.len()` と `reported.len()` を format マクロ内で直接呼び出すように変更。いずれも 1 回のみ使用される O(1) 呼び出しで、変数名が意味的価値を追加していなかった。

## Changes

### [src/main.rs](file:////Users/thkt/GitHub/claude-reviews/src/main.rs)

```diff
@@ -38,9 +38,6 @@ fn build_output(results: &[tools::ToolResult]) -> Option<String> {
         .filter(|r| r.success && !r.output.is_empty())
         .collect();

-    let total = results.len();
-    let count = reported.len();
-
     let mut context = String::from("# Pre-flight Analysis Results\n\n");
@@ -52,7 +49,7 @@
-        "reason": format!("Pre-flight: {}/{} tools reported", count, total),
+        "reason": format!("Pre-flight: {}/{} tools reported", reported.len(), results.len()),
```

> [!NOTE]
>
> - `total` (`results.len()`) と `count` (`reported.len()`) を format マクロ内にインライン化
> - 両方とも 1 回のみ使用、O(1) 操作のため性能影響なし

> [!TIP]
>
> - **インライン化**: 変数名 `total`/`count` は `results`/`reported` から自明であり命名的価値が低い
> - **Not adopted**: 変数維持 — 可読性の向上が見られないため不採用

---

### git diff --stat

```
 src/main.rs | 5 +----
 1 file changed, 1 insertion(+), 4 deletions(-)
```
