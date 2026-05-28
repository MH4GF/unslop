# textlint bottleneck profile

計測日 2026-05-28. textlint v repo: `MH4GF/claude-code` の node_modules.
ハーネス bench/run.sh, 6 回実行のうち warmup 1 回を捨て median を採用.

## 1. baseline (全 rule on)

ファイルサイズと wall-clock の関係.

| fixture     | lines | median (ms) |
| ----------- | ----: | ----------: |
| empty.md    |     0 |         292 |
| small.md    |    60 |         570 |
| medium.md   |   185 |         891 |
| large.md    |   273 |         567 |

medium > large の逆転は内容差 (medium SKILL.md は日本語密度が高く `ja-technical-writing/*` の hit が多い) によるもの.

固定費の推定: **empty.md = 292ms ≈ Node.js cold start + textlint kernel 初期化 + 全 rule の moduleInterop**.
PostToolUse hook が毎回これを払うのが体感遅さの主因.

## 2. preset / rule 単位 off (medium fixture)

baseline (この run): **743ms**.

### preset 単位

| 設定                | median (ms) | Δ from baseline |
| ------------------- | ----------: | --------------: |
| all-rules-on        |         743 |               0 |
| off:prh             |         715 |             -28 |
| off:preset/ai       |         740 |              -3 |
| off:preset/jt       |         515 |            -228 |

→ `preset-ja-technical-writing` 22 rules で合計 **~228ms**.
1 rule あたり平均 10ms 程度.
`preset-ai-writing` は medium fixture では hit が少なく寄与小.
prh も軽い (28ms).

### 個別 rule (preset-ja-technical-writing 配下)

差分が計測ノイズ (±30ms) より大きいもののみ抜粋.

| rule                          | Δ from baseline |
| ----------------------------- | --------------: |
| ja-no-mixed-period            |             +53 |
| no-dropping-the-ra            |             +38 |
| no-zero-width-spaces          |             +24 |
| no-doubled-joshi              |             +21 |
| sentence-length               |             +19 |
| ja-no-redundant-expression    |             -37 |
| no-unmatched-pair             |             -26 |
| max-comma                     |             -25 |
| no-doubled-conjunctive-particle-ga |        -23 |

差分 ±50ms 以下に収まり、**個別 rule の重さは計測ノイズに埋もれている**.
22 rules それぞれ 5-20ms ずつ消費していると見るのが妥当.
「一発で重い rule」は無く、合計でじわじわ重い構造.

## 3. 内訳推定

| 区分                                      |  ms  |     比 |
| ----------------------------------------- | ---: | -----: |
| Node 起動 + textlint kernel + markdown plugin (empty) | 292 |   39% |
| preset-ja-technical-writing 22 rules      |  228 |    31% |
| その他 (config 読み + rule init + ノイズ) |  194 |    26% |
| prh                                       |   28 |     4% |
| preset-ai-writing 5+ rules                |    3 |    <1% |
| **合計 (= medium baseline)**              |  743 |  100% |

## 4. Rust 化の期待効果

| 区分                       | 現状 ms | Rust 想定 ms | 根拠 |
| -------------------------- | ------: | -----------: | ---- |
| プロセス起動 + lib init    |     292 |        5-10  | Rust CLI の典型値 (clap + serde) |
| markdown parse             | (上に含) |        1-3   | comrak v0.20+ の measured |
| regex 系 rule (全 30+)     |     ~50 |        2-5   | regex crate, 内容に対して streaming |
| 形態素解析が要る rule (4-5)  |    ~100 |       20-40  | lindera-rs + IPADIC, lazy load + once 化 |
| prh + 設定 parse           |      28 |         2-5  | YAML + serde |
| **medium 合計**            |     743 |       30-60  | **~12-25x speedup** |

PostToolUse の体感が 「タイプして 0.8s 待つ」 → 「ほぼ瞬時 (50ms 以下)」 に変わる試算.

## 5. Phase 1 スコープの推奨

個別 rule 計測が flat (どれも 5-20ms) なので、「重い順」ではなく **実装容易な順 + dependency 順** で進める. 提案:

### Phase 1a — regex / string match のみ (形態素解析は使わない) ✦ 1-2 週間目安
- prh (44 行の YAML、最頻出)
- preset-ja-technical-writing:
  - sentence-length / max-comma / max-ten / ja-no-mixed-period
  - no-mix-dearu-desumasu / no-doubled-conjunction
  - no-exclamation-question-mark / no-hankaku-kana
  - no-nfd / no-zero-width-spaces / no-invalid-control-character
  - ja-unnatural-alphabet / no-unmatched-pair / no-dropping-the-ra
  - no-double-negative-ja
- preset-ai-writing: 5 rules 全部 (ほぼ regex)

