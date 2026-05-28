//! preset-ai-writing/ai-tech-writing-guideline
//! upstream: src/rules/ai-tech-writing-guideline.ts
//!
//! Phase 1a 範囲は Paragraph 内の各 category regex
//! (redundancy/voice/clarity/consistency/structure) + DocumentExit 相当の品質サマリレポート。
//! Document-level の Paragraph→List 隣接検出 (detectMechanicalListIntroPattern) は
//! 構造解析が必要なので Phase 1b 以降に回す。

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "@textlint-ja/preset-ai-writing/ai-tech-writing-guideline";

#[derive(Clone, Copy)]
enum Category {
    Redundancy,
    Voice,
    Clarity,
    Consistency,
    Structure,
}

struct Pattern {
    re: Regex,
    message: &'static str,
    category: Category,
}

fn pat(re: &str, message: &'static str, category: Category) -> Pattern {
    Pattern {
        re: Regex::new(re).unwrap(),
        message,
        category,
    }
}

static PATTERNS: Lazy<Vec<Pattern>> = Lazy::new(|| {
    use Category::*;
    vec![
        // Redundancy
        pat(
            r"まず最初に",
            "【簡潔性】冗長表現が検出されました。「まず最初に」→「まず」または「最初に」への簡潔化を検討してください。",
            Redundancy,
        ),
        pat(
            r"あらかじめ予測",
            "【簡潔性】冗長表現が検出されました。「あらかじめ予測」→「予測」への簡潔化を検討してください。",
            Redundancy,
        ),
        pat(
            r"することができます",
            "【簡潔性】冗長な助動詞表現が検出されました。「できます」または「します」への簡潔化を検討してください。",
            Redundancy,
        ),
        pat(
            r"する必要があります",
            "【簡潔性】冗長な義務表現が検出されました。「してください」または「します」への直接的な表現を検討してください。",
            Redundancy,
        ),
        pat(
            r"言うまでもなく",
            "【簡潔性】不要な前置き表現が検出されました。核心から始める簡潔な文章構成を検討してください。",
            Redundancy,
        ),
        // Voice
        pat(
            r"が行われ(て|る|ます)",
            "【明確性】受動的で抽象的な表現が検出されました。具体的な動詞を使った能動態への変更を検討してください（例：「実行する」「処理する」）。",
            Voice,
        ),
        pat(
            r"の変更を行",
            "【明確性】名詞化された表現が検出されました。「を変更する」のような直接的な動詞表現を検討してください。",
            Voice,
        ),
        pat(
            r"の実装を実施",
            "【明確性】二重の名詞化表現が検出されました。「を実装する」への簡潔化を検討してください。",
            Voice,
        ),
        pat(
            r"によって[実行処理実施]され",
            "【明確性】受動態表現が検出されました。「○○が△△を実行する」のような能動態への変更を検討してください。",
            Voice,
        ),
        pat(
            r"がシステムによって実行される",
            "【明確性】受動態表現が検出されました。「システムが○○を実行する」のような能動態への変更を検討してください。",
            Voice,
        ),
        pat(
            r"によって実行され",
            "【明確性】受動態表現が検出されました。「システムが○○を実行する」のような能動態への変更を検討してください。",
            Voice,
        ),
        // Clarity
        pat(
            r"高速な(?:パフォーマンス|処理|動作)",
            "【具体性】抽象的な性能表現が検出されました。具体的な数値基準の提示を検討してください（例：「50ms未満の応答時間」）。",
            Clarity,
        ),
        pat(
            r"大幅に(?:向上|改善|削減)",
            "【具体性】定量化されていない変化表現が検出されました。具体的な数値や割合の提示を検討してください。",
            Clarity,
        ),
        pat(
            r"効率的な",
            "【具体性】抽象的な評価表現が検出されました。何に対してどのように効率的なのか、具体的な説明を検討してください。",
            Clarity,
        ),
        pat(
            r"適切な",
            "【具体性】曖昧な判断表現が検出されました。何を基準として適切なのか、具体的な条件や基準の明示を検討してください。",
            Clarity,
        ),
        pat(
            r"必要に応じて",
            "【具体性】曖昧な条件表現が検出されました。どのような状況で必要なのか、具体的な判断基準の明示を検討してください。",
            Clarity,
        ),
        // Consistency
        pat(
            r"(ユーザー.*?(?:クライアント|顧客))|(?:(?:クライアント|顧客).*?ユーザー)",
            "【一貫性】同一対象を指す用語の混在が検出されました。文書全体で統一した用語の使用を検討してください。",
            Consistency,
        ),
        pat(
            r"(設定画面.*?(?:設定ページ|環境設定))|(?:(?:設定ページ|環境設定).*?設定画面)",
            "【一貫性】機能名称の表記揺れが検出されました。プロジェクト内で統一した名称の使用を検討してください。",
            Consistency,
        ),
        pat(
            r"(です。.*?である。)|(である。.*?です。)",
            "【一貫性】文体の混在が検出されました。「です・ます調」または「だ・である調」への統一を検討してください。",
            Consistency,
        ),
        // Structure
        pat(
            r"また、.*?また、",
            "【構造化】接続表現の重複が検出されました。箇条書きや段落分けによる情報整理を検討してください。",
            Structure,
        ),
        pat(
            r"(?:第一に|まず).*?(?:第二に|次に).*?(?:第三に|最後に)",
            "【構造化】連続的な手順説明が検出されました。番号付きリストまたは見出し構造での整理を検討してください。",
            Structure,
        ),
    ]
});

