use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub has_package_json: bool,
    pub has_tsconfig: bool,
    pub has_react: bool,
}

impl ProjectInfo {
    pub fn detect(dir: &Path) -> Self {
        let root = Self::find_root(dir);
        let has_package_json = root.join("package.json").exists();
        let has_tsconfig = root.join("tsconfig.json").exists();
        let has_react = has_package_json && Self::detect_react(&root);

        Self {
            root,
            has_package_json,
            has_tsconfig,
            has_react,
        }
    }

    fn find_root(start: &Path) -> PathBuf {
        let mut dir = Some(start.to_path_buf());
        while let Some(d) = dir {
            if d.join(".git").exists() {
                return d;
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
        start.to_path_buf()
    }

    fn detect_react(root: &Path) -> bool {
        let pkg_path = root.join("package.json");
        let content = match std::fs::read_to_string(&pkg_path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return false,
        };

        for key in ["dependencies", "devDependencies", "peerDependencies"] {
            if let Some(deps) = json.get(key).and_then(|v| v.as_object())
                && deps.contains_key("react")
            {
                return true;
            }
        }
        false
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
    fn detects_package_json() {
        let tmp = make_temp_dir("project-pkg");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join("package.json"), "{}").unwrap();

        let info = ProjectInfo::detect(&tmp);
        assert!(info.has_package_json);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn no_package_json() {
        let tmp = make_temp_dir("project-nopkg");
        fs::create_dir_all(tmp.join(".git")).unwrap();

        let info = ProjectInfo::detect(&tmp);
        assert!(!info.has_package_json);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn detects_react_dependency() {
        let tmp = make_temp_dir("project-react");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(
            tmp.join("package.json"),
            r#"{"dependencies": {"react": "^19.0.0"}}"#,
        ).unwrap();

        let info = ProjectInfo::detect(&tmp);
        assert!(info.has_react);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn detects_react_in_dev_dependencies() {
        let tmp = make_temp_dir("project-react-dev");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(
            tmp.join("package.json"),
            r#"{"devDependencies": {"react": "^19.0.0"}}"#,
        ).unwrap();

        let info = ProjectInfo::detect(&tmp);
        assert!(info.has_react);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn no_react_dependency() {
        let tmp = make_temp_dir("project-noreact");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(
            tmp.join("package.json"),
            r#"{"dependencies": {"vue": "^3.0.0"}}"#,
        ).unwrap();

        let info = ProjectInfo::detect(&tmp);
        assert!(!info.has_react);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn detects_react_in_peer_dependencies() {
        let tmp = make_temp_dir("project-react-peer");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(
            tmp.join("package.json"),
            r#"{"peerDependencies": {"react": ">=18"}}"#,
        ).unwrap();

        let info = ProjectInfo::detect(&tmp);
        assert!(info.has_react);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn malformed_package_json_no_react() {
        let tmp = make_temp_dir("project-malformed-pkg");
        fs::create_dir_all(tmp.join(".git")).unwrap();
        fs::write(tmp.join("package.json"), "not valid json").unwrap();

        let info = ProjectInfo::detect(&tmp);
        assert!(!info.has_react);

        fs::remove_dir_all(&tmp).unwrap();
    }
}
