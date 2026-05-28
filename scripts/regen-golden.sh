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
entries = []
for line in src.splitlines():
    if re.match(r"\s*\d+:\d+\s+(error|warning|info)", line):
        entries.append([line])
    elif entries and not re.match(r"^\s*✖", line) and line.strip():
        entries[-1].append(line)
out = []
for parts in entries:
    joined = " ".join(p.strip() for p in parts)
    m = re.match(r"(\d+):(\d+)\s+(error|warning|info)\s+(.+?)\s{2,}([\w@/\-\.]+)\s*$", joined)
    if not m:
        continue
    ln, col, sev, msg, rule = m.groups()
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
