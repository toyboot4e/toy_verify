use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeStatus {
    AC,
    WA,
    RE,
    TLE,
}

impl fmt::Display for JudgeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JudgeStatus::AC => write!(f, "AC"),
            JudgeStatus::WA => write!(f, "WA"),
            JudgeStatus::RE => write!(f, "RE"),
            JudgeStatus::TLE => write!(f, "TLE"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TestCaseResult {
    pub name: String,
    pub status: JudgeStatus,
    pub elapsed: Duration,
}

#[derive(Debug, Clone)]
pub struct TestSummary {
    pub success: bool,
    pub results: Vec<TestCaseResult>,
    pub elapsed: Duration,
}

#[derive(Debug, Clone)]
pub struct ProblemInfo {
    pub problem_id: String,
    pub url: String,
    pub source_dir: PathBuf,
    pub file: PathBuf,
}
