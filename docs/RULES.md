# Rules

unslop が実装している rule の一覧。
simplified なものは upstream の挙動と部分的に異なる。詳細は [ADR 001](./adr/001-textlint-compatibility-policy.md) を参照。

## preset-ai-writing

| rule | 実装 | simplified | compat test |
|------|:----:|:----------:|:-----------:|
| no-ai-emphasis-patterns | ✅ | - | ✅ |
| no-ai-hype-expressions | ✅ | - | ✅ |
| no-ai-list-formatting | ✅ | - | ✅ |
| ai-tech-writing-guideline | ✅ | サマリレポート位置が最後の paragraph | ✅ |
| no-ai-colon-continuation | ✅ | 構造判定を segment 隣接で近似 (Document AST 不使用) | - |

upstream: <https://github.com/textlint-ja/textlint-rule-preset-ai-writing>

## preset-ja-technical-writing

| rule | 実装 | simplified | compat test |
|------|:----:|:----------:|:-----------:|
| sentence-length | ✅ | char 数で測定 (textlint は markup 剥がし後の length) | - |
| max-comma | ✅ | sentence-splitter 不使用 (改行と句点で split) | - |
| max-ten | ✅ | sentence-splitter 不使用 / 括弧内例外なし | - |
| ja-no-mixed-period | ❌ | config で disable 済 | - |
| no-mix-dearu-desumasu | ✅ | 末尾文字判定のみ。multi-section の preference 未対応 | - |
| no-doubled-conjunction | ✅ | sentence-splitter 不使用 | - |
| no-doubled-joshi | ✅ | 例外パターン主要 4 個のみ (の/を/て/並立) | - |
| no-double-negative-ja | ✅ | 主要 8 patterns | - |
| no-zero-width-spaces | ✅ | - | - |
| no-hankaku-kana | ✅ | - | - |
| no-nfd | ✅ | - | ✅ |
| no-invalid-control-character | ✅ | options (checkCode/checkImage) 未対応 | - |
| no-exclamation-question-mark | ✅ | options (allow*Mark) 未対応 / `Yahoo!` のみ allow | - |
| ja-unnatural-alphabet | ✅ | サロゲートペア未対応 / allowCommonCase 未対応 | - |
| no-unmatched-pair | ✅ | sentence-splitter 不使用 / pair list は upstream と同じ | - |
| no-dropping-the-ra | ❌ | 公式 repo 不明 | - |

`compat test` 列は `tests/compat.rs` で exact match を取れているもの。
それ以外は `tests/golden.rs` の粗粒度差分 + 手動検証で担保している。

## prh

| rule | 実装 | simplified |
|------|:----:|:----------:|
| prh | ✅ | YAML の `version: 1` + `rules: [{ expected, pattern }]` 形式のみ。`$1` 置換と複数 YAML import は未対応 |

## unslop-original (textlint 非対応)

textlint に対応 rule がない unslop 独自の防波堤。`.textlintrc.json` の rules へ直接書いて有効化する。

| rule | 実装 | 概要 |
|------|:----:|------|
| no-mid-sentence-break | ✅ | 段落 (引用配下を除く) の生テキストを走査し、文末記号以外の直後の改行を文中改行として検出する。長い一文の途中へ改行を挟み sentence-length をすり抜ける書き方を防ぐ。検出のみ |

## options 対応状況

| 設定 | 対応 |
|------|:---:|
| `preset-ja-technical-writing.sentence-length.max` | ✅ |
| `preset-ja-technical-writing.max-comma.max` | ✅ |
| `preset-ja-technical-writing.max-ten.max` | ✅ |
| `prh.rulePaths` (相対パス) | ✅ |
| preset 子 rule の bool 無効化 (`{ "rule-name": false }`) | ✅ |
| その他の rule options (allow 等) | ❌ |
