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

    /// T-009: no config file → all tools enabled (default)
    #[test]
    fn t009_no_config_file_returns_all_defaults() {
        let dir = temp_dir("t009");

        // No .claude-reviews.json exists
        let config = Config::load(&dir);

        assert!(config.enabled);
        assert!(config.tools.knip);
        assert!(config.tools.oxlint);
        assert!(config.tools.tsgo);
        assert!(config.tools.react_doctor);

        cleanup(&dir);
    }

    /// T-010: partial config {tools: {knip: false}} → knip disabled, others default
    #[test]
    fn t010_partial_config_merges_with_defaults() {
        let dir = temp_dir("t010");

        let config_path = dir.join(".claude-reviews.json");
        fs::write(
            &config_path,
            r#"{"tools": {"knip": false}}"#,
        )
        .unwrap();

        let config = Config::load(&dir);

        assert!(config.enabled);
        assert!(!config.tools.knip);
        assert!(config.tools.oxlint);
        assert!(config.tools.tsgo);
        assert!(config.tools.react_doctor);

        cleanup(&dir);
    }

    /// T-011: enabled: false → all skip
    #[test]
    fn t011_enabled_false_disables_everything() {
        let dir = temp_dir("t011");

        let config_path = dir.join(".claude-reviews.json");
        fs::write(&config_path, r#"{"enabled": false}"#).unwrap();

        let config = Config::load(&dir);

        assert!(!config.enabled);

        cleanup(&dir);
    }

    /// T-012: invalid JSON config → stderr warning + default used
    #[test]
    fn t012_invalid_json_falls_back_to_defaults() {
        let dir = temp_dir("t012");

        let config_path = dir.join(".claude-reviews.json");
        fs::write(&config_path, "{ this is not valid json }").unwrap();

        let config = Config::load(&dir);

        // Should fall back to defaults despite invalid JSON
        assert!(config.enabled);
        assert!(config.tools.knip);
        assert!(config.tools.oxlint);
        assert!(config.tools.tsgo);
        assert!(config.tools.react_doctor);

        cleanup(&dir);
    }
}
