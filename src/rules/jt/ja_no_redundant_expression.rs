//! preset-ja-technical-writing/ja-no-redundant-expression
//!
//! 冗長表現の検出。upstream は morpheme-match による品詞ベース判定だが、
//! unslop では下記 4 パターン (dict1-dict4) を正規表現の simplified 範囲で実装する。
//!
//! - dict1/dict2: 「すること[助詞](不)可能」「すること[助詞]できる」
//! - dict3: 「であると言えます」「であると言える」
//! - dict4: 「であると考えている」「であると考えています」
//!
//! upstream の dict5/dict6 (「[サ変名詞]を行う」「[サ変名詞]を実行する」) は
//! ai_tech_writing_guideline の `の変更を行` `の実装を実施` `によって実行され` と一部重複し、
//! また誤検知を避けるための allows リストが上流側で広く設定されているため、
//! simplified 実装の安定性を優先して対象外とする。
//!
//! コードスパンとリンク URL は除外する (segment.code_ranges / link_url_ranges 経由)。

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::{Document, SegmentKind, TextSegment};
use crate::rule::{Fix, Issue, Rule, Severity};

const RULE_ID: &str = "ja-no-redundant-expression";

struct Pattern {
    id: &'static str,
    re: Regex,
    /// 検出時に表示する match の人間説明 (text)。
    text: &'static str,
    /// 簡潔化案 (空なら fix なし)。
    replacement: Option<&'static str>,
    message: &'static str,
}

fn pat(
    id: &'static str,
    re: &str,
    text: &'static str,
    message: &'static str,
    replacement: Option<&'static str>,
) -> Pattern {
    Pattern {
        id,
        re: Regex::new(re).unwrap(),
        text,
        message,
        replacement,
    }
}

static PATTERNS: Lazy<Vec<Pattern>> = Lazy::new(|| {
    vec![
        // dict1: することが可能/することも可能/することは可能
        // upstream は capture group ベースで助動詞や時制を保つが、unslop は simplified 範囲とし
        // auto-fix なしで報告のみとする (機械置換で意味が崩れるのを避けるため)。
        pat(
            "dict1",
            r"すること(が|も|は)可能",
            "することが可能",
            "\"することが\"を省き簡潔な表現にすると文章が明瞭になります。",
            None,
        ),
        // dict2: することができる/することができます (も/は も許容)
        pat(
            "dict2",
            r"すること(が|も|は)でき(る|ます|ない|ません)",
            "することができる",
            "\"することが\"を省き簡潔な表現にすると文章が明瞭になります。",
            // 簡潔化は活用形を残せないので fix なしにする
            None,
        ),
        // dict3: であると言えます / であると言える / であると言えるでしょう
        pat(
            "dict3",
            r"であると言え(る|ます)",
            "であると言える",
            "\"である\" または \"と言える\"を省き簡潔な表現にすると文章が明瞭になります。",
            None,
        ),
        // dict4: であると考えている / であると考えています
        pat(
            "dict4",
            r"であると考えて(いる|います)",
            "であると考えている",
            "\"である\" または \"と考えている\"を省き簡潔な表現にすると文章が明瞭になります。",
            None,
        ),
    ]
});

pub struct JaNoRedundantExpression;

impl Rule for JaNoRedundantExpression {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.in_block_quote {
                continue;
            }
            if !matches!(
                seg.kind,
                SegmentKind::Paragraph | SegmentKind::ListItem | SegmentKind::TableCell
            ) {
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
                        format!("【{}】 \"{}\"は冗長な表現です。{}", p.id, p.text, p.message),
                        line,
                        column,
                        Severity::Error,
                    );
                    if let Some(rep) = p.replacement {
                        let abs_start = seg.start_byte + s;
                        let abs_end = seg.start_byte + e;
                        issue = issue.with_fix(Fix {
                            range: abs_start..abs_end,
                            replacement: rep.to_string(),
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
        JaNoRedundantExpression.check(&doc).len()
    }

    #[test]
    fn detects_suru_koto_ga_kanou() {
        assert_eq!(count("これは省略することが可能だ。\n"), 1);
    }

    #[test]
    fn detects_suru_koto_ga_dekiru() {
        assert_eq!(count("これは省略することができる。\n"), 1);
    }

    #[test]
    fn detects_de_aru_to_ieru() {
        assert_eq!(count("これは正しいであると言える。\n"), 1);
    }

    #[test]
    fn detects_de_aru_to_kangaeteiru() {
        assert_eq!(count("妥当であると考えている。\n"), 1);
    }

    #[test]
    fn skips_code_span() {
        assert_eq!(count("`することが可能` という冗長表現について。\n"), 0);
    }

    #[test]
    fn skips_blockquote() {
        assert_eq!(count("> これは省略することが可能だ。\n"), 0);
    }
}
