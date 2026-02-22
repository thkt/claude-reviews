#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Create a unique temp directory for test isolation
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("claude-reviews-test")
            .join(name)
            .join(format!("{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    /// Clean up temp directory after test
    fn cleanup(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    /// T-006: package.json exists → has_package_json == true
    #[test]
    fn t006_detects_package_json() {
        let dir = temp_dir("t006");

        fs::write(dir.join("package.json"), r#"{"name": "test"}"#).unwrap();

        let info = ProjectInfo::detect(&dir);

        assert!(info.has_package_json);

        cleanup(&dir);
    }

    /// T-007: no package.json → has_package_json == false
    #[test]
    fn t007_no_package_json() {
        let dir = temp_dir("t007");

        // Empty directory, no package.json
        let info = ProjectInfo::detect(&dir);

        assert!(!info.has_package_json);

        cleanup(&dir);
    }

    /// T-008: package.json with react dependency → has_react == true
    #[test]
    fn t008_detects_react_dependency() {
        let dir = temp_dir("t008");

        fs::write(
            dir.join("package.json"),
            r#"{
                "name": "test",
                "dependencies": {
                    "react": "^19.0.0",
                    "react-dom": "^19.0.0"
                }
            }"#,
        )
        .unwrap();

        let info = ProjectInfo::detect(&dir);

        assert!(info.has_package_json);
        assert!(info.has_react);

        cleanup(&dir);
    }
}
