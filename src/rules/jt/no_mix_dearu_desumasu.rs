//! preset-ja-technical-writing/no-mix-dearu-desumasu (simplified)
//!
//! 各 segment の文末を見て desumasu / dearu に分類し、Document 全体で多数派と異なるものを報告する。
//! upstream は preferInHeader/Body/List を区別するが、ここでは body のみ。Phase 1b 拡張で項目別判定。

use crate::document::{Document, SegmentKind};
use crate::morph::tokenize;
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "no-mix-dearu-desumasu";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    Desumasu,
    Dearu,
}

pub struct NoMixDearuDesumasu;

impl Rule for NoMixDearuDesumasu {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut classified: Vec<(&_, Style)> = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            for (s_start, s_text) in split_sentences(&seg.text) {
                if let Some(style) = classify(s_text) {
                    classified.push((seg, style));
                    let _ = s_start;
                }
            }
        }

        let desumasu_count = classified.iter().filter(|(_, s)| *s == Style::Desumasu).count();
        let dearu_count = classified.iter().filter(|(_, s)| *s == Style::Dearu).count();
        if desumasu_count == 0 || dearu_count == 0 {
            return Vec::new();
        }
        let majority = if desumasu_count >= dearu_count {
            Style::Desumasu
        } else {
            Style::Dearu
        };
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            for (s_start, s_text) in split_sentences(&seg.text) {
                let style = match classify(s_text) {
                    Some(s) => s,
                    None => continue,
                };
                if style != majority {
                    let (line, column) = doc.pos_at(seg, s_start);
                    let label = match style {
                        Style::Desumasu => "ですます",
                        Style::Dearu => "である",
                    };
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: format!(
                            "本文: \"{label}\"調 が混在しています。本文全体で統一してください。"
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

fn classify(sentence: &str) -> Option<Style> {
    let tokens = tokenize(sentence);
    let mut iter = tokens.iter().rev();
    let last = iter.find(|t| t.pos != "記号")?;
    let surface = last.surface.as_str();
    let base = last.base_form.as_str();
    if surface == "です" || surface == "ます" || base == "です" || base == "ます" {
        return Some(Style::Desumasu);
    }
    if surface == "だ" || base == "だ" || surface == "である" || base == "である" {
        return Some(Style::Dearu);
    }
    if last.pos == "助動詞" {
        let s = surface;
        if s.ends_with("です") || s.ends_with("ます") {
            return Some(Style::Desumasu);
        }
        if s.ends_with("だ") || s.ends_with("である") {
            return Some(Style::Dearu);
        }
    }
    None
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
