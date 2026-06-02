mod above;
mod at;
mod below;
mod external;
mod helpers;
mod report;

use std::fmt;
use std::path::PathBuf;

use chrono::Local;
use thiserror::Error;
use tokio::fs;

use crate::config::NucleusConfig;
use report::{SecurityReport, Verdict};

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("results directory creation failed: {0}")]
    ResultsDir(std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Layer {
    All,
    Below,
    At,
    Above,
    Tiers,
    Darkforest,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Below => write!(f, "below"),
            Self::At => write!(f, "at"),
            Self::Above => write!(f, "above"),
            Self::Tiers => write!(f, "tiers"),
            Self::Darkforest => write!(f, "darkforest"),
        }
    }
}

pub struct SecurityArgs {
    pub layer: Layer,
    pub tunnel_url: Option<String>,
    pub target_host: Option<String>,
    pub results_dir: Option<PathBuf>,
}

pub async fn run(cfg: &NucleusConfig, args: &SecurityArgs) -> Result<bool, SecurityError> {
    let target = args.target_host.as_deref().unwrap_or(&cfg.bind_address);

    let results_dir = args.results_dir.clone().unwrap_or_else(|| {
        cfg.project_root
            .join("validation")
            .join(format!("security-{}", Local::now().format("%Y%m%d-%H%M%S")))
    });

    fs::create_dir_all(&results_dir)
        .await
        .map_err(SecurityError::ResultsDir)?;

    let mut report = SecurityReport::new();

    report.log("═══════════════════════════════════════════════════════════");
    report.log("  Security Validation Pipeline — Five-Layer Pen Testing");
    report.log(&format!("  Target: {target}"));
    report.log(&format!("  Layer: {}", args.layer));
    report.log(&format!("  Results: {}", results_dir.display()));
    report.log("═══════════════════════════════════════════════════════════");

    helpers::notify_skunkbat(&mut report, target, cfg.port_for("skunkbat")).await;

    let layer = args.layer;

    if layer == Layer::All || layer == Layer::Below {
        below::layer_below(&mut report, target, cfg).await;
    }

    if layer == Layer::All || layer == Layer::At {
        at::layer_at(&mut report, target, cfg).await;
    }

    if layer == Layer::All || layer == Layer::Above {
        above::layer_above(&mut report, target, cfg, args.tunnel_url.as_deref()).await;
    }

    if layer == Layer::All || layer == Layer::Tiers {
        external::layer_tiers(&mut report, cfg).await;
    }

    if layer == Layer::All || layer == Layer::Darkforest {
        external::layer_darkforest(&mut report, cfg).await;
    }

    helpers::collect_skunkbat_metrics(&mut report, target, cfg.port_for("skunkbat"), &results_dir)
        .await;

    let pass = report.count(Verdict::Pass);
    let fail = report.count(Verdict::Fail);
    let warn = report.count(Verdict::Warn);
    let info = report.count(Verdict::Info);

    report.log("");
    report.log("═══════════════════════════════════════════════════════════");
    report.log("  Security Validation Complete");
    report.log(&format!("  PASS: {pass}"));
    report.log(&format!("  FAIL: {fail}"));
    report.log(&format!("  WARN: {warn}"));
    report.log(&format!("  INFO: {info}"));
    report.log(&format!("  Results: {}", results_dir.display()));
    report.log("═══════════════════════════════════════════════════════════");

    write_report(&report, &results_dir, target, &args.layer).await?;
    write_log(&report, &results_dir).await?;

    Ok(!report.has_failures())
}

async fn write_report(
    report: &SecurityReport,
    results_dir: &std::path::Path,
    target: &str,
    layer: &Layer,
) -> Result<(), SecurityError> {
    let content = format!(
        "# Security Validation — {}\n\n\
        **Target**: {target}\n\
        **Layer**: {layer}\n\n\
        ## Summary\n\n\
        | Metric | Count |\n\
        |--------|-------|\n\
        | PASS | {} |\n\
        | FAIL | {} |\n\
        | WARN | {} |\n\
        | INFO | {} |\n",
        Local::now().to_rfc3339(),
        report.count(Verdict::Pass),
        report.count(Verdict::Fail),
        report.count(Verdict::Warn),
        report.count(Verdict::Info),
    );

    fs::write(results_dir.join("SECURITY_RESULTS.md"), content).await?;
    Ok(())
}

async fn write_log(
    report: &SecurityReport,
    results_dir: &std::path::Path,
) -> Result<(), SecurityError> {
    let content = report.log_lines.join("\n");
    fs::write(results_dir.join("security.log"), content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_display_roundtrip() {
        assert_eq!(Layer::All.to_string(), "all");
        assert_eq!(Layer::Below.to_string(), "below");
        assert_eq!(Layer::Darkforest.to_string(), "darkforest");
    }

    #[test]
    fn report_counts_correctly() {
        let mut report = SecurityReport::new();
        report.pass("test pass");
        report.pass("test pass 2");
        report.fail("test fail");
        report.warn("test warn");
        report.info("test info");

        assert_eq!(report.count(Verdict::Pass), 2);
        assert_eq!(report.count(Verdict::Fail), 1);
        assert_eq!(report.count(Verdict::Warn), 1);
        assert_eq!(report.count(Verdict::Info), 1);
        assert!(report.has_failures());
    }

    #[test]
    fn report_no_failures_when_clean() {
        let mut report = SecurityReport::new();
        report.pass("all good");
        report.warn("minor");
        assert!(!report.has_failures());
    }
}
