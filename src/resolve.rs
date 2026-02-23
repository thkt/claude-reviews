use std::path::{Path, PathBuf};

pub fn resolve_bin(name: &str, start: &Path) -> PathBuf {
    crate::traverse::walk_ancestors(start, |dir| {
        let candidate = dir.join("node_modules/.bin").join(name);
        if candidate.exists() {
            eprintln!("reviews: resolved {} -> {}", name, candidate.display());
            Some(candidate)
        } else {
            None
        }
    })
    .unwrap_or_else(|| PathBuf::from(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TempDir;
    use std::fs;

    #[test]
    fn finds_bin_in_node_modules() {
        let tmp = TempDir::new("resolve-find");
        let bin_dir = tmp.join("node_modules/.bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let bin_path = bin_dir.join("knip");
        fs::write(&bin_path, "").unwrap();

        let result = resolve_bin("knip", &tmp);
        assert_eq!(result, bin_path);
    }

    #[test]
    fn falls_back_to_bare_name_when_no_node_modules() {
        let tmp = TempDir::new("resolve-nomod");
        fs::create_dir_all(tmp.join(".git")).unwrap();

        let result = resolve_bin("knip", &tmp);
        assert_eq!(result, PathBuf::from("knip"));
    }

    #[test]
    fn stops_at_git_boundary() {
        let tmp = TempDir::new("resolve-git");
        let project = tmp.join("project");
        fs::create_dir_all(project.join(".git")).unwrap();
        let bin_dir = tmp.join("node_modules/.bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("knip"), "").unwrap();
        let subdir = project.join("src");
        fs::create_dir_all(&subdir).unwrap();

        let result = resolve_bin("knip", &subdir);
        assert_eq!(result, PathBuf::from("knip"));
    }
}
