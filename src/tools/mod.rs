pub mod knip;
pub mod oxlint;
pub mod react_doctor;
pub mod tsgo;

use std::process::Output;
use crate::sanitize;

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
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else {
        format!("{}\n{}", stdout, stderr)
    };
    sanitize::sanitize(&combined)
}
