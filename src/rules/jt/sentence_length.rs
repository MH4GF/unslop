//! preset-ja-technical-writing/sentence-length (simplified)

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "sentence-length";

pub struct SentenceLength {
    pub max: usize,
}

impl Default for SentenceLength {
    fn default() -> Self {
        Self { max: 100 }
    }
}

fn is_sentence_terminator(c: char) -> bool {
    matches!(c, '。' | '！' | '？' | '\n')
}

impl Rule for SentenceLength {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !matches!(
                seg.kind,
                SegmentKind::Paragraph | SegmentKind::Heading | SegmentKind::ListItem
            ) {
                continue;
            }
            let mut sentence_start = 0usize;
            let bytes = seg.text.as_bytes();
            let mut i = 0usize;
            while i <= bytes.len() {
                let at_end = i == bytes.len();
                let cur = if at_end {
                    None
                } else {
                    seg.text[i..].chars().next()
                };
                let terminator = cur.map(is_sentence_terminator).unwrap_or(false);
                if terminator || at_end {
                    let s = sentence_start;
                    let e = if at_end {
                        i
                    } else {
                        i + cur.unwrap().len_utf8()
                    };
                    let snippet = seg.text[s..e].trim();
                    let len = snippet.encode_utf16().count();
                    if len > self.max {
                        let (line, column) = doc.pos_at(seg, s);
                        let over = len - self.max;
                        issues.push(Issue {
                            rule_id: RULE_ID.to_string(),
                            message: format!(
                                "Line {} sentence length({}) exceeds the maximum sentence length of {}. Over {} characters.",
                                line, len, self.max, over
                            ),
                            line,
                            column,
                            severity: Severity::Error,
                        });
                    }
                    sentence_start = e;
                }
                if at_end {
                    break;
                }
                i += cur.unwrap().len_utf8();
            }
        }
        issues
    }
}
