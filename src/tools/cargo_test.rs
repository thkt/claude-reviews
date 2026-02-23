use super::{run_cargo_command, ToolResult};
use crate::project::ProjectInfo;

pub fn run(info: &ProjectInfo) -> ToolResult {
    if !info.has_cargo_toml {
        return ToolResult::skipped("test");
    }
    run_cargo_command("test", &["test", "--no-fail-fast"], info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn skips_without_cargo_toml() {
        let info = ProjectInfo {
            root: PathBuf::from("/tmp/nonexistent"),
            has_package_json: false,
            has_tsconfig: false,
            has_react: false,
            has_cargo_toml: false,
        };
        let result = run(&info);
        assert!(!result.success);
        assert!(result.output.is_empty());
    }
}
