//! preset-ja-technical-writing/ja-no-abusage
//!
//! よくある誤用の検出。upstream は morpheme-match + prh の 2 段構成だが、unslop では
//! 高頻度かつ誤検知の少ないパターンに絞って正規表現の simplified 範囲で実装する。
//!
//! 取り込み対象:
//! - 「を適応」 → 「を適用」 (morpheme-match 由来)
//! - 「可変する」 (morpheme-match 由来)
//! - 動詞連用形 + 「ずらい」 → 「づらい」 (morpheme-match 由来)
//! - 「値を返却する」 → 「値を返す」 (prh.yml)
//! - 「例外を補足」 → 「例外を捕捉」 (prh.yml)
//! - 「こんにちわ」 → 「こんにちは」 (prh.yml)
//! - 「うる覚え」 → 「うろ覚え」 (prh.yml)
//!
//! 残りの prh エントリは個別 prh.yml で扱う前提で本 rule のスコープ外とする。

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::{Document, TextSegment};
use crate::rule::{Fix, Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "ja-no-abusage";

struct Pattern {
    re: Regex,
    message: &'static str,
    /// match 全体を置換する文字列。fix なしなら None。
    replacement: Option<&'static str>,
}

fn pat(re: &str, message: &'static str, replacement: Option<&'static str>) -> Pattern {
    Pattern {
        re: Regex::new(re).unwrap(),
        message,
        replacement,
    }
}

static PATTERNS: Lazy<Vec<Pattern>> = Lazy::new(|| {
    vec![
        pat(
            r"を適応",
            "\"適用\"の誤用である可能性があります。適応 => 適用",
            Some("を適用"),
        ),
        pat(
            r"可変する",
            "「可変する」という使い方は適切ではありません。「可逆」と同じ使い方になります。\nhttp://qiita.com/scivola/items/f02589968a4ca27bc52b",
            None,
        ),
        pat(
            r"([ぁ-ん])ずらい",
            "動詞の連用形+辛い（つらい）の場合は、「ずらい」ではなく「づらい」が適切です。",
            // 後段で動的に置換する (capture group 利用のため)
            Some("__ZURAI_DZURAI__"),
        ),
        pat(
            r"値を返却する",
            "「値を返却する」は冗長な誤用の可能性があります。「値を返す」が一般的です。",
            Some("値を返す"),
        ),
        pat(
            r"例外を補足",
            "「補足」は補い足すこと。例外は「捕捉」する。",
            Some("例外を捕捉"),
        ),
        pat(
            r"こんにちわ",
            "「こんにちわ」は誤りです。「こんにちは」が正しい表記です。",
            Some("こんにちは"),
        ),
        pat(
            r"うる覚え",
            "「うる覚え」は誤りです。「うろ覚え」が正しい表記です。",
            Some("うろ覚え"),
        ),
    ]
});

pub struct JaNoAbusage;

impl Rule for JaNoAbusage {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            if seg.in_block_quote {
                continue;
            }
            for p in PATTERNS.iter() {
                let mut from = 0usize;
                while let Ok(Some(m)) = p.re.find_from_pos(&seg.text, from) {
                    let s = m.start();
                    let e = m.end();
                    from = e.max(s + 1);
                    if range_in_excluded(seg, s, e) {
                        continue;
                    }
                    let (line, column) = doc.pos_at(seg, s);
                    let mut issue = Issue::new(
                        RULE_ID,
                        p.message.to_string(),
                        line,
                        column,
                        Severity::Error,
                    );
                    if let Some(rep) = p.replacement {
                        let abs_start = seg.start_byte + s;
                        let abs_end = seg.start_byte + e;
                        let matched = m.as_str();
                        let replacement = if rep == "__ZURAI_DZURAI__" {
                            // matched は "[ぁ-ん]ずらい" → 先頭 1 文字を残して "ずらい" を "づらい" に
                            let mut chars = matched.chars();
                            let head = chars.next().map(|c| c.to_string()).unwrap_or_default();
                            format!("{head}づらい")
                        } else {
                            rep.to_string()
                        };
                        issue = issue.with_fix(Fix {
                            range: abs_start..abs_end,
                            replacement,
                        });
                    }
                    issues.push(issue);
                }
            }
        }
        issues
    }
}

fn range_in_excluded(seg: &TextSegment, s: usize, e: usize) -> bool {
    seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e)
        || seg.link_url_ranges.iter().any(|&(cs, ce)| s < ce && cs < e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(src: &str) -> usize {
        let doc = Document::parse(src);
        JaNoAbusage.check(&doc).len()
    }

    fn fix_applied(src: &str) -> String {
        let doc = Document::parse(src);
        let mut buf = src.to_string();
        let fixes: Vec<_> = JaNoAbusage
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
    fn detects_wo_tekiou() {
        assert_eq!(count("法律を適応する\n"), 1);
    }

    #[test]
    fn ignores_correct_tekiou() {
        // "変化に適応する" は正用
        assert_eq!(count("変化に適応する\n"), 0);
    }

    #[test]
    fn detects_kahen_suru() {
        assert_eq!(count("ウインドウ幅が可変すると\n"), 1);
    }

    #[test]
    fn ignores_kahen_da() {
        assert_eq!(count("長さは可変だ\n"), 0);
    }

    #[test]
    fn fixes_yomizurai() {
        assert_eq!(fix_applied("この本は読みずらい\n"), "この本は読みづらい\n");
    }

    #[test]
    fn fixes_konnichiwa() {
        assert_eq!(fix_applied("こんにちわ世界\n"), "こんにちは世界\n");
    }

    #[test]
    fn fixes_uruoboe() {
        assert_eq!(fix_applied("これはうる覚えです\n"), "これはうろ覚えです\n");
    }

    #[test]
    fn fixes_henkyaku() {
        assert_eq!(
            fix_applied("関数で値を返却するべき\n"),
            "関数で値を返すべき\n"
        );
    }

    #[test]
    fn fixes_reigaiwo_hosoku() {
        assert_eq!(
            fix_applied("try で例外を補足する\n"),
            "try で例外を捕捉する\n"
        );
    }
}
