//! preset-ja-technical-writing/ja-no-mixed-period
//!
//! Paragraph 末尾が `。` で終わっていない場合に error。
//! 末尾が ASCII `.` の場合は auto-fix で `。` に置換する。
//! upstream の `forceAppendPeriod` は default off に合わせ、句点なしの段落に対して
//! auto-fix では「追加しない」。
//!
//! Simplification:
//! - 段落内に日本語 (ひらがな/カタカナ/漢字) を含まない場合は対象外。
//! - 段落末尾が markdown 構造的終端 (`)` `>` `*` `_` backtick) なら link/code/emphasis 末端と
//!   判断してスキップ。upstream の `lastNode !== Str` 判定の簡略版。

use crate::document::{Document, SegmentKind};
use crate::rule::{Fix, Issue, Rule, Severity};

const RULE_ID: &str = "ja-no-mixed-period";
const PREFER: &str = "。";

const ALLOWED_LAST_CHARS: &[char] = &[
    '。', '．', '!', '?', '！', '？', '」', '』', '）', '〕', '〉', '》', '】', '］', '｝', '〙',
    '〛', ')', ']', '}', '>',
];

pub struct JaNoMixedPeriod;

impl Rule for JaNoMixedPeriod {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if seg.kind != SegmentKind::Paragraph {
                continue;
            }
            if seg.in_block_quote {
                continue;
            }
            if !contains_japanese(&seg.text) {
                continue;
            }
            // 末尾の空白 (改行含む) を除いた byte 長と最後の char を取る。
            let trimmed = seg.text.trim_end();
            if trimmed.is_empty() {
                continue;
            }
            let last_char = match trimmed.chars().next_back() {
                Some(c) => c,
                None => continue,
            };
            let last_char_byte_end = trimmed.len();
            let last_char_byte_start = last_char_byte_end - last_char.len_utf8();

            // markdown 構造的な終端 (link/image の `)` / code span の backtick / emphasis の `_`/`*`
            // / autolink の `>` ) は upstream の lastNode != Str に相当するためスキップ。
            // 例外的に `)` `]` などは ja_no_mixed_period の許可リストにも含めているので
            // どちらにせよ OK 扱いになる。
            if matches!(last_char, '`' | '*' | '_') {
                continue;
            }
            if matches!(last_char, ')' | ']' | '>') && ends_in_markdown_structure(seg, trimmed) {
                continue;
            }

            if ALLOWED_LAST_CHARS.contains(&last_char) {
                continue;
            }

            let (line, column) = doc.pos_at(seg, last_char_byte_start);
            let mut issue = Issue::new(
                RULE_ID,
                format!(
                    "文末が\"{PREFER}\"で終わっていません。\n理由: 句点は文の境界を明確にし、読み手の理解を助けます\n修正: 適切な文末表現で文を完結させ、句点を追加してください\n例: 「〜です{PREFER}」「〜ます{PREFER}」「〜でした{PREFER}」など"
                ),
                line,
                column,
                Severity::Error,
            );
            // ASCII `.` のときだけ classic-period として `。` に auto-fix する。
            if last_char == '.' {
                let abs_start = seg.start_byte + last_char_byte_start;
                let abs_end = seg.start_byte + last_char_byte_end;
                issue = issue.with_fix(Fix {
                    range: abs_start..abs_end,
                    replacement: PREFER.to_string(),
                });
            }
            issues.push(issue);
        }
        issues
    }
}

fn contains_japanese(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c,
            '々' | '〇' | '〻'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{3041}'..='\u{309F}'  // ひらがな
            | '\u{30A0}'..='\u{30FF}'  // カタカナ
        )
    })
}

/// 段落末尾が link/image/code/autolink の終端文字に該当するかを判定。
/// `seg.link_url_ranges` や `seg.code_ranges` が末尾近くまでカバーしていれば markdown 構造とみなす。
fn ends_in_markdown_structure(seg: &crate::document::TextSegment, trimmed: &str) -> bool {
    let end = trimmed.len();
    // 末尾の数 byte (最大 8 byte) を含むレンジがあれば構造と判断する。
    // link_url_ranges は URL 本体 (閉じ `)` の手前まで) を指すので end-1 を含むかで判定する。
    let target = end.saturating_sub(1);
    seg.link_url_ranges
        .iter()
        .any(|&(s, e)| s <= target && target < e + 2)
        || seg
            .code_ranges
            .iter()
            .any(|&(s, e)| s <= target && target < e + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(src: &str) -> usize {
        let doc = Document::parse(src);
        JaNoMixedPeriod.check(&doc).len()
    }

    fn fix_applied(src: &str) -> String {
        let doc = Document::parse(src);
        let mut buf = src.to_string();
        let fixes: Vec<_> = JaNoMixedPeriod
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
    fn ok_when_ends_with_kuten() {
        assert_eq!(count("これは問題ないです。\n"), 0);
    }

    #[test]
    fn ng_when_missing_kuten() {
        assert_eq!(count("これは句点がありません\n"), 1);
    }

    #[test]
    fn ascii_period_is_fixed_to_kuten() {
        assert_eq!(fix_applied("これはダメ.\n"), "これはダメ。\n");
    }

    #[test]
    fn english_only_is_ignored() {
        assert_eq!(count("english only\n"), 0);
    }

    #[test]
    fn heading_is_ignored() {
        assert_eq!(count("# 見出しは無視\n"), 0);
    }

    #[test]
    fn blockquote_is_ignored() {
        assert_eq!(count("> 引用は無視\n"), 0);
    }

    #[test]
    fn list_item_is_ignored() {
        assert_eq!(count("- 箇条書きは無視\n"), 0);
    }

    #[test]
    fn link_only_paragraph_is_ignored() {
        assert_eq!(count("[リンクの説明も無視される](http://example.com)\n"), 0);
    }

    #[test]
    fn image_only_paragraph_is_ignored() {
        assert_eq!(count("![画像の説明も無視される](img/img.png)\n"), 0);
    }

    #[test]
    fn ends_with_exclamation_is_ok() {
        assert_eq!(count("末尾に感嘆符はある!\n"), 0);
    }
}
