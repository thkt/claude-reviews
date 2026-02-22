use std::sync::LazyLock;
use regex::Regex;

static ANSI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap());
static MULTI_BLANK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

pub fn sanitize(input: &str) -> String {
    let s = ANSI_RE.replace_all(input, "");

    let s: String = s
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    let s = MULTI_BLANK.replace_all(&s, "\n\n");

    s.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_ansi_escape_codes() {
        let input = "\x1b[31mError:\x1b[0m something failed";
        let result = sanitize(input);
        assert_eq!(result, "Error: something failed");
    }

    #[test]
    fn removes_complex_ansi_codes() {
        let input = "\x1b[1;32m✓\x1b[0m test passed \x1b[38;5;240m(0.5s)\x1b[0m";
        let result = sanitize(input);
        assert_eq!(result, "✓ test passed (0.5s)");
    }

    #[test]
    fn compresses_consecutive_blank_lines() {
        let input = "line1\n\n\n\nline2";
        let result = sanitize(input);
        assert_eq!(result, "line1\n\nline2");
    }

    #[test]
    fn preserves_single_blank_line() {
        let input = "line1\n\nline2";
        let result = sanitize(input);
        assert_eq!(result, "line1\n\nline2");
    }

    #[test]
    fn removes_trailing_whitespace() {
        let input = "hello   \nworld\t";
        let result = sanitize(input);
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn json_passes_through_unchanged() {
        let input = r#"{"files": ["a.ts"], "count": 3}"#;
        let result = sanitize(input);
        assert_eq!(result, input);
    }

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(sanitize(""), "");
    }
}
