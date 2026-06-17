//! preset-ja-technical-writing/no-doubled-conjunctive-particle-ga
//!
//! 一文内で接続助詞「が」が 2 回以上現れたら error。逆接の「が」を多用すると
//! 文の主述関係が読みにくくなるための rule。auto-fix なし (意味解釈が要る)。

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
            for (s_start, s_text) in split_sentences(&seg.text) {
                let tokens = tokenize(s_text);
                let mut hits: Vec<&crate::morph::Token> = Vec::new();
                for t in &tokens {
                    if t.pos == "助詞" && t.pos_detail_1 == "接続助詞" && t.surface == "が" {
                        hits.push(t);
                    }
                }
                if hits.len() < 2 {
                    continue;
                }
                let second = hits[1];
                let (line, column) = doc.pos_at(seg, s_start + second.byte_start);
                issues.push(Issue {
                    rule_id: RULE_ID.to_string(),
                    message: "一文に二回以上利用されている接続助詞 \"が\" がみつかりました。"
                        .to_string(),
                    line,
                    column,
                    severity: Severity::Error,
                    fix: None,
                });
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
    fn passes_single_ga() {
        assert_eq!(count("雨が降ったが、出かけた。"), 0);
    }

    #[test]
    fn flags_doubled_ga() {
        assert_eq!(count("雨が降ったが、出かけたが、楽しかった。"), 1);
    }

    #[test]
    fn does_not_count_subject_ga() {
        // 主格 (が) は格助詞であって接続助詞ではないので、複数あっても OK
        assert_eq!(count("私が君が好きだ。"), 0);
    }

    #[test]
    fn separates_per_sentence() {
        assert_eq!(count("雨が降ったが、晴れた。風が吹いたが、止んだ。"), 0);
    }
}
