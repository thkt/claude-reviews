mod config;
mod project;
mod resolve;
mod sanitize;
mod tools;

use serde::Deserialize;
use std::io::{self, Read};

const MAX_INPUT_SIZE: usize = 10_000_000;

#[derive(Deserialize)]
struct HookInput {
    tool_input: SkillInput,
}

#[derive(Deserialize)]
struct SkillInput {
    skill: Option<String>,
}

fn parse_audit_skill(input: &str) -> Option<()> {
    let hook: HookInput = serde_json::from_str(input).ok()?;
    let skill = hook.tool_input.skill.as_deref()?;
    if skill == "audit" { Some(()) } else { None }
}

fn build_output(results: &[tools::ToolResult]) -> Option<String> {
    let successful: Vec<_> = results
        .iter()
        .filter(|r| r.success && !r.output.is_empty())
        .collect();

    if successful.is_empty() {
        return None;
    }

    let total = results.len();
    let reported = successful.len();

    let mut context = String::from("# Pre-flight Analysis Results\n\n");
    for result in &successful {
        context.push_str(&format!("## {}\n\n```\n{}\n```\n\n", result.name, result.output));
    }

    let output = serde_json::json!({
        "decision": "approve",
        "reason": format!("Pre-flight: {}/{} tools reported", reported, total),
        "additionalContext": context.trim_end()
    });

    Some(output.to_string())
}

fn main() {
    let mut input_str = String::new();
    let bytes_read = match io::stdin().take(MAX_INPUT_SIZE as u64).read_to_string(&mut input_str) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("reviews: stdin read error: {}", e);
            return;
        }
    };

    if bytes_read == MAX_INPUT_SIZE {
        eprintln!("reviews: error: input too large (>={}B limit)", MAX_INPUT_SIZE);
        return;
    }

    if parse_audit_skill(&input_str).is_none() {
        return;
    }

    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("reviews: cannot determine cwd: {}", e);
            return;
        }
    };
    let config = config::Config::load(&cwd);

    if !config.enabled {
        return;
    }

    let project = project::ProjectInfo::detect(&cwd);
    let results = run_tools_parallel(&config, &project);

    if let Some(json) = build_output(&results) {
        println!("{}", json);
    }
}

type ToolRunFn = fn(&project::ProjectInfo) -> tools::ToolResult;

fn run_tools_parallel(
    config: &config::Config,
    project: &project::ProjectInfo,
) -> Vec<tools::ToolResult> {
    use std::thread;

    let runners: Vec<(bool, &'static str, ToolRunFn)> = vec![
        (config.tools.knip, "knip", tools::knip::run),
        (config.tools.oxlint, "oxlint", tools::oxlint::run),
        (config.tools.tsgo, "tsgo", tools::tsgo::run),
        (config.tools.react_doctor, "react-doctor", tools::react_doctor::run),
    ];

    let handles: Vec<_> = runners
        .into_iter()
        .filter(|(enabled, _, _)| *enabled)
        .map(|(_, name, run_fn)| {
            let p = project.clone();
            (name, thread::spawn(move || run_fn(&p)))
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
    fn parse_audit_skill_valid() {
        let input = r#"{"tool_name": "Skill", "tool_input": {"skill": "audit"}}"#;
        assert!(parse_audit_skill(input).is_some());
    }

    #[test]
    fn parse_audit_skill_invalid_json() {
        assert!(parse_audit_skill("not json{{{").is_none());
    }

    #[test]
    fn parse_audit_skill_non_audit() {
        let input = r#"{"tool_name": "Skill", "tool_input": {"skill": "commit"}}"#;
        assert!(parse_audit_skill(input).is_none());
    }

    #[test]
    fn parse_audit_skill_null() {
        let input = r#"{"tool_name": "Skill", "tool_input": {}}"#;
        assert!(parse_audit_skill(input).is_none());
    }

    #[test]
    fn parse_audit_skill_with_args() {
        let input = r#"{"tool_name": "Skill", "tool_input": {"skill": "audit", "args": "--verbose"}}"#;
        assert!(parse_audit_skill(input).is_some());
    }

    #[test]
    fn build_output_partial_success() {
        let results = vec![
            tools::ToolResult { name: "knip", output: "result1".into(), success: true },
            tools::ToolResult { name: "oxlint", output: "result2".into(), success: true },
            tools::ToolResult { name: "tsgo", output: "result3".into(), success: true },
            tools::ToolResult { name: "react-doctor", output: String::new(), success: false },
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
    fn build_output_no_results() {
        let results = vec![
            tools::ToolResult { name: "knip", output: String::new(), success: false },
            tools::ToolResult { name: "oxlint", output: String::new(), success: false },
        ];
        assert!(build_output(&results).is_none());
    }

    #[test]
    fn build_output_empty_slice() {
        assert!(build_output(&[]).is_none());
    }

    #[test]
    fn build_output_excludes_successful_but_empty() {
        let results = vec![
            tools::ToolResult { name: "knip", output: String::new(), success: true },
            tools::ToolResult { name: "oxlint", output: "issues".into(), success: true },
        ];
        let json = build_output(&results).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["reason"].as_str().unwrap().contains("1/2"));
        let ctx = parsed["additionalContext"].as_str().unwrap();
        assert!(!ctx.contains("knip"));
    }
}
