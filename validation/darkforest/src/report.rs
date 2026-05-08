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
    println!("  Dark Forest — Pure Rust Security Validator v{}", env!("CARGO_PKG_VERSION"));
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
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, json)
}
