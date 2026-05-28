//! preset-ai-writing/no-ai-emphasis-patterns
//!
//! upstream: tests/upstream/textlint-rule-preset-ai-writing/src/rules/no-ai-emphasis-patterns.ts
//!
//! 検出対象:
//! 1. 絵文字 + 太字 (`<emoji>\s*\*\*<text>\*\*`) を Paragraph / ListItem で検出
//! 2. 情報系プレフィックス太字 (`\*\*(注意|重要|...)([:：]...?)?\*\*`) を Paragraph / ListItem で検出
//!    - 上の絵文字+太字マッチと重複する箇所はスキップ
//! 3. 見出し内の任意の太字 (`(**|__).+?\1`) を Heading で検出

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "@textlint-ja/preset-ai-writing/no-ai-emphasis-patterns";

const INFO_PATTERNS: &[&str] = &[
    "注意", "重要", "ポイント", "メモ", "参考", "補足", "確認", "チェック", "推奨", "おすすめ",
    "検出される例", "推奨される表現", "良い例", "悪い例", "例", "サンプル", "使用例", "設定例",
];

// 絵文字 + 任意空白 + **text**
static EMOJI_EMPHASIS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(ℹ️|🔍|✅|❌|⚠️|💡|📝|📋|📌|🔗|🎯|🚀|⭐|✨|💯|🔥|📊|📈)\s*\*\*([^*]+)\*\*")
        .unwrap()
});

// **(prefix)([:：]...?)?**
static INFO_PREFIX: Lazy<Regex> = Lazy::new(|| {
    let alt = INFO_PATTERNS.join("|");
    let pat = format!(r"\*\*({alt})([：:].*?)?\*\*");
    Regex::new(&pat).unwrap()
});

// 見出し内 (**X** or __X__)
static HEADING_EMPHASIS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\*\*|__)(.*?)\1").unwrap());

pub struct NoAiEmphasisPatterns;

impl Rule for NoAiEmphasisPatterns {
    fn id(&self) -> &str {
        RULE_ID
    }

    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            match seg.kind {
                SegmentKind::Paragraph => {
                    let (mut a, e_ranges) = emoji_emphasis_issues(doc, seg, false);
                    issues.append(&mut a);
                    let mut b = info_prefix_issues(doc, seg, &e_ranges, false);
                    issues.append(&mut b);
                }
                SegmentKind::ListItem => {
                    let (mut a, e_ranges) = emoji_emphasis_issues(doc, seg, true);
                    issues.append(&mut a);
                    let mut b = info_prefix_issues(doc, seg, &e_ranges, true);
                    issues.append(&mut b);
                }
                SegmentKind::Heading => {
                    let mut a = heading_emphasis_issues(doc, seg);
                    issues.append(&mut a);
                }
                _ => {}
            }
        }
        issues
    }
}

fn emoji_emphasis_issues(
    doc: &Document,
    seg: &crate::document::TextSegment,
    list_item: bool,
) -> (Vec<Issue>, Vec<(usize, usize)>) {
    let mut issues = Vec::new();
    let mut ranges = Vec::new();
    let mut start_from = 0usize;
    while let Ok(Some(m)) = EMOJI_EMPHASIS.find_from_pos(&seg.text, start_from) {
        let s = m.start();
        let e = m.end();
        ranges.push((s, e));
        let (line, column) = doc.pos_at(seg, s);
        let msg = if list_item {
            "リストアイテムで絵文字と太字の組み合わせは機械的な印象を与える可能性があります。より自然な表現を検討してください。"
        } else {
            "絵文字と太字の組み合わせは機械的な印象を与える可能性があります。より自然な表現を検討してください。"
        };
        issues.push(Issue {
            rule_id: RULE_ID.to_string(),
            message: msg.to_string(),
            line,
            column,
            severity: Severity::Error,
        });
        start_from = e.max(s + 1);
    }
    (issues, ranges)
}

fn info_prefix_issues(
    doc: &Document,
    seg: &crate::document::TextSegment,
    emoji_ranges: &[(usize, usize)],
    list_item: bool,
) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut start_from = 0usize;
    while let Ok(Some(m)) = INFO_PREFIX.find_from_pos(&seg.text, start_from) {
        let s = m.start();
        let e = m.end();
        let overlaps = emoji_ranges.iter().any(|&(es, ee)| s < ee && e > es);
        if !overlaps {
            let prefix = m
                .as_str()
                .strip_prefix("**")
                .and_then(|t| t.split("**").next())
                .and_then(|t| t.split(':').next())
                .and_then(|t| t.split('：').next())
                .unwrap_or("");
            let (line, column) = doc.pos_at(seg, s);
            let msg = if list_item {
                format!(
                    "リストアイテムで「**{prefix}**」のような太字の情報プレフィックスは機械的な印象を与える可能性があります。より自然な表現を検討してください。"
                )
            } else {
                format!(
                    "「**{prefix}**」のような太字の情報プレフィックスは機械的な印象を与える可能性があります。より自然な表現を検討してください。"
                )
            };
            issues.push(Issue {
                rule_id: RULE_ID.to_string(),
                message: msg,
                line,
                column,
                severity: Severity::Error,
            });
        }
        start_from = e.max(s + 1);
    }
    issues
}

fn heading_emphasis_issues(doc: &Document, seg: &crate::document::TextSegment) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut start_from = 0usize;
    while let Ok(Some(m)) = HEADING_EMPHASIS.find_from_pos(&seg.text, start_from) {
        let s = m.start();
        let e = m.end();
        let (line, column) = doc.pos_at(seg, s);
        issues.push(Issue {
            rule_id: RULE_ID.to_string(),
            message: "見出し内の太字は不要です。見出し自体が強調のため、追加の太字は冗長です。"
                .to_string(),
            line,
            column,
            severity: Severity::Error,
        });
        start_from = e.max(s + 1);
    }
    issues
}
