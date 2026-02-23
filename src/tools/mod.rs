pub mod audit;
pub mod cargo_check;
pub mod cargo_test;
pub mod clippy;
pub mod knip;
pub mod machete;
pub mod oxlint;
pub mod react_doctor;
pub mod tsgo;

use crate::sanitize;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

const TOOL_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_OUTPUT_SIZE: usize = 102_400;

#[derive(Debug)]
pub struct ToolResult {
    pub name: &'static str,
    pub output: String,
    pub success: bool,
}

impl ToolResult {
    pub fn skipped(name: &'static str) -> Self {
        Self {
            name,
            output: String::new(),
            success: false,
        }
    }
}

pub fn combine_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let sanitized = if stdout.is_empty() {
        sanitize::sanitize(&stderr)
    } else if stderr.is_empty() {
        sanitize::sanitize(&stdout)
    } else {
        sanitize::sanitize(&format!("{}\n{}", stdout, stderr))
    };
    truncate_output(&sanitized)
}

fn truncate_output(s: &str) -> String {
    if s.len() <= MAX_OUTPUT_SIZE {
        s.to_string()
    } else {
        let mut truncated = s[..s.floor_char_boundary(MAX_OUTPUT_SIZE)].to_string();
        truncated.push_str("\n[output truncated]");
        truncated
    }
}

fn run_with_timeout(name: &'static str, mut cmd: Command) -> ToolResult {
    let mut child = match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("reviews: {} spawn error: {}", name, e);
            return ToolResult::skipped(name);
        }
    };

    let deadline = std::time::Instant::now() + TOOL_TIMEOUT;
    let poll_interval = Duration::from_millis(100);

    loop {
        match child.try_wait() {
            Ok(Some(_)) => match child.wait_with_output() {
                Ok(output) => {
                    return ToolResult {
                        name,
                        success: output.status.success(),
                        output: combine_output(&output),
                    };
                }
                Err(e) => {
                    eprintln!("reviews: {} output read error: {}", name, e);
                    return ToolResult::skipped(name);
                }
            },
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    eprintln!(
                        "reviews: {} timed out after {}s, killing process",
                        name,
                        TOOL_TIMEOUT.as_secs()
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                    return ToolResult::skipped(name);
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => {
                eprintln!("reviews: {} wait error: {}", name, e);
                return ToolResult::skipped(name);
            }
        }
    }
}

pub(crate) fn run_cargo_command(
    name: &'static str,
    args: &[&str],
    info: &crate::project::ProjectInfo,
) -> ToolResult {
    let mut cmd = Command::new("cargo");
    cmd.args(args).current_dir(&info.root);
    run_with_timeout(name, cmd)
}

pub(crate) fn run_js_command(
    name: &'static str,
    bin: &Path,
    args: &[&str],
    info: &crate::project::ProjectInfo,
) -> ToolResult {
    let mut cmd = Command::new(bin);
    cmd.args(args).current_dir(&info.root);
    run_with_timeout(name, cmd)
}

pub(crate) fn is_command_available(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("command -v \"$0\"")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    #[test]
    fn skipped_result_is_empty_and_failed() {
        let r = ToolResult::skipped("test-tool");
        assert_eq!(r.name, "test-tool");
        assert!(!r.success);
        assert!(r.output.is_empty());
    }

    #[test]
    fn combine_output_stdout_only() {
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: b"hello world".to_vec(),
            stderr: vec![],
        };
        assert_eq!(combine_output(&output), "hello world");
    }

    #[test]
    fn combine_output_stderr_only() {
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: b"error msg".to_vec(),
        };
        assert_eq!(combine_output(&output), "error msg");
    }

    #[test]
    fn combine_output_both_streams() {
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: b"out".to_vec(),
            stderr: b"err".to_vec(),
        };
        assert_eq!(combine_output(&output), "out\nerr");
    }

    #[test]
    fn combine_output_truncates_large_output() {
        let big = "x".repeat(MAX_OUTPUT_SIZE + 1000);
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: big.into_bytes(),
            stderr: vec![],
        };
        let result = combine_output(&output);
        assert!(result.len() <= MAX_OUTPUT_SIZE + 50);
        assert!(result.ends_with("[output truncated]"));
    }

    #[test]
    fn run_with_timeout_success() {
        let mut cmd = Command::new("echo");
        cmd.arg("hello");
        let result = run_with_timeout("echo-test", cmd);
        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[test]
    fn run_with_timeout_handles_missing_command() {
        let cmd = Command::new("nonexistent-command-12345");
        let result = run_with_timeout("missing", cmd);
        assert!(!result.success);
        assert!(result.output.is_empty());
    }

    #[test]
    fn run_with_timeout_captures_exit_code() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "echo fail >&2; exit 1"]);
        let result = run_with_timeout("fail-test", cmd);
        assert!(!result.success);
        assert!(result.output.contains("fail"));
    }

    #[test]
    fn is_command_available_for_existing() {
        assert!(is_command_available("sh"));
    }

    #[test]
    fn is_command_available_for_missing() {
        assert!(!is_command_available("nonexistent-tool-xyz-99999"));
    }
}
