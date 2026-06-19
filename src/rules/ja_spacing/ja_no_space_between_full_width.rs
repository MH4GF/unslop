//! preset-ja-spacing/ja-no-space-between-full-width
//! upstream: textlint-rule-ja-no-space-between-full-width
//!
//! 全角文字どうしの間にある半角スペース 1 文字を検出する。
//! ただしカタカナ複合語 (両側が `[ァ-ヶ]`) は例外として除外する (upstream 互換)。
//! auto-fix は該当の半角スペース 1 文字を削除する。

use crate::document::Document;
use crate::rule::{Fix, Issue, Rule, Severity};
use crate::rules::is_str_bearing;

use super::{is_full_width, is_katakana_in_compound, range_in_excluded};

const RULE_ID: &str = "ja-no-space-between-full-width";
const MESSAGE: &str = "原則として、全角文字どうしの間にスペースを入れません。";

pub struct JaNoSpaceBetweenFullWidth;

impl Rule for JaNoSpaceBetweenFullWidth {
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
            if chars.len() < 3 {
                continue;
            }
            for w in chars.windows(3) {
                let (_, prev) = w[0];
                let (sp_byte, sp) = w[1];
                let (_, next) = w[2];
                if sp != ' ' {
                    continue;
                }
                if !is_full_width(prev) || !is_full_width(next) {
                    continue;
                }
                if is_katakana_in_compound(prev) && is_katakana_in_compound(next) {
                    continue;
                }
                if range_in_excluded(seg, sp_byte, sp_byte + 1) {
                    continue;
                }
                let (line, column) = doc.pos_at(seg, sp_byte);
                let abs = seg.start_byte + sp_byte;
                issues.push(
                    Issue::new(RULE_ID, MESSAGE, line, column, Severity::Error).with_fix(Fix {
                        range: abs..abs + 1,
                        replacement: String::new(),
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
        JaNoSpaceBetweenFullWidth.check(&doc).len()
    }

    #[test]
    fn detects_hiragana_katakana_boundary() {
        assert_eq!(count("これは ダメ\n"), 1);
    }

    #[test]
    fn ignores_katakana_compound() {
        // ユーザー インターフェース: 境界は ー と イ。ー は `[ァ-ヶ]` 範囲外なので Zen ではない。
        assert_eq!(count("ユーザー インターフェース\n"), 0);
    }

    #[test]
    fn katakana_to_katakana_in_compound_excluded() {
        // 「ア イ」: 両方 [ァ-ヶ]。katakakana 例外で対象外。
        assert_eq!(count("ア イ\n"), 0);
    }

    #[test]
    fn detects_two_occurrences() {
        // 「同じ トランザクション で」のようなケースで 2 件発火する。
        assert_eq!(count("同じ トランザクション で\n"), 2);
    }

    #[test]
    fn ignores_half_width_neighbour() {
        assert_eq!(count("This is 大丈夫\n"), 0);
    }

    #[test]
    fn skips_code_span() {
        assert_eq!(count("`これは ダメ` というのは正しい\n"), 0);
    }

    #[test]
    fn fix_removes_single_space() {
        let src = "これは ダメ\n";
        let doc = Document::parse(src);
        let issue = JaNoSpaceBetweenFullWidth
            .check(&doc)
            .into_iter()
            .next()
            .unwrap();
        let f = issue.fix.unwrap();
        let mut buf = src.to_string();
        buf.replace_range(f.range, &f.replacement);
        assert_eq!(buf, "これはダメ\n");
    }
}
