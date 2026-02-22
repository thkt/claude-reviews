use crate::project::ProjectInfo;
use crate::resolve;
use super::{ToolResult, combine_output};
use std::process::Command;

pub fn run(project: &ProjectInfo) -> ToolResult {
    if !project.has_tsconfig {
        return ToolResult::skipped("tsgo");
    }

    let bin = resolve::resolve_bin("tsgo", &project.root);

    let output = match Command::new(&bin)
        .arg("--noEmit")
        .current_dir(&project.root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("reviews: tsgo: {}", e);
            return ToolResult::skipped("tsgo");
        }
    };

    ToolResult {
        name: "tsgo",
        output: combine_output(&output),
        success: output.status.success(),
    }
}
