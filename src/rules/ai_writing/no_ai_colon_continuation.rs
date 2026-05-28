//! preset-ai-writing/no-ai-colon-continuation (simplified)
//!
//! Paragraph が `:` `：` で終わり、直後の segment が List/BlockQuote/Table/CodeBlock 相当で、
//! コロン直前の text 末尾品詞が 動詞 / 形容詞 / 助動詞 / 接続詞 の場合に error。
//!
//! upstream は Document AST を見るが、ここでは Document.segments の隣接判定で近似する。

use crate::document::{Document, SegmentKind};
use crate::morph::tokenize;
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "@textlint-ja/preset-ai-writing/no-ai-colon-continuation";

pub struct NoAiColonContinuation;

impl Rule for NoAiColonContinuation {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for i in 0..doc.segments.len() {
            let seg = &doc.segments[i];
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            let trimmed = seg.text.trim_end();
            let ends_full = trimmed.ends_with('：');
            let ends_half = trimmed.ends_with(':');
            if !ends_full && !ends_half {
                continue;
            }
            let next = match doc.segments.get(i + 1) {
                Some(n) => n,
                None => continue,
            };
            if !matches!(next.kind, SegmentKind::ListItem | SegmentKind::TableCell) {
                continue;
            }
            let colon_char = if ends_full { "：" } else { ":" };
            let before = trimmed.trim_end_matches(colon_char).trim();
            if before.chars().count() <= 2 {
                continue;
            }
            if is_english_only(before) {
                continue;
            }
            let tokens = tokenize(before);
            let last = match tokens.last() {
                Some(t) => t,
                None => continue,
            };
            let problematic = matches!(last.pos.as_str(), "動詞" | "形容詞" | "助動詞" | "接続詞");
            if !problematic {
                continue;
            }
            let colon_byte = match seg.text.rfind(colon_char) {
                Some(b) => b,
                None => continue,
            };
            let (line, column) = doc.pos_at(seg, colon_byte);
            issues.push(Issue {
                rule_id: RULE_ID.to_string(),
                message: format!(
                    "「{before}{colon_char}」のような述語とコロンで終わるパターンは、読み手によっては英語の構文を直訳したような印象を与える場合があります。「次のように〜します。」のような自然な日本語表現を検討してください。"
                ),
                line,
                column,
                severity: Severity::Error,
            });
        }
        issues
    }
}

fn is_english_only(s: &str) -> bool {
    let trimmed = s.trim();
    if !trimmed.chars().any(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
}
