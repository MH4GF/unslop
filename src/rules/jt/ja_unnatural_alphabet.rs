//! preset-ja-technical-writing/ja-unnatural-alphabet (simplified)

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::Document;
use crate::rule::{Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "ja-unnatural-alphabet";

static UNNATURAL: Lazy<Regex> = Lazy::new(|| {
    let ja = r"(?:[々〇〻\u{3400}-\u{4DBF}\u{4E00}-\u{9FFF}\u{F900}-\u{FAFF}\u{FF00}-\u{FFEF}ぁ-んァ-ヶー。、・−])";
    let alpha = r"([a-zA-Zａ-ｚＡ-Ｚ])";
    Regex::new(&format!("{ja}{alpha}{ja}")).unwrap()
});

fn allowed(c: char) -> bool {
    matches!(c, 'a' | 'i' | 'u' | 'e' | 'o' | 'n') || c.is_ascii_uppercase()
}

pub struct JaUnnaturalAlphabet;

impl Rule for JaUnnaturalAlphabet {
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
            while let Ok(Some(m)) = UNNATURAL.captures_from_pos(&seg.text, from) {
                let alpha_match = m.get(1).unwrap();
                let alpha_char = alpha_match.as_str().chars().next().unwrap();
                let alpha_start = alpha_match.start();
                let full = m.get(0).unwrap();
                if !allowed(alpha_char) {
                    let (line, column) = doc.pos_at(seg, alpha_start);
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: format!(
                            "不自然なアルファベットがあります: \"{}\"",
                            full.as_str()
                        ),
                        line,
                        column,
                        severity: Severity::Error,
                    });
                }
                from = (alpha_start + 1).max(from + 1);
            }
        }
        issues
    }
}
