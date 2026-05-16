#![forbid(unsafe_code)]

mod check;
mod crypto;
mod discovery;
mod fuzz;
mod net;
mod observer;
mod pentest;
mod report;

use check::iso_now;
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "darkforest",
    about = "Dark Forest — Pure Rust auditable security validator for NUCLEUS",
    version
)]
struct Cli {
    /// Test suite: all, pentest, fuzz, crypto, external, compute, readonly, observer
    #[arg(long, default_value = "all")]
    suite: String,

    /// Bind address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Timing analysis rounds
    #[arg(long, default_value_t = 5)]
    rounds: u32,

    /// Write JSON report to this path
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let host = &cli.host;
    let start = Instant::now();
    let start_ts = iso_now();

    report::print_banner(&cli.suite, &start_ts);

    let mut results: Vec<check::CheckResult> = Vec::new();

    let run_pentest = matches!(
        cli.suite.as_str(),
        "all" | "pentest" | "external" | "compute" | "readonly"
    );
    let run_fuzz = matches!(cli.suite.as_str(), "all" | "fuzz");
    let run_crypto = matches!(cli.suite.as_str(), "all" | "crypto");

    if run_pentest {
        if matches!(cli.suite.as_str(), "all" | "pentest" | "external") {
            let before = results.len();
            pentest::run_external(host, &mut results);
            report::print_pipe(&results[before..]);
        }
        if matches!(cli.suite.as_str(), "all" | "pentest" | "compute") {
            let before = results.len();
            pentest::run_compute(host, &mut results);
            report::print_pipe(&results[before..]);
        }
        if matches!(cli.suite.as_str(), "all" | "pentest" | "readonly") {
            let before = results.len();
            pentest::run_readonly(host, &mut results);
            report::print_pipe(&results[before..]);
        }
    }

    if run_fuzz {
        let before = results.len();
        fuzz::run_primals(host, cli.rounds, &mut results);
        fuzz::run_hub(host, &mut results);
        report::print_pipe(&results[before..]);
    }

    if run_crypto {
        let before = results.len();
        crypto::run(host, &mut results);
        report::print_pipe(&results[before..]);
    }

    if matches!(cli.suite.as_str(), "all" | "observer") {
        let before = results.len();
        observer::run(host, &mut results);
        report::print_pipe(&results[before..]);
    }

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let rpt = report::Report::build(results, host, &cli.suite, &start_ts, duration_ms);

    report::print_summary(&rpt);

    if let Some(ref path) = cli.output {
        match report::write_json(&rpt, path) {
            Ok(()) => println!("\nJSON report written to: {}", path.display()),
            Err(e) => eprintln!("\nERROR writing JSON report: {e}"),
        }
    }

    let exit_code = i32::from(rpt.summary.fail > 0);
    std::process::exit(exit_code);
}