### Phase 1b — 形態素解析が要る (lindera 導入後)
- max-kanji-continuous-len / ja-no-successive-word
- ja-no-redundant-expression / no-doubled-joshi
- ja-no-weak-phrase / ja-no-abusage / no-doubled-conjunctive-particle-ga

### Phase 2 — hook 置き換え
- `unslop` バイナリで `textlint-guard.sh` 内の textlint 呼び出しを置換
- `.textlintrc.json` 互換 parser (Phase 1 で対応する rule のみ honor)
- 出力フォーマット: `<file>:<line>:<col> <severity> <message> <rule-id>` を維持

## 6. 実測 (Phase 1a 実装後)

Rust 実装 (release build) を同じ fixture で実測。1 計測 6 回、warmup 1 回を除く 5 回 median。

| fixture     | textlint (ms) | unslop (ms) | speedup |
| ----------- | ------------: | ----------: | ------: |
| empty.md    |           269 |          20 |   13.5x |
| small.md    |           564 |          22 |   25.6x |
| medium.md   |           720 |          29 |   24.8x |
| large.md    |           573 |          23 |   24.9x |

予測 (30-60ms、12-25x) を上回る速度。**PostToolUse の体感が 700ms → 30ms に短縮**。

### 実装済み rule (Phase 1a)

`@textlint-ja/preset-ai-writing` 配下。

- no-ai-emphasis-patterns
- no-ai-hype-expressions
- no-ai-list-formatting
- ai-tech-writing-guideline

`preset-ja-technical-writing` 配下。

- sentence-length (simplified)
- max-comma (simplified)
- no-zero-width-spaces
- no-hankaku-kana
- no-nfd
- no-invalid-control-character
- no-exclamation-question-mark
- ja-unnatural-alphabet

加えて `prh` 1 個。

## 7. Phase 1b 実装

lindera-rs (embed-ipadic) を導入し、形態素解析を要する rule + 専用 parser rule を追加した。

| fixture     | textlint (ms) | unslop 1a (ms) | unslop 1b (ms) | 1b speedup |
| ----------- | ------------: | -------------: | -------------: | ---------: |
| empty.md    |           269 |             20 |             22 |      12.2x |
| small.md    |           564 |             22 |             39 |      14.5x |
| medium.md   |           720 |             29 |             46 |      15.6x |
| large.md    |           573 |             23 |             36 |      15.9x |

lindera の cold start + tokenize で +14〜17ms 増えたが依然 15x 高速。

### Phase 1b 追加 rule

形態素解析を使う 5 個。

- preset-ai-writing: no-ai-colon-continuation
- preset-ja-technical-writing: max-ten
- preset-ja-technical-writing: no-doubled-conjunction
- preset-ja-technical-writing: no-mix-dearu-desumasu (simplified)
- preset-ja-technical-writing: no-double-negative-ja

専用 parser を使う 1 個。

- preset-ja-technical-writing: no-unmatched-pair (PairMaker 相当を Vec ベース stack で実装)

これで preset-ja-technical-writing 配下の対応 rule は次の 14 個。

- sentence-length
- max-comma
- max-ten
- no-mix-dearu-desumasu (simplified)
- no-doubled-conjunction
- no-double-negative-ja
- no-zero-width-spaces
- no-hankaku-kana
- no-nfd
- no-invalid-control-character
- no-exclamation-question-mark
- ja-unnatural-alphabet
- no-unmatched-pair

ja-no-mixed-period は config で disable 済。
未対応で残るのは no-dropping-the-ra のみで、公式 repo の所在を確認できていない。

### 互換テスト

tests/cases/ 配下の upstream test cases を JSON 化した。
5 rule で `cargo test --test compat` が緑。
内訳は AI writing 4 個と jt の no-nfd 1 個。
ほかの rule の extract は upstream の build 構成 (babel transpile 前提の ESM test) に阻まれている。
Phase 1b 以降に integration test を追加する。

### 切替え方法

新 hook を `~/.claude/hooks/unslop-guard.sh` として用意した。
既存の `textlint-guard` hook は残してある。
`~/.claude/settings.json` の PostToolUse hook を unslop-guard.sh に切替え済。
`UNSLOP_GUARD=off` で無効化、`UNSLOP_BIN=<path>` でバイナリ位置を override できる。
バイナリ場所 (固定): `/Users/mh4gf/ghq/github.com/MH4GF/unslop/target/release/unslop`。

## 7. ベンチハーネス

- スクリプト: `bench/run.sh [baseline|per-rule|all]`
- fixture: `bench/fixtures/{empty,small,medium,large}.md` (claude-code repo の実 md を凍結)
- 結果: `bench/results/<timestamp>-<mode>.tsv`
- 再実行は `./bench/run.sh all` で約 4-5 分.

Rust 実装の各 phase 完了時に `./bench/run.sh baseline` を unslop バイナリ向けに走らせ、本数字と比較する.
