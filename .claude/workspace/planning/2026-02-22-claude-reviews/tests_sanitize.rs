#[cfg(test)]
mod tests {
    use super::*;

    /// T-013: ANSI escape codes removed
    #[test]
    fn t013_removes_ansi_escape_codes() {
        let input = "\x1b[31mError:\x1b[0m something failed\x1b[1m!\x1b[0m";
        let result = sanitize(input);
        assert_eq!(result, "Error: something failed!");
    }

    /// T-014: 3 consecutive blank lines → 1 blank line
    #[test]
    fn t014_collapses_consecutive_blank_lines() {
        let input = "line1\n\n\n\nline2\n\n\n\n\nline3";
        let result = sanitize(input);
        assert_eq!(result, "line1\n\nline2\n\nline3");
    }

    /// T-015: trailing whitespace removed
    #[test]
    fn t015_removes_trailing_whitespace() {
        let input = "hello   \nworld\t\t\nfoo  \t  \n";
        let result = sanitize(input);
        // Each line should have trailing whitespace removed
        for line in result.lines() {
            assert_eq!(line, line.trim_end(), "line has trailing whitespace: {:?}", line);
        }
        assert_eq!(result, "hello\nworld\nfoo\n");
    }

    /// T-016: JSON text passes through unchanged
    #[test]
    fn t016_json_passes_through_unchanged() {
        let input = r#"{"errors": [{"file": "src/main.ts", "line": 42, "message": "unused variable"}]}"#;
        let result = sanitize(input);
        assert_eq!(result, input);
    }
}
