#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TEXTLINT_REPO="${TEXTLINT_GUARD_REPO:-/Users/mh4gf/ghq/github.com/MH4GF/claude-code}"
TEXTLINT="$TEXTLINT_REPO/node_modules/.bin/textlint"
CONFIG="$ROOT/tests/golden/textlintrc.json"
FIX_DIR="$ROOT/tests/golden/fixtures"
EXP_DIR="$ROOT/tests/golden/expected"

mkdir -p "$EXP_DIR"

normalize() {
  python3 - "$1" <<'PY'
import re, sys, pathlib
src = pathlib.Path(sys.argv[1]).read_text(errors="replace")
# 1. textlint の各エラーは "<line>:<col>  [✓ ]error/warning/info  <message>" で始まり、
#    複数行 message が続き、最後に長い空白を挟んで rule 名で終わる。
# 2. 開始行 (LINE_HEAD) を見つけてエントリ単位にまとめる。
# 3. エントリ内の各行末で `\s{2,}<rule>$` を探し、最初に見つかったものを rule とする。
LINE_HEAD = re.compile(r"\s*\d+:\d+\s+(?:✓\s+)?(error|warning|info)")
TRAILING_RULE = re.compile(r"\s{2,}([\w@/\-\.]+)\s*$")
SUMMARY = re.compile(r"^\s*(✖|✓)")
entries = []
for line in src.splitlines():
    if LINE_HEAD.match(line):
        entries.append([line])
        continue
    if not entries or SUMMARY.match(line) or not line.strip():
        continue
    entries[-1].append(line)
out = []
for parts in entries:
    head = parts[0]
    m_head = re.match(r"\s*(\d+):(\d+)\s+(?:✓\s+)?(error|warning|info)", head)
    if not m_head:
        continue
    ln = m_head.group(1)
    rule = None
    for p in parts:
        m_rule = TRAILING_RULE.search(p)
        if m_rule:
            rule = m_rule.group(1)
            break
    if not rule:
        continue
    short = rule.rsplit("/", 1)[-1]
    out.append(f"L{ln} [{short}]")
out.sort()
print("\n".join(out))
PY
}

for f in "$FIX_DIR"/*.md; do
  name=$(basename "$f" .md)
  raw="$EXP_DIR/${name}.textlint.raw"
  exp="$EXP_DIR/${name}.expected.txt"
  echo "[gen] $name"
  (cd "$TEXTLINT_REPO" && "$TEXTLINT" -c "$CONFIG" --no-color "$f" >"$raw" 2>&1 || true)
  normalize "$raw" >"$exp"
done

echo "done: $(ls "$EXP_DIR"/*.expected.txt | wc -l | tr -d ' ') expected files"
