//! preset-ai-writing/ai-tech-writing-guideline
//! upstream: src/rules/ai-tech-writing-guideline.ts
//!
//! Paragraph 内の各 category regex (redundancy/voice/clarity/consistency/structure) を検出する。
//! upstream の DocumentExit 品質サマリは LLM の直しループ要因になるため出力しない。
//! Document-level の Paragraph→List 隣接検出 (detectMechanicalListIntroPattern) は
//! 構造解析が必要なので未対応。

use fancy_regex::Regex;
use once_cell::sync::Lazy;

use crate::document::{Document, SegmentKind};
use crate::rule::{Issue, Rule, Severity};

const RULE_ID: &str = "@textlint-ja/preset-ai-writing/ai-tech-writing-guideline";

struct Pattern {
    re: Regex,
    message: &'static str,
}

fn pat(re: &str, message: &'static str) -> Pattern {
    Pattern {
        re: Regex::new(re).unwrap(),
        message,
    }
}

static PATTERNS: Lazy<Vec<Pattern>> = Lazy::new(|| {
    vec![
        // Redundancy
        pat(
            r"まず最初に",
            "【簡潔性】冗長表現が検出されました。「まず最初に」→「まず」または「最初に」への簡潔化を検討してください。",
        ),
        pat(
            r"あらかじめ予測",
            "【簡潔性】冗長表現が検出されました。「あらかじめ予測」→「予測」への簡潔化を検討してください。",
        ),
        pat(
            r"することができます",
            "【簡潔性】冗長な助動詞表現が検出されました。「できます」または「します」への簡潔化を検討してください。",
        ),
        pat(
            r"する必要があります",
            "【簡潔性】冗長な義務表現が検出されました。「してください」または「します」への直接的な表現を検討してください。",
        ),
        pat(
            r"言うまでもなく",
            "【簡潔性】不要な前置き表現が検出されました。核心から始める簡潔な文章構成を検討してください。",
        ),
        // Voice
        pat(
            r"が行われ(て|る|ます)",
            "【明確性】受動的で抽象的な表現が検出されました。具体的な動詞を使った能動態への変更を検討してください（例：「実行する」「処理する」）。",
        ),
        pat(
            r"の変更を行",
            "【明確性】名詞化された表現が検出されました。「を変更する」のような直接的な動詞表現を検討してください。",
        ),
        pat(
            r"の実装を実施",
            "【明確性】二重の名詞化表現が検出されました。「を実装する」への簡潔化を検討してください。",
        ),
        pat(
            r"によって[実行処理実施]され",
            "【明確性】受動態表現が検出されました。「○○が△△を実行する」のような能動態への変更を検討してください。",
        ),
        pat(
            r"がシステムによって実行される",
            "【明確性】受動態表現が検出されました。「システムが○○を実行する」のような能動態への変更を検討してください。",
        ),
        pat(
            r"によって実行され",
            "【明確性】受動態表現が検出されました。「システムが○○を実行する」のような能動態への変更を検討してください。",
        ),
        // Clarity
        pat(
            r"高速な(?:パフォーマンス|処理|動作)",
            "【具体性】抽象的な性能表現が検出されました。具体的な数値基準の提示を検討してください（例：「50ms未満の応答時間」）。",
        ),
        pat(
            r"大幅に(?:向上|改善|削減)",
            "【具体性】定量化されていない変化表現が検出されました。具体的な数値や割合の提示を検討してください。",
        ),
        pat(
            r"効率的な",
            "【具体性】抽象的な評価表現が検出されました。何に対してどのように効率的なのか、具体的な説明を検討してください。",
        ),
        pat(
            r"適切な",
            "【具体性】曖昧な判断表現が検出されました。何を基準として適切なのか、具体的な条件や基準の明示を検討してください。",
        ),
        pat(
            r"必要に応じて",
            "【具体性】曖昧な条件表現が検出されました。どのような状況で必要なのか、具体的な判断基準の明示を検討してください。",
        ),
        // Consistency
        pat(
            r"(ユーザー.*?(?:クライアント|顧客))|(?:(?:クライアント|顧客).*?ユーザー)",
            "【一貫性】同一対象を指す用語の混在が検出されました。文書全体で統一した用語の使用を検討してください。",
        ),
        pat(
            r"(設定画面.*?(?:設定ページ|環境設定))|(?:(?:設定ページ|環境設定).*?設定画面)",
            "【一貫性】機能名称の表記揺れが検出されました。プロジェクト内で統一した名称の使用を検討してください。",
        ),
        pat(
            r"(です。.*?である。)|(である。.*?です。)",
            "【一貫性】文体の混在が検出されました。「です・ます調」または「だ・である調」への統一を検討してください。",
        ),
        // Structure
        pat(
            r"また、.*?また、",
            "【構造化】接続表現の重複が検出されました。箇条書きや段落分けによる情報整理を検討してください。",
        ),
        pat(
            r"(?:第一に|まず).*?(?:第二に|次に).*?(?:第三に|最後に)",
            "【構造化】連続的な手順説明が検出されました。番号付きリストまたは見出し構造での整理を検討してください。",
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

        for seg in &doc.segments {
            if !matches!(seg.kind, SegmentKind::Paragraph | SegmentKind::ListItem) {
                continue;
            }

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
                        fix: None,
                    });
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

    #[test]
    fn summary_no_longer_emitted() {
        let doc = Document::parse("まず最初に設定ファイルを開きます。\n");
        let issues = AiTechWritingGuideline.check(&doc);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.starts_with("【簡潔性】"));
    }

    #[test]
    fn no_summary_when_multiple_individual_issues() {
        let doc = Document::parse("まず最初にすることができます。\n");
        let issues = AiTechWritingGuideline.check(&doc);
        for i in &issues {
            assert!(
                !i.message.contains("品質分析"),
                "summary should not appear: {}",
                i.message
            );
        }
    }
}
