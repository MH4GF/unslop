//! preset-ja-technical-writing/max-comma (simplified)

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "max-comma";

pub struct MaxComma {
    pub max: usize,
}

impl Default for MaxComma {
    fn default() -> Self {
        Self { max: 3 }
    }
}

impl Rule for MaxComma {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            for sentence in split_sentences(&seg.text) {
                let count = sentence.text.matches(',').count();
                if count > self.max {
                    let last_comma = sentence.text.rfind(',').unwrap_or(0);
                    let abs_offset = sentence.start + last_comma;
                    let (line, column) = doc.pos_at(seg, abs_offset);
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: format!(
                            "This sentence exceeds the maximum count of comma. Maximum is {}.",
                            self.max
                        ),
                        line,
                        column,
                        severity: Severity::Error,
                        fix: None,
                    });
                }
            }
        }
        issues
    }
}

struct Sentence<'a> {
    text: &'a str,
    start: usize,
}

fn split_sentences(text: &str) -> Vec<Sentence<'_>> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    for c in text.chars() {
        let next = i + c.len_utf8();
        if matches!(c, '。' | '！' | '？' | '\n') {
            out.push(Sentence {
                text: &text[start..next],
                start,
            });
            start = next;
        }
        i = next;
    }
    if start < text.len() {
        out.push(Sentence {
            text: &text[start..],
            start,
        });
    }
    out
}
