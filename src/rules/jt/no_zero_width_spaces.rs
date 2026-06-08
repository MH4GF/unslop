//! preset-ja-technical-writing/no-zero-width-spaces
//! upstream: textlint-rule-no-zero-width-spaces
//!
//! 対象は U+200B のみ (upstream 互換)。auto-fix で削除する。

use crate::document::Document;
use crate::rule::{Fix, Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "no-zero-width-spaces";

pub struct NoZeroWidthSpaces;

impl Rule for NoZeroWidthSpaces {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            for (i, c) in seg.text.char_indices() {
                if c == '\u{200B}' {
                    let (line, column) = doc.pos_at(seg, i);
                    let abs = seg.start_byte + i;
                    issues.push(
                        Issue::new(
                            RULE_ID,
                            "Zero width space is disallowed.",
                            line,
                            column,
                            Severity::Error,
                        )
                        .with_fix(Fix {
                            range: abs..abs + c.len_utf8(),
                            replacement: String::new(),
                        }),
                    );
                }
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_fix_range(src: &str) -> Option<std::ops::Range<usize>> {
        let doc = Document::parse(src);
        NoZeroWidthSpaces
            .check(&doc)
            .into_iter()
            .next()
            .and_then(|i| i.fix.map(|f| f.range))
    }

    #[test]
    fn detects_zwsp() {
        let doc = Document::parse("あ\u{200B}い");
        assert_eq!(NoZeroWidthSpaces.check(&doc).len(), 1);
    }

    #[test]
    fn fix_targets_three_byte_zwsp() {
        let r = first_fix_range("あ\u{200B}い").expect("fix expected");
        assert_eq!(r.end - r.start, '\u{200B}'.len_utf8());
    }

    #[test]
    fn fix_applied_removes_zwsp() {
        let src = "あ\u{200B}い";
        let doc = Document::parse(src);
        let issue = NoZeroWidthSpaces.check(&doc).into_iter().next().unwrap();
        let f = issue.fix.unwrap();
        let mut buf = src.to_string();
        buf.replace_range(f.range, &f.replacement);
        assert_eq!(buf, "あい");
    }
}
