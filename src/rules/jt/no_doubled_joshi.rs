//! preset-ja-technical-writing/no-doubled-joshi
//!
//! 同一文内に同じ助詞 (surface + pos_detail_1) が複数回出てきたら error。
//! 例外: 連体化「の」、格助詞「を」、接続助詞「て」、並立助詞、〜か〜か。

use std::collections::BTreeMap;

use crate::document::{Document, SegmentKind};
use crate::morph::{Token, tokenize};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "no-doubled-joshi";

pub struct NoDoubledJoshi;

impl Rule for NoDoubledJoshi {
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
                let mut groups: BTreeMap<(String, String), Vec<&Token>> = BTreeMap::new();
                for t in &tokens {
                    if t.pos != "助詞" {
                        continue;
                    }
                    let key = (t.surface.clone(), t.pos_detail_1.clone());
                    groups.entry(key).or_default().push(t);
                }
                for ((surface, _detail), members) in &groups {
                    if members.len() < 2 {
                        continue;
                    }
                    if is_exception(members) {
                        continue;
                    }
                    let second = members[1];
                    let (line, column) = doc.pos_at(seg, s_start + second.byte_start);
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: format!(
                            "一文に二回以上利用されている助詞 \"{surface}\" がみつかりました。"
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

fn is_exception(members: &[&Token]) -> bool {
    let first = members[0];
    if first.pos_detail_1 == "連体化" {
        return true;
    }
    if first.pos_detail_1 == "格助詞" && first.surface == "を" {
        return true;
    }
    if first.pos_detail_1 == "接続助詞" && first.surface == "て" {
        return true;
    }
    if members.len() == 2
        && first.pos_detail_1 == "並立助詞"
        && members[1].pos_detail_1 == "並立助詞"
    {
        return true;
    }
    false
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
