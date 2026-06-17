//! preset-ja-technical-writing/no-doubled-conjunctive-particle-ga
//!
//! 一文中に接続助詞「が」が 2 回以上現れたら error。
//! 形態素解析で品詞「助詞」/品詞細分類「接続助詞」かつ surface == "が" を検出する。
//! upstream は最初の出現位置を index に置く。auto-fix なし。

use crate::document::{Document, SegmentKind};
use crate::morph::tokenize;
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "no-doubled-conjunctive-particle-ga";

pub struct NoDoubledConjunctiveParticleGa;

impl Rule for NoDoubledConjunctiveParticleGa {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            if seg.in_block_quote {
                continue;
            }
            for (s_start, s_text) in split_sentences(&seg.text) {
                let tokens = tokenize(s_text);
                let ga_tokens: Vec<_> = tokens
                    .iter()
                    .filter(|t| {
                        t.pos == "助詞" && t.pos_detail_1 == "接続助詞" && t.surface == "が"
                    })
                    .collect();
                if ga_tokens.len() < 2 {
                    continue;
                }
                let first = ga_tokens[0];
                let (line, column) = doc.pos_at(seg, s_start + first.byte_start);
                issues.push(Issue::new(
                    RULE_ID,
                    "文中に逆接の接続助詞 \"が\" が二回以上使われています。",
                    line,
                    column,
                    Severity::Error,
                ));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn count(src: &str) -> usize {
        let doc = Document::parse(src);
        NoDoubledConjunctiveParticleGa.check(&doc).len()
    }

    #[test]
    fn detects_doubled_ga() {
        assert_eq!(
            count("今日は早朝から出発したが、定刻には間に合わなかったが、無事会場に到着した。\n"),
            1
        );
    }

    #[test]
    fn ok_when_split_by_kuten() {
        // 文を区切れば error にならない
        assert_eq!(
            count("今日は早朝から出発したが、定刻には間に合わなかった。が、無事会場に到着した。\n"),
            0
        );
    }

    #[test]
    fn ok_when_only_once() {
        assert_eq!(count("この文章が問題となっています。\n"), 0);
    }
}
