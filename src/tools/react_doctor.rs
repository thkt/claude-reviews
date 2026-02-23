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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn skips_without_react() {
        let info = ProjectInfo {
            root: PathBuf::from("/tmp/nonexistent"),
            has_package_json: true,
            has_tsconfig: false,
            has_react: false,
            has_cargo_toml: false,
        };
        let result = run(&info);
        assert!(!result.success);
        assert!(result.output.is_empty());
    }
}
