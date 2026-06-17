use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;

use unslop::config::TextlintRc;
use unslop::log::{Mode, RunStats, maybe_write};
use unslop::rule::Issue;

#[derive(Parser, Debug)]
#[command(
    name = "unslop",
    version,
    about = "Fast textlint-compatible Japanese writing linter"
)]
struct Cli {
    #[arg(short = 'c', long = "config")]
    config: Option<PathBuf>,

    #[arg(long = "no-color", default_value_t = false)]
    no_color: bool,

    /// 検出した違反のうち auto-fix 可能なものをファイルに書き戻す
    #[arg(long = "fix", default_value_t = false)]
    fix: bool,

    /// fix simulation のみ実行する (ファイルへの書き戻しは行わない)
    #[arg(long = "fix-dry-run", default_value_t = false)]
    fix_dry_run: bool,

    files: Vec<PathBuf>,
}

fn display_path(path: &Path, cwd: Option<&Path>) -> String {
    if let Some(cwd) = cwd
        && let Ok(rel) = path.strip_prefix(cwd)
        && !rel.as_os_str().is_empty()
    {
        return rel.display().to_string();
    }
    path.display().to_string()
}

fn short_rule_id(rule_id: &str) -> &str {
    rule_id.rsplit('/').next().unwrap_or(rule_id)
}

/// (rule_id, message, [(line, column)]) の指摘グループ。
type IssueGroup<'a> = (&'a str, &'a str, Vec<(usize, usize)>);

/// AI agent 向けのトークン効率出力。ヘッダにパスを 1 回だけ出し、
/// 同一 (rule_id, message) の指摘は位置をカンマ区切りでまとめる。
fn format_file_issues(display_path: &str, issues: &[Issue]) -> String {
    if issues.is_empty() {
        return String::new();
    }
    let mut groups: Vec<IssueGroup> = Vec::new();
    for issue in issues {
        let loc = (issue.line, issue.column);
        match groups
            .iter_mut()
            .find(|(r, m, _)| *r == issue.rule_id && *m == issue.message)
        {
            Some((_, _, locs)) => {
                if locs.last() != Some(&loc) {
                    locs.push(loc);
                }
            }
            None => groups.push((&issue.rule_id, &issue.message, vec![loc])),
        }
    }
    let mut out = format!("{display_path}\n");
    for (rule_id, message, locs) in groups {
        let locs: Vec<String> = locs.iter().map(|(l, c)| format!("{l}:{c}")).collect();
        out.push_str(&format!(
            "{} {} {}\n",
            locs.join(","),
            short_rule_id(rule_id),
            message.replace('\n', " ")
        ));
    }
    out
}

