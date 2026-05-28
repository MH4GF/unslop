//! preset-ja-technical-writing/no-zero-width-spaces
//! upstream: textlint-rule-no-zero-width-spaces

use crate::document::Document;
use crate::rule::{Issue, Rule, Severity};
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
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: "Zero width space is disallowed.".to_string(),
                        line,
                        column,
                        severity: Severity::Error,
                    });
                }
            }
        }
        issues
    }
}
