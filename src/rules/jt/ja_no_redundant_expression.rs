//! preset-ja-technical-writing/ja-no-redundant-expression (simplified)
//!
//! 冗長表現を辞書ベースで検出し、auto-fix で置換する。upstream の辞書は大規模
//! だが、`ai-tech-writing-guideline` と重複する範囲が広いため、本実装は重複が
//! 少なく確度の高いパターンに絞った。

use crate::document::Document;
use crate::rule::{Fix, Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "ja-no-redundant-expression";

struct Entry {
    pattern: &'static str,
    replacement: &'static str,
}

const ENTRIES: &[Entry] = &[
    Entry {
        pattern: "することが可能です",
        replacement: "できます",
    },
    Entry {
        pattern: "することが可能",
        replacement: "できる",
    },
    Entry {
        pattern: "することが出来ます",
        replacement: "できます",
    },
    Entry {
        pattern: "することが出来る",
        replacement: "できる",
    },
    Entry {
        pattern: "ということが言えます",
        replacement: "といえます",
    },
    Entry {
        pattern: "ということが言える",
        replacement: "といえる",
    },
];

pub struct JaNoRedundantExpression;

impl Rule for JaNoRedundantExpression {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            let text = &seg.text;
            // 長いパターンを先に走らせ、後段の短いパターン (substring) が同じ範囲を再検出しないように consumed を保持する。
            let mut consumed: Vec<(usize, usize)> = Vec::new();
            for entry in ENTRIES {
                let mut from = 0usize;
                while let Some(rel) = text[from..].find(entry.pattern) {
                    let s = from + rel;
                    let e = s + entry.pattern.len();
                    let in_code = seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                    let in_link = seg.link_url_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                    let overlaps = consumed.iter().any(|&(cs, ce)| s < ce && cs < e);
                    if !in_code && !in_link && !overlaps {
                        let (line, column) = doc.pos_at(seg, s);
                        let abs = seg.start_byte + s;
                        issues.push(
                            Issue::new(
                                RULE_ID,
                                format!(
                                    "冗長表現 \"{}\" がみつかりました。\"{}\" への簡潔化を検討してください。",
                                    entry.pattern, entry.replacement
                                ),
                                line,
                                column,
                                Severity::Error,
                            )
                            .with_fix(Fix {
                                range: abs..abs + entry.pattern.len(),
                                replacement: entry.replacement.to_string(),
                            }),
                        );
                        consumed.push((s, e));
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

    fn count(src: &str) -> usize {
        let doc = Document::parse(src);
        JaNoRedundantExpression.check(&doc).len()
    }

    #[test]
    fn flags_kanou() {
        assert_eq!(count("これを実行することが可能です。"), 1);
    }

    #[test]
    fn flags_dekiru() {
        assert_eq!(count("これを実行することが出来る。"), 1);
    }

    #[test]
    fn passes_normal() {
        assert_eq!(count("これを実行できます。"), 0);
    }

    #[test]
    fn skips_code_span() {
        assert_eq!(count("`することが可能` という文字列です。"), 0);
    }

    #[test]
    fn fix_replaces() {
        let src = "これを実行することが可能です。";
        let doc = Document::parse(src);
        let issue = JaNoRedundantExpression
            .check(&doc)
            .into_iter()
            .next()
            .unwrap();
        let f = issue.fix.unwrap();
        let mut buf = src.to_string();
        buf.replace_range(f.range, &f.replacement);
        assert_eq!(buf, "これを実行できます。");
    }
}
