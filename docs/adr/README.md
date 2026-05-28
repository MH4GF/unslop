# Architecture Decision Records

設計上の判断を凍結する。実装で迷ったら ADR を読み、議論を蒸し返さない。
状態は Accepted / Superseded / Deprecated のいずれか。新規追加は連番。

| # | タイトル | 状態 |
|---|---------|---|
| [001](./001-textlint-compatibility-policy.md) | textlint との互換性方針 | Accepted |
| [002](./002-tokenizer-choice.md) | 形態素解析に lindera (IPADIC) を採用 | Accepted |
| [003](./003-document-segment-extraction.md) | Markdown segment 抽出を source slice ベースで行う | Accepted |
| [004](./004-test-strategy.md) | テスト戦略 (compat + golden の二段構え) | Accepted |
