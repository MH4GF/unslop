//! Golden fixture test.
//!
//! tests/golden/fixtures/<name>.md を unslop で lint し、tests/golden/expected/<name>.expected.txt
//! (本家 textlint で生成した golden) と diff する。
//!
//! expected を作り直すには `scripts/regen-golden.sh` を実行する。
//! unslop 側の許容差分は tests/golden/known-diffs/<name>.diff として記録する想定。

use std::fs;
use std::path::PathBuf;

use unslop::config::TextlintRc;
use unslop::lint;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/fixtures")
}

fn expected_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/expected")
}

fn textlint_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/textlintrc.json")
}

fn normalize_unslop(issues: &[unslop::rule::Issue]) -> String {
    let lines: std::collections::BTreeSet<String> = issues
        .iter()
        .map(|i| {
            let short = i.rule_id.rsplit('/').next().unwrap_or(&i.rule_id);
            format!("L{} [{}]", i.line, short)
        })
        .collect();
    lines.into_iter().collect::<Vec<_>>().join("\n")
}

struct Coverage {
    common: usize,
    textlint_only: Vec<String>,
    unslop_only: Vec<String>,
}

fn diff(expected: &str, actual: &str) -> Coverage {
    let exp: std::collections::BTreeSet<&str> =
        expected.lines().filter(|l| !l.is_empty()).collect();
    let act: std::collections::BTreeSet<&str> = actual.lines().filter(|l| !l.is_empty()).collect();
    Coverage {
        common: exp.intersection(&act).count(),
        textlint_only: exp.difference(&act).map(|s| s.to_string()).collect(),
        unslop_only: act.difference(&exp).map(|s| s.to_string()).collect(),
    }
}

fn run_fixture(name: &str) -> (Coverage, String, String) {
    let fixture = fixtures_dir().join(format!("{name}.md"));
    let expected_path = expected_dir().join(format!("{name}.expected.txt"));
    let source = fs::read_to_string(&fixture).expect("read fixture");
    let expected = fs::read_to_string(&expected_path).expect("read expected");

    let config_path = textlint_config();
    let base_dir = config_path.parent().unwrap().to_path_buf();
    let rc = TextlintRc::from_path(&config_path).expect("load textlintrc");
    let rules = unslop::build_rules(&rc, &base_dir);
    let issues = lint(&source, &rules);
    let actual = normalize_unslop(&issues);

    let cov = diff(&expected, &actual);
    (cov, expected, actual)
}

fn report(name: &str, cov: &Coverage, expected: &str, actual: &str) -> String {
    let total_expected = expected.lines().filter(|l| !l.is_empty()).count();
    let total_actual = actual.lines().filter(|l| !l.is_empty()).count();
    let mut s = format!(
        "[{name}] textlint={total_expected} unslop={total_actual} common={} textlint-only={} unslop-only={}\n",
        cov.common,
        cov.textlint_only.len(),
        cov.unslop_only.len()
    );
    if !cov.textlint_only.is_empty() {
        s.push_str("  textlint-only (unslop で取りこぼし):\n");
        for l in &cov.textlint_only {
            s.push_str(&format!("    - {l}\n"));
        }
    }
    if !cov.unslop_only.is_empty() {
        s.push_str("  unslop-only (unslop が過剰検出):\n");
        for l in &cov.unslop_only {
            s.push_str(&format!("    + {l}\n"));
        }
    }
    s
}

/// 各 fixture について、textlint との一致率を assert する。
/// floor は「過去より悪化していない」を保証する閾値。下げる修正は意識的に行う。
fn assert_coverage(name: &str, floor_common: usize, ceiling_unslop_only: usize) {
    let (cov, expected, actual) = run_fixture(name);
    let r = report(name, &cov, &expected, &actual);
    eprintln!("{r}");
    assert!(
        cov.common >= floor_common,
        "[{name}] common {} fell below floor {floor_common}\n{r}",
        cov.common,
    );
    assert!(
        cov.unslop_only.len() <= ceiling_unslop_only,
        "[{name}] unslop-only {} exceeded ceiling {ceiling_unslop_only}\n{r}",
        cov.unslop_only.len(),
    );
}

#[test]
fn claude_code_readme() {
    assert_coverage("claude-code-readme", 5, 1);
}

#[test]
fn thermo_nuclear_skill() {
    assert_coverage("thermo-nuclear-skill", 15, 3);
}

#[test]
fn empirical_prompt_tuning_skill() {
    assert_coverage("empirical-prompt-tuning-skill", 16, 18);
}
