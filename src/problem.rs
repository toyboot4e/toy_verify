#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

use crate::types::{ProblemInfo, TestCase};

static PULLED: OnceLock<bool> = OnceLock::new();

const REPO_URL: &str = "https://github.com/yosupo06/library-checker-problems";

pub fn from_url(url: &str) -> Option<String> {
    let re = Regex::new(r"^https?://judge\.yosupo\.jp/problem/([a-z0-9_]+)$").unwrap();
    re.captures(url).map(|c| c[1].to_string())
}

pub fn repo_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("library-checker-problems")
}

pub fn update_repository(cache_dir: &Path) -> Result<()> {
    let path = repo_path(cache_dir);

    if path.exists() {
        PULLED.get_or_init(|| {
            eprintln!("Pulling library-checker-problems...");
            let status = Command::new("git")
                .args(["-C", path.to_str().unwrap(), "pull"])
                .status()
                .expect("failed to run git pull");
            if !status.success() {
                eprintln!("warning: git pull failed");
            }
            true
        });
    } else {
        std::fs::create_dir_all(cache_dir).context("failed to create cache directory")?;
        eprintln!("Cloning library-checker-problems...");
        let status = Command::new("git")
            .args(["clone", REPO_URL, path.to_str().unwrap()])
            .status()
            .context("failed to run git clone")?;
        if !status.success() {
            bail!("git clone failed");
        }
    }

    Ok(())
}

pub fn source_directory(cache_dir: &Path, problem_id: &str) -> Result<PathBuf> {
    let repo = repo_path(cache_dir);
    let pattern = format!("{}/**/{}/info.toml", repo.display(), problem_id);

    let matches: Vec<_> = glob::glob(&pattern)
        .context("invalid glob pattern")?
        .filter_map(|r| r.ok())
        .collect();

    match matches.len() {
        0 => bail!("problem '{}' not found in repository", problem_id),
        1 => Ok(matches[0].parent().unwrap().to_path_buf()),
        _ => bail!(
            "multiple matches for problem '{}': {:?}",
            problem_id,
            matches
        ),
    }
}

pub fn generate_test_cases(cache_dir: &Path, problem_id: &str) -> Result<()> {
    let repo = repo_path(cache_dir);
    let source_dir = source_directory(cache_dir, problem_id)?;
    let info_toml = source_dir.join("info.toml");
    let generate_py = repo.join("generate.py");

    if !generate_py.exists() {
        bail!("generate.py not found at {}", generate_py.display());
    }

    eprintln!("Generating test cases for {}...", problem_id);
    let status = Command::new("python3")
        .args([generate_py.to_str().unwrap(), info_toml.to_str().unwrap()])
        .status()
        .context("failed to run generate.py")?;

    if !status.success() {
        bail!("generate.py failed for {}", problem_id);
    }

    Ok(())
}

pub fn discover_test_cases(source_dir: &Path) -> Result<Vec<TestCase>> {
    let in_dir = source_dir.join("in");
    let out_dir = source_dir.join("out");

    if !in_dir.exists() {
        bail!("input directory not found: {}", in_dir.display());
    }

    let pattern = format!("{}/*.in", in_dir.display());
    let mut cases: Vec<TestCase> = glob::glob(&pattern)
        .context("invalid glob pattern")?
        .filter_map(|r| r.ok())
        .filter_map(|input_path| {
            let stem = input_path.file_stem()?.to_str()?.to_string();
            let output_path = out_dir.join(format!("{}.out", stem));
            if output_path.exists() {
                Some(TestCase {
                    name: stem,
                    input_path,
                    output_path,
                })
            } else {
                None
            }
        })
        .collect();

    cases.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(cases)
}

pub fn download_and_generate(
    cache_dir: &Path,
    problem_id: &str,
    url: &str,
    file: &Path,
) -> Result<(ProblemInfo, Vec<TestCase>)> {
    update_repository(cache_dir)?;
    generate_test_cases(cache_dir, problem_id)?;
    let source_dir = source_directory(cache_dir, problem_id)?;
    let cases = discover_test_cases(&source_dir)?;
    let info = ProblemInfo {
        problem_id: problem_id.to_string(),
        url: url.to_string(),
        source_dir,
        file: file.to_path_buf(),
    };
    Ok((info, cases))
}
