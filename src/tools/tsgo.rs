use super::ToolResult;
use crate::project::ProjectInfo;
use crate::resolve;

pub fn run(project: &ProjectInfo) -> ToolResult {
    if !project.has_tsconfig {
        return ToolResult::skipped("tsgo");
    }

    let bin = resolve::resolve_bin("tsgo", &project.root);
    super::run_js_command("tsgo", &bin, &["--noEmit"], project)
}
