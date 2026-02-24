mod config;
mod project;
mod resolve;
mod sanitize;
#[cfg(test)]
mod test_utils;
mod tools;
mod traverse;

use serde::Deserialize;
use std::io::{self, Read};
use std::path::Path;

const MAX_INPUT_SIZE: usize = 10_000_000;

#[derive(Deserialize)]
struct HookInput {
    tool_input: SkillInput,
}

#[derive(Deserialize)]
struct SkillInput {
    skill: Option<String>,
}

fn is_audit_skill(input: &str) -> bool {
    let Ok(hook) = serde_json::from_str::<HookInput>(input) else {
        return false;
    };
    hook.tool_input.skill.as_deref() == Some("audit")
}

fn build_output(results: &[tools::ToolResult]) -> Option<String> {
    if results.is_empty() {
        return None;
    }

    let with_output: Vec<_> = results.iter().filter(|r| !r.output.is_empty()).collect();

    let mut context = String::from("# Pre-flight Analysis Results\n\n");
    for result in &with_output {
        let escaped = result.output.replace("```", "` ` `");
        context.push_str(&format!("## {}\n\n```\n{}\n```\n\n", result.name, escaped));
    }

    // Advisory-only: always approve, inject tool output as context
    let with_issues = with_output.iter().filter(|r| !r.success).count();
    let reason = if with_issues > 0 {
        format!(
            "Pre-flight: {}/{} tools reported ({} with issues)",
            with_output.len(),
            results.len(),
            with_issues
        )
    } else {
        format!(
            "Pre-flight: {}/{} tools reported",
            with_output.len(),
            results.len()
        )
    };
    let output = serde_json::json!({
        "decision": "approve",
        "reason": reason,
        "additionalContext": context.trim_end()
    });

    Some(output.to_string())
}

fn run(input: &str, cwd: &Path) -> Option<String> {
    if !is_audit_skill(input) {
        return None;
    }

    let config = config::Config::load(cwd);
    if !config.enabled {
        return None;
    }

    let project = project::ProjectInfo::detect(cwd);
    let results = run_tools_parallel(&config, &project);
    build_output(&results)
}

fn main() {
    let mut input_str = String::new();
    let bytes_read = match io::stdin()
        .take((MAX_INPUT_SIZE + 1) as u64)
        .read_to_string(&mut input_str)
    {
        Ok(n) => n,
        Err(e) => {
            eprintln!("reviews: stdin read error: {}", e);
            std::process::exit(1);
        }
    };

    if bytes_read > MAX_INPUT_SIZE {
        eprintln!(
            "reviews: error: input too large (>{}B limit)",
            MAX_INPUT_SIZE
        );
        std::process::exit(1);
    }

    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("reviews: cannot determine cwd: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(json) = run(&input_str, &cwd) {
        println!("{}", json);
    }
}

type ToolRunFn = fn(&project::ProjectInfo) -> tools::ToolResult;

struct ToolEntry {
    enabled: bool,
    name: &'static str,
    run: ToolRunFn,
}

fn run_tools_parallel(
    config: &config::Config,
    project: &project::ProjectInfo,
) -> Vec<tools::ToolResult> {
    use std::thread;

    let entries = vec![
        ToolEntry {
            enabled: config.tools.knip,
            name: "knip",
            run: tools::knip::run,
        },
        ToolEntry {
            enabled: config.tools.oxlint,
            name: "oxlint",
            run: tools::oxlint::run,
        },
        ToolEntry {
            enabled: config.tools.tsgo,
            name: "tsgo",
            run: tools::tsgo::run,
        },
        ToolEntry {
            enabled: config.tools.react_doctor,
            name: "react-doctor",
            run: tools::react_doctor::run,
        },
    ];

    let handles: Vec<_> = entries
        .into_iter()
        .filter(|e| e.enabled)
        .map(|e| {
            let p = project.clone();
            (e.name, thread::spawn(move || (e.run)(&p)))
        })
        .collect();

    handles
        .into_iter()
        .map(|(name, handle)| match handle.join() {
            Ok(result) => result,
            Err(e) => {
                eprintln!("reviews: {} thread panicked: {:?}", name, e);
                tools::ToolResult::skipped(name)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_audit_skill_valid() {
        let input = r#"{"tool_name": "Skill", "tool_input": {"skill": "audit"}}"#;
        assert!(is_audit_skill(input));
    }

    #[test]
    fn is_audit_skill_invalid_json() {
        assert!(!is_audit_skill("not json{{{"));
    }

    #[test]
    fn is_audit_skill_non_audit() {
        let input = r#"{"tool_name": "Skill", "tool_input": {"skill": "commit"}}"#;
        assert!(!is_audit_skill(input));
    }

    #[test]
    fn is_audit_skill_null() {
        let input = r#"{"tool_name": "Skill", "tool_input": {}}"#;
        assert!(!is_audit_skill(input));
    }

    #[test]
    fn is_audit_skill_with_args() {
        let input =
            r#"{"tool_name": "Skill", "tool_input": {"skill": "audit", "args": "--verbose"}}"#;
        assert!(is_audit_skill(input));
    }

    #[test]
    fn build_output_partial_success() {
        let results = vec![
            tools::ToolResult {
                name: "knip",
                output: "result1".into(),
                success: true,
            },
            tools::ToolResult {
                name: "oxlint",
                output: "result2".into(),
                success: true,
            },
            tools::ToolResult {
                name: "tsgo",
                output: "result3".into(),
                success: true,
            },
            tools::ToolResult {
                name: "react-doctor",
                output: String::new(),
                success: false,
            },
        ];
        let json = build_output(&results).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["decision"], "approve");
        assert!(parsed["reason"].as_str().unwrap().contains("3/4"));
        let ctx = parsed["additionalContext"].as_str().unwrap();
        assert!(ctx.contains("knip"));
        assert!(ctx.contains("oxlint"));
        assert!(ctx.contains("tsgo"));
    }

    #[test]
    fn build_output_all_empty_output() {
        let results = vec![
            tools::ToolResult {
                name: "knip",
                output: String::new(),
                success: false,
            },
            tools::ToolResult {
                name: "oxlint",
                output: String::new(),
                success: false,
            },
        ];
        let json = build_output(&results).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["decision"], "approve");
        assert!(parsed["reason"].as_str().unwrap().contains("0/2"));
    }

    #[test]
    fn build_output_empty_slice() {
        assert!(build_output(&[]).is_none());
    }

    #[test]
    fn build_output_includes_failed_with_output() {
        let results = vec![
            tools::ToolResult {
                name: "oxlint",
                output: "warning: unused variable".into(),
                success: false,
            },
            tools::ToolResult {
                name: "knip",
                output: String::new(),
                success: false,
            },
        ];
        let json = build_output(&results).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let reason = parsed["reason"].as_str().unwrap();
        assert!(reason.contains("1/2"));
        assert!(reason.contains("1 with issues"));
        let ctx = parsed["additionalContext"].as_str().unwrap();
        assert!(ctx.contains("oxlint"));
        assert!(ctx.contains("warning: unused variable"));
    }

    #[test]
    fn build_output_excludes_successful_but_empty() {
        let results = vec![
            tools::ToolResult {
                name: "knip",
                output: String::new(),
                success: true,
            },
            tools::ToolResult {
                name: "oxlint",
                output: "issues".into(),
                success: true,
            },
        ];
        let json = build_output(&results).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["reason"].as_str().unwrap().contains("1/2"));
        let ctx = parsed["additionalContext"].as_str().unwrap();
        assert!(!ctx.contains("knip"));
    }
}
