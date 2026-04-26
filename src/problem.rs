//! Problem handlers for online judges.

#[cfg(test)]
mod tests;

mod library_checker;

use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::types::{ProblemInfo, TestCase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProblemId {
    LibraryChecker(String),
}

impl fmt::Display for ProblemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProblemId::LibraryChecker(id) => write!(f, "{}", id),
        }
    }
}

/// Extracts a problem ID from a supported judge URL.
pub fn from_url(url: &str) -> Option<ProblemId> {
    library_checker::from_url(url).map(ProblemId::LibraryChecker)
}

/// Downloads and generates test cases for the given problem.
pub fn download_and_generate(
    cache_dir: &Path,
    problem_id: &ProblemId,
    url: &str,
    file: &Path,
) -> Result<(ProblemInfo, Vec<TestCase>)> {
    match problem_id {
        ProblemId::LibraryChecker(id) => {
            library_checker::download_and_generate(cache_dir, id, url, file)
        }
    }
}

/// Returns the Library Checker repository path.
pub fn repo_path(cache_dir: &Path) -> PathBuf {
    library_checker::repo_path(cache_dir)
}
