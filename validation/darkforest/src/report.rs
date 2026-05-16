use crate::check::{CheckResult, Status};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
pub struct Report {
    pub darkforest_version: String,
    pub timestamp: String,
    pub host: String,
    pub suite: String,
    pub duration_ms: u64,
    pub summary: Summary,
    pub checks: Vec<CheckResult>,
}

#[derive(Serialize)]
pub struct Summary {
    pub total: u32,
    pub pass: u32,
    pub fail: u32,
    pub known_gap: u32,
    pub dark_forest: u32,
}

impl Report {
    pub fn build(
        checks: Vec<CheckResult>,
        host: &str,
        suite: &str,
        start_ts: &str,
        duration_ms: u64,
    ) -> Self {
        let mut pass = 0u32;
        let mut fail = 0u32;
        let mut gap = 0u32;
        let mut df = 0u32;
        for c in &checks {
            match c.status {
                Status::Pass => pass += 1,
                Status::Fail => fail += 1,
                Status::KnownGap => gap += 1,
                Status::DarkForest => df += 1,
            }
        }
        Self {
            darkforest_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: start_ts.to_string(),
            host: host.to_string(),
            suite: suite.to_string(),
            duration_ms,
            summary: Summary {
                total: pass + fail + gap + df,
                pass,
                fail,
                known_gap: gap,
                dark_forest: df,
            },
            checks,
        }
    }
}

pub fn print_pipe(checks: &[CheckResult]) {
    for c in checks {
        println!("{}|{}|{}|{}", c.pipe_tag(), c.suite, c.id, c.title);
    }
}

pub fn print_banner(suite: &str, ts: &str) {
    println!("═══════════════════════════════════════════════════");
    println!(
        "  Dark Forest — Pure Rust Security Validator v{}",
        env!("CARGO_PKG_VERSION")
    );
    println!("  Date: {ts}");
    println!("  Suite: {suite}");
    println!("═══════════════════════════════════════════════════");
}

pub fn print_summary(report: &Report) {
    let s = &report.summary;
    println!();
    println!("═══════════════════════════════════════════════════");
    println!(
        "  Results: {} PASS, {} FAIL, {} KNOWN_GAP, {} DARK_FOREST",
        s.pass, s.fail, s.known_gap, s.dark_forest
    );
    println!("═══════════════════════════════════════════════════");

    if s.fail > 0 {
        println!();
        println!("FAILURES: Active security boundaries are broken.");
    }
    if s.dark_forest > 0 {
        println!();
        println!(
            "DARK FOREST: {} information leaks or attack surface findings.",
            s.dark_forest
        );
    }
}

pub fn write_json(report: &Report, path: &Path) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(report).map_err(std::io::Error::other)?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::{Category, CheckBuilder, Severity};
    use std::io::Read;

    fn sample_checks() -> Vec<CheckResult> {
        vec![
            CheckBuilder::new("A", "s", Category::Crypto, Severity::High).pass("ok", "ev"),
            CheckBuilder::new("B", "s", Category::Network, Severity::Critical).fail("bad", "ev"),
            CheckBuilder::new("C", "s", Category::Fuzz, Severity::Low).dark("leak", "ev"),
            CheckBuilder::new("D", "s", Category::Auth, Severity::Medium).known_gap("gap", "ev"),
        ]
    }

    #[test]
    fn report_summary_counts_correctly() {
        let report = Report::build(sample_checks(), "localhost", "all", "2026-01-01", 100);
        assert_eq!(report.summary.total, 4);
        assert_eq!(report.summary.pass, 1);
        assert_eq!(report.summary.fail, 1);
        assert_eq!(report.summary.dark_forest, 1);
        assert_eq!(report.summary.known_gap, 1);
    }

    #[test]
    fn report_empty_checks() {
        let report = Report::build(vec![], "host", "s", "ts", 0);
        assert_eq!(report.summary.total, 0);
        assert_eq!(report.summary.pass, 0);
        assert_eq!(report.summary.fail, 0);
    }

    #[test]
    fn report_preserves_metadata() {
        let report = Report::build(vec![], "10.0.0.1", "crypto", "2026-05-16T00:00:00", 42);
        assert_eq!(report.host, "10.0.0.1");
        assert_eq!(report.suite, "crypto");
        assert_eq!(report.timestamp, "2026-05-16T00:00:00");
        assert_eq!(report.duration_ms, 42);
    }

    #[test]
    fn report_json_roundtrip() {
        let report = Report::build(sample_checks(), "h", "s", "t", 1);
        let json = serde_json::to_string(&report).expect("serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(value["summary"]["total"], 4);
        assert_eq!(value["summary"]["pass"], 1);
        assert_eq!(value["summary"]["fail"], 1);
    }

    #[test]
    fn write_json_creates_valid_file() {
        let dir = std::env::temp_dir().join("darkforest_test_report");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_report.json");

        let report = Report::build(sample_checks(), "h", "s", "t", 1);
        write_json(&report, &path).expect("write");

        let mut contents = String::new();
        fs::File::open(&path)
            .expect("open")
            .read_to_string(&mut contents)
            .expect("read");

        let value: serde_json::Value = serde_json::from_str(&contents).expect("parse");
        assert_eq!(value["summary"]["total"], 4);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
