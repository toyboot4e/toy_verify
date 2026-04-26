//! `toy_verify` is a tool for running online judge problem solutions.

use std::path::PathBuf;
use std::process;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

mod config;

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
    #[arg(long, default_value = "toy_verify/cache")]
    cache_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Download and generate test cases for problems
    Download(Download),
    /// Download test cases and run a solution against them
    Test(Test),
}

#[derive(Parser)]
struct Download {
    /// Problem URLs (e.g. https://judge.yosupo.jp/problem/aplusb)
    urls: Vec<String>,
}

#[derive(Parser)]
struct Test {
    /// Problem URLs (e.g. https://judge.yosupo.jp/problem/aplusb)
    urls: Vec<String>,
    /// Time limit in seconds
    #[arg(long)]
    tle: Option<f64>,
}

impl Download {
    fn run(&self, cache_dir: &PathBuf) -> Result<()> {
        for url in &self.urls {
            let problem_id = problem::from_url(url);
            let problem_id = match problem_id {
                Some(id) => id,
                None => bail!("invalid Library Checker URL: {}", url),
            };

            let (_, cases) =
                problem::download_and_generate(cache_dir, &problem_id, url)?;
            eprintln!("Downloaded {} test cases for '{}'", cases.len(), problem_id);
        }
        Ok(())
    }
}

impl Test {
    fn run(&self, cache_dir: &PathBuf) -> Result<()> {
        let config_path = PathBuf::from("toy_verify/config.toml");
        let cfg = config::parse_config(&config_path)
            .context("failed to load toy_verify/config.toml")?;
        let timeout = self.tle.map(Duration::from_secs_f64);
        let mut all_success = true;

        for url in &self.urls {
            let problem_id = problem::from_url(url);
            let problem_id = match problem_id {
                Some(id) => id,
                None => bail!("invalid Library Checker URL: {}", url),
            };

            let (info, cases) =
                problem::download_and_generate(cache_dir, &problem_id, url)?;

            if let Some(ref compile_template) = cfg.compile {
                let compile_cmd = config::expand(compile_template, &info);
                eprintln!("Compiling: {}", compile_cmd);
                let status = std::process::Command::new("sh")
                    .args(["-c", &compile_cmd])
                    .status()
                    .context("failed to run compile command")?;
                if !status.success() {
                    bail!("compile command failed for '{}'", problem_id);
                }
            }

            let execute_cmd = config::expand(&cfg.execute, &info);
            eprintln!(
                "Running {} test cases for '{}'...\n",
                cases.len(),
                problem_id
            );

            let summary = judge::run_test_suite(&execute_cmd, &cases, None, timeout)?;
            if !summary.success {
                all_success = false;
            }
        }

        if !all_success {
            process::exit(1);
        }
        Ok(())
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Download(cmd) => cmd.run(&cli.cache_dir),
        Commands::Test(cmd) => cmd.run(&cli.cache_dir),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        process::exit(1);
    }
}
