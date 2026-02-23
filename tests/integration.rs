use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("reviews-integ-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run_reviews_in(dir: &std::path::Path, input: &str) -> (String, String, bool) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_reviews"))
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn reviews");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

fn run_reviews(input: &str) -> (String, String, bool) {
    let tmp = TempDir::new("default");
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    run_reviews_in(tmp.path(), input)
}

#[test]
fn non_audit_skill_exits_silently() {
    let input = r#"{"tool_name": "Skill", "tool_input": {"skill": "commit"}}"#;
    let (stdout, _, success) = run_reviews(input);
    assert!(success);
    assert!(stdout.is_empty());
}

#[test]
fn invalid_json_exits_silently() {
    let (stdout, _, success) = run_reviews("not json{{{");
    assert!(success);
    assert!(stdout.is_empty());
}

#[test]
fn empty_input_exits_silently() {
    let (stdout, _, success) = run_reviews("");
    assert!(success);
    assert!(stdout.is_empty());
}

#[test]
fn disabled_config_exits_silently() {
    let tmp = TempDir::new("disabled");
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    std::fs::write(
        tmp.path().join(".claude-reviews.json"),
        r#"{"enabled": false}"#,
    )
    .unwrap();

    let input = r#"{"tool_name": "Skill", "tool_input": {"skill": "audit"}}"#;
    let (stdout, _, success) = run_reviews_in(tmp.path(), input);
    assert!(success);
    assert!(stdout.is_empty());
}

#[test]
fn audit_skill_produces_valid_json() {
    let tmp = TempDir::new("audit-json");
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    // Empty project with no tools installed — all tools will be skipped

    let input = r#"{"tool_name": "Skill", "tool_input": {"skill": "audit"}}"#;
    let (stdout, _, success) = run_reviews_in(tmp.path(), input);
    assert!(success);

    if !stdout.is_empty() {
        let parsed: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("invalid JSON output: {e}\nstdout: {stdout}"));
        assert_eq!(parsed["decision"], "approve");
        assert!(parsed["reason"].as_str().is_some());
    }
}
