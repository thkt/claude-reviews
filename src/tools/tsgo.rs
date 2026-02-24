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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn skips_without_tsconfig() {
        let info = ProjectInfo {
            root: PathBuf::from("/tmp/nonexistent"),
            has_package_json: true,
            has_tsconfig: false,
            has_react: false,
        };
        let result = run(&info);
        assert!(!result.success);
        assert!(result.output.is_empty());
    }
}
