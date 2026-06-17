//! preset-ja-technical-writing/ja-no-abusage (simplified)
//!
//! 慣用句・表現の誤用を辞書ベースで検出し、auto-fix で正用に置換する。
//! upstream の辞書は広いが、本実装は誤用が定着しがちで価値の高い代表例に絞る。

use crate::document::Document;
use crate::rule::{Fix, Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "ja-no-abusage";

struct Entry {
    pattern: &'static str,
    replacement: &'static str,
}

const ENTRIES: &[Entry] = &[
    Entry {
        pattern: "足元をすくわれ",
        replacement: "足をすくわれ",
    },
    Entry {
        pattern: "足元をすくう",
        replacement: "足をすくう",
    },
    Entry {
        pattern: "明るみになる",
        replacement: "明るみに出る",
    },
    Entry {
        pattern: "危機一発",
        replacement: "危機一髪",
    },
    Entry {
        pattern: "采配を振るう",
        replacement: "采配を振る",
    },
    Entry {
        pattern: "飛ぶ鳥跡を濁さず",
        replacement: "立つ鳥跡を濁さず",
    },
    Entry {
        pattern: "袖振り合うも他生の縁",
        replacement: "袖振り合うも多生の縁",
    },
];

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
            let text = &seg.text;
            for entry in ENTRIES {
                let mut from = 0usize;
                while let Some(rel) = text[from..].find(entry.pattern) {
                    let s = from + rel;
                    let e = s + entry.pattern.len();
                    let in_code = seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                    let in_link = seg.link_url_ranges.iter().any(|&(cs, ce)| s < ce && cs < e);
                    if !in_code && !in_link {
                        let (line, column) = doc.pos_at(seg, s);
                        let abs = seg.start_byte + s;
                        issues.push(
                            Issue::new(
                                RULE_ID,
                                format!(
                                    "誤用表現 \"{}\" がみつかりました。\"{}\" を検討してください。",
                                    entry.pattern, entry.replacement
                                ),
                                line,
                                column,
                                Severity::Error,
                            )
                            .with_fix(Fix {
                                range: abs..abs + entry.pattern.len(),
                                replacement: entry.replacement.to_string(),
                            }),
                        );
                    }
                    from = e.max(s + 1);
                }
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
        JaNoAbusage.check(&doc).len()
    }

    #[test]
    fn flags_kiki_ippatsu() {
        assert_eq!(count("ここで危機一発の事態となった。"), 1);
    }

    #[test]
    fn flags_ashimoto() {
        assert_eq!(count("足元をすくわれてしまった。"), 1);
    }

    #[test]
    fn passes_correct() {
        assert_eq!(count("ここで危機一髪の事態となった。"), 0);
    }

    #[test]
    fn fix_replaces() {
        let src = "ここで危機一発の事態となった。";
        let doc = Document::parse(src);
        let issue = JaNoAbusage.check(&doc).into_iter().next().unwrap();
        let f = issue.fix.unwrap();
        let mut buf = src.to_string();
        buf.replace_range(f.range, &f.replacement);
        assert_eq!(buf, "ここで危機一髪の事態となった。");
    }
}
