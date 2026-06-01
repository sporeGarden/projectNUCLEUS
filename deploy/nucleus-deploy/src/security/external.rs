use tokio::process::Command;

use super::helpers::run_external_test;
use super::report::SecurityReport;
use crate::config::NucleusConfig;

pub async fn layer_tiers(report: &mut SecurityReport, cfg: &NucleusConfig) {
    report.log("");
    report.log("══ Layer 4: ABG Tier Enforcement ══");

    let tier_script = cfg.project_root.join("deploy/tier_enforcement_test.sh");
    run_external_test(report, &tier_script, "OS-level tier enforcement", "4a").await;

    let tier_api_script = cfg.project_root.join("deploy/jupyterhub_tier_test.py");
    if tier_api_script.exists() {
        report.log("");
        report.log("── 4b: JupyterHub API Tier Enforcement ──");

        let output = Command::new("python3").arg(&tier_api_script).output().await;

        match output {
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                let pass_count = text.matches("PASS|").count();
                let fail_count = text.matches("FAIL|").count();
                if fail_count == 0 {
                    report.pass(format!(
                        "JupyterHub API tier enforcement: {pass_count} assertions pass"
                    ));
                } else {
                    report.fail(format!(
                        "JupyterHub API tier enforcement: {fail_count} failures"
                    ));
                }
            }
            Err(_) => report.warn("Could not run jupyterhub_tier_test.py"),
        }
    }
}

pub async fn layer_darkforest(report: &mut SecurityReport, cfg: &NucleusConfig) {
    report.log("");
    report.log("══ Layer 5: Dark Forest ══");

    let release_bin = cfg
        .project_root
        .join("validation/darkforest/target/release/darkforest");
    let debug_bin = cfg
        .project_root
        .join("validation/darkforest/target/debug/darkforest");

    let bin = if release_bin.exists() {
        release_bin
    } else if debug_bin.exists() {
        debug_bin
    } else {
        report
            .warn("darkforest binary not found — build with: cargo build --release -p darkforest");
        return;
    };

    report.log("Running Rust darkforest binary (all suites)...");

    let output = Command::new(&bin).args(["--suite", "all"]).output().await;

    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            let pass_count = text.matches("PASS|").count();
            let fail_count = text.matches("FAIL|").count();
            let gap_count = text.matches("KNOWN_GAP|").count();
            let df_count = text.matches("DARK_FOREST|").count();

            if fail_count == 0 {
                report.pass(format!(
                    "Dark Forest (Rust): {pass_count} pass, {gap_count} gaps, {df_count} dark forest findings"
                ));
            } else {
                report.fail(format!(
                    "Dark Forest (Rust): {fail_count} failures out of {} assertions",
                    pass_count + fail_count
                ));
            }
        }
        Err(e) => report.warn(format!("darkforest execution failed: {e}")),
    }
}
