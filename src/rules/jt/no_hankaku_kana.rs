//! preset-ja-technical-writing/no-hankaku-kana
//!
//! auto-fix では match 範囲を NFKC 正規化して全角カナへ変換する。
//! NFKC は U+FF61-U+FF9F の compat decomposition を含むため、半角濁点・半濁点も
//! 直前文字と結合した precomposed form (例: ｶﾞ → ガ) に揃う。

use fancy_regex::Regex;
use once_cell::sync::Lazy;
use unicode_normalization::UnicodeNormalization;

use crate::document::Document;
use crate::rule::{Fix, Issue, Rule, Severity};
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
                let abs_start = seg.start_byte + s;
                let abs_end = seg.start_byte + e;
                let replacement: String = m.as_str().nfkc().collect();
                issues.push(
                    Issue::new(
                        RULE_ID,
                        format!("Disallow to use 半角カタカナ: \"{}\"", m.as_str()),
                        line,
                        column,
                        Severity::Error,
                    )
                    .with_fix(Fix {
                        range: abs_start..abs_end,
                        replacement,
                    }),
                );
                from = e.max(s + 1);
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix_applied(src: &str) -> String {
        let doc = Document::parse(src);
        let mut buf = src.to_string();
        let fixes: Vec<_> = NoHankakuKana
            .check(&doc)
            .into_iter()
            .filter_map(|i| i.fix)
            .collect();
        for f in fixes.iter().rev() {
            buf.replace_range(f.range.clone(), &f.replacement);
        }
        buf
    }

    #[test]
    fn fix_converts_simple() {
        assert_eq!(fix_applied("ｱｲｳ です"), "アイウ です");
    }

    #[test]
    fn fix_converts_with_dakuten() {
        // ｶﾞ + ｷﾞ → ガギ
        assert_eq!(fix_applied("ｶﾞｷﾞ"), "ガギ");
    }

    #[test]
    fn fix_converts_with_handakuten() {
        // ﾊﾟ + ﾋﾟ → パピ
        assert_eq!(fix_applied("ﾊﾟﾋﾟ"), "パピ");
    }

    #[test]
    fn fix_converts_mixed_run() {
        // ｲﾝﾌﾗ → インフラ
        assert_eq!(fix_applied("ｲﾝﾌﾗ で動く"), "インフラ で動く");
    }

    #[test]
    fn fix_converts_punctuation() {
        // ｡ ｢ ｣ ､ ･ も全角に
        assert_eq!(fix_applied("ｱ｢ｲ｣"), "ア「イ」");
    }
}
