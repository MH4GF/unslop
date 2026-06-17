//! preset-ja-technical-writing/ja-no-mixed-period (simplified)
//!
//! 文末の終止符を `。` に統一する。文末が `.` / `．` / `:` / `：` なら error と
//! し、`。` への置換 fix を出す。`?` / `！` / `？` / `!` 等は対象外。
//!
//! upstream は文末 emoji 許可や allowPeriods 設定など細かい option を持つが、
//! 本実装は preset の default 相当のみで simplified。code span / link URL 内は
//! 除外する。

use crate::document::{Document, SegmentKind};
use crate::rule::{Fix, Issue, Rule, Severity};

const RULE_ID: &str = "ja-no-mixed-period";

pub struct JaNoMixedPeriod;

impl Rule for JaNoMixedPeriod {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            // 日本語文字を 1 つも含まない英文 paragraph は対象外。
            // 英文末尾の ASCII `.` が大量に false positive になるのを避ける。
            if !contains_japanese_char(&seg.text) {
                continue;
            }
            for (s_start, s_text) in split_sentences(&seg.text) {
                let trimmed_len = s_text.trim_end_matches(['\n', '\r', ' ', '\t']).len();
                let trimmed = &s_text[..trimmed_len];
                let Some((last_off, last_ch)) = trimmed.char_indices().next_back() else {
                    continue;
                };
                if !matches!(last_ch, '.' | '．' | ':' | '：') {
                    continue;
                }
                let rel = s_start + last_off;
                let end_rel = rel + last_ch.len_utf8();
                let in_code = seg
                    .code_ranges
                    .iter()
                    .any(|&(cs, ce)| rel < ce && cs < end_rel);
                let in_link = seg
                    .link_url_ranges
                    .iter()
                    .any(|&(cs, ce)| rel < ce && cs < end_rel);
                if in_code || in_link {
                    continue;
                }
                let (line, column) = doc.pos_at(seg, rel);
                let abs = seg.start_byte + rel;
                issues.push(
                    Issue::new(
                        RULE_ID,
                        format!(
                            "Disallow to use \"{last_ch}\" as sentence terminator; use \"。\"."
                        ),
                        line,
                        column,
                        Severity::Error,
                    )
                    .with_fix(Fix {
                        range: abs..abs + last_ch.len_utf8(),
                        replacement: "。".to_string(),
                    }),
                );
            }
        }
        issues
    }
}

fn contains_japanese_char(text: &str) -> bool {
    text.chars().any(|c| {
        ('\u{3040}'..='\u{309F}').contains(&c)
            || ('\u{30A0}'..='\u{30FF}').contains(&c)
            || ('\u{4E00}'..='\u{9FFF}').contains(&c)
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(src: &str) -> Vec<(usize, String)> {
        let doc = Document::parse(src);
        JaNoMixedPeriod
            .check(&doc)
            .into_iter()
            .map(|i| (i.line, i.message))
            .collect()
    }

    #[test]
    fn passes_japanese_period() {
        assert!(messages("これは正しい文です。").is_empty());
    }

    #[test]
    fn flags_ascii_period() {
        let got = messages("これは混在の文です.");
        assert_eq!(got.len(), 1);
    }

    #[test]
    fn flags_fullwidth_period() {
        let got = messages("これは混在の文です．");
        assert_eq!(got.len(), 1);
    }

    #[test]
    fn flags_trailing_colon() {
        let got = messages("以下のとおり:");
        assert_eq!(got.len(), 1);
    }

    #[test]
    fn passes_question_or_exclamation() {
        assert!(messages("本当?").is_empty());
        assert!(messages("本当？").is_empty());
    }

    #[test]
    fn skips_code_span_period() {
        // `obj.method.` は code span 内、末尾 `.` は code 内なのでスキップ
        assert!(messages("呼び出しは `obj.method.`").is_empty());
    }

    #[test]
    fn fix_replaces_with_japanese_period() {
        let src = "これは混在の文です.";
        let doc = Document::parse(src);
        let issue = JaNoMixedPeriod.check(&doc).into_iter().next().unwrap();
        let f = issue.fix.unwrap();
        let mut buf = src.to_string();
        buf.replace_range(f.range, &f.replacement);
        assert_eq!(buf, "これは混在の文です。");
    }
}
