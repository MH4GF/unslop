pub mod config;
pub mod document;
mod fixer;
pub mod log;
pub mod morph;
pub mod rule;
pub mod rules;

use std::path::Path;

use crate::config::TextlintRc;
use crate::document::Document;
use crate::fixer::{MAX_PASSES, apply_fixes};
use crate::rule::{ByteRange, Issue, Rule};

pub fn build_rules(rc: &TextlintRc, base_dir: &Path) -> Vec<Box<dyn Rule>> {
    let mut out: Vec<Box<dyn Rule>> = Vec::new();

    if rc.preset_child_enabled("@textlint-ja/preset-ai-writing", "no-ai-emphasis-patterns") {
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
    if rc.preset_child_enabled(
        "@textlint-ja/preset-ai-writing",
        "ai-tech-writing-guideline",
    ) {
        out.push(Box::new(
            rules::ai_writing::ai_tech_writing_guideline::AiTechWritingGuideline,
        ));
    }
    if rc.preset_child_enabled("@textlint-ja/preset-ai-writing", "no-ai-colon-continuation") {
        out.push(Box::new(
            rules::ai_writing::no_ai_colon_continuation::NoAiColonContinuation,
        ));
    }

    if rc.preset_child_enabled("preset-ja-technical-writing", "sentence-length") {
        let max = rc
            .preset_child_option("preset-ja-technical-writing", "sentence-length", "max")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(100);
        out.push(Box::new(rules::jt::sentence_length::SentenceLength { max }));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "max-comma") {
        let max = rc
            .preset_child_option("preset-ja-technical-writing", "max-comma", "max")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(3);
        out.push(Box::new(rules::jt::max_comma::MaxComma { max }));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "max-ten") {
        let max = rc
            .preset_child_option("preset-ja-technical-writing", "max-ten", "max")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(3);
        out.push(Box::new(rules::jt::max_ten::MaxTen { max }));
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
    if rc.preset_child_enabled(
        "preset-ja-technical-writing",
        "no-invalid-control-character",
    ) {
        out.push(Box::new(
            rules::jt::no_invalid_control_character::NoInvalidControlCharacter,
        ));
    }
    if rc.preset_child_enabled(
        "preset-ja-technical-writing",
        "no-exclamation-question-mark",
    ) {
        out.push(Box::new(
            rules::jt::no_exclamation_question_mark::NoExclamationQuestionMark,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "ja-unnatural-alphabet") {
        out.push(Box::new(
            rules::jt::ja_unnatural_alphabet::JaUnnaturalAlphabet,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-doubled-conjunction") {
        out.push(Box::new(
            rules::jt::no_doubled_conjunction::NoDoubledConjunction,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-doubled-joshi") {
        out.push(Box::new(rules::jt::no_doubled_joshi::NoDoubledJoshi));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-mix-dearu-desumasu") {
        out.push(Box::new(
            rules::jt::no_mix_dearu_desumasu::NoMixDearuDesumasu,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-double-negative-ja") {
        out.push(Box::new(
            rules::jt::no_double_negative_ja::NoDoubleNegativeJa,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "no-unmatched-pair") {
        out.push(Box::new(rules::jt::no_unmatched_pair::NoUnmatchedPair));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "ja-no-mixed-period") {
        out.push(Box::new(rules::jt::ja_no_mixed_period::JaNoMixedPeriod));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "ja-no-redundant-expression") {
        out.push(Box::new(
            rules::jt::ja_no_redundant_expression::JaNoRedundantExpression,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "ja-no-successive-word") {
        out.push(Box::new(
            rules::jt::ja_no_successive_word::JaNoSuccessiveWord,
        ));
    }
    if rc.preset_child_enabled(
        "preset-ja-technical-writing",
        "no-doubled-conjunctive-particle-ga",
    ) {
        out.push(Box::new(
            rules::jt::no_doubled_conjunctive_particle_ga::NoDoubledConjunctiveParticleGa,
        ));
    }
    if rc.preset_child_enabled("preset-ja-technical-writing", "ja-no-abusage") {
        out.push(Box::new(rules::jt::ja_no_abusage::JaNoAbusage));
    }

    if rc.preset_child_enabled("preset-ja-spacing", "ja-no-space-between-full-width") {
        out.push(Box::new(
            rules::ja_spacing::ja_no_space_between_full_width::JaNoSpaceBetweenFullWidth,
        ));
    }
    if rc.preset_child_enabled("preset-ja-spacing", "ja-space-between-half-and-full-width") {
        out.push(Box::new(
            rules::ja_spacing::ja_space_between_half_and_full_width::JaSpaceBetweenHalfAndFullWidth,
        ));
    }

    // unslop-original rule (textlint に該当 rule なし)。standalone gate で有効化する。
    if rc.rule_enabled("no-mid-sentence-break") {
        out.push(Box::new(
            rules::unslop::no_mid_sentence_break::NoMidSentenceBreak,
        ));
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
    all.sort_by_key(|a| (a.line, a.column));
    all
}

#[derive(Debug, Clone)]
pub struct AppliedFix {
    pub rule_id: String,
    pub range: ByteRange,
    pub replacement: String,
    pub pass: usize,
}

#[derive(Debug, Clone)]
pub struct FixResult {
    pub fixed_source: String,
    pub applied_fixes: Vec<AppliedFix>,
    pub remaining_issues: Vec<Issue>,
    pub passes: usize,
    pub hit_max_passes: bool,
}

pub fn fix(source: &str, rules: &[Box<dyn Rule>]) -> FixResult {
    let mut current = source.to_string();
    let mut applied_fixes: Vec<AppliedFix> = Vec::new();
    let mut passes = 0;
    let mut hit_max = false;

    loop {
        passes += 1;
        let issues = lint(&current, rules);
        let pending: Vec<(String, crate::rule::Fix)> = issues
            .iter()
            .filter_map(|i| i.fix.clone().map(|f| (i.rule_id.clone(), f)))
            .collect();
        if pending.is_empty() {
            break;
        }
        let fixes: Vec<crate::rule::Fix> = pending.iter().map(|(_, f)| f.clone()).collect();
        let (next, applied, _dropped) = apply_fixes(&current, &fixes);
        if applied.is_empty() {
            break;
        }
        for af in &applied {
            let rid = pending
                .iter()
                .find(|(_, f)| f.range == af.range && f.replacement == af.replacement)
                .map(|(rid, _)| rid.clone())
                .unwrap_or_default();
            applied_fixes.push(AppliedFix {
                rule_id: rid,
                range: af.range.clone(),
                replacement: af.replacement.clone(),
                pass: passes,
            });
        }
        current = next;
        if passes >= MAX_PASSES {
            hit_max = true;
            break;
        }
    }

    let remaining_issues = lint(&current, rules);
    FixResult {
        fixed_source: current,
        applied_fixes,
        remaining_issues,
        passes,
        hit_max_passes: hit_max,
    }
}
