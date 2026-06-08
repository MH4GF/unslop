//! preset-ja-technical-writing/max-ten

use crate::document::{Document, SegmentKind};
use crate::morph::tokenize;
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "max-ten";

pub struct MaxTen {
    pub max: usize,
}

impl Default for MaxTen {
    fn default() -> Self {
        Self { max: 3 }
    }
}

impl Rule for MaxTen {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            for (s_start, s_text) in split_sentences(&seg.text) {
                let tokens = tokenize(s_text);
                let touten_positions: Vec<usize> = tokens
                    .iter()
                    .filter(|t| t.surface == "、" && t.pos == "記号")
                    .map(|t| t.byte_start)
                    .collect();
                if touten_positions.len() > self.max {
                    let last = *touten_positions.last().unwrap();
                    let (line, column) = doc.pos_at(seg, s_start + last);
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: format!(
                            "This sentence exceeds the maximum count of ten. Maximum is {}.",
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

fn split_sentences(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    for c in text.chars() {
        let next = i + c.len_utf8();
        if matches!(c, '。' | '！' | '？' | '!' | '?' | '．' | '\n') {
            out.push((start, &text[start..next]));
            start = next;
        }
        i = next;
    }
    if start < text.len() {
        out.push((start, &text[start..]));
    }
    out
}
