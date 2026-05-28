#!/usr/bin/env bash
# unslop の medium fixture を 6 回計測し、warmup を除いた 5 回の median を出す。
# UNSLOP_SMOKE_MAX_MS (default 300) を超えたら exit 1。
# CI と手元の両方で動作する想定。CI runner は手元より遅いので threshold は緩め。

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
UNSLOP="${UNSLOP_BIN:-$ROOT/target/release/unslop}"
CONFIG="$ROOT/tests/golden/textlintrc.json"
FIXTURE="$ROOT/bench/fixtures/medium.md"
MAX_MS="${UNSLOP_SMOKE_MAX_MS:-300}"

if [ ! -x "$UNSLOP" ]; then
  echo "unslop binary not found at $UNSLOP. build with 'cargo build --release' first." >&2
  exit 1
fi

samples=()
for _ in 1 2 3 4 5 6; do
  start=$(python3 -c 'import time; print(int(time.time()*1000))')
  "$UNSLOP" -c "$CONFIG" "$FIXTURE" >/dev/null 2>&1 || true
  end=$(python3 -c 'import time; print(int(time.time()*1000))')
  samples+=("$((end - start))")
done

kept=("${samples[@]:1}")
median=$(printf '%s\n' "${kept[@]}" | sort -n | awk 'NR==3{print}')

echo "samples: ${samples[*]}  kept: ${kept[*]}  median: ${median}ms (threshold: ${MAX_MS}ms)"

if [ "$median" -gt "$MAX_MS" ]; then
  echo "FAIL: median ${median}ms exceeded threshold ${MAX_MS}ms" >&2
  exit 1
fi
