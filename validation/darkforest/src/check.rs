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
    pub name: String,
    pub port: u16,
}

/// Compiled defaults — used when no env overrides are set
const DEFAULT_PRIMALS: &[(&str, &str, u16)] = &[
    ("barracuda",     "BARRACUDA_PORT",     9740),
    ("beardog",       "BEARDOG_PORT",       9100),
    ("biomeos",       "BIOMEOS_PORT",       9800),
    ("coralreef",     "CORALREEF_PORT",     9730),
    ("loamspine",     "LOAMSPINE_PORT",     9700),
    ("nestgate",      "NESTGATE_PORT",      9500),
    ("petaltongue",   "PETALTONGUE_PORT",   9900),
    ("rhizocrypt",    "RHIZOCRYPT_PORT",    9601),
    ("rhizocrypt-rpc","RHIZOCRYPT_RPC_PORT",9602),
    ("skunkbat",      "SKUNKBAT_PORT",      9140),
    ("songbird",      "SONGBIRD_PORT",      9200),
    ("squirrel",      "SQUIRREL_PORT",      9300),
    ("sweetgrass",    "SWEETGRASS_PORT",    9850),
    ("toadstool",     "TOADSTOOL_PORT",     9400),
];

/// Loads primal list, honoring env-var overrides from nucleus_config.sh
pub fn load_primals() -> Vec<Primal> {
    DEFAULT_PRIMALS
        .iter()
        .map(|(name, env_key, default_port)| {
            let port = std::env::var(env_key)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(*default_port);
            Primal { name: (*name).to_string(), port }
        })
        .collect()
}

pub fn hub_port() -> u16 {
    std::env::var("JUPYTERHUB_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000)
}

pub const COMPUTE_USER: &str = "tamison";
pub const REVIEWER_USER: &str = "abgreviewer";
pub const OBSERVER_USER: &str = "abg-test";
