use serde::Serialize;
use std::process::Command;
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
    pub fn pipe_tag(&self) -> &'static str {
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
            elapsed_ms: self.start.elapsed().as_millis() as u64,
            timestamp: iso_now(),
        }
    }
}

pub fn iso_now() -> String {
    Command::new("date")
        .arg("-Iseconds")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| String::from("unknown"))
}

pub struct Primal {
    pub name: &'static str,
    pub port: u16,
}

pub const PRIMALS: &[Primal] = &[
    Primal { name: "barracuda", port: 9740 },
    Primal { name: "beardog", port: 9100 },
    Primal { name: "biomeos", port: 9800 },
    Primal { name: "coralreef", port: 9730 },
    Primal { name: "loamspine", port: 9700 },
    Primal { name: "nestgate", port: 9500 },
    Primal { name: "petaltongue", port: 9900 },
    Primal { name: "rhizocrypt", port: 9601 },
    Primal { name: "skunkbat", port: 9140 },
    Primal { name: "songbird", port: 9200 },
    Primal { name: "squirrel", port: 9300 },
    Primal { name: "sweetgrass", port: 9850 },
    Primal { name: "toadstool", port: 9400 },
];

pub const HUB_PORT: u16 = 8000;
pub const COMPUTE_USER: &str = "tamison";
pub const REVIEWER_USER: &str = "abgreviewer";
pub const OBSERVER_USER: &str = "abg-test";
