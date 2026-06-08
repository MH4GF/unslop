# unslop

Fast Rust 実装の textlint 互換 Japanese writing linter。
PostToolUse hook で textlint の代替として 15-25x 高速化する用途で使う。

## ハーネスのレイヤ

| 何 | どこ |
|---|---|
| CLI / lint engine / rules | `src/` |
| 形態素解析 (lindera + IPADIC) | `src/morph.rs` |
| upstream test ケースの compat 検証 | `tests/compat.rs` + `tests/cases/` |
| 実 md fixture で textlint と diff | `tests/golden.rs` + `tests/golden/` |
| 性能ベンチ | `bench/run.sh` + `bench/results/report.md` |
| PostToolUse hook | `~/.claude/hooks/unslop-guard.sh` |

## rule を追加するときの手順

1. upstream の src を読む (`tests/upstream/` に shallow clone)。simplified 範囲を判断
2. `src/rules/<group>/<rule>.rs` を作る。形態素が要るなら `morph::tokenize` を使う
3. `src/rules/<group>/mod.rs` と `src/lib.rs::build_rules` に登録
4. `tests/golden/fixtures/` に該当 rule がトリガーする md を追加
5. `scripts/regen-golden.sh` で expected を再生成し、`cargo test --test golden` で floor/ceiling を baseline 化

## auto-fix を追加するときの手順

1. 置換が機械的に決まる rule のみ対象とする (sentence-length など意味解釈は対象外)
2. `check` 内で `Issue::new(...).with_fix(Fix { range, replacement })` を chain する
3. `range` は source 全体の absolute byte offset (`seg.start_byte + rel_offset` で得る)
4. 1 match で最も保守的な単一 fix を出す。連鎖は `lib::fix()` の loop が捌く
5. `tests/golden/fixtures/auto-fix-basics.md` に該当 rule のトリガーを追加し、expected source も更新する

## textlint との互換性方針

- **完全一致を目指さない**。message・line は寄せるが、column / 数値 / sentence-split 位置は simplified を許容する
- 互換性は `tests/golden.rs` の `(line, rule_id)` 集合一致 + floor/ceiling 閾値で担保する
- `unslop-only` (過剰検出) は閾値で凍結。下げる修正は意識的に行う

## 禁止 / 注意

- hook script (`~/.claude/hooks/*.sh`) は agent から編集しない (auto mode が self-modification として拒否する)
- `tests/upstream/` は git ignore。設定スクリプトで再取得する想定
- `Cargo.toml` の dependency 追加は最小限。ビルド時間が伸びる (lindera 含めて release で 1 分超)
- `panic` マクロは test 内のみ。lib 側はエラーを `Result` で返すか silently スキップする

## 確認コマンド

- `cargo test --all-targets` — compat + golden + unit
- `cargo clippy --all-targets -- -D warnings` — CI と同じ
- `cargo fmt --all -- --check` — CI と同じ
- `bench/run.sh compare` — textlint と unslop の速度比較

## 参考

- 設計判断と進捗は `bench/results/report.md`
- upstream rule の現状リスト・simplification メモは report の §7 を参照
