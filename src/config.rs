use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub enabled: bool,
    pub tools: ToolsConfig,
}

#[derive(Debug, Clone)]
pub struct ToolsConfig {
    pub knip: bool,
    pub oxlint: bool,
    pub tsgo: bool,
    pub react_doctor: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            tools: ToolsConfig::default(),
        }
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            knip: true,
            oxlint: true,
            tsgo: true,
            react_doctor: true,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProjectConfig {
    enabled: Option<bool>,
    tools: Option<ProjectToolsConfig>,
}

#[derive(Debug, Deserialize)]
struct ProjectToolsConfig {
    knip: Option<bool>,
    oxlint: Option<bool>,
    tsgo: Option<bool>,
    react_doctor: Option<bool>,
}

const CONFIG_FILE: &str = ".claude-reviews.json";

impl Config {
    /// Load config by searching from `start` up to .git root.
    pub fn load(start: &Path) -> Self {
        let default = Self::default();
        let Some(config_path) = Self::find_config(start) else {
            return default;
        };
        Self::load_from(&config_path, default)
    }

    fn find_config(start: &Path) -> Option<PathBuf> {
        let mut dir = Some(start.to_path_buf());
        while let Some(d) = dir {
            let candidate = d.join(CONFIG_FILE);
            if candidate.exists() {
                return Some(candidate);
            }
            if d.join(".git").exists() {
                break;
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
        None
    }

    fn load_from(path: &Path, default: Config) -> Config {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("reviews: warning: failed to read config: {}", e);
                return default;
            }
        };
        let project: ProjectConfig = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("reviews: warning: invalid config JSON: {}", e);
                return default;
            }
        };
        default.merge(project)
    }

    fn merge(mut self, project: ProjectConfig) -> Self {
        if let Some(enabled) = project.enabled {
            self.enabled = enabled;
        }
        if let Some(tools) = project.tools {
            if let Some(v) = tools.knip {
                self.tools.knip = v;
            }
            if let Some(v) = tools.oxlint {
                self.tools.oxlint = v;
            }
            if let Some(v) = tools.tsgo {
                self.tools.tsgo = v;
            }
            if let Some(v) = tools.react_doctor {
                self.tools.react_doctor = v;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("claude-reviews-test-{}-{}", prefix, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn default_config_all_tools_enabled() {
        let tmp = make_temp_dir("config-default");
        fs::create_dir_all(tmp.join(".git")).unwrap();

        let config = Config::load(&tmp);
        assert!(config.enabled);
        assert!(config.tools.knip);
        assert!(config.tools.oxlint);
        assert!(config.tools.tsgo);
        assert!(config.tools.react_doctor);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn partial_config_merges_with_defaults() {
        let tmp = make_temp_dir("config-partial");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(
            tmp.join(CONFIG_FILE),
            r#"{"tools": {"knip": false}}"#,
        ).unwrap();

        let config = Config::load(&tmp);
        assert!(config.enabled);
        assert!(!config.tools.knip);
        assert!(config.tools.oxlint);
        assert!(config.tools.tsgo);
        assert!(config.tools.react_doctor);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn enabled_false_disables_all() {
        let tmp = make_temp_dir("config-disabled");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(
            tmp.join(CONFIG_FILE),
            r#"{"enabled": false}"#,
        ).unwrap();

        let config = Config::load(&tmp);
        assert!(!config.enabled);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn invalid_json_falls_back_to_default() {
        let tmp = make_temp_dir("config-invalid");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join(CONFIG_FILE), "not valid json{{{").unwrap();

        let config = Config::load(&tmp);
        assert!(config.enabled);
        assert!(config.tools.knip);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn finds_config_in_parent_directory() {
        let tmp = make_temp_dir("config-parent");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join(CONFIG_FILE), r#"{"tools": {"knip": false}}"#).unwrap();
        let subdir = tmp.join("src").join("components");
        fs::create_dir_all(&subdir).unwrap();

        let config = Config::load(&subdir);
        assert!(!config.tools.knip);

        fs::remove_dir_all(&tmp).unwrap();
    }
}
