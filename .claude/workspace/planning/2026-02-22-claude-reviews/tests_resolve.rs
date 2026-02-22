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

    /// T-021: node_modules/.bin/knip exists → returns local path
    #[test]
    fn t021_resolve_bin_finds_local_node_modules_bin() {
        let dir = temp_dir("t021");

        // Setup: create node_modules/.bin/knip
        let bin_dir = dir.join("node_modules").join(".bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let knip_bin = bin_dir.join("knip");
        fs::write(&knip_bin, "").unwrap();

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&knip_bin, fs::Permissions::from_mode(0o755)).unwrap();
        }

        let result = resolve_bin("knip", &dir);

        assert_eq!(result, knip_bin);

        cleanup(&dir);
    }

    /// T-022: no node_modules → returns bare name (PATH fallback)
    #[test]
    fn t022_resolve_bin_returns_bare_name_without_node_modules() {
        let dir = temp_dir("t022");

        // No node_modules directory exists
        let result = resolve_bin("knip", &dir);

        assert_eq!(result, PathBuf::from("knip"));

        cleanup(&dir);
    }

    /// T-023: .git boundary stops traversal, PATH fallback
    #[test]
    fn t023_resolve_bin_stops_at_git_boundary() {
        let dir = temp_dir("t023");

        // Setup directory structure:
        //   dir/
        //     .git/          (git boundary)
        //     subdir/
        //       (CWD - no node_modules here)
        //   dir/../
        //     node_modules/.bin/knip  (above .git - should NOT be found)

        let git_dir = dir.join(".git");
        fs::create_dir_all(&git_dir).unwrap();

        let subdir = dir.join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        // Place node_modules above the .git boundary (at parent of dir)
        // This simulates a monorepo where the binary is outside the git root
        // The resolver should NOT traverse past .git
        let parent_bin = dir.parent().unwrap().join("node_modules").join(".bin");
        fs::create_dir_all(&parent_bin).unwrap();
        let knip_bin = parent_bin.join("knip");
        fs::write(&knip_bin, "").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&knip_bin, fs::Permissions::from_mode(0o755)).unwrap();
        }

        // Resolve from subdir - should traverse up to dir (has .git), then stop
        let result = resolve_bin("knip", &subdir);

        // Should fall back to bare name since .git stops traversal
        assert_eq!(result, PathBuf::from("knip"));

        // Cleanup
        let _ = fs::remove_dir_all(&parent_bin.parent().unwrap().parent().unwrap().join("node_modules"));
        cleanup(&dir);
    }
}
