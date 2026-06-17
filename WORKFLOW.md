---
tracker:
  kind: linear
  project_slug: "ai-native-workspace-202646c35423"
  api_key: $LINEAR_API_KEY
  active_states: ["Todo"]
  terminal_states: ["In Review", "Done", "Canceled", "Duplicate"]
  required_labels: ["unslop"]

workspace:
  root: /Users/mh4gf/.symphony/workspaces/unslop

hooks:
  after_create: |
    set -eu
    git clone --depth 1 git@github.com:MH4GF/unslop.git .

agent:
  max_concurrent_agents: 2
  max_turns: 6

codex:
  command: claude
  claude_args: ["--permission-mode", "auto"]
  stall_timeout_ms: 600000
  turn_timeout_ms: 1800000
---

MH4GF/unslop (Rust 製 textlint 互換 Japanese writing linter) の clone で作業する。repo 構造とビルド・テスト手順の把握は root の `CLAUDE.md` を起点にする。

## Issue

{{ issue.identifier }} - {{ issue.title }}

## Body

{{ issue.description }}

## Identifier ルール

`{{ issue.identifier }}` を branch 名と PR body にそのまま埋め込む。Linear の GitHub linking は identifier 完全一致で動くため、URL slug や title から推論した別形を書かない。

## PR ルール

- `main` 直接 push 禁止。必ず `gh pr create` で PR を出す
- PR body 冒頭に `Closes {{ issue.identifier }}` を独立行で必須記載。末尾に {{ issue.url }} を併記
- issue が曖昧 (acceptance criteria が不明) なら、PR body に plan と質問を書いた draft PR を開いて止まる

## スコープ外

issue が次のいずれかを含むなら、止まって ユーザー に label 修正を依頼する。

- vault 内容の編集 (`MH4GF/works`)
- `MH4GF/claude-code` の編集 (claude-code workflow の管轄)
- Symphony orchestrator のコード (`MH4GF/symphony`)
- `~/.claude/hooks/*.sh` の直接編集 (本 repo の `CLAUDE.md` 禁止事項。auto mode が self-modification として拒否する)
