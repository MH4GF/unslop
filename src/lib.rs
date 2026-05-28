pub mod config;
pub mod document;
pub mod rule;
pub mod rules;

use std::path::Path;

use crate::config::TextlintRc;
use crate::document::Document;
use crate::rule::{Issue, Rule};

pub fn build_rules(rc: &TextlintRc, base_dir: &Path) -> Vec<Box<dyn Rule>> {
    let mut out: Vec<Box<dyn Rule>> = Vec::new();

    if rc.preset_child_enabled(
        "@textlint-ja/preset-ai-writing",
        "no-ai-emphasis-patterns",
    ) {
        out.push(Box::new(
            rules::ai_writing::no_ai_emphasis_patterns::NoAiEmphasisPatterns,
        ));
    }
    if rc.preset_child_enabled("@textlint-ja/preset-ai-writing", "no-ai-hype-expressions") {
        out.push(Box::new(
            rules::ai_writing::no_ai_hype_expressions::NoAiHypeExpressions,
        ));
    }
    if rc.preset_child_enabled("@textlint-ja/preset-ai-writing", "no-ai-list-formatting") {
        out.push(Box::new(
            rules::ai_writing::no_ai_list_formatting::NoAiListFormatting,
        ));
    }
    if rc.preset_child_enabled("@textlint-ja/preset-ai-writing", "ai-tech-writing-guideline") {
        out.push(Box::new(
            rules::ai_writing::ai_tech_writing_guideline::AiTechWritingGuideline,
        ));
    }

    if rc.preset_child_enabled("preset-ja-technical-writing", "sentence-length") {
        out.push(Box::new(rules::jt::sentence_length::SentenceLength::default()));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "max-comma") {
        out.push(Box::new(rules::jt::max_comma::MaxComma::default()));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-zero-width-spaces") {
        out.push(Box::new(rules::jt::no_zero_width_spaces::NoZeroWidthSpaces));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-hankaku-kana") {
        out.push(Box::new(rules::jt::no_hankaku_kana::NoHankakuKana));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-nfd") {
        out.push(Box::new(rules::jt::no_nfd::NoNfd));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-invalid-control-character") {
        out.push(Box::new(
            rules::jt::no_invalid_control_character::NoInvalidControlCharacter,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-exclamation-question-mark") {
        out.push(Box::new(
            rules::jt::no_exclamation_question_mark::NoExclamationQuestionMark,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "ja-unnatural-alphabet") {
        out.push(Box::new(rules::jt::ja_unnatural_alphabet::JaUnnaturalAlphabet));
    }

    if rc.rule_enabled("prh") {
        for p in rc.prh_rule_paths(base_dir) {
            match rules::prh::Prh::from_yaml_path(&p) {
                Ok(r) => out.push(Box::new(r)),
                Err(e) => eprintln!("[unslop] failed to load prh {}: {e}", p.display()),
            }
        }
    }

    out
}

pub fn lint(source: &str, rules: &[Box<dyn Rule>]) -> Vec<Issue> {
    let doc = Document::parse(source);
    let mut all = Vec::new();
    for r in rules {
        all.extend(r.check(&doc));
    }
    all.sort_by(|a, b| (a.line, a.column).cmp(&(b.line, b.column)));
    all
}
