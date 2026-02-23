use super::ToolResult;
use crate::project::ProjectInfo;
use crate::resolve;

pub fn run(project: &ProjectInfo) -> ToolResult {
    if !project.has_package_json {
        return ToolResult::skipped("knip");
    }

    let bin = resolve::resolve_bin("knip", &project.root);
    super::run_js_command(
        "knip",
        &bin,
        &["--reporter", "json", "--no-exit-code"],
        project,
    )
}
