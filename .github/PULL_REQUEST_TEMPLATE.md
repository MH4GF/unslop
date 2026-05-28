## Summary

何を変更したか書く (Rule 追加 / 修正 / refactor / docs など)。

## Rule 追加の場合

- [ ] upstream link (例: `https://github.com/textlint-rule/textlint-rule-X`)
- [ ] simplification を `docs/RULES.md` に追記した
- [ ] `tests/golden/fixtures/` に発火する md を追加した
- [ ] `scripts/regen-golden.sh` で expected を更新した
- [ ] `tests/golden.rs` の floor / ceiling を更新した
- [ ] (可能なら) compat test を `tests/compat.rs` に追加した

## 既存 rule の修正の場合

- [ ] `tests/golden.rs` の floor / ceiling 変動を確認し、必要なら閾値を更新した
- [ ] 過剰検出を減らした場合は ceiling を下げた

## 設計判断を伴う場合

- [ ] `docs/adr/` に ADR を追加した

## 確認

- [ ] `cargo test --all-targets` 緑
- [ ] `cargo clippy --all-targets -- -D warnings` 緑
- [ ] `cargo fmt --all -- --check` 緑
- [ ] `bench/smoke.sh` 緑 (medium fixture < 300ms)
