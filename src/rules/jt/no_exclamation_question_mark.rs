//! preset-ja-technical-writing/no-exclamation-question-mark
//! upstream: textlint-rule-no-exclamation-question-mark

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::Document;
use crate::rule::{Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "no-exclamation-question-mark";
const BUILTIN_ALLOW: &[&str] = &["Yahoo!"];

static HALF_EXCL: Lazy<Regex> = Lazy::new(|| Regex::new(r"!").unwrap());
static FULL_EXCL: Lazy<Regex> = Lazy::new(|| Regex::new(r"！").unwrap());
static HALF_Q: Lazy<Regex> = Lazy::new(|| Regex::new(r"\?").unwrap());
static FULL_Q: Lazy<Regex> = Lazy::new(|| Regex::new(r"？").unwrap());

pub struct NoExclamationQuestionMark;

impl Rule for NoExclamationQuestionMark {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            // builtin allow list の範囲を予め計算 (重複検出回避)
            let ignored_ranges = compute_ignored_ranges(&seg.text);
            for re in [&*HALF_EXCL, &*FULL_EXCL, &*HALF_Q, &*FULL_Q] {
                let mut from = 0usize;
                while let Ok(Some(m)) = re.find_from_pos(&seg.text, from) {
                    let s = m.start();
                    let e = m.end();
                    let text = m.as_str().to_string();
                    let in_code = seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                    let in_link_url = seg.link_url_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                    let in_allow = ignored_ranges.iter().any(|(rs, re_)| *rs <= s && s <= *re_);
                    if !in_code && !in_link_url && !in_allow {
                        let (line, column) = doc.pos_at(seg, s);
                        issues.push(Issue {
                            rule_id: RULE_ID.to_string(),
                            message: format!("Disallow to use \"{text}\"."),
                            line,
                            column,
                            severity: Severity::Error,
                        });
                    }
                    from = e.max(s + 1);
                }
            }
        }
        issues
    }
}

fn compute_ignored_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for word in BUILTIN_ALLOW {
        let mut from = 0usize;
        while let Some(i) = text[from..].find(word) {
            let abs = from + i;
            out.push((abs, abs + word.len()));
            from = abs + word.len();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    fn messages(src: &str) -> Vec<(usize, String)> {
        let doc = Document::parse(src);
        NoExclamationQuestionMark
            .check(&doc)
            .into_iter()
            .map(|i| (i.line, i.message))
            .collect()
    }

    #[test]
    fn flags_plain_text_question_mark() {
        let got = messages("これ本当?");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].1, "Disallow to use \"?\".");
    }

    #[test]
    fn passes_yahoo_allowlist() {
        assert!(messages("Yahoo!検索").is_empty());
    }

    #[test]
    fn skips_inline_link_url() {
        let src = "[週次](obsidian://open?vault=w&file=a.md)";
        assert!(messages(src).is_empty(), "got = {:?}", messages(src));
    }

    #[test]
    fn skips_angle_autolink_url() {
        assert!(messages("<https://x.test/a?b=1>").is_empty());
    }

    #[test]
    fn skips_image_url() {
        assert!(messages("![alt](/p/q.png?x=1)").is_empty());
    }

    #[test]
    fn skips_inline_code_span() {
        assert!(messages("呼び出しは `obj.method?` です。").is_empty());
    }

    #[test]
    fn flags_label_question_with_clean_url() {
        // label 内の全角 ？ は flag、URL の `?` は skip
        let got = messages("[ヤバ？い](https://x.test)");
        assert_eq!(got.len(), 1, "got = {got:?}");
        assert_eq!(got[0].1, "Disallow to use \"？\".");
    }
}
