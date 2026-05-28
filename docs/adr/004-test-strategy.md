# 004. テスト戦略 (compat + golden の二段構え)

- 日付: 2026-05-28
- 状態: Accepted

## コンテキスト

textlint 互換 linter として、unslop の動作仕様を継続的に担保したい。
本家 upstream には豊富な test cases があるが、build 構成 (babel transpile 前提の ESM) のため
全 rule の test を機械的に取り込むのが難しい。
また実 markdown corpus での挙動も追跡したい。

## 決定

テストを 2 層で構成する。

### Layer 1: compat test (`tests/compat.rs` + `tests/cases/`)

upstream の textlint-tester ベース test を `tests/upstream-loader/hijack.cjs` で hijack し、
`tester.run(name, rule, { valid, invalid })` の引数を JSON dump する。
JSON を Rust 側で読み、各 rule 実装に対して exact match を取る。
取り込みできた rule (5 個) のみカバー。

### Layer 2: golden fixture diff (`tests/golden.rs` + `tests/golden/`)

実 md fixture を本家 textlint で lint した結果を `expected.txt` として凍結し、
unslop の出力と `(line, rule_id)` 集合で diff を取る。
`floor_common` と `ceiling_unslop_only` を fixture ごとに baseline 化する。
過剰検出と過小検出の両方を閾値として凍結し、回帰を CI から検知する。

## 帰結

- 5 rule は exact match で担保 (compat)、19 rule は実 corpus で粗粒度に担保 (golden)
- 新 rule 追加時は `tests/golden/fixtures/` にその rule が発火する md を 1 個追加し floor を上げる
- textlint upstream が変わったら `scripts/regen-golden.sh` で expected 再生成、閾値を手動更新
- compat test の取り込みできない rule (max-ten 等) は golden だけで担保するため、message exact 担保は無い

## 代替案

- snapshot test (insta crate) のみ: 差分が出ても何が悪化したか粒度が荒い
- 各 rule に独自 fixture: メンテ重く、textlint と直接 diff が取れない
- textlint を CI で再実行: Node.js + npm install のコストが大きく、CI 時間が +30s 以上
