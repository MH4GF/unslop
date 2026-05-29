//! prh: YAML 辞書ベースの用語置換 lint。
//! upstream: textlint-rule-prh + prh
//!
//! 対応スコープ (Phase 1a, simplified):
//! - `version: 1` + `rules: [{ expected, pattern }]` 形式
//! - pattern は `/regex/flags` 形式 (i フラグのみ尊重) または literal string
//! - message format: `<actual> => <expected>`

use fancy_regex::Regex;
use serde::Deserialize;
use std::path::Path;

use crate::document::Document;
use crate::rule::{Issue, Rule, Severity};
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
            compiled.push(CompiledRule {
                expected: r.expected,
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
                    // 本家 textlint-rule-prh は text ノードだけを見て Code ノードを無視する。
                    // code span (区間 overlap) に重なるマッチはスキップする。
                    let in_code_span = seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                    let actual = m.as_str().to_string();
                    if !in_code_span && actual != rule.expected {
                        let (line, column) = doc.pos_at(seg, s);
                        issues.push(Issue {
                            rule_id: RULE_ID.to_string(),
                            message: format!("{actual} => {}", rule.expected),
                            line,
                            column,
                            severity: Severity::Error,
                        });
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
                re: Regex::new(r"\bworker\b").unwrap(),
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

    #[test]
    fn flags_plain_text() {
        // 地の文の英単語は従来どおり検出される (退行なし)。
        let got = messages("これは worker です。");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].1, "worker => ワーカー");
    }

    #[test]
    fn skips_inline_code_span() {
        // バックティックで囲んだ識別子はスキップする。
        assert!(messages("これは `worker` です。").is_empty());
    }

    #[test]
    fn skips_double_backtick_code_span() {
        assert!(messages("これは ``worker`` です。").is_empty());
    }

    #[test]
    fn skips_fenced_code_block() {
        // fenced code block の中身も対象外。
        assert!(messages("```\nworker\n```\n").is_empty());
    }

    #[test]
    fn flags_plain_but_skips_code_span_when_mixed() {
        // 地の文とコードスパンの混在では地の文側のみ検出する。
        let got = messages("地の worker と `worker` を併記する。");
        assert_eq!(got.len(), 1, "got = {got:?}");
        assert_eq!(got[0].1, "worker => ワーカー");
    }

    #[test]
    fn skips_code_span_nested_in_emphasis() {
        // emphasis にネストした code span も範囲に含めて拾う。
        assert!(messages("これは *`worker`* です。").is_empty());
    }
}
