---
tracker:
  kind: linear
  project_slug: "ai-native-workspace-202646c35423"
  api_key: $LINEAR_API_KEY
  active_states: ["Todo", "In Progress", "Merging", "Rework"]
  terminal_states: ["Human Review", "Done", "Canceled", "Duplicate"]
  required_labels: ["unslop"]

workspace:
  root: /Users/hermes/.symphony/workspaces/unslop

hooks:
  after_create: |
    set -eu
    git clone --depth 1 git@github.com:MH4GF/unslop.git .

agent:
  max_concurrent_agents: 2
  max_turns: 20

codex:
  command: /Users/hermes/.local/bin/safe-claude
  claude_args: ["--permission-mode", "bypassPermissions"]
  stall_timeout_ms: 600000
  turn_timeout_ms: 1800000
---

MH4GF/unslop (Rust 製 textlint 互換 Japanese writing linter) の clone で作業する。repo 構造とビルド・テスト手順は root の `CLAUDE.md` を起点に把握する。

## Issue

{{ issue.identifier }} - {{ issue.title }}

## Body

{{ issue.description }}

{% if attempt %}
## Continuation context

- これはリトライ attempt #{{ attempt }}。チケットが active state のため再ディスパッチされた。
- 最初からやり直さず、現在の workspace 状態から resume する。
- 完了済みの調査や validation を繰り返さない。新規コード変更で必要な場合は除く。
- issue が active state の間は turn を終わらせない。required permissions/secrets が missing で blocked の場合は除く。
{% endif %}

## Workflow protocol

- session 起動直後に `symphony-workflow` skill を呼ぶ。status routing / workpad 運用 / 実装 / sweep / `Human Review` 遷移 / land まで、進行は全て同 skill の手順に従う
- `symphony-workflow` skill が利用できない環境では実装に入らない。Linear issue に blocker comment (何が missing か / unblock に必要な人間の action) を 1 件書き、issue を `Human Review` へ動かして shutdown する
- 下の「本 repo 固有ルール」は skill の共通手順を上書きする

## 本 repo 固有ルール

なし。skill の共通手順に従う。

## スコープ外

issue が次のいずれかを含むなら、止まってユーザーに label 修正を依頼する。

- vault 内容の編集 (`MH4GF/works`)
- `MH4GF/claude-code` の編集 (claude-code workflow の管轄)
- Symphony orchestrator のコード (`MH4GF/symphony`)
- `~/.claude/hooks/*.sh` の直接編集 (本 repo の `CLAUDE.md` 禁止事項。auto mode が self-modification として拒否する)
