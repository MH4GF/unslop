//! preset-ja-technical-writing/no-nfd
//! upstream: textlint-rule-no-nfd
//!
//! NFD 形式の濁点・半濁点 (`゛`, `゜`, `゚`, `゙`) を検出。
//! index 0 (先頭文字) は skip し、前文字と合わせて NFC 正規化候補を提示する。

use crate::document::Document;
use crate::rule::{Fix, Issue, Rule, Severity};
use crate::rules::is_str_bearing;
use unicode_normalization::UnicodeNormalization;

const RULE_ID: &str = "no-nfd";

pub struct NoNfd;

fn is_target(c: char) -> bool {
    matches!(c, '\u{309B}' | '\u{309C}' | '\u{309A}' | '\u{3099}')
}

impl Rule for NoNfd {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            // text を char_indices で歩き、前文字を保持しながら target を検出。
            let mut prev: Option<(usize, char)> = None;
            for (i, c) in seg.text.char_indices() {
                if is_target(c)
                    && let Some((pi, _pc)) = prev
                {
                    let pair = &seg.text[pi..i + c.len_utf8()];
                    // upstream の文字置換: ゛ → ゙, ゜ → ゚
                    let normalized_pair: String = pair
                        .chars()
                        .map(|c| match c {
                            '\u{309B}' => '\u{3099}',
                            '\u{309C}' => '\u{309A}',
                            other => other,
                        })
                        .collect();
                    let nfc: String = normalized_pair.nfc().collect();
                    let (line, column) = doc.pos_at(seg, i);
                    let abs_start = seg.start_byte + pi;
                    let abs_end = seg.start_byte + i + c.len_utf8();
                    issues.push(
                        Issue::new(
                            RULE_ID,
                            format!(
                                "Disallow to use NFD(well-known as UTF8-MAC 濁点): \"{pair}\" => \"{nfc}\""
                            ),
                            line,
                            column,
                            Severity::Error,
                        )
                        .with_fix(Fix {
                            range: abs_start..abs_end,
                            replacement: nfc,
                        }),
                    );
                }
                prev = Some((i, c));
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_replaces_nfd_pair() {
        // ホ + ゚ (NFD) → ポ (NFC)
        let src = "ホ\u{309A}ケット";
        let doc = Document::parse(src);
        let issues = NoNfd.check(&doc);
        assert_eq!(issues.len(), 1);
        let f = issues[0].fix.clone().unwrap();
        let mut buf = src.to_string();
        buf.replace_range(f.range, &f.replacement);
        assert_eq!(buf, "ポケット");
    }
}
