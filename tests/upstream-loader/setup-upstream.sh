#!/usr/bin/env bash
# 各 textlint rule の repo を tests/upstream/ に shallow clone し、npm install まで実行する。
# 既に clone 済みの repo は skip。
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
UP="$ROOT/upstream"
mkdir -p "$UP"

# Phase 1a: regex-only な ja-technical-writing 子 rule
REPOS=(
  "https://github.com/textlint-ja/textlint-rule-max-comma.git"
  "https://github.com/textlint-ja/textlint-rule-max-ten.git"
  "https://github.com/textlint-ja/textlint-rule-ja-no-mixed-period.git"
  "https://github.com/textlint-ja/textlint-rule-no-mix-dearu-desumasu.git"
  "https://github.com/textlint-ja/textlint-rule-no-doubled-conjunction.git"
  "https://github.com/textlint-rule/textlint-rule-no-exclamation-question-mark.git"
  "https://github.com/textlint-ja/textlint-rule-no-hankaku-kana.git"
  "https://github.com/textlint-ja/textlint-rule-no-nfd.git"
  "https://github.com/textlint-rule/textlint-rule-no-zero-width-spaces.git"
  "https://github.com/textlint-rule/textlint-rule-no-invalid-control-character.git"
  "https://github.com/textlint-ja/textlint-rule-ja-unnatural-alphabet.git"
  "https://github.com/textlint-rule/textlint-rule-no-unmatched-pair.git"
  "https://github.com/textlint-rule/textlint-rule-no-dropping-the-ra.git"
  "https://github.com/textlint-ja/textlint-rule-no-double-negative-ja.git"
)

for url in "${REPOS[@]}"; do
  name="${url##*/}"
  name="${name%.git}"
  dest="$UP/$name"
  if [ -d "$dest" ]; then
    echo "[skip] $name (already cloned)"
    continue
  fi
  echo "[clone] $name"
  git clone --depth 1 "$url" "$dest" 2>&1 | tail -1
done

# npm install は extract する直前に個別実行する方が efficient なので別 step。
echo "done. run: tests/upstream-loader/extract-all.sh"
