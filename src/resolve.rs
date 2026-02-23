use std::path::{Path, PathBuf};

const MAX_TRAVERSAL_DEPTH: usize = 20;

pub fn resolve_bin(name: &str, start: &Path) -> PathBuf {
    let mut dir = Some(start);
    let mut depth = 0;

    while let Some(d) = dir {
        if depth >= MAX_TRAVERSAL_DEPTH {
            break;
        }

        let candidate = d.join("node_modules/.bin").join(name);
        if candidate.exists() {
            eprintln!("reviews: resolved {} -> {}", name, candidate.display());
            return candidate;
        }

        if d.join(".git").exists() {
            break;
        }

        dir = d.parent();
        depth += 1;
    }

    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::make_temp_dir;
    use std::fs;

    #[test]
    fn finds_bin_in_node_modules() {
        let tmp = make_temp_dir("resolve-find");
        let bin_dir = tmp.join("node_modules/.bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let bin_path = bin_dir.join("knip");
        fs::write(&bin_path, "").unwrap();

        let result = resolve_bin("knip", &tmp);
        assert_eq!(result, bin_path);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn falls_back_to_bare_name_when_no_node_modules() {
        let tmp = make_temp_dir("resolve-nomod");
        fs::create_dir_all(tmp.join(".git")).unwrap();

        let result = resolve_bin("knip", &tmp);
        assert_eq!(result, PathBuf::from("knip"));

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn stops_at_git_boundary() {
        let tmp = make_temp_dir("resolve-git");
        let project = tmp.join("project");
        fs::create_dir_all(project.join(".git")).unwrap();
        let bin_dir = tmp.join("node_modules/.bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("knip"), "").unwrap();
        let subdir = project.join("src");
        fs::create_dir_all(&subdir).unwrap();

        let result = resolve_bin("knip", &subdir);
        assert_eq!(result, PathBuf::from("knip"));

        fs::remove_dir_all(&tmp).unwrap();
    }
}
