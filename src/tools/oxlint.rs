use super::{ToolResult, combine_output};
use crate::project::ProjectInfo;
use crate::resolve;
use std::process::Command;

pub fn run(project: &ProjectInfo) -> ToolResult {
    if !project.has_package_json {
        return ToolResult::skipped("oxlint");
    }

    let bin = resolve::resolve_bin("oxlint", &project.root);

    let output = match Command::new(&bin)
        .args(["--format", "json", "."])
        .current_dir(&project.root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("reviews: oxlint: {}", e);
            return ToolResult::skipped("oxlint");
        }
    };

    ToolResult {
        name: "oxlint",
        output: combine_output(&output),
        success: output.status.success(),
    }
}
