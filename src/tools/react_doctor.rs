use super::ToolResult;
use crate::project::ProjectInfo;
use crate::resolve;

pub fn run(project: &ProjectInfo) -> ToolResult {
    if !project.has_react {
        return ToolResult::skipped("react-doctor");
    }

    let bin = resolve::resolve_bin("react-doctor", &project.root);
    super::run_js_command("react-doctor", &bin, &[".", "--verbose"], project)
}