pub struct AiTechWritingGuideline;

impl Rule for AiTechWritingGuideline {
    fn id(&self) -> &str {
        RULE_ID
    }

    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        let mut metrics = [0usize; 5];
        let mut had_any = false;
        let mut last_paragraph_pos: Option<(usize, usize)> = None;

        for seg in &doc.segments {
            if !matches!(seg.kind, SegmentKind::Paragraph | SegmentKind::ListItem) {
                continue;
            }
            last_paragraph_pos = Some((seg.start_line, seg.start_column));

            for p in PATTERNS.iter() {
                let mut from = 0usize;
                while let Ok(Some(m)) = p.re.find_from_pos(&seg.text, from) {
                    let s = m.start();
                    let e = m.end();
                    let (line, column) = doc.pos_at(seg, s);
                    issues.push(Issue {
                        rule_id: RULE_ID.to_string(),
                        message: p.message.to_string(),
                        line,
                        column,
                        severity: Severity::Error,
                    });
                    let idx = p.category as usize;
                    metrics[idx] += 1;
                    had_any = true;
                    from = e.max(s + 1);
                }
            }
        }

        if had_any {
            let total: usize = metrics.iter().sum();
            let labels = ["簡潔性", "明確性", "具体性", "一貫性", "構造化"];
            let details: Vec<String> = labels
                .iter()
                .zip(metrics.iter())
                .filter(|&(_, &c)| c > 0)
                .map(|(l, &c)| format!("{l}: {c}件"))
                .collect();
            let details_text = if details.is_empty() {
                String::new()
            } else {
                format!(" [内訳: {}]", details.join(", "))
            };
            let (line, column) = last_paragraph_pos.unwrap_or((1, 1));
            issues.push(Issue {
                rule_id: RULE_ID.to_string(),
                message: format!(
                    "【テクニカルライティング品質分析】この文書で{total}件の改善提案が見つかりました{details_text}。効果的なテクニカルライティングの7つのC（Clear, Concise, Correct, Coherent, Concrete, Complete, Courteous）の原則に基づいて見直しを検討してください。詳細なガイドライン: https://github.com/textlint-ja/textlint-rule-preset-ai-writing/blob/main/docs/tech-writing-guidelines.md"
                ),
                line,
                column,
                severity: Severity::Error,
            });
        }

        issues
    }
}
