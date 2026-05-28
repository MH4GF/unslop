#!/usr/bin/env bash
set -uo pipefail

# textlint bottleneck profiler.
#
# Modes:
#   baseline       全 rule on で empty/small/medium/large を計測
#   per-rule       medium fixture で各 rule を 1 つずつ off にして差分を取る
#   all            上記すべて
#
# 1 計測あたり 6 回実行し、warmup の 1 回目を捨て後半 5 回の median を返す。
# 結果は bench/results/<timestamp>-<mode>.tsv に追記。

ROOT="$(cd "$(dirname "$0")" && pwd)"
REPO="${TEXTLINT_REPO:-/Users/mh4gf/ghq/github.com/MH4GF/claude-code}"
BIN="$REPO/node_modules/.bin/textlint"
BASE_CONFIG="$REPO/.textlintrc.json"
FIXTURES_DIR="$ROOT/fixtures"
CONFIGS_DIR="$ROOT/configs"
RESULTS_DIR="$ROOT/results"
RUNS=6
DROP_WARMUP=1

mode="${1:-all}"
ts="$(date +%Y%m%d-%H%M%S)"
UNSLOP="$ROOT/../target/release/unslop"

mkdir -p "$CONFIGS_DIR" "$RESULTS_DIR"

if [ ! -x "$BIN" ]; then
  echo "textlint not found at $BIN" >&2
  exit 1
fi
if [ ! -f "$BASE_CONFIG" ]; then
  echo "base config not found at $BASE_CONFIG" >&2
  exit 1
fi

# ms 単位の wall-clock を返す。python3 を使うのは date +%N が macOS で使えないため。
now_ms() {
  python3 -c 'import time; print(int(time.time()*1000))'
}

