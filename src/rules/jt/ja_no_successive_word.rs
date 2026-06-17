//! preset-ja-technical-writing/ja-no-successive-word (simplified)
//!
//! 形態素解析で隣接する 2 token の surface が一致したら error。
//! 「私はは行く」「行く行く」のような同一語連続を検出する。auto-fix なし。
//!
//! 助詞 / 助動詞 / 記号 / 接頭詞 / フィラー / その他 は対象外。これらは
//! 文法上連続することがある (「の の」「々」など) ため過剰検出を避ける。

use crate::document::{Document, SegmentKind};
use crate::morph::tokenize;
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "ja-no-successive-word";

fn skippable_pos(pos: &str) -> bool {
    matches!(
        pos,
        "助詞" | "助動詞" | "記号" | "接頭詞" | "フィラー" | "その他"
    )
}

pub struct JaNoSuccessiveWord;

impl Rule for JaNoSuccessiveWord {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph
                && seg.kind != SegmentKind::Heading
                && seg.kind != SegmentKind::ListItem
                && seg.kind != SegmentKind::TableCell
            {
                continue;
            }
            let tokens = tokenize(&seg.text);
            for pair in tokens.windows(2) {
                let a = &pair[0];
                let b = &pair[1];
                if a.surface != b.surface {
                    continue;
                }
                if skippable_pos(&a.pos) || skippable_pos(&b.pos) {
                    continue;
                }
                // 1 文字の繰り返し (ひらがな・カタカナ 1 文字単独 token) は除外する。
                // 「ぱ ぱ」のような擬音や、textlint upstream で false positive となるため。
                if a.surface.chars().count() == 1 {
                    continue;
                }
                let (line, column) = doc.pos_at(seg, b.byte_start);
                issues.push(Issue {
                    rule_id: RULE_ID.to_string(),
                    message: format!("\"{}\" が連続して2回使われています。", a.surface),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn count(src: &str) -> usize {
        let doc = Document::parse(src);
        JaNoSuccessiveWord.check(&doc).len()
    }

    #[test]
    fn flags_repeated_noun() {
        assert!(count("これは技術技術の話です。") >= 1);
    }

    #[test]
    fn passes_normal_text() {
        assert_eq!(count("これは普通の文章です。"), 0);
    }

    #[test]
    fn passes_repeated_particle() {
        // 助詞は skip
        assert_eq!(count("そうですよ ね ね。"), 0);
    }
}
