//! `toy_verify` is a tool for running online judge problem solutions.

use std::path::{Path, PathBuf};
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
    /// Sub command. If it's empty, it falls back to the help command.
    #[command(subcommand)]
    command: Option<Commands>,

    /// Cache directory for problem generation repositories and test cases.
    #[arg(long)]
    cache_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Download and generate test cases for problems
    Download(Download),
    /// Download test cases and run a solution against them
    Test(Test),
    /// Show configuration paths and environment info
    Info(Info),
}

#[derive(Parser)]
struct Download {
    /// Problem URLs (e.g. https://judge.yosupo.jp/problem/aplusb)
    urls: Vec<String>,
}

#[derive(Parser)]
struct Info {}

#[derive(Parser)]
struct Test {
    /// Source files containing `[verify]: <URL>` directives
    files: Vec<PathBuf>,
    /// Time limit in seconds
    #[arg(long)]
    tle: Option<f64>,
}

impl Download {
    fn run(&self, cache_dir: &Path) -> Result<()> {
        for url in &self.urls {
            let problem_id = match problem::from_url(url) {
                Some(id) => id,
                None => bail!("unsupported problem URL: {}", url),
            };

            let (_, cases) =
                problem::download_and_generate(cache_dir, &problem_id, url, Path::new(""))?;
            eprintln!("Downloaded {} test cases for '{}'", cases.len(), problem_id);
        }
        Ok(())
    }
}

impl Test {
    fn extract_url(path: &Path) -> Result<Option<String>> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read file: {}", path.display()))?;
        for line in content.lines() {
            if let Some(pos) = line.find("[verify]:") {
                let rest = line[pos + "[verify]:".len()..].trim();
                if !rest.is_empty() {
                    return Ok(Some(rest.to_string()));
                }
            }
        }
        Ok(None)
    }

    fn run(&self, cache_dir: &PathBuf) -> Result<()> {
        let config_path = PathBuf::from("toy_verify/config.toml");
        let cfg =
            config::parse_config(&config_path).context("failed to load toy_verify/config.toml")?;
        let timeout = Duration::from_secs_f64(self.tle.unwrap_or(30.0));

        let mut summaries = Vec::new();
        for file in &self.files {
            let url = match Self::extract_url(file)? {
                Some(url) => url,
                None => {
                    eprintln!(
                        "warning: no [verify]: directive found in {}",
                        file.display()
                    );
                    continue;
                }
            };

            let problem_id = match problem::from_url(&url) {
                Some(id) => id,
                None => bail!("unsupported problem URL: {}", url),
            };

            let (info, cases) = problem::download_and_generate(cache_dir, &problem_id, &url, file)?;

            if let Some(ref compile_template) = cfg.compile {
                let compile_cmd = config::expand_compile(compile_template, &info);
                eprintln!("Compiling: {}", compile_cmd);
                let status = std::process::Command::new("sh")
                    .args(["-c", &compile_cmd])
                    .status()
                    .context("failed to run compile command")?;
                if !status.success() {
                    bail!("compile command failed for '{}'", problem_id);
                }
            }

            let execute_cmd = config::expand_execute(&cfg.execute, &info);
            eprintln!(
                "Running {} test cases for '{}'...\n",
                cases.len(),
                problem_id
            );

            summaries.push(judge::run_test_suite(&execute_cmd, &cases, timeout)?);
        }

        if !summaries.iter().all(|s| s.success) {
            process::exit(1);
        }
        Ok(())
    }
}

fn default_cache_dir() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .expect("could not determine a cache directory for your platform")
        .join("toy_verify")
}

impl Info {
    fn run(&self, cache_dir: &Path) -> Result<()> {
        let config_path = PathBuf::from("toy_verify/config.toml");
        let repo_dir = problem::repo_path(cache_dir);

        println!("toy_verify {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!(
            "Config path:  {}",
            config_path
                .canonicalize()
                .unwrap_or(config_path.clone())
                .display()
        );
        println!("Cache dir:    {}", cache_dir.display());
        println!("Repo dir:     {}", repo_dir.display());
        println!("Repo exists:  {}", repo_dir.exists());

        match config::parse_config(&config_path) {
            Ok(cfg) => {
                if let Some(ref compile) = cfg.compile {
                    println!("Compile:      {}", compile);
                }
                println!("Execute:      {}", cfg.execute);
            }
            Err(_) => {
                println!("Config:       not found or invalid");
            }
        }

        Ok(())
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let cache_dir = cli.cache_dir.unwrap_or_else(default_cache_dir);

    match cli.command {
        Some(Commands::Download(cmd)) => cmd.run(&cache_dir),
        Some(Commands::Test(cmd)) => cmd.run(&cache_dir),
        Some(Commands::Info(cmd)) => cmd.run(&cache_dir),
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            Ok(())
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        process::exit(1);
    }
}
