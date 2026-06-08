//! no-mid-sentence-break (unslop-original / textlint 非対応)
//!
//! 段落 (BlockQuote 配下を除く Paragraph) の生テキストを走査し、文末記号以外の直後の改行を
//! 「文中改行」として報告する。長い一文の途中へ改行を挟んで sentence-length などの文分割
//! チェックをすり抜ける書き方を防ぐ。句点や閉じ記号の直後の改行 (句点改行) は許可する。
//!
//! detection のみ。本来分けるべき箇所まで結合する誤修正リスクがあるため auto-fix は実装しない。

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "no-mid-sentence-break";

const MESSAGE: &str = "文の途中で改行されています。一文の途中の改行は sentence-length などの文分割チェックをすり抜けるため、句点で区切ってください。";

pub struct NoMidSentenceBreak;

/// 直前がこれらの文末・閉じ記号なら、その後の改行は句点改行として許可する。
fn is_sentence_ender(c: char) -> bool {
    matches!(
        c,
        '。' | '．'
            | '.'
            | '！'
            | '!'
            | '？'
            | '?'
            | '」'
            | '』'
            | '）'
            | ')'
            | '】'
            | '〕'
            | '》'
            | '〉'
            | '｝'
            | '］'
            | '}'
            | ']'
            | '…'
    )
}

impl Rule for NoMidSentenceBreak {
    fn id(&self) -> &str {
        RULE_ID
    }

    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            // 段落相当のみ。リスト・コードブロックは kind で、引用は flag で除外する。
            if seg.kind != SegmentKind::Paragraph || seg.in_block_quote {
                continue;
            }
            let mut last_non_ws: Option<char> = None;
            for (idx, ch) in seg.text.char_indices() {
                if ch == '\n' {
                    // 直前の非空白文字が文末記号でなければ文中改行。
                    if let Some(prev) = last_non_ws
                        && !is_sentence_ender(prev)
                    {
                        let (line, column) = doc.pos_at(seg, idx);
                        issues.push(Issue {
                            rule_id: RULE_ID.to_string(),
                            message: MESSAGE.to_string(),
                            line,
                            column,
                            severity: Severity::Error,
                            fix: None,
                        });
                    }
                    continue;
                }
                if !ch.is_whitespace() {
                    last_non_ws = Some(ch);
                }
            }
        }
        issues
    }
}