# median_ms <ms ms ms ...> → median 整数
median_ms() {
  python3 -c '
import sys
vals = sorted(int(x) for x in sys.argv[1:])
n = len(vals)
print(vals[n//2] if n % 2 else (vals[n//2-1] + vals[n//2]) // 2)
' "$@"
}

# run_once <config> <fixture> → ms
run_once() {
  local config="$1" fixture="$2" start end
  start=$(now_ms)
  "$BIN" -c "$config" --no-color "$fixture" >/dev/null 2>&1 || true
  end=$(now_ms)
  echo $((end - start))
}

# bench <label> <config> <fixture> → median ms (TSV 1 行を stdout に出力)
bench() {
  local label="$1" config="$2" fixture="$3"
  local samples=()
  local i=0
  while [ "$i" -lt "$RUNS" ]; do
    samples+=("$(run_once "$config" "$fixture")")
    i=$((i + 1))
  done
  # 先頭 DROP_WARMUP 件を捨てる
  local kept=("${samples[@]:$DROP_WARMUP}")
  local med
  med=$(median_ms "${kept[@]}")
  printf '%s\t%s\t%s\t%s\t%s\n' "$label" "$(basename "$fixture")" "$med" "${kept[*]}" "${samples[*]}"
}

run_baseline() {
  local out="$RESULTS_DIR/${ts}-baseline.tsv"
  {
    echo "# baseline: all rules on"
    echo "# label\tfixture\tmedian_ms\tkept_samples\tall_samples"
    for f in empty small medium large; do
      bench "all-rules-on" "$BASE_CONFIG" "$FIXTURES_DIR/$f.md"
    done
  } | tee "$out"
  echo "→ $out" >&2
}

run_unslop() {
  local out="$RESULTS_DIR/${ts}-unslop.tsv"
  if [ ! -x "$UNSLOP" ]; then
    echo "unslop binary not found at $UNSLOP. cargo build --release first." >&2
    return 1
  fi
  local saved_bin="$BIN"
  BIN="$UNSLOP"
  {
    echo "# unslop release build"
    echo "# label\tfixture\tmedian_ms\tkept_samples\tall_samples"
    for f in empty small medium large; do
      bench "unslop" "$BASE_CONFIG" "$FIXTURES_DIR/$f.md"
    done
  } | tee "$out"
  BIN="$saved_bin"
  echo "→ $out" >&2
}

# .textlintrc.json から rule 名一覧を抽出
list_rules() {
  python3 - "$BASE_CONFIG" <<'PY'
import json, sys
cfg = json.load(open(sys.argv[1]))
rules = cfg.get("rules", {})
for name, val in rules.items():
    if name.startswith("preset-") or name.startswith("@textlint-ja/preset-"):
        # preset 配下の子 rule を node_modules から推測
        # ここでは preset 単位 off の差分を取る方針 (子 rule 名は別途上書きが必要)
        print(f"PRESET:{name}")
    else:
        print(f"RULE:{name}")
PY
}

# 子 rule (preset-ja-technical-writing 配下) を 1 つずつ off にした config を生成
# preset-ai-writing 配下も同様に扱う
# 相対パスを base config からの絶対パスに直した中間 config を 1 度だけ作る
BASE_CONFIG_ABS="$CONFIGS_DIR/_base.json"
build_base_abs() {
  python3 - "$BASE_CONFIG" "$BASE_CONFIG_ABS" <<'PY'
import json, os, sys
src, out = sys.argv[1], sys.argv[2]
base_dir = os.path.dirname(os.path.abspath(src))
cfg = json.load(open(src))
prh = cfg.get("rules", {}).get("prh")
if isinstance(prh, dict) and "rulePaths" in prh:
    prh["rulePaths"] = [
        p if os.path.isabs(p) else os.path.normpath(os.path.join(base_dir, p))
        for p in prh["rulePaths"]
    ]
json.dump(cfg, open(out, "w"), ensure_ascii=False, indent=2)
PY
}

gen_off_config() {
  local preset="$1" child="$2" out="$3"
  python3 - "$BASE_CONFIG_ABS" "$preset" "$child" "$out" <<'PY'
import json, sys
src, preset, child, out = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
cfg = json.load(open(src))
rules = cfg.setdefault("rules", {})
node = rules.get(preset)
if not isinstance(node, dict):
    node = {}
node[child] = False
rules[preset] = node
json.dump(cfg, open(out, "w"), ensure_ascii=False, indent=2)
PY
}

gen_off_root() {
  local name="$1" out="$2"
  python3 - "$BASE_CONFIG_ABS" "$name" "$out" <<'PY'
import json, sys
src, name, out = sys.argv[1], sys.argv[2], sys.argv[3]
cfg = json.load(open(src))
cfg.setdefault("rules", {})[name] = False
json.dump(cfg, open(out, "w"), ensure_ascii=False, indent=2)
PY
}

run_per_rule() {
  build_base_abs
  local out="$RESULTS_DIR/${ts}-per-rule.tsv"
  local fixture="$FIXTURES_DIR/medium.md"
  {
    echo "# per-rule off delta on $(basename "$fixture")"
    echo "# label\tfixture\tmedian_ms\tkept_samples\tall_samples"
    # baseline 用に同じ run でもう一度 all-on を取る (この run 内の比較基準)
    bench "all-rules-on" "$BASE_CONFIG_ABS" "$fixture"

    # preset-ja-technical-writing の子 rule
    local jt_rules=(
      sentence-length max-comma max-ten max-kanji-continuous-len
      no-mix-dearu-desumasu ja-no-mixed-period
      no-doubled-conjunction no-doubled-conjunctive-particle-ga
      no-double-negative-ja no-doubled-joshi no-dropping-the-ra
      no-nfd no-exclamation-question-mark no-hankaku-kana
      no-invalid-control-character ja-no-weak-phrase
      ja-no-successive-word ja-no-abusage ja-no-redundant-expression
      ja-unnatural-alphabet no-unmatched-pair no-zero-width-spaces
    )
    for r in "${jt_rules[@]}"; do
      local cfg="$CONFIGS_DIR/off-jt-$r.json"
      gen_off_config "preset-ja-technical-writing" "$r" "$cfg"
      bench "off:jt/$r" "$cfg" "$fixture"
    done

    # preset-ai-writing の子 rule
    local ai_rules=(
      no-ai-list-formatting no-ai-hype-expressions
      ai-tech-writing-guideline no-filler-phrases
      no-ai-emphasis-patterns no-ai-colon-continuation
    )
    for r in "${ai_rules[@]}"; do
      local cfg="$CONFIGS_DIR/off-ai-$r.json"
      gen_off_config "@textlint-ja/preset-ai-writing" "$r" "$cfg"
      bench "off:ai/$r" "$cfg" "$fixture"
    done

    # prh は単独 rule
    local cfg="$CONFIGS_DIR/off-prh.json"
    gen_off_root "prh" "$cfg"
    bench "off:prh" "$cfg" "$fixture"

    # preset 丸ごと off も比較用に
    local cfg2="$CONFIGS_DIR/off-preset-jt.json"
    gen_off_root "preset-ja-technical-writing" "$cfg2"
    bench "off:preset/jt" "$cfg2" "$fixture"
    local cfg3="$CONFIGS_DIR/off-preset-ai.json"
    gen_off_root "@textlint-ja/preset-ai-writing" "$cfg3"
    bench "off:preset/ai" "$cfg3" "$fixture"
  } | tee "$out"
  echo "→ $out" >&2
}

case "$mode" in
  baseline) run_baseline ;;
  per-rule) run_per_rule ;;
  unslop) run_unslop ;;
  compare) run_baseline; run_unslop ;;
  all) run_baseline; run_per_rule; run_unslop ;;
  *) echo "unknown mode: $mode" >&2; exit 1 ;;
esac
