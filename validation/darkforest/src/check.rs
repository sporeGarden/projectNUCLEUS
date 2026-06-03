use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    KnownGap,
    DarkForest,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Network,
    Crypto,
    Auth,
    Isolation,
    Fuzz,
    InfoLeak,
    Observer,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub id: String,
    pub suite: String,
    pub category: Category,
    pub severity: Severity,
    pub status: Status,
    pub title: String,
    pub evidence: String,
    pub remediation: String,
    pub elapsed_ms: u64,
    pub timestamp: String,
}

impl CheckResult {
    pub const fn pipe_tag(&self) -> &'static str {
        match self.status {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
            Status::KnownGap => "KNOWN_GAP",
            Status::DarkForest => "DARK_FOREST",
        }
    }
}

pub struct CheckBuilder {
    id: String,
    suite: String,
    category: Category,
    severity: Severity,
    remediation: String,
    start: Instant,
}

impl CheckBuilder {
    pub fn new(id: &str, suite: &str, category: Category, severity: Severity) -> Self {
        Self {
            id: id.to_string(),
            suite: suite.to_string(),
            category,
            severity,
            remediation: String::new(),
            start: Instant::now(),
        }
    }

    pub fn remediation(mut self, r: &str) -> Self {
        self.remediation = r.to_string();
        self
    }

    pub fn pass(self, title: &str, evidence: &str) -> CheckResult {
        self.finish(Status::Pass, title, evidence)
    }

    pub fn fail(self, title: &str, evidence: &str) -> CheckResult {
        self.finish(Status::Fail, title, evidence)
    }

    pub fn dark(self, title: &str, evidence: &str) -> CheckResult {
        self.finish(Status::DarkForest, title, evidence)
    }

    pub fn known_gap(self, title: &str, evidence: &str) -> CheckResult {
        self.finish(Status::KnownGap, title, evidence)
    }

    fn finish(self, status: Status, title: &str, evidence: &str) -> CheckResult {
        CheckResult {
            id: self.id,
            suite: self.suite,
            category: self.category,
            severity: self.severity,
            status,
            title: title.to_string(),
            evidence: evidence.to_string(),
            remediation: self.remediation,
            elapsed_ms: u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX),
            timestamp: iso_now(),
        }
    }
}

pub fn iso_now() -> String {
    use std::time::SystemTime;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();

    // Convert to UTC broken-down time (no external crate needed)
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Days since 1970-01-01 to Y-M-D (civil_from_days algorithm)
    #[allow(clippy::cast_possible_wrap)]
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    #[allow(clippy::cast_possible_wrap)]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}+00:00")
}

pub struct Primal {
    pub name: String,
    pub port: u16,
}

/// Loads primal list via capability-based discovery with env/default fallback.
///
/// Resolution cascade:
/// 1. biomeOS `primal.list` (live topology) — if biomeOS is reachable
/// 2. Per-primal `{NAME}_PORT` env vars (ops override)
/// 3. Compiled defaults (last resort)
pub fn load_primals() -> Vec<Primal> {
    let host = std::env::var("DARKFOREST_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    crate::discovery::resolve_primals(&host)
        .into_iter()
        .map(|rp| Primal {
            name: rp.name,
            port: rp.port,
        })
        .collect()
}

pub fn hub_port() -> u16 {
    std::env::var("JUPYTERHUB_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000)
}

pub fn compute_user() -> String {
    std::env::var("DARKFOREST_COMPUTE_USER").unwrap_or_else(|_| "tamison".into())
}

pub fn reviewer_user() -> String {
    std::env::var("DARKFOREST_REVIEWER_USER").unwrap_or_else(|_| "abgreviewer".into())
}

pub fn observer_user() -> String {
    std::env::var("DARKFOREST_OBSERVER_USER").unwrap_or_else(|_| "abg-test".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_tag_maps_correctly() {
        let mk = |s: Status| CheckResult {
            id: String::new(),
            suite: String::new(),
            category: Category::Network,
            severity: Severity::Info,
            status: s,
            title: String::new(),
            evidence: String::new(),
            remediation: String::new(),
            elapsed_ms: 0,
            timestamp: String::new(),
        };
        assert_eq!(mk(Status::Pass).pipe_tag(), "PASS");
        assert_eq!(mk(Status::Fail).pipe_tag(), "FAIL");
        assert_eq!(mk(Status::KnownGap).pipe_tag(), "KNOWN_GAP");
        assert_eq!(mk(Status::DarkForest).pipe_tag(), "DARK_FOREST");
    }

    #[test]
    fn check_builder_pass_records_fields() {
        let result = CheckBuilder::new("TST-01", "test.suite", Category::Crypto, Severity::High)
            .remediation("fix it")
            .pass("all good", "no issues");
        assert_eq!(result.id, "TST-01");
        assert_eq!(result.suite, "test.suite");
        assert_eq!(result.status, Status::Pass);
        assert_eq!(result.category, Category::Crypto);
        assert_eq!(result.severity, Severity::High);
        assert_eq!(result.remediation, "fix it");
        assert_eq!(result.title, "all good");
        assert_eq!(result.evidence, "no issues");
    }

    #[test]
    fn check_builder_fail_records_status() {
        let result = CheckBuilder::new("TST-02", "s", Category::Auth, Severity::Critical)
            .fail("broken", "evidence");
        assert_eq!(result.status, Status::Fail);
    }

    #[test]
    fn check_builder_dark_records_status() {
        let result = CheckBuilder::new("TST-03", "s", Category::InfoLeak, Severity::Low)
            .dark("leak found", "data");
        assert_eq!(result.status, Status::DarkForest);
    }

    #[test]
    fn check_builder_known_gap_records_status() {
        let result = CheckBuilder::new("TST-04", "s", Category::Fuzz, Severity::Medium)
            .known_gap("gap", "reason");
        assert_eq!(result.status, Status::KnownGap);
    }

    #[test]
    fn load_primals_returns_15_defaults() {
        let primals = load_primals();
        assert_eq!(primals.len(), 15, "should have 15 default primals");
    }

    #[test]
    fn load_primals_includes_all_nucleus_primals() {
        let primals = load_primals();
        let names: Vec<&str> = primals.iter().map(|p| p.name.as_str()).collect();
        for expected in [
            "beardog",
            "songbird",
            "skunkbat",
            "toadstool",
            "barracuda",
            "coralreef",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
            "biomeos",
            "petaltongue",
            "squirrel",
        ] {
            assert!(names.contains(&expected), "missing primal: {expected}");
        }
    }

    #[test]
    fn status_serde_roundtrip() {
        let json = serde_json::to_string(&Status::DarkForest).unwrap();
        assert_eq!(json, "\"dark_forest\"");
    }

    #[test]
    fn severity_serde_roundtrip() {
        let json = serde_json::to_string(&Severity::Critical).unwrap();
        assert_eq!(json, "\"critical\"");
    }

    #[test]
    fn category_serde_roundtrip() {
        let json = serde_json::to_string(&Category::InfoLeak).unwrap();
        assert_eq!(json, "\"info_leak\"");
    }

    #[test]
    fn iso_now_returns_valid_iso8601() {
        let ts = iso_now();
        assert!(
            ts.contains('T'),
            "timestamp should contain T separator: {ts}"
        );
        assert!(
            ts.contains('-'),
            "timestamp should contain date dashes: {ts}"
        );
        assert!(
            ts.contains(':'),
            "timestamp should contain time colons: {ts}"
        );
        assert!(ts.len() >= 19, "timestamp too short: {ts}");
    }
}
