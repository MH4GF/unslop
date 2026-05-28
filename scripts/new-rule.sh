#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<EOF
usage: scripts/new-rule.sh <group> <rule-name>

  group     ai_writing | jt
  rule-name kebab-case (例: no-doubled-joshi)

例:
  scripts/new-rule.sh jt no-foo-bar

scaffold:
  src/rules/<group>/<snake_name>.rs を生成
  src/rules/<group>/mod.rs に pub mod を追記
  src/lib.rs に登録のヒントコメントを表示
  tests/golden/ への fixture 追加と floor 更新の手順を表示
EOF
  exit 1
}

[ $# -eq 2 ] || usage
group="$1"
rule_kebab="$2"

case "$group" in
  ai_writing|jt) ;;
  *) echo "error: group must be ai_writing or jt" >&2; exit 1 ;;
esac

rule_snake="${rule_kebab//-/_}"
type_name="$(echo "$rule_kebab" | awk -F'-' '{for(i=1;i<=NF;i++){printf toupper(substr($i,1,1)) substr($i,2)}}')"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
rule_file="$ROOT/src/rules/$group/${rule_snake}.rs"
mod_file="$ROOT/src/rules/$group/mod.rs"

if [ -e "$rule_file" ]; then
  echo "error: $rule_file already exists" >&2
  exit 1
fi

rule_id="$rule_kebab"
if [ "$group" = "ai_writing" ]; then
  rule_id="@textlint-ja/preset-ai-writing/$rule_kebab"
fi

cat >"$rule_file" <<EOF
//! $rule_id
//!
//! TODO: upstream src と挙動を要確認。
//! upstream: tests/upstream/textlint-rule-$rule_kebab/

use crate::document::Document;
use crate::rule::{Issue, Rule, Severity};
use crate::rules::is_str_bearing;

const RULE_ID: &str = "$rule_id";

pub struct $type_name;

impl Rule for $type_name {
    fn id(&self) -> &str {
        RULE_ID
    }

    fn check(&self, doc: &Document) -> Vec<Issue> {
        let mut issues = Vec::new();
        for seg in &doc.segments {
            if !is_str_bearing(seg.kind) {
                continue;
            }
            // TODO: implement
            let _ = seg;
        }
        issues
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn _silence_unused() {
    let _ = Severity::Error;
}
EOF

if ! grep -q "pub mod ${rule_snake};" "$mod_file"; then
  echo "pub mod ${rule_snake};" >>"$mod_file"
  sort -o "$mod_file" "$mod_file"
fi

cat <<EOF
[done] scaffold:
  $rule_file
  $mod_file (pub mod ${rule_snake};)

次の手順:

1. upstream を clone (まだなら)
   git clone --depth 1 https://github.com/textlint-XX/textlint-rule-$rule_kebab \\
     tests/upstream/textlint-rule-$rule_kebab

2. upstream src を読み、Rust 実装を埋める ($rule_file)

3. src/lib.rs::build_rules に登録:
   if rc.preset_child_enabled("<preset>", "$rule_kebab") {
       out.push(Box::new(rules::$group::${rule_snake}::${type_name}));
   }

4. golden fixture を追加し floor を上げる:
   - tests/golden/fixtures/ にこの rule が発火する md を 1 個追加
   - scripts/regen-golden.sh で expected を再生成
   - cargo test --test golden で実態の floor / ceiling を確認
   - tests/golden.rs の assert_coverage(...) 引数を更新

5. (可能なら) compat test を追加:
   - upstream の test を tests/upstream-loader/extract-all.sh に追記
   - tests/cases/<group>/$rule_kebab.json を再生成
   - tests/compat.rs にテストを追加

6. docs/RULES.md に rule を追記
EOF
