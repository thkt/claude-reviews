use super::ToolResult;
use crate::project::ProjectInfo;
use crate::resolve;

pub fn run(project: &ProjectInfo) -> ToolResult {
    if !project.has_package_json {
        return ToolResult::skipped("oxlint");
    }

    let bin = resolve::resolve_bin("oxlint", &project.root);
    super::run_js_command("oxlint", &bin, &["--format", "json", "."], project)
}
