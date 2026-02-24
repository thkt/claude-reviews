use super::ToolResult;
use crate::project::ProjectInfo;
use crate::resolve;

pub fn run(project: &ProjectInfo) -> ToolResult {
    let bin = resolve::resolve_bin("oxlint", &project.root);
    super::run_js_command("oxlint", &bin, &["--format", "json"], project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn runs_without_package_json() {
        let info = ProjectInfo {
            root: PathBuf::from("/tmp/nonexistent"),
            has_package_json: false,
            has_tsconfig: false,
            has_react: false,
        };
        let result = run(&info);
        assert_eq!(result.name, "oxlint");
    }
}
