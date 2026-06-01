use chrono::Local;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    Warn,
    Info,
}

pub struct Finding {
    pub verdict: Verdict,
    #[expect(dead_code, reason = "findings stored for potential JSON export")]
    pub message: String,
}

pub struct SecurityReport {
    pub findings: Vec<Finding>,
    pub log_lines: Vec<String>,
}

impl SecurityReport {
    pub const fn new() -> Self {
        Self {
            findings: Vec::new(),
            log_lines: Vec::new(),
        }
    }

    pub fn log(&mut self, msg: &str) {
        let line = format!("[{}] {msg}", Local::now().format("%H:%M:%S"));
        eprintln!("{line}");
        self.log_lines.push(line);
    }

    pub fn pass(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log(&format!("  [PASS] {m}"));
        self.findings.push(Finding {
            verdict: Verdict::Pass,
            message: m,
        });
    }

    pub fn fail(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log(&format!("  [FAIL] {m}"));
        self.findings.push(Finding {
            verdict: Verdict::Fail,
            message: m,
        });
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log(&format!("  [WARN] {m}"));
        self.findings.push(Finding {
            verdict: Verdict::Warn,
            message: m,
        });
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log(&format!("  [INFO] {m}"));
        self.findings.push(Finding {
            verdict: Verdict::Info,
            message: m,
        });
    }

    pub fn count(&self, verdict: Verdict) -> usize {
        self.findings
            .iter()
            .filter(|f| f.verdict == verdict)
            .count()
    }

    pub fn has_failures(&self) -> bool {
        self.count(Verdict::Fail) > 0
    }
}
