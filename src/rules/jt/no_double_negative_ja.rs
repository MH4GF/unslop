//! preset-ja-technical-writing/no-double-negative-ja
//!
//! 主要 7 系統 (なくはない/なくもない / ないでもない/ないではない / ないことはない/ないこともない /
//! ないものではない/ないものでもない / ないわけではない/ないわけでもない /
//! ないとはいいきれない / ないとはかぎらない) の二重否定を検出する。
//! upstream の matchTokenStream 相当のステートマシンを TokenSpec の列で持つ。

use crate::document::Document;
use crate::morph::{Token, tokenize};
use crate::rule::{Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "no-double-negative-ja";

#[derive(Default, Clone)]
struct Spec {
    surface: Option<&'static [&'static str]>,
    base: Option<&'static [&'static str]>,
    pos: Option<&'static [&'static str]>,
    reading: Option<&'static [&'static str]>,
    conjugated_form: Option<&'static [&'static str]>,
}

fn matches_spec(t: &Token, s: &Spec) -> bool {
    if let Some(v) = s.surface {
        if !v.contains(&t.surface.as_str()) {
            return false;
        }
    }
    if let Some(v) = s.base {
        if !v.contains(&t.base_form.as_str()) {
            return false;
        }
    }
    if let Some(v) = s.pos {
        if !v.contains(&t.pos.as_str()) {
            return false;
        }
    }
    if let Some(v) = s.reading {
        if !v.contains(&t.reading.as_str()) {
            return false;
        }
    }
    if let Some(v) = s.conjugated_form {
        if !v.contains(&t.conjugated_form.as_str()) {
            return false;
        }
    }
    true
}

struct Pattern {
    message: &'static str,
    specs: Vec<Spec>,
}

const NAI: &[&str] = &["ない", "無い"];
const KEIYO: &[&str] = &["形容詞"];
const JOSHI: &[&str] = &["助詞"];
const MEISHI: &[&str] = &["名詞"];

fn patterns() -> Vec<Pattern> {
    vec![
        Pattern {
            message: "二重否定: 〜なくもない",
            specs: vec![
                Spec { base: Some(NAI), ..Spec_const() },
                Spec { surface: Some(&["も"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { base: Some(NAI), pos: Some(KEIYO), ..Spec_const() },
            ],
        },
        Pattern {
            message: "二重否定: 〜なくはない",
            specs: vec![
                Spec { base: Some(NAI), ..Spec_const() },
                Spec { surface: Some(&["は"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { base: Some(NAI), pos: Some(KEIYO), ..Spec_const() },
            ],
        },
        Pattern {
            message: "二重否定: 〜ないでもない",
            specs: vec![
                Spec { base: Some(NAI), ..Spec_const() },
                Spec { surface: Some(&["で"]), conjugated_form: Some(&["連用形"]), ..Spec_const() },
                Spec { surface: Some(&["も"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { base: Some(NAI), pos: Some(KEIYO), ..Spec_const() },
            ],
        },
        Pattern {
            message: "二重否定: 〜ないではない",
            specs: vec![
                Spec { base: Some(NAI), ..Spec_const() },
                Spec { surface: Some(&["で"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { surface: Some(&["は"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { base: Some(NAI), pos: Some(KEIYO), ..Spec_const() },
            ],
        },
        Pattern {
            message: "二重否定: 〜ないことはない",
            specs: vec![
                Spec { base: Some(NAI), ..Spec_const() },
                Spec { reading: Some(&["コト"]), pos: Some(MEISHI), ..Spec_const() },
                Spec { surface: Some(&["は"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { base: Some(NAI), ..Spec_const() },
            ],
        },
        Pattern {
            message: "二重否定: 〜ないこともない",
            specs: vec![
                Spec { base: Some(NAI), ..Spec_const() },
                Spec { reading: Some(&["コト"]), pos: Some(MEISHI), ..Spec_const() },
                Spec { surface: Some(&["も"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { base: Some(NAI), ..Spec_const() },
            ],
        },
        Pattern {
            message: "二重否定: 〜ないわけではない",
            specs: vec![
                Spec { base: Some(NAI), ..Spec_const() },
                Spec { reading: Some(&["ワケ"]), pos: Some(MEISHI), ..Spec_const() },
                Spec { surface: Some(&["で"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { surface: Some(&["は"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { base: Some(NAI), pos: Some(KEIYO), ..Spec_const() },
            ],
        },
        Pattern {
            message: "二重否定: 〜ないとはかぎらない",
            specs: vec![
                Spec { base: Some(NAI), ..Spec_const() },
                Spec { surface: Some(&["と"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { surface: Some(&["は"]), pos: Some(JOSHI), ..Spec_const() },
                Spec { reading: Some(&["カギラ"]), ..Spec_const() },
                Spec { base: Some(NAI), ..Spec_const() },
            ],
        },
    ]
}

#[allow(non_snake_case)]
const fn Spec_const() -> Spec {
    Spec {
        surface: None,
        base: None,
        pos: None,
        reading: None,
        conjugated_form: None,
    }
}

pub struct NoDoubleNegativeJa;

impl Rule for NoDoubleNegativeJa {
    fn id(&self) -> &str {
        RULE_ID
    }
    fn check(&self, doc: &Document) -> Vec<Issue> {
        let pats = patterns();
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            let tokens = tokenize(&seg.text);
            for pat in &pats {
                let mut state = 0usize;
                let mut anchor_byte = 0usize;
                for t in &tokens {
                    if matches_spec(t, &pat.specs[state]) {
                        if state == 0 {
                            anchor_byte = t.byte_start;
                        }
                        state += 1;
                        if state == pat.specs.len() {
                            let (line, column) = doc.pos_at(seg, anchor_byte);
                            issues.push(Issue {
                                rule_id: RULE_ID.to_string(),
                                message: pat.message.to_string(),
                                line,
                                column,
                                severity: Severity::Error,
                            });
                            state = 0;
                        }
                    } else {
                        state = 0;
                    }
                }
            }
        }
        issues
    }
}
