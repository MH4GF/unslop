//! preset-ja-technical-writing/no-unmatched-pair
//!
//! 各文の中で対 char (括弧・引用符) が閉じていないものを検出する。
//! upstream PairMaker と同じ pair list。文 split は `。!？!?．` 等で行う。

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "no-unmatched-pair";

struct Pair {
    key: &'static str,
    start: char,
    end: char,
}

const PAIRS: &[Pair] = &[
    Pair { key: "double quote", start: '"', end: '"' },
    Pair { key: "angled bracket[]", start: '[', end: ']' },
    Pair { key: "round bracket()", start: '(', end: ')' },
    Pair { key: "curly brace{}", start: '{', end: '}' },
    Pair { key: "かぎ括弧「」", start: '「', end: '」' },
    Pair { key: "丸括弧（）", start: '（', end: '）' },
    Pair { key: "二重かぎ括弧『』", start: '『', end: '』' },
    Pair { key: "波括弧｛｝", start: '｛', end: '｝' },
    Pair { key: "角括弧［］", start: '［', end: '］' },
    Pair { key: "重角括弧〚〛", start: '〚', end: '〛' },
    Pair { key: "隅付き括弧【】", start: '【', end: '】' },
    Pair { key: "double guillemet «»", start: '«', end: '»' },
    Pair { key: "single guillemet ‹›", start: '‹', end: '›' },
];

pub struct NoUnmatchedPair;

impl Rule for NoUnmatchedPair {
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
                let mut stack: Vec<(usize, &Pair)> = Vec::new();
                let mut i = 0usize;
                for c in s_text.chars() {
                    let on_top_same_end = stack
                        .last()
                        .map(|(_, p)| p.end == c)
                        .unwrap_or(false);
                    if on_top_same_end {
                        stack.pop();
                    } else if let Some(p) = PAIRS.iter().find(|p| p.start == c) {
                        stack.push((i, p));
                    } else if let Some(p) = PAIRS.iter().find(|p| p.end == c) {
                        let _ = p;
                    }
                    i += c.len_utf8();
                }
                for (off, p) in stack {
                    let (line, column) = doc.pos_at(seg, s_start + off);
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: format!(
                            "Cannot find a pairing character for {}. You should close this sentence with {}. This pair of marks is called {}.",
                            p.start, p.end, p.key
                        ),
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
