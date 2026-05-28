use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use unslop::config::TextlintRc;
use unslop::rule::Severity;

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

    files: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let config_path = cli
        .config
        .unwrap_or_else(|| PathBuf::from(".textlintrc.json"));
    let base_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let rc = match TextlintRc::from_path(&config_path) {
        Ok(rc) => rc,
        Err(e) => {
            eprintln!("[unslop] cannot load config {}: {e}", config_path.display());
            return ExitCode::from(2);
        }
    };

    let rules = unslop::build_rules(&rc, &base_dir);

    let mut had_error = false;
    for file in &cli.files {
        let src = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[unslop] cannot read {}: {e}", file.display());
                had_error = true;
                continue;
            }
        };
        let issues = unslop::lint(&src, &rules);
        for issue in &issues {
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
        if !issues.is_empty() {
            had_error = true;
        }
    }

    if had_error {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
