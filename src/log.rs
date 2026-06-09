//! `UNSLOP_LOG=<path>` で opt-in する構造化ログ機構。1 起動 1 行 JSON Lines。

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Lint,
    Fix,
    FixDryRun,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Lint => "lint",
            Mode::Fix => "fix",
            Mode::FixDryRun => "fix-dry-run",
        }
    }
}

#[derive(Debug, Default)]
pub struct RunStats {
    pub files: Vec<String>,
    pub mode: Mode,
    pub fixes_applied: u64,
    pub fixes_per_rule: BTreeMap<String, u64>,
    pub remaining_issues: u64,
    pub remaining_per_rule: BTreeMap<String, u64>,
}

const ENV_KEY: &str = "UNSLOP_LOG";

/// env `UNSLOP_LOG` が空でなく path として有効なら 1 行 append する。
/// それ以外は no-op。失敗時は stderr に warn を 1 行出すのみ。
pub fn maybe_write(stats: &RunStats, exit_code: u8, duration: Duration) {
    let Ok(path) = std::env::var(ENV_KEY) else {
        return;
    };
    if path.is_empty() {
        return;
    }
    if let Err(e) = write_line(
        Path::new(&path),
        stats,
        exit_code,
        duration,
        SystemTime::now(),
    ) {
        eprintln!("[unslop] failed to write UNSLOP_LOG to {path}: {e}");
    }
}

/// `maybe_write` の本体。SystemTime と path を注入できる。
#[doc(hidden)]
pub fn write_line(
    path: &Path,
    stats: &RunStats,
    exit_code: u8,
    duration: Duration,
    now: SystemTime,
) -> std::io::Result<()> {
    let ts = format_iso8601_millis(now);
    let value = serde_json::json!({
        "ts": ts,
        "files": stats.files,
        "mode": stats.mode.as_str(),
        "fixes_applied": stats.fixes_applied,
        "fixes_per_rule": stats.fixes_per_rule,
        "remaining_issues": stats.remaining_issues,
        "remaining_per_rule": stats.remaining_per_rule,
        "exit_code": exit_code,
        "duration_ms": u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
    });
    let mut line = serde_json::to_vec(&value)?;
    line.push(b'\n');
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    // 1 回の write_all で 4KB 以下のレコードは POSIX O_APPEND の保証で atomic
    f.write_all(&line)
}

/// `SystemTime` を `YYYY-MM-DDTHH:MM:SS.mmmZ` (UTC) に整形する。
#[doc(hidden)]
pub fn format_iso8601_millis(t: SystemTime) -> String {
    let d = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = d.as_secs() as i64;
    let millis = d.subsec_millis();
    let days = total_secs.div_euclid(86_400);
    let secs_of_day = total_secs.rem_euclid(86_400);
    let h = secs_of_day / 3600;
    let m = (secs_of_day / 60) % 60;
    let s = secs_of_day % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}.{millis:03}Z")
}

// Howard Hinnant の date algorithms: http://howardhinnant.github.io/date_algorithms.html
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m_raw = if mp < 10 { mp + 3 } else { mp - 9 };
    let m = m_raw as u32;
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m, d)
}
