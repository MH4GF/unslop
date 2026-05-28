//! preset-ai-writing/no-ai-list-formatting
//! upstream: src/rules/no-ai-list-formatting.ts

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "@textlint-ja/preset-ai-writing/no-ai-list-formatting";

// upstream の boldListPattern と完全一致
static BOLD_LIST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[\s]*(?:[-*+]|\d+[.)])\s+\*\*([^*]+)\*\*(?:\s*([:：])|\s+([-—–])(?=\s))").unwrap()
});

const FLASHY_EMOJIS: &[&str] = &[
    "✅", "❌", "⭐", "✨", "💯", "⚠️", "❗", "❓", "💥", "🔥", "⚡", "💪", "🚀", "💡", "🤔", "💭",
    "🧠", "🎯", "📈", "📊", "🏆", "👍", "👎", "😊", "😎", "🎉", "🌟", "📝", "📋", "✏️", "🖊️", "💼",
];

static FLASHY_EMOJI: Lazy<Regex> = Lazy::new(|| {
    let parts: Vec<String> = FLASHY_EMOJIS
        .iter()
        .map(|e| fancy_regex::escape(e).to_string())
        .collect();
    Regex::new(&format!("({})", parts.join("|"))).unwrap()
});

pub struct NoAiListFormatting;

impl Rule for NoAiListFormatting {
    fn id(&self) -> &str {
        RULE_ID
    }

    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::ListItem {
                continue;
            }

            // bold list pattern (first match only)
            if let Ok(Some(m)) = BOLD_LIST.captures(&seg.text) {
                let full = m.get(0).unwrap();
                let s = full.start();
                let sep = m
                    .get(2)
                    .or_else(|| m.get(3))
                    .map(|x| x.as_str())
                    .unwrap_or("");
                let sep_name = match sep {
                    ":" | "：" => "コロン",
                    "-" => "ハイフン",
                    "—" | "–" => "ダッシュ",
                    _ => "区切り文字",
                };
                let (line, column) = doc.pos_at(seg, s);
                issues.push(Issue {
                    rule_id: RULE_ID.to_string(),
                    message: format!(
                        "リストアイテムで強調（**）と{sep_name}（{sep}）の組み合わせは機械的な印象を与える可能性があります。より自然な表現を検討してください。"
                    ),
                    line,
                    column,
                    severity: Severity::Error,
                });
            }

            // flashy emoji (first match only)
            if let Ok(Some(m)) = FLASHY_EMOJI.find(&seg.text) {
                let emoji = m.as_str();
                let s = m.start();
                let (line, column) = doc.pos_at(seg, s);
                issues.push(Issue {
                    rule_id: RULE_ID.to_string(),
                    message: format!(
                        "リストアイテムでの絵文字「{emoji}」の使用は、読み手によっては機械的な印象を与える場合があります。テキストベースの表現も検討してみてください。"
                    ),
                    line,
                    column,
                    severity: Severity::Error,
                });
            }
        }
        issues
    }
}
