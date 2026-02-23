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
use std::sync::mpsc;
use std::time::Duration;

const TOOL_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_OUTPUT_SIZE: usize = 102_400; // 100KB

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
        let mut truncated = s[..MAX_OUTPUT_SIZE].to_string();
        truncated.push_str("\n[output truncated]");
        truncated
    }
}

fn run_with_timeout(name: &'static str, mut cmd: Command) -> ToolResult {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(cmd.output());
    });

    match rx.recv_timeout(TOOL_TIMEOUT) {
        Ok(Ok(output)) => ToolResult {
            name,
            success: output.status.success(),
            output: combine_output(&output),
        },
        Ok(Err(e)) => {
            eprintln!("reviews: {} execution error: {}", name, e);
            ToolResult::skipped(name)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!(
                "reviews: {} timed out after {}s",
                name,
                TOOL_TIMEOUT.as_secs()
            );
            ToolResult::skipped(name)
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            eprintln!("reviews: {} thread error", name);
            ToolResult::skipped(name)
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
    Command::new("which")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
