//! preset-ja-technical-writing/no-hankaku-kana

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::Document;
use crate::rule::{Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "no-hankaku-kana";

static HANKAKU: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\u{ff61}-\u{ff9f}]+)").unwrap());

pub struct NoHankakuKana;

impl Rule for NoHankakuKana {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            let mut from = 0usize;
            while let Ok(Some(m)) = HANKAKU.find_from_pos(&seg.text, from) {
                let s = m.start();
                let e = m.end();
                let (line, column) = doc.pos_at(seg, s);
                issues.push(Issue {
                    rule_id: RULE_ID.to_string(),
                    message: format!("Disallow to use 半角カタカナ: \"{}\"", m.as_str()),
                    line,
                    column,
                    severity: Severity::Error,
                });
                from = e.max(s + 1);
            }
        }
        issues
    }
}
