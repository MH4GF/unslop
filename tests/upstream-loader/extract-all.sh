#!/usr/bin/env bash
# 各 rule repo を `npm install` (必要なら) → tests/upstream-loader/extract.cjs で
# tests/cases/jt/<rule>.json を生成する。
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
UP="$ROOT/upstream"
OUT_DIR="$ROOT/cases/jt"
mkdir -p "$OUT_DIR"
EXTRACT="$ROOT/upstream-loader/extract.cjs"

# repo name : path-to-test-file (test ファイルが複数あれば一番典型的なもの)
declare -a JOBS=(
  "textlint-rule-no-zero-width-spaces:test/index-test.ts"
  "textlint-rule-no-hankaku-kana:test/textlint-rule-no-hankaku-kana-test.ts"
  "textlint-rule-no-nfd:test/textlint-rule-no-nfd-test.ts"
  "textlint-rule-no-invalid-control-character:test/textlint-rule-no-invalid-control-character-test.js"
  "textlint-rule-no-exclamation-question-mark:test/textlint-rule-no-exclamation-question-mark-test.js"
  "textlint-rule-ja-no-mixed-period:test/textlint-rule-ja-no-mixed-period-test.ts"
  "textlint-rule-no-double-negative-ja:test/no-doubled-negative-ja-test.js"
  "textlint-rule-no-mix-dearu-desumasu:test/no-mix-dearu-desumasu-test.js"
  "textlint-rule-no-doubled-conjunction:test/no-doubled-conjunction.js"
  "textlint-rule-max-ten:test/max-ten-test.js"
  "textlint-rule-ja-unnatural-alphabet:test/textlint-rule-ja-unnatural-alphabet-test.js"
  "textlint-rule-no-unmatched-pair:test/textlint-rule-no-unmatched-pair-test.js"
)

for job in "${JOBS[@]}"; do
  repo="${job%%:*}"
  test_rel="${job##*:}"
  dir="$UP/$repo"
  test_file="$dir/$test_rel"
  rule_name="${repo#textlint-rule-}"
  out="$OUT_DIR/${rule_name}.json"

  if [ ! -d "$dir" ]; then
    echo "[skip] $repo (not cloned)"
    continue
  fi
  if [ ! -f "$test_file" ]; then
    # fallback: 先頭の test/*.{ts,js} を 1 つ使う
    test_file=$(find "$dir/test" -maxdepth 2 -name '*-test.ts' -o -name '*-test.js' -o -name '*.test.ts' -o -name '*.test.js' -o -name 'test.ts' -o -name 'test.js' 2>/dev/null | head -1)
    if [ -z "$test_file" ]; then
      echo "[skip] $repo (no test file)"
      continue
    fi
  fi

  if [ ! -d "$dir/node_modules" ]; then
    echo "[install] $repo"
    (cd "$dir" && npm install --no-audit --ignore-scripts --silent 2>&1 | tail -1)
  fi

  echo "[extract] $repo → $out"
  (cd "$dir" && UNSLOP_CASES_OUT="$out" node "$EXTRACT" "$test_file" 2>&1 | grep -E "hijack|Error" | head -3)
done

echo "done. cases:"
ls -1 "$OUT_DIR"
