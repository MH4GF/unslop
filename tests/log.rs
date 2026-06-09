use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use unslop::log::{Mode, RunStats, format_iso8601_millis, write_line};

fn unique_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "unslop-log-test-{}-{}-{}.jsonl",
        label,
        std::process::id(),
        nanos,
    ))
}

struct TempPath(PathBuf);

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
fn format_iso8601_millis_epoch() {
    assert_eq!(
        format_iso8601_millis(UNIX_EPOCH),
        "1970-01-01T00:00:00.000Z"
    );
}

#[test]
fn format_iso8601_millis_known_points() {
    let one_day = UNIX_EPOCH + Duration::from_secs(86_400);
    assert_eq!(format_iso8601_millis(one_day), "1970-01-02T00:00:00.000Z");

    // 2023-11-14T22:13:20.000Z
    let known = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    assert_eq!(format_iso8601_millis(known), "2023-11-14T22:13:20.000Z");

    // 2026-01-01T00:00:00.000Z (closure-of-year boundary)
    let new_year_2026 = UNIX_EPOCH + Duration::from_secs(1_767_225_600);
    assert_eq!(
        format_iso8601_millis(new_year_2026),
        "2026-01-01T00:00:00.000Z"
    );
}

#[test]
fn format_iso8601_millis_subsec() {
    let t = UNIX_EPOCH + Duration::from_millis(1_700_000_000_123);
    assert_eq!(format_iso8601_millis(t), "2023-11-14T22:13:20.123Z");
}

#[test]
fn mode_strings() {
    assert_eq!(Mode::Lint.as_str(), "lint");
    assert_eq!(Mode::Fix.as_str(), "fix");
    assert_eq!(Mode::FixDryRun.as_str(), "fix-dry-run");
}

#[test]
fn write_line_creates_and_appends() {
    let tmp = TempPath(unique_path("append"));
    let mut fixes = BTreeMap::new();
    fixes.insert("prh".to_string(), 3u64);
    let mut remaining = BTreeMap::new();
    remaining.insert("sentence-length".to_string(), 2u64);

    let stats = RunStats {
        files: vec!["a.md".to_string(), "b.md".to_string()],
        mode: Mode::Fix,
        fixes_applied: 3,
        fixes_per_rule: fixes,
        remaining_issues: 2,
        remaining_per_rule: remaining,
    };

    write_line(&tmp.0, &stats, 1, Duration::from_millis(42), UNIX_EPOCH).expect("first write");
    write_line(&tmp.0, &stats, 0, Duration::from_millis(7), UNIX_EPOCH).expect("second write");

    let body = fs::read_to_string(&tmp.0).expect("read appended file");
    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 lines, got: {body:?}");
    for line in &lines {
        let _: serde_json::Value = serde_json::from_str(line).expect("each line valid JSON");
    }
}

#[test]
fn write_line_keys_and_types() {
    let tmp = TempPath(unique_path("keys"));
    let mut fixes = BTreeMap::new();
    fixes.insert("prh".to_string(), 5u64);
    let stats = RunStats {
        files: vec!["x.md".to_string()],
        mode: Mode::FixDryRun,
        fixes_applied: 5,
        fixes_per_rule: fixes,
        remaining_issues: 0,
        remaining_per_rule: BTreeMap::new(),
    };

    write_line(&tmp.0, &stats, 0, Duration::from_millis(123), UNIX_EPOCH).expect("write");

    let body = fs::read_to_string(&tmp.0).expect("read file");
    let v: serde_json::Value = serde_json::from_str(body.trim_end()).expect("valid JSON");

    assert!(v["ts"].is_string(), "ts should be string");
    let ts = v["ts"].as_str().unwrap();
    assert!(
        ts.ends_with('Z') && ts.contains('T'),
        "ts should be ISO8601 UTC, got {ts}"
    );

    assert_eq!(v["mode"].as_str(), Some("fix-dry-run"));
    assert!(v["files"].is_array());
    assert_eq!(v["files"].as_array().unwrap().len(), 1);

    assert!(v["fixes_per_rule"].is_object());
    assert_eq!(v["fixes_per_rule"]["prh"].as_u64(), Some(5));
    assert!(v["remaining_per_rule"].is_object());

    assert_eq!(v["fixes_applied"].as_u64(), Some(5));
    assert_eq!(v["remaining_issues"].as_u64(), Some(0));
    assert_eq!(v["exit_code"].as_u64(), Some(0));
    assert_eq!(v["duration_ms"].as_u64(), Some(123));
}
