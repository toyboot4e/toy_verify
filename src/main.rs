//! `toy_verify` is a tool for running online judge problem solutions.

use std::path::PathBuf;
use std::process;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

mod judge;
mod problem;
mod types;

#[derive(Parser)]
#[command(
    name = "toy_verify",
    about = "Download and verify online judge problems"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Cache directory for repository and test cases
    #[arg(long, default_value = ".toy_verify/cache")]
    cache_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Download and generate test cases for a problem
    Download {
        /// Problem URL (e.g. https://judge.yosupo.jp/problem/aplusb)
        url: String,
    },
    /// Download test cases and run a solution against them
    Test {
        /// Problem URL (e.g. https://judge.yosupo.jp/problem/aplusb)
        url: String,
        /// Command to run the solution
        command: String,
        /// Time limit in seconds
        #[arg(long)]
        tle: Option<f64>,
    },
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Download { url } => {
            let problem_id = problem::from_url(&url);
            let problem_id = match problem_id {
                Some(id) => id,
                None => bail!("invalid Library Checker URL: {}", url),
            };

            let cases = problem::download_and_generate(&cli.cache_dir, &problem_id)?;
            eprintln!("Downloaded {} test cases for '{}'", cases.len(), problem_id);
        }
        Commands::Test { url, command, tle } => {
            let problem_id = problem::from_url(&url);
            let problem_id = match problem_id {
                Some(id) => id,
                None => bail!("invalid Library Checker URL: {}", url),
            };

            let cases = problem::download_and_generate(&cli.cache_dir, &problem_id)?;
            eprintln!(
                "Running {} test cases for '{}'...\n",
                cases.len(),
                problem_id
            );

            let timeout = tle.map(Duration::from_secs_f64);
            let summary = judge::run_test_suite(&command, &cases, None, timeout)?;

            if !summary.success {
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        process::exit(1);
    }
}
