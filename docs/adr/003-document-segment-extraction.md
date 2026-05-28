# 003. Markdown segment 抽出を source slice ベースで行う

- 日付: 2026-05-28
- 状態: Accepted

## コンテキスト

textlint の rule は `getSource(node)` から markdown source の生 string を受け取る。
bold 記法 (`**` 囲みの強調等) を含んだ raw text に対して regex を回す。
Rust 側で「parsed plain text」を渡すと markup 記号を含む rule
(no-ai-emphasis-patterns / no-ai-list-formatting) が動かない。

## 決定

`Document::parse` で comrak の AST を構築した後、Paragraph / Heading / ListItem / TableCell
の **sourcepos 範囲を source 文字列から slice** して `TextSegment.text` に保持する。
Markup を剥がさず、textlint 互換の挙動になる。

各 segment は次を保持する。

- `text` = source markdown のその range の slice
- `start_byte` / `start_line` / `start_column` = source 内位置 (1-based)
- `kind` = Paragraph / Heading / ListItem / TableCell / CodeBlock

byte offset → (line, column) の解決は `Document::pos_at` で行う。

## 帰結

- markup 検出系の rule をそのまま port できる
- code span 内の token も検出対象になり、prh が inline code 内の英単語を過剰検出する等の副作用がある
  (Phase 2 で code span 除外を入れる予定)
- ListItem の中に入れ子の Paragraph は textlint と同じく親 ListItem 側で処理する
  (Phase 1a で発生した bug を fix 済)
- comrak の `sourcepos.column` は byte ベース (1-based) なので、char ベースとの変換ロジックが要る

## 代替案

- plain text 抽出 (markup 剥がし): markup 検出系の rule が壊れる
- textlint と同じ Str ノード単位 (inline runs): 実装が煩雑で position mapping が複雑になる
- comrak の代わりに pulldown-cmark: source slice を取りにくい (event-based parser)
