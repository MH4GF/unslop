# 001. textlint との互換性方針

- 日付: 2026-05-28
- 状態: Accepted

## コンテキスト

textlint の Rust 置き換えとして実装するが、textlint の全挙動を 1:1 で再現するのは現実的でない。
本家は markdown plugin / Str ノード解析 / kuromoji 連携 / preset 解決などで多層の依存を持つ。
完全互換を目指すと初期実装の工数が爆発し、Phase 1a/1b を出せない。

## 決定

互換性を 3 段階に分けて目標を定める。

1. rule の発火対象 (line + rule_id) は 90% 以上一致させる。`tests/golden/` で floor を凍結する
2. メッセージ本文の 1 行目を末尾の `.`/`。` 抜きで一致させる。`compat.rs` で exact match を取る
3. column と数値は simplified 実装の差を許容する
   - sentence-length の `length` 値、`Over X characters` 値
   - max-comma / max-ten の検出 column
   - サマリレポート位置 (ai-tech-writing-guideline)

互換性は `tests/golden.rs` の `(line, rule_id)` 集合一致 + floor/ceiling 閾値で常時担保する。
過剰検出 (`unslop_only`) も閾値で固定し、下げる修正は意識的に行う。

## 帰結

- 短期で 19 rule を出せた (Phase 1a + 1b)。
- column や数値ベースで Issue を取りに来る downstream consumer (IDE 等) は互換にならない。
  Phase 2 で IDE 統合の要求が出たら寄せる。
- upstream の挙動が変わった時は手動で expected を再生成する。`scripts/regen-golden.sh` を用意済み。

## 代替案

- 完全互換: 工数 5-10x。形態素 token の位置情報、markdown plugin 全互換が必要。後回し。
- ノー互換 (独自 message): 既存 hook / IDE 統合が壊れる。却下。
