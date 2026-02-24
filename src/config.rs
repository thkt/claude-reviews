use serde::Deserialize;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = ".claude-reviews.json";

/// Generates `ToolsConfig` (all-bool, default true) and `ProjectToolsConfig`
/// (all Option<bool> for JSON merge). `apply()` merges overrides into defaults.
macro_rules! define_tools {
    ($($field:ident),+ $(,)?) => {
        #[derive(Debug, Clone)]
        pub struct ToolsConfig {
            $(pub $field: bool,)+
        }

        impl Default for ToolsConfig {
            fn default() -> Self {
                Self { $($field: true,)+ }
            }
        }

        #[derive(Debug, Deserialize)]
        struct ProjectToolsConfig {
            $($field: Option<bool>,)+
        }

        impl ToolsConfig {
            fn apply(&mut self, overrides: &ProjectToolsConfig) {
                $(if let Some(v) = overrides.$field { self.$field = v; })+
            }
        }
    };
}

define_tools! {
    knip,
    oxlint,
    tsgo,
    react_doctor,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub enabled: bool,
    pub tools: ToolsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            tools: ToolsConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProjectConfig {
    enabled: Option<bool>,
    tools: Option<ProjectToolsConfig>,
}

impl Config {
    pub fn load(start: &Path) -> Self {
        let default = Self::default();
        let Some(config_path) = Self::find_config(start) else {
            return default;
        };
        Self::load_from(&config_path, default)
    }

    fn find_config(start: &Path) -> Option<PathBuf> {
        crate::traverse::walk_ancestors(start, |dir| {
            let candidate = dir.join(CONFIG_FILE);
            candidate.exists().then_some(candidate)
        })
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
        if let Some(ref tools) = project.tools {
            self.tools.apply(tools);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TempDir;
    use std::fs;

    #[test]
    fn default_config_all_tools_enabled() {
        let tmp = TempDir::new("config-default");
        fs::create_dir_all(tmp.join(".git")).unwrap();

        let config = Config::load(&tmp);
        assert!(config.enabled);
        assert!(config.tools.knip);
        assert!(config.tools.oxlint);
        assert!(config.tools.tsgo);
        assert!(config.tools.react_doctor);
    }

    #[test]
    fn partial_config_merges_with_defaults() {
        let tmp = TempDir::new("config-partial");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join(CONFIG_FILE), r#"{"tools": {"knip": false}}"#).unwrap();

        let config = Config::load(&tmp);
        assert!(config.enabled);
        assert!(!config.tools.knip);
        assert!(config.tools.oxlint);
        assert!(config.tools.tsgo);
        assert!(config.tools.react_doctor);
    }

    #[test]
    fn enabled_false_disables_all() {
        let tmp = TempDir::new("config-disabled");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join(CONFIG_FILE), r#"{"enabled": false}"#).unwrap();

        let config = Config::load(&tmp);
        assert!(!config.enabled);
    }

    #[test]
    fn invalid_json_falls_back_to_default() {
        let tmp = TempDir::new("config-invalid");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join(CONFIG_FILE), "not valid json{{{").unwrap();

        let config = Config::load(&tmp);
        assert!(config.enabled);
        assert!(config.tools.knip);
    }

    #[test]
    fn finds_config_in_parent_directory() {
        let tmp = TempDir::new("config-parent");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join(CONFIG_FILE), r#"{"tools": {"knip": false}}"#).unwrap();
        let subdir = tmp.join("src").join("components");
        fs::create_dir_all(&subdir).unwrap();

        let config = Config::load(&subdir);
        assert!(!config.tools.knip);
    }
}
