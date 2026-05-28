# 002. 形態素解析に lindera (IPADIC) を採用

- 日付: 2026-05-28
- 状態: Accepted

## コンテキスト

Phase 1b の rule 群は形態素解析が必須となる。
具体的には max-ten / no-doubled-conjunction / no-doubled-joshi / no-mix-dearu-desumasu /
no-double-negative-ja / no-ai-colon-continuation が対象。
textlint 本家では kuromojin (JavaScript の kuromoji wrapper) を使い、辞書として IPADIC を採用している。
Rust 側で同じ品詞・読み・基本形を得る必要がある。

## 決定

`lindera = "3"` を `embed-ipadic` feature 付きで採用する。
辞書をバイナリに同梱し、CLI 起動時の辞書 load を省く。
`src/morph.rs::tokenize` を共通ラッパとして提供し、各 rule から呼ぶ。

## 帰結

- token の品詞分類 (`pos` / `pos_detail_1..3`) が kuromoji と互換。upstream rule のロジックを直訳できる
- バイナリサイズは +50MB 程度。release build で 1 分以上。許容範囲
- 起動 overhead が cold で +14ms (Phase 1a の 22ms → Phase 1b の 39ms)。15x speedup は維持
- token の position (`byte_start`) は kuromoji の `word_position` (1-based char) と異なる。upstream rule の port 時に offset 計算を直す必要がある

## 代替案

- vibrato: より高速な形態素解析。辞書 build の手間 (IPADIC を別途用意) があり今回は見送り
- sudachi.rs: 辞書サイズが大きく、kuromoji 互換性で確認すべき項目が多い
- 形態素解析なし (regex のみ): Phase 1b の rule 群が成立しない
