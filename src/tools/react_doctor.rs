use crate::project::ProjectInfo;
use crate::resolve;
use super::{ToolResult, combine_output};
use std::process::Command;

pub fn run(project: &ProjectInfo) -> ToolResult {
    if !project.has_react {
        return ToolResult::skipped("react-doctor");
    }

    let bin = resolve::resolve_bin("react-doctor", &project.root);

    let output = match Command::new(&bin)
        .args([".", "--verbose"])
        .current_dir(&project.root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("reviews: react-doctor: {}", e);
            return ToolResult::skipped("react-doctor");
        }
    };

    ToolResult {
        name: "react-doctor",
        output: combine_output(&output),
        success: output.status.success(),
    }
}
