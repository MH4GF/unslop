//! preset-ja-technical-writing/no-invalid-control-character

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::Document;
use crate::rule::{Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "no-invalid-control-character";

// upstream と同じ集合: ASCII control (\t \r \n 除く) + C1 control + BiDi format
static PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"([\x00-\x08\x0B\x0C\x0E-\x1F\x7F\u{0080}-\u{009F}\u{202A}-\u{202E}])",
    )
    .unwrap()
});

const NAMES: &[(char, &str)] = &[
    ('\u{0000}', "NULL"),
    ('\u{0001}', "START OF HEADING"),
    ('\u{0002}', "START OF TEXT"),
    ('\u{0003}', "END OF TEXT"),
    ('\u{0004}', "END OF TRANSMISSION"),
    ('\u{0005}', "ENQUIRY"),
    ('\u{0006}', "ACKNOWLEDGE"),
    ('\u{0007}', "BELL"),
    ('\u{0008}', "BACKSPACE"),
    ('\u{000B}', "LINE TABULATION"),
    ('\u{000C}', "FORM FEED"),
    ('\u{000E}', "SHIFT OUT"),
    ('\u{000F}', "SHIFT IN"),
    ('\u{0010}', "DATA LINK ESCAPE"),
    ('\u{0011}', "DEVICE CONTROL ONE"),
    ('\u{0012}', "DEVICE CONTROL TWO"),
    ('\u{0013}', "DEVICE CONTROL THREE"),
    ('\u{0014}', "DEVICE CONTROL FOUR"),
    ('\u{0015}', "NEGATIVE ACKNOWLEDGE"),
    ('\u{0016}', "SYNCHRONOUS IDLE"),
    ('\u{0017}', "END OF TRANSMISSION BLOCK"),
    ('\u{0018}', "CANCEL"),
    ('\u{0019}', "END OF MEDIUM"),
    ('\u{001A}', "SUBSTITUTE"),
    ('\u{001B}', "ESCAPE"),
    ('\u{001C}', "INFORMATION SEPARATOR FOUR"),
    ('\u{001D}', "INFORMATION SEPARATOR THREE"),
    ('\u{001E}', "INFORMATION SEPARATOR TWO"),
    ('\u{001F}', "INFORMATION SEPARATOR ONE"),
    ('\u{007F}', "DELETE"),
    ('\u{202A}', "LEFT-TO-RIGHT EMBEDDING"),
    ('\u{202B}', "RIGHT-TO-LEFT EMBEDDING"),
    ('\u{202C}', "POP DIRECTIONAL FORMATTING"),
    ('\u{202D}', "LEFT-TO-RIGHT OVERRIDE"),
    ('\u{202E}', "RIGHT-TO-LEFT OVERRIDE"),
];

fn name_of(c: char) -> &'static str {
    NAMES
        .iter()
        .find_map(|(ch, n)| if *ch == c { Some(*n) } else { None })
        .unwrap_or("")
}

fn unicode_escape(c: char) -> String {
    format!("\\u{:04x}", c as u32)
}

pub struct NoInvalidControlCharacter;

impl Rule for NoInvalidControlCharacter {
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
            while let Ok(Some(m)) = PATTERN.find_from_pos(&seg.text, from) {
                let s = m.start();
                let e = m.end();
                let c = m.as_str().chars().next().unwrap();
                let name = name_of(c);
                let esc = unicode_escape(c);
                let (line, column) = doc.pos_at(seg, s);
                issues.push(Issue {
                    rule_id: RULE_ID.to_string(),
                    message: format!("Found invalid control character({name} {esc})"),
                    line,
                    column,
                    severity: Severity::Error,
                });
                from = e.max(s + 1);
            }
        }
        issues
    }
}
