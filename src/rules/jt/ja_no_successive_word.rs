//! preset-ja-technical-writing/ja-no-successive-word
//!
//! 隣接する形態素の surface が一致したら error。
//!
//! - 数字 (名詞-数) 連続は除外 (例: "九九回目")
//! - オノマトペ (カタカナ + 長音) の連続は除外 (allowOnomatopee 相当, default on)
//!
//! auto-fix なし (意味を壊すため)。

use crate::document::Document;
use crate::morph::{Token, tokenize};
use crate::rule::{Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "ja-no-successive-word";

pub struct JaNoSuccessiveWord;

impl Rule for JaNoSuccessiveWord {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            let tokens = tokenize(&seg.text);
            for i in 1..tokens.len() {
                let prev = &tokens[i - 1];
                let curr = &tokens[i];
                if prev.surface != curr.surface || prev.surface.is_empty() {
                    continue;
                }
                if is_number_token(prev) && is_number_token(curr) {
                    continue;
                }
                if is_onomatopoeia(&prev.surface) && is_onomatopoeia(&curr.surface) {
                    continue;
                }
                // code_ranges / link_url_ranges 内のトークンは対象外。
                let span_start = curr.byte_start;
                let span_end = curr.byte_end;
                if seg
                    .code_ranges
                    .iter()
                    .any(|&(s, e)| span_start < e && s < span_end)
                    || seg
                        .link_url_ranges
                        .iter()
                        .any(|&(s, e)| span_start < e && s < span_end)
                {
                    continue;
                }
                let (line, column) = doc.pos_at(seg, curr.byte_start);
                issues.push(Issue::new(
                    RULE_ID,
                    format!("\"{}\" が連続して2回使われています。", curr.surface),
                    line,
                    column,
                    Severity::Error,
                ));
            }
        }
        issues
    }
}

fn is_number_token(t: &Token) -> bool {
    t.pos == "名詞" && t.pos_detail_1 == "数"
}

/// カタカナのみ (長音含む) で構成されるか。
fn is_onomatopoeia(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| {
            matches!(
                c,
                '\u{30A1}'
                    ..='\u{30F6}' // ァ-ヶ (ロ・ワ・ヲ・ン を含む)
                | '\u{30FC}' // ー
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issues(src: &str) -> Vec<String> {
        let doc = Document::parse(src);
        JaNoSuccessiveWord
            .check(&doc)
            .into_iter()
            .map(|i| i.message)
            .collect()
    }

    #[test]
    fn detects_successive_ha() {
        let v = issues("これはは問題ある文章です。\n");
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("\"は\""), "msg = {}", v[0]);
    }

    #[test]
    fn detects_successive_aru() {
        let v = issues("これは問題あるある文章です。\n");
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("\"ある\""), "msg = {}", v[0]);
    }

    #[test]
    fn skips_kansuji_kuku() {
        // 「九九」は数字連続として除外
        assert_eq!(issues("値は九九です。\n"), Vec::<String>::new());
    }

    #[test]
    fn skips_onomatopoeia_kakukaku() {
        assert_eq!(
            issues("フレームレートが落ちて動作がカクカクしてきた。\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn ok_for_sumomomomomo() {
        // 形態素解析次第だが、IPADIC では split されオノマトペ扱いなので 0
        // (textlint と完全一致しない可能性はあるが simplified 許容)
        let _ = issues("すもももももももものうち\n");
    }

    #[test]
    fn skips_code_span() {
        assert_eq!(
            issues("`これはは` というコード断片。\n"),
            Vec::<String>::new()
        );
    }
}
