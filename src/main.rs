use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;

use unslop::config::TextlintRc;
use unslop::log::{Mode, RunStats, maybe_write};
use unslop::rule::{Issue, Severity};

#[derive(Parser, Debug)]
#[command(
    name = "unslop",
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

fn print_issue(file: &Path, issue: &Issue) {
    let sev = match issue.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    };
    println!(
        "{}:{}:{}  {}  {}  {}",
        file.display(),
        issue.line,
        issue.column,
        sev,
        issue.message.replace('\n', " "),
        issue.rule_id
    );
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

    for file in &cli.files {
        let src = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[unslop] cannot read {}: {e}", file.display());
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
                eprintln!("[unslop] cannot write {}: {e}", file.display());
                had_io_err = true;
            }
            if applied_count > 0 {
                let verb = if write_back { "fixed" } else { "would fix" };
                eprintln!(
                    "[unslop] {} {} issue(s) in {} ({} pass)",
                    verb,
                    applied_count,
                    file.display(),
                    result.passes
                );
            }
            if result.hit_max_passes {
                eprintln!(
                    "[unslop] warn: fix loop hit MAX_PASSES for {}, residual issues remain",
                    file.display()
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

        for issue in &issues {
            print_issue(file, issue);
        }
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
