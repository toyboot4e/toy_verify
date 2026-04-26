#[cfg(test)]
mod tests;

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::types::ProblemInfo;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub compile: Option<String>,
    pub execute: String,
}

pub fn parse_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("failed to parse config file: {}", path.display()))?;
    if config.execute.is_empty() {
        bail!("'execute' must not be empty in {}", path.display());
    }
    Ok(config)
}

pub fn expand(template: &str, info: &ProblemInfo) -> String {
    template
        .replace("{problem}", &info.problem_id)
        .replace("{url}", &info.url)
        .replace("{source_dir}", &info.source_dir.to_string_lossy().as_ref())
        .replace("{file}", &info.file.to_string_lossy())
}
