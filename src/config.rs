use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::types::ProblemInfo;

#[derive(Debug, Clone)]
pub struct Config {
    pub compile: Option<String>,
    pub execute: String,
}

pub fn parse_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;

    let mut compile = None;
    let mut execute = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, rest)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = rest.trim();

        // Strip surrounding quotes
        let value = if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            &value[1..value.len() - 1]
        } else {
            value
        };

        match key {
            "compile" => compile = Some(value.to_string()),
            "execute" => execute = Some(value.to_string()),
            _ => {}
        }
    }

    let Some(execute) = execute else {
        bail!("missing required key 'execute' in {}", path.display());
    };

    Ok(Config { compile, execute })
}

pub fn expand(template: &str, info: &ProblemInfo) -> String {
    template
        .replace("{problem}", &info.problem_id)
        .replace("{url}", &info.url)
        .replace("{source_dir}", &info.source_dir.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_config_full() {
        let dir = std::env::temp_dir().join("toy_verify_test_config_full");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            "compile = \"g++ -o {problem} {source_dir}/solution.cpp\"\nexecute = \"./{problem}\"\n",
        )
        .unwrap();

        let config = parse_config(&path).unwrap();
        assert_eq!(
            config.compile.as_deref(),
            Some("g++ -o {problem} {source_dir}/solution.cpp")
        );
        assert_eq!(config.execute, "./{problem}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_config_execute_only() {
        let dir = std::env::temp_dir().join("toy_verify_test_config_exec");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(&path, "execute = \"echo hello\"\n").unwrap();

        let config = parse_config(&path).unwrap();
        assert!(config.compile.is_none());
        assert_eq!(config.execute, "echo hello");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_config_missing_execute() {
        let dir = std::env::temp_dir().join("toy_verify_test_config_miss");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(&path, "compile = \"gcc foo.c\"\n").unwrap();

        let result = parse_config(&path);
        assert!(result.is_err());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_config_comments_and_blanks() {
        let dir = std::env::temp_dir().join("toy_verify_test_config_comments");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            "# This is a comment\n\nexecute = \"./run\"\n\n# Another comment\n",
        )
        .unwrap();

        let config = parse_config(&path).unwrap();
        assert!(config.compile.is_none());
        assert_eq!(config.execute, "./run");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_expand_all_vars() {
        let info = ProblemInfo {
            problem_id: "aplusb".to_string(),
            url: "https://judge.yosupo.jp/problem/aplusb".to_string(),
            source_dir: PathBuf::from("/tmp/problems/aplusb"),
        };

        assert_eq!(
            expand("g++ -o {problem} {source_dir}/sol.cpp", &info),
            "g++ -o aplusb /tmp/problems/aplusb/sol.cpp"
        );
        assert_eq!(expand("./{problem}", &info), "./aplusb");
        assert_eq!(
            expand("echo {url}", &info),
            "echo https://judge.yosupo.jp/problem/aplusb"
        );
    }

    #[test]
    fn test_expand_no_vars() {
        let info = ProblemInfo {
            problem_id: "x".to_string(),
            url: "u".to_string(),
            source_dir: PathBuf::from("d"),
        };

        assert_eq!(expand("echo hello", &info), "echo hello");
    }
}