fn main() -> ExitCode {
    let started = Instant::now();
    let cli = Cli::parse();
    let config_path = cli
        .config
        .unwrap_or_else(|| PathBuf::from(".textlintrc.json"));
    let base_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let files: Vec<String> = cli.files.iter().map(|p| p.display().to_string()).collect();

    let rc = match TextlintRc::from_path(&config_path) {
        Ok(rc) => rc,
        Err(e) => {
            eprintln!("[unslop] cannot load config {}: {e}", config_path.display());
            let stats = RunStats {
                files,
                mode: Mode::Lint,
                ..Default::default()
            };
            maybe_write(&stats, 2, started.elapsed());
            return ExitCode::from(2);
        }
    };

    let rules = unslop::build_rules(&rc, &base_dir);

    let mut stats = RunStats {
        files,
        mode: if cli.fix_dry_run {
            Mode::FixDryRun
        } else if cli.fix {
            Mode::Fix
        } else {
            Mode::Lint
        },
        ..Default::default()
    };

    let mut had_lint_err = false;
    let mut had_io_err = false;
    let fix_mode = cli.fix || cli.fix_dry_run;
    let write_back = cli.fix && !cli.fix_dry_run;
    let cwd = std::env::current_dir().ok();

    for file in &cli.files {
        let shown = display_path(file, cwd.as_deref());
        let src = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[unslop] cannot read {shown}: {e}");
                had_io_err = true;
                continue;
            }
        };

        let issues = if fix_mode {
            let result = unslop::fix(&src, &rules);
            let applied_count = result.applied_fixes.len();
            if write_back
                && result.fixed_source != src
                && let Err(e) = std::fs::write(file, &result.fixed_source)
            {
                eprintln!("[unslop] cannot write {shown}: {e}");
                had_io_err = true;
            }
            if applied_count > 0 {
                let verb = if write_back { "fixed" } else { "would fix" };
                eprintln!(
                    "[unslop] {} {} issue(s) in {} ({} pass)",
                    verb, applied_count, shown, result.passes
                );
            }
            if result.hit_max_passes {
                eprintln!(
                    "[unslop] warn: fix loop hit MAX_PASSES for {shown}, residual issues remain"
                );
            }
            stats.fixes_applied += applied_count as u64;
            for af in &result.applied_fixes {
                *stats.fixes_per_rule.entry(af.rule_id.clone()).or_default() += 1;
            }
            result.remaining_issues
        } else {
            unslop::lint(&src, &rules)
        };

        print!("{}", format_file_issues(&shown, &issues));
        stats.remaining_issues += issues.len() as u64;
        for i in &issues {
            *stats
                .remaining_per_rule
                .entry(i.rule_id.clone())
                .or_default() += 1;
        }
        if !issues.is_empty() {
            had_lint_err = true;
        }
    }

    let exit = if had_io_err {
        2u8
    } else if had_lint_err {
        1u8
    } else {
        0u8
    };
    maybe_write(&stats, exit, started.elapsed());
    ExitCode::from(exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use unslop::rule::Severity;

    fn iss(rule: &str, msg: &str, line: usize, column: usize) -> Issue {
        Issue::new(rule, msg, line, column, Severity::Error)
    }

    #[test]
    fn groups_identical_rule_and_message() {
        let issues = vec![
            iss("prh", "worker => ワーカー", 1, 2),
            iss("prh", "worker => ワーカー", 3, 4),
            iss("prh", "worker => ワーカー", 10, 1),
        ];
        assert_eq!(
            format_file_issues("a.md", &issues),
            "a.md\n1:2,3:4,10:1 prh worker => ワーカー\n"
        );
    }

    #[test]
    fn dedupes_repeated_identical_location() {
        let issues = vec![
            iss("prh", "trigger => トリガー", 23, 36),
            iss("prh", "trigger => トリガー", 23, 36),
        ];
        assert_eq!(
            format_file_issues("a.md", &issues),
            "a.md\n23:36 prh trigger => トリガー\n"
        );
    }

    #[test]
    fn same_rule_different_message_not_merged() {
        let issues = vec![
            iss("prh", "worker => ワーカー", 1, 1),
            iss("prh", "commit => コミット", 2, 1),
        ];
        assert_eq!(
            format_file_issues("a.md", &issues),
            "a.md\n1:1 prh worker => ワーカー\n2:1 prh commit => コミット\n"
        );
    }

    #[test]
    fn different_rule_same_message_not_merged() {
        let issues = vec![
            iss("rule-a", "同じ文言", 1, 1),
            iss("rule-b", "同じ文言", 2, 1),
        ];
        assert_eq!(
            format_file_issues("a.md", &issues),
            "a.md\n1:1 rule-a 同じ文言\n2:1 rule-b 同じ文言\n"
        );
    }

    #[test]
    fn groups_ordered_by_first_occurrence() {
        let issues = vec![
            iss("rule-a", "メッセージA", 1, 1),
            iss("rule-b", "メッセージB", 2, 1),
            iss("rule-a", "メッセージA", 5, 1),
        ];
        assert_eq!(
            format_file_issues("a.md", &issues),
            "a.md\n1:1,5:1 rule-a メッセージA\n2:1 rule-b メッセージB\n"
        );
    }

    #[test]
    fn shortens_preset_rule_id() {
        let issues = vec![iss(
            "@textlint-ja/preset-ai-writing/no-ai-list-formatting",
            "msg",
            9,
            1,
        )];
        assert_eq!(
            format_file_issues("a.md", &issues),
            "a.md\n9:1 no-ai-list-formatting msg\n"
        );
    }

    #[test]
    fn replaces_newline_in_message() {
        let issues = vec![iss("r", "前半\n後半", 1, 1)];
        assert_eq!(
            format_file_issues("a.md", &issues),
            "a.md\n1:1 r 前半 後半\n"
        );
    }

    #[test]
    fn empty_issues_returns_empty_string() {
        assert_eq!(format_file_issues("a.md", &[]), "");
    }

    #[test]
    fn display_path_relativizes_under_cwd() {
        assert_eq!(
            display_path(Path::new("/proj/docs/a.md"), Some(Path::new("/proj"))),
            "docs/a.md"
        );
    }

    #[test]
    fn display_path_keeps_path_outside_cwd() {
        assert_eq!(
            display_path(Path::new("/tmp/x.md"), Some(Path::new("/proj"))),
            "/tmp/x.md"
        );
    }

    #[test]
    fn display_path_keeps_path_without_cwd() {
        assert_eq!(display_path(Path::new("/proj/a.md"), None), "/proj/a.md");
    }

    #[test]
    fn display_path_keeps_relative_input() {
        assert_eq!(
            display_path(Path::new("docs/a.md"), Some(Path::new("/proj"))),
            "docs/a.md"
        );
    }
}
