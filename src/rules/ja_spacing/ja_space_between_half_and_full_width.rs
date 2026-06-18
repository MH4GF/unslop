//! preset-ja-spacing/ja-space-between-half-and-full-width
//! upstream: textlint-rule-ja-space-between-half-and-full-width
//!
//! 半角 (`[A-Za-z0-9]`) と全角 が直接隣接した境界に半角スペース 1 文字を挿入する。
//! `space: "always"` (`alphabets: true, numbers: true`, `exceptPunctuation: true` のデフォルト) 固定相当。
//! 句読点 `、。` 側は除外する。
//!
//! 検出方向は次の 2 件
//!   A) `[A-Za-z0-9]` 直後に全角 — 全角側を report 位置の参考にする (upstream は alnum 側 column)
//!   B) 全角 直後に `[A-Za-z0-9]` — 全角側 column を report する
//!
//! auto-fix は半角 ASCII space `' '` を境界に挿入する。全角スペースは使わない。

use crate::document::Document;
use crate::rule::{Fix, Issue, Rule, Severity};
use crate::rules::is_str_bearing;

use super::{is_full_width, is_half_width_alnum, is_zen_punctuation, range_in_excluded};

const RULE_ID: &str = "ja-space-between-half-and-full-width";
const MESSAGE: &str = "原則として、全角文字と半角文字の間にスペースを入れます。";

pub struct JaSpaceBetweenHalfAndFullWidth;

impl Rule for JaSpaceBetweenHalfAndFullWidth {
    fn id(&self) -> &str {
        RULE_ID
    }

    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            let chars: Vec<(usize, char)> = seg.text.char_indices().collect();
            if chars.len() < 2 {
                continue;
            }
            for w in chars.windows(2) {
                let (l_byte, left) = w[0];
                let (r_byte, right) = w[1];

                let han_to_zen = is_half_width_alnum(left) && is_full_width(right);
                let zen_to_han = is_full_width(left) && is_half_width_alnum(right);
                if !han_to_zen && !zen_to_han {
                    continue;
                }
                // upstream の exceptPunctuation: true 相当 (always のデフォルト)。
                // 句読点側は対象外にする。
                let zen_side = if han_to_zen { right } else { left };
                if is_zen_punctuation(zen_side) {
                    continue;
                }
                if range_in_excluded(seg, l_byte, r_byte + right.len_utf8()) {
                    continue;
                }
                let (line, column) = doc.pos_at(seg, l_byte);
                let insert_at = seg.start_byte + r_byte;
                issues.push(
                    Issue::new(RULE_ID, MESSAGE, line, column, Severity::Error).with_fix(Fix {
                        range: insert_at..insert_at,
                        replacement: " ".to_string(),
                    }),
                );
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
        JaSpaceBetweenHalfAndFullWidth.check(&doc).len()
    }

    #[test]
    fn detects_alnum_to_zen() {
        assert_eq!(count("JTF標準\n"), 1);
    }

    #[test]
    fn detects_zen_to_alnum() {
        assert_eq!(count("日本語とenglish\n"), 1);
    }

    #[test]
    fn ignores_existing_space() {
        assert_eq!(count("JTF 標準\n"), 0);
    }

    #[test]
    fn ignores_punctuation_boundary() {
        // 、J や 。X は除外 (exceptPunctuation 相当)。
        assert_eq!(count("これは、Always\n"), 0);
        assert_eq!(count("Always。これは\n"), 0);
    }

    #[test]
    fn ignores_pure_alnum_sequence() {
        assert_eq!(count("This is a pen\n"), 0);
    }

    #[test]
    fn skips_code_span() {
        assert_eq!(count("`JTF標準` というのは正しい\n"), 0);
    }

    #[test]
    fn fix_inserts_space_after_alnum() {
        let src = "JTF標準\n";
        let doc = Document::parse(src);
        let issue = JaSpaceBetweenHalfAndFullWidth
            .check(&doc)
            .into_iter()
            .next()
            .unwrap();
        let f = issue.fix.unwrap();
        let mut buf = src.to_string();
        buf.replace_range(f.range, &f.replacement);
        assert_eq!(buf, "JTF 標準\n");
    }

    #[test]
    fn fix_inserts_space_after_zen() {
        let src = "日本語とenglish\n";
        let doc = Document::parse(src);
        let issue = JaSpaceBetweenHalfAndFullWidth
            .check(&doc)
            .into_iter()
            .next()
            .unwrap();
        let f = issue.fix.unwrap();
        let mut buf = src.to_string();
        buf.replace_range(f.range, &f.replacement);
        assert_eq!(buf, "日本語と english\n");
    }
}
