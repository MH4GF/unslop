//! upstream の textlint-tester ケースを使った互換テスト。
//!
//! tests/cases/<group>/<rule>.json を読み、各 rule 実装に対して
//! valid → issue 0、invalid → 期待 message 数と一致 (中身は exact match) を検証する。

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use unslop::document::Document;
use unslop::rule::{Issue, Rule};
use unslop::rules::ai_writing::ai_tech_writing_guideline::AiTechWritingGuideline;
use unslop::rules::ai_writing::no_ai_emphasis_patterns::NoAiEmphasisPatterns;
use unslop::rules::ai_writing::no_ai_hype_expressions::NoAiHypeExpressions;
use unslop::rules::ai_writing::no_ai_list_formatting::NoAiListFormatting;
use unslop::rules::jt::no_nfd::NoNfd;

#[derive(Debug, Deserialize)]
struct Suite {
    name: String,
    valid: Vec<ValidCase>,
    invalid: Vec<InvalidCase>,
}

#[derive(Debug, Deserialize)]
struct ValidCase {
    text: String,
    #[serde(default)]
    options: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct InvalidCase {
    text: String,
    #[serde(default)]
    options: Option<serde_json::Value>,
    errors: Vec<ExpectedError>,
}

#[derive(Debug, Deserialize)]
struct ExpectedError {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    line: Option<usize>,
    #[serde(default)]
    column: Option<usize>,
}

fn cases_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases")
}

fn load_suite(path: &str) -> Vec<Suite> {
    let p = cases_dir().join(path);
    let s = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("parse {}: {e}", p.display()))
}

fn run_rule(rule: &dyn Rule, text: &str) -> Vec<Issue> {
    let doc = Document::parse(text);
    let mut issues = rule.check(&doc);
    issues.sort_by(|a, b| (a.line, a.column).cmp(&(b.line, b.column)));
    issues
}

fn check_suite(rule: &dyn Rule, suites: Vec<Suite>) -> Report {
    let mut report = Report::default();
    for suite in suites {
        for (i, case) in suite.valid.iter().enumerate() {
            // options 未対応は skip マーク
            if has_options(&case.options) {
                report.skipped_valid += 1;
                continue;
            }
            let issues = run_rule(rule, &case.text);
            if !issues.is_empty() {
                let entry = report
                    .valid_failures
                    .entry(suite.name.clone())
                    .or_default();
                entry.push(FailureValid {
                    index: i,
                    text: trunc(&case.text),
                    issues: issues
                        .iter()
                        .map(|i| format!("L{}:{} {}", i.line, i.column, trunc(&i.message)))
                        .collect(),
                });
            } else {
                report.valid_ok += 1;
            }
        }
        for (i, case) in suite.invalid.iter().enumerate() {
            if has_options(&case.options) {
                report.skipped_invalid += 1;
                continue;
            }
            let issues = run_rule(rule, &case.text);
            let mismatch = !errors_match(&issues, &case.errors);
            if mismatch {
                let entry = report
                    .invalid_failures
                    .entry(suite.name.clone())
                    .or_default();
                entry.push(FailureInvalid {
                    index: i,
                    text: trunc(&case.text),
                    expected: case
                        .errors
                        .iter()
                        .map(|e| {
                            format!(
                                "L{}:{} {}",
                                e.line.unwrap_or(0),
                                e.column.unwrap_or(0),
                                e.message.as_deref().map(trunc).unwrap_or("<no msg>".into())
                            )
                        })
                        .collect(),
                    actual: issues
                        .iter()
                        .map(|i| format!("L{}:{} {}", i.line, i.column, trunc(&i.message)))
                        .collect(),
                });
            } else {
                report.invalid_ok += 1;
            }
        }
    }
    report
}

fn has_options(opts: &Option<serde_json::Value>) -> bool {
    matches!(opts, Some(v) if !v.is_null() && !(v.is_object() && v.as_object().unwrap().is_empty()))
}

fn errors_match(actual: &[Issue], expected: &[ExpectedError]) -> bool {
    if actual.len() != expected.len() {
        return false;
    }
    for (a, e) in actual.iter().zip(expected) {
        if let Some(em) = &e.message {
            if &a.message != em {
                return false;
            }
        }
        if let Some(el) = e.line {
            if a.line != el {
                return false;
            }
        }
        if let Some(ec) = e.column {
            if a.column != ec {
                return false;
            }
        }
    }
    true
}

fn trunc(s: &str) -> String {
    let max = 80;
    let one_line = s.replace('\n', "\\n");
    if one_line.chars().count() > max {
        let truncated: String = one_line.chars().take(max).collect();
        format!("{truncated}…")
    } else {
        one_line
    }
}

#[derive(Default)]
struct Report {
    valid_ok: usize,
    invalid_ok: usize,
    skipped_valid: usize,
    skipped_invalid: usize,
    valid_failures: BTreeMap<String, Vec<FailureValid>>,
    invalid_failures: BTreeMap<String, Vec<FailureInvalid>>,
}

struct FailureValid {
    index: usize,
    text: String,
    issues: Vec<String>,
}

struct FailureInvalid {
    index: usize,
    text: String,
    expected: Vec<String>,
    actual: Vec<String>,
}

impl Report {
    fn assert_clean(&self, rule_name: &str) {
        let mut out = String::new();
        if !self.valid_failures.is_empty() {
            out.push_str(&format!("[{rule_name}] valid cases reported issues:\n"));
            for (suite, fails) in &self.valid_failures {
                for f in fails {
                    out.push_str(&format!(
                        "  {suite}#{}: text={:?}\n    got: {:?}\n",
                        f.index, f.text, f.issues
                    ));
                }
            }
        }
        if !self.invalid_failures.is_empty() {
            out.push_str(&format!("[{rule_name}] invalid cases did not match:\n"));
            for (suite, fails) in &self.invalid_failures {
                for f in fails {
                    out.push_str(&format!(
                        "  {suite}#{}: text={:?}\n    expected: {:?}\n    actual:   {:?}\n",
                        f.index, f.text, f.expected, f.actual
                    ));
                }
            }
        }
        eprintln!(
            "[{rule_name}] valid_ok={} invalid_ok={} skipped(valid)={} skipped(invalid)={}",
            self.valid_ok, self.invalid_ok, self.skipped_valid, self.skipped_invalid,
        );
        if !out.is_empty() {
            panic!("{out}");
        }
    }
}

#[test]
fn no_ai_emphasis_patterns() {
    let suites = load_suite("ai-writing/no-ai-emphasis-patterns.json");
    let report = check_suite(&NoAiEmphasisPatterns, suites);
    report.assert_clean("no-ai-emphasis-patterns");
}

#[test]
fn no_ai_hype_expressions() {
    let suites = load_suite("ai-writing/no-ai-hype-expressions.json");
    let report = check_suite(&NoAiHypeExpressions, suites);
    report.assert_clean("no-ai-hype-expressions");
}

#[test]
fn no_ai_list_formatting() {
    let suites = load_suite("ai-writing/no-ai-list-formatting.json");
    let report = check_suite(&NoAiListFormatting, suites);
    report.assert_clean("no-ai-list-formatting");
}

#[test]
fn ai_tech_writing_guideline() {
    let suites = load_suite("ai-writing/ai-tech-writing-guideline.json");
    let report = check_suite(&AiTechWritingGuideline, suites);
    report.assert_clean("ai-tech-writing-guideline");
}

#[test]
fn no_nfd() {
    let suites = load_suite("jt/no-nfd.json");
    let report = check_suite(&NoNfd, suites);
    report.assert_clean("no-nfd");
}
