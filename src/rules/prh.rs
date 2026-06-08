//! prh: YAML 辞書ベースの用語置換 lint。
//! upstream: textlint-rule-prh + prh
//!
//! 対応スコープ (Phase 1a, simplified):
//! - `version: 1` + `rules: [{ expected, pattern }]` 形式
//! - pattern は `/regex/flags` 形式 (i フラグのみ尊重) または literal string
//! - message format: `<actual> => <merged>` (case merge 後の文字列を表示)
//! - auto-fix 対応: actual の case を merge して expected を出力する

use fancy_regex::Regex;
use serde::Deserialize;
use std::path::Path;

use crate::document::Document;
use crate::rule::{Fix, Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "prh";

#[derive(Debug, Deserialize)]
struct PrhFile {
    #[allow(dead_code)]
    #[serde(default)]
    version: u32,
    rules: Vec<RawRule>,
}

#[derive(Debug, Deserialize)]
struct RawRule {
    expected: String,
    pattern: String,
}

struct CompiledRule {
    expected: String,
    expected_is_ascii: bool,
    re: Regex,
}

pub struct Prh {
    rules: Vec<CompiledRule>,
}

impl Prh {
    pub fn from_yaml_path(path: &Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let parsed: PrhFile = serde_yaml::from_str(&s)?;
        let mut compiled = Vec::with_capacity(parsed.rules.len());
        for r in parsed.rules {
            let re = compile_pattern(&r.pattern)?;
            let expected_is_ascii = r.expected.is_ascii();
            compiled.push(CompiledRule {
                expected: r.expected,
                expected_is_ascii,
                re,
            });
        }
        Ok(Prh { rules: compiled })
    }
}

fn compile_pattern(src: &str) -> anyhow::Result<Regex> {
    let (body, flags) = if src.starts_with('/') {
        if let Some(last) = src.rfind('/') {
            if last > 0 {
                (&src[1..last], &src[last + 1..])
            } else {
                (src, "")
            }
        } else {
            (src, "")
        }
    } else {
        (src, "")
    };
    let prefix = if flags.contains('i') { "(?i)" } else { "" };
    Ok(Regex::new(&format!("{prefix}{body}"))?)
}

/// 本家 prh の case 推論を再現する。
/// expected が ASCII 純粋でないとき、または actual の case が混在のときは
/// expected をそのまま返す。それ以外は actual の case (lower / upper / title)
/// を expected に伝播する。
fn case_merge(actual: &str, expected: &str, expected_is_ascii: bool) -> String {
    if !expected_is_ascii {
        return expected.to_string();
    }
    let ascii_letters: Vec<char> = actual.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    if ascii_letters.is_empty() {
        return expected.to_string();
    }
    let all_lower = ascii_letters.iter().all(|c| c.is_ascii_lowercase());
    let all_upper = ascii_letters.iter().all(|c| c.is_ascii_uppercase());
    let title = ascii_letters[0].is_ascii_uppercase()
        && ascii_letters.iter().skip(1).all(|c| c.is_ascii_lowercase());

    if all_lower {
        expected.to_ascii_lowercase()
    } else if all_upper {
        expected.to_ascii_uppercase()
    } else if title {
        let mut chars = expected.chars();
        match chars.next() {
            Some(first) => {
                let mut out = String::with_capacity(expected.len());
                for c in first.to_uppercase() {
                    out.push(c);
                }
                for c in chars {
                    for lc in c.to_lowercase() {
                        out.push(lc);
                    }
                }
                out
            }
            None => expected.to_string(),
        }
    } else {
        expected.to_string()
    }
}

impl Rule for Prh {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            for rule in &self.rules {
                let mut from = 0usize;
                while let Ok(Some(m)) = rule.re.find_from_pos(&seg.text, from) {
                    let s = m.start();
                    let e = m.end();
                    let in_code_span = seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                    let actual = m.as_str().to_string();
                    let merged = case_merge(&actual, &rule.expected, rule.expected_is_ascii);
                    if !in_code_span && actual != merged {
                        let (line, column) = doc.pos_at(seg, s);
                        let abs_start = seg.start_byte + s;
                        let abs_end = seg.start_byte + e;
                        issues.push(
                            Issue::new(
                                RULE_ID,
                                format!("{actual} => {merged}"),
                                line,
                                column,
                                Severity::Error,
                            )
                            .with_fix(Fix {
                                range: abs_start..abs_end,
                                replacement: merged,
                            }),
                        );
                    }
                    from = e.max(s + 1);
                }
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    /// `worker` → ワーカー の単一ルールを持つ Prh を組む。
    fn prh_worker() -> Prh {
        Prh {
            rules: vec![CompiledRule {
                expected: "ワーカー".to_string(),
                expected_is_ascii: false,
                re: Regex::new(r"\bworker\b").unwrap(),
            }],
        }
    }

    /// ASCII 同士の case merge を試す `worker` → `process` (i フラグ)。
    fn prh_ascii_process() -> Prh {
        Prh {
            rules: vec![CompiledRule {
                expected: "process".to_string(),
                expected_is_ascii: true,
                re: Regex::new(r"(?i)\bworker\b").unwrap(),
            }],
        }
    }

    fn messages(src: &str) -> Vec<(usize, String)> {
        let doc = Document::parse(src);
        prh_worker()
            .check(&doc)
            .into_iter()
            .map(|i| (i.line, i.message))
            .collect()
    }

    fn first_fix_replacement(rule: &Prh, src: &str) -> Option<String> {
        let doc = Document::parse(src);
        rule.check(&doc)
            .into_iter()
            .next()
            .and_then(|i| i.fix.map(|f| f.replacement))
    }

    #[test]
    fn flags_plain_text() {
        let got = messages("これは worker です。");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].1, "worker => ワーカー");
    }

    #[test]
    fn skips_inline_code_span() {
        assert!(messages("これは `worker` です。").is_empty());
    }

    #[test]
    fn skips_double_backtick_code_span() {
        assert!(messages("これは ``worker`` です。").is_empty());
    }

    #[test]
    fn skips_fenced_code_block() {
        assert!(messages("```\nworker\n```\n").is_empty());
    }

    #[test]
    fn flags_plain_but_skips_code_span_when_mixed() {
        let got = messages("地の worker と `worker` を併記する。");
        assert_eq!(got.len(), 1, "got = {got:?}");
        assert_eq!(got[0].1, "worker => ワーカー");
    }

    #[test]
    fn skips_code_span_nested_in_emphasis() {
        assert!(messages("これは *`worker`* です。").is_empty());
    }

    #[test]
    fn fix_replaces_with_japanese_expected() {
        let got = first_fix_replacement(&prh_worker(), "これは worker です。");
        assert_eq!(got.as_deref(), Some("ワーカー"));
    }

    #[test]
    fn case_merge_lower() {
        assert_eq!(case_merge("worker", "process", true), "process");
    }

    #[test]
    fn case_merge_upper() {
        assert_eq!(case_merge("WORKER", "process", true), "PROCESS");
    }

    #[test]
    fn case_merge_title() {
        assert_eq!(case_merge("Worker", "process", true), "Process");
    }

    #[test]
    fn case_merge_mixed_skips() {
        assert_eq!(case_merge("wOrker", "process", true), "process");
    }

    #[test]
    fn case_merge_japanese_expected_unchanged() {
        assert_eq!(case_merge("Worker", "ワーカー", false), "ワーカー");
    }

    #[test]
    fn fix_propagates_title_case_to_ascii_expected() {
        let got = first_fix_replacement(&prh_ascii_process(), "Worker is here.");
        assert_eq!(got.as_deref(), Some("Process"));
    }

    #[test]
    fn fix_propagates_upper_case_to_ascii_expected() {
        let got = first_fix_replacement(&prh_ascii_process(), "WORKER is here.");
        assert_eq!(got.as_deref(), Some("PROCESS"));
    }

    #[test]
    fn fix_propagates_lower_case_to_ascii_expected() {
        let got = first_fix_replacement(&prh_ascii_process(), "the worker now.");
        assert_eq!(got.as_deref(), Some("process"));
    }

    #[test]
    fn no_issue_when_already_matches_merged() {
        let doc = Document::parse("the process now.");
        let issues = prh_ascii_process().check(&doc);
        assert!(issues.is_empty());
    }
}
