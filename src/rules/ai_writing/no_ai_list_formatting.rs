//! preset-ai-writing/no-ai-list-formatting
//! upstream: src/rules/no-ai-list-formatting.ts

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "@textlint-ja/preset-ai-writing/no-ai-list-formatting";

// upstream の boldListPattern から bold-colon 検出を除いた版 (本人運用 hook の FP 抑止)
static BOLD_LIST: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\s]*(?:[-*+]|\d+[.)])\s+\*\*([^*]+)\*\*\s+([-—–])(?=\s)").unwrap());

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
                let e = full.end();
                let in_code = seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                if !in_code {
                    let sep = m.get(2).map(|x| x.as_str()).unwrap_or("");
                    let sep_name = match sep {
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
                        fix: None,
                    });
                }
            }

            // flashy emoji (first match only)
            if let Ok(Some(m)) = FLASHY_EMOJI.find(&seg.text) {
                let emoji = m.as_str();
                let s = m.start();
                let e = m.end();
                let in_code = seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                if !in_code {
                    let (line, column) = doc.pos_at(seg, s);
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: format!(
                            "リストアイテムでの絵文字「{emoji}」の使用は、読み手によっては機械的な印象を与える場合があります。テキストベースの表現も検討してみてください。"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    fn rule_ids(src: &str) -> Vec<String> {
        let doc = Document::parse(src);
        NoAiListFormatting
            .check(&doc)
            .into_iter()
            .map(|i| i.message)
            .collect()
    }

    #[test]
    fn bold_colon_no_longer_detected() {
        assert!(rule_ids("- **重要**: これは重要な項目です\n").is_empty());
        assert!(rule_ids("- **重要情報**：これは重要な項目です\n").is_empty());
    }

    #[test]
    fn bold_dash_still_detected() {
        assert_eq!(rule_ids("- **重要** — 説明文がここに入る\n").len(), 1);
    }

    #[test]
    fn emoji_still_detected() {
        assert_eq!(rule_ids("- ✅ チェック項目\n").len(), 1);
    }

    #[test]
    fn bold_dash_in_code_span_skipped() {
        // ListItem 全体が code span 化されている
        assert!(rule_ids("- `**foo** — bar`\n").is_empty());
    }

    #[test]
    fn emoji_in_code_span_skipped() {
        assert!(rule_ids("- `✅` チェック項目\n").is_empty());
    }
}
