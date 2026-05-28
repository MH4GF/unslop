//! preset-ja-technical-writing/no-doubled-conjunction
//!
//! 隣接する文の冒頭にある接続詞が同じなら error。

use crate::document::{Document, SegmentKind};
use crate::morph::tokenize;
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "no-doubled-conjunction";

pub struct NoDoubledConjunction;

impl Rule for NoDoubledConjunction {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            let mut prev_conjunction: Option<String> = None;
            for (s_start, s_text) in split_sentences(&seg.text) {
                let tokens = tokenize(s_text);
                let first_conj = tokens.iter().enumerate().find(|(i, t)| {
                    if t.pos != "接続詞" {
                        return false;
                    }
                    if let Some(prev) = i.checked_sub(1).and_then(|j| tokens.get(j))
                        && prev.pos_detail_1 == "空白"
                    {
                        return false;
                    }
                    true
                });
                if let Some((_, conj)) = first_conj {
                    if let Some(prev) = &prev_conjunction
                        && prev == &conj.surface
                    {
                        let (line, column) = doc.pos_at(seg, s_start + conj.byte_start);
                        issues.push(Issue {
                            rule_id: RULE_ID.to_string(),
                            message: format!(
                                "同じ接続詞（{}）が連続して使われています。",
                                conj.surface
                            ),
                            line,
                            column,
                            severity: Severity::Error,
                        });
                    }
                    prev_conjunction = Some(conj.surface.clone());
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
