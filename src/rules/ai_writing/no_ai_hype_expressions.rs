//! preset-ai-writing/no-ai-hype-expressions
//! upstream: src/rules/no-ai-hype-expressions.ts

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "@textlint-ja/preset-ai-writing/no-ai-hype-expressions";

struct PatternEntry {
    pattern: &'static str,
    message: &'static str,
}

const ABSOLUTENESS: &[PatternEntry] = &[
    PatternEntry { pattern: "革命的な", message: "「革命的な」という表現は過度に誇張的である可能性があります。具体的な改善点を述べることを検討してください。" },
    PatternEntry { pattern: "ゲームチェンジャー", message: "「ゲームチェンジャー」という表現は機械的な印象を与える可能性があります。具体的な変化を説明することを検討してください。" },
    PatternEntry { pattern: "世界初の", message: "「世界初の」という表現は過度に強調的である可能性があります。事実に基づいた表現を検討してください。" },
    PatternEntry { pattern: "究極の", message: "「究極の」という表現は誇張的である可能性があります。より具体的で控えめな表現を検討してください。" },
    PatternEntry { pattern: "完全に", message: "「完全に」という絶対的な表現は過度に断定的である可能性があります。「多くの場合」などの表現を検討してください。" },
    PatternEntry { pattern: "完璧な", message: "「完璧な」という表現は過度に理想化している可能性があります。具体的な利点を述べることを検討してください。" },
    PatternEntry { pattern: "最高の", message: "「最高の」という表現は主観的で誇張的である可能性があります。より客観的な評価を示すことを検討してください。" },
    PatternEntry { pattern: "最先端の", message: "「最先端の」という表現は定型的である可能性があります。具体的な技術的特徴を説明することを検討してください。" },
    PatternEntry { pattern: "大幅に", message: "「大幅に」という表現は誇張的である可能性があります。具体的な数値や割合を示すことを検討してください。" },
];

const ABSTRACT: &[PatternEntry] = &[
    PatternEntry { pattern: "魔法のように", message: "「魔法のように」という比喩的表現は現実味に欠ける可能性があります。具体的な仕組みを説明することを検討してください。" },
    PatternEntry { pattern: "奇跡的な", message: "「奇跡的な」という表現は過度に感情的である可能性があります。具体的な成果を示すことを検討してください。" },
    PatternEntry { pattern: "驚異的な", message: "「驚異的な」という表現は誇張的である可能性があります。数値や事実に基づいた表現を検討してください。" },
    PatternEntry { pattern: "可能性を解き放つ", message: "「可能性を解き放つ」という抽象的な表現は曖昧である可能性があります。具体的な利益を説明することを検討してください。" },
    PatternEntry { pattern: "潜在能力を引き出す", message: "「潜在能力を引き出す」という表現は抽象的である可能性があります。具体的な効果を説明することを検討してください。" },
    PatternEntry { pattern: "民主化する", message: "「民主化する」という表現は技術文脈では曖昧である可能性があります。「利用しやすくする」などの具体的な表現を検討してください。" },
    PatternEntry { pattern: "スーパーチャージ", message: "「スーパーチャージ」という表現は機械的な印象を与える可能性があります。具体的な改善内容を説明することを検討してください。" },
    PatternEntry { pattern: "驚嘆させ", message: "「驚嘆させる」という表現は過度に感情的である可能性があります。客観的な評価を示すことを検討してください。" },
];

const PREDICTIVE: &[PatternEntry] = &[
    PatternEntry { pattern: "業界を再定義", message: "「業界を再定義する」という表現は誇張的である可能性があります。具体的な変化を説明することを検討してください。" },
    PatternEntry { pattern: "未来を変える", message: "「未来を変える」という表現は大げさである可能性があります。具体的な改善点を述べることを検討してください。" },
    PatternEntry { pattern: "パラダイムシフト", message: "「パラダイムシフト」という表現は定型的である可能性があります。具体的な変化を説明することを検討してください。" },
    PatternEntry { pattern: "不可避の", message: "「不可避の」という表現は過度に断定的である可能性があります。「可能性が高い」などの表現を検討してください。" },
    PatternEntry { pattern: "新たな基準を設定", message: "「新たな基準を設定」という表現は誇張的である可能性があります。具体的な改善内容を説明することを検討してください。" },
    PatternEntry { pattern: "次世代の", message: "「次世代の」という表現は定型的である可能性があります。具体的な技術的進歩を説明することを検討してください。" },
    PatternEntry { pattern: "フロンティアを開拓", message: "「フロンティアを開拓」という比喩的表現は抽象的である可能性があります。具体的な取り組みを説明することを検討してください。" },
    PatternEntry { pattern: "根本的に変革", message: "「根本的に変革」という表現は誇張的である可能性があります。具体的な変化を説明することを検討してください。" },
];

struct CompiledPattern {
    re: Regex,
    message: &'static str,
}

fn compile(entries: &[&PatternEntry]) -> Vec<CompiledPattern> {
    entries
        .iter()
        .map(|e| CompiledPattern {
            re: Regex::new(&fancy_regex::escape(e.pattern)).unwrap(),
            message: e.message,
        })
        .collect()
}

static ALL_PATTERNS: Lazy<Vec<CompiledPattern>> = Lazy::new(|| {
    let all: Vec<&PatternEntry> = ABSOLUTENESS
        .iter()
        .chain(ABSTRACT.iter())
        .chain(PREDICTIVE.iter())
        .collect();
    compile(&all)
});

pub struct NoAiHypeExpressions;

impl Rule for NoAiHypeExpressions {
    fn id(&self) -> &str {
        RULE_ID
    }

    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !matches!(
                seg.kind,
                SegmentKind::Paragraph
                    | SegmentKind::ListItem
                    | SegmentKind::Heading
                    | SegmentKind::TableCell
            ) {
                continue;
            }
            for cp in ALL_PATTERNS.iter() {
                let mut from = 0usize;
                while let Ok(Some(m)) = cp.re.find_from_pos(&seg.text, from) {
                    let s = m.start();
                    let e = m.end();
                    let (line, column) = doc.pos_at(seg, s);
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: cp.message.to_string(),
                        line,
                        column,
                        severity: Severity::Error,
                    });
                    from = e.max(s + 1);
                }
            }
        }
        issues
    }
}
