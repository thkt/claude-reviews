use crate::project::ProjectInfo;
use crate::resolve;
use super::{ToolResult, combine_output};
use std::process::Command;

pub fn run(project: &ProjectInfo) -> ToolResult {
    if !project.has_package_json {
        return ToolResult::skipped("knip");
    }

    let bin = resolve::resolve_bin("knip", &project.root);

    let output = match Command::new(&bin)
        .args(["--reporter", "json", "--no-exit-code"])
        .current_dir(&project.root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("reviews: knip: {}", e);
            return ToolResult::skipped("knip");
        }
    };

    ToolResult {
        name: "knip",
        output: combine_output(&output),
        success: output.status.success(),
    }
}
