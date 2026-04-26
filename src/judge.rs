//! The judge system.

#[cfg(test)]
mod tests;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

use anyhow::{Context, Result};

use crate::problem::ProblemId;
use crate::types::{JudgeStatus, TestCase, TestCaseResult, TestSummary};

/// Execution result. `stderr` is immediately outputted to terminal and is lost.
pub(crate) enum ExecResult {
    /// Not TLE.
    Completed {
        stdout: String,
        exitcode: i32,
        elapsed: Duration,
    },
    /// TLE.
    TimedOut { stdout: String, elapsed: Duration },
}

impl ExecResult {
    fn stdout(&self) -> &str {
        match self {
            ExecResult::Completed { stdout, .. } | ExecResult::TimedOut { stdout, .. } => stdout,
        }
    }

    fn elapsed(&self) -> Duration {
        match self {
            ExecResult::Completed { elapsed, .. } | ExecResult::TimedOut { elapsed, .. } => {
                *elapsed
            }
        }
    }

    /// Returns the [`JudgeStatus`].
    fn judge_status(&self, expected: &str) -> JudgeStatus {
        match self {
            ExecResult::TimedOut { .. } => JudgeStatus::TLE,
            ExecResult::Completed {
                stdout, exitcode, ..
            } => {
                if *exitcode != 0 {
                    JudgeStatus::RE
                } else if compare_output(stdout, expected) {
                    JudgeStatus::AC
                } else {
                    JudgeStatus::WA
                }
            }
        }
    }
}

/// How we compare the result and the expected code. Currently, it's word_based.
fn compare_output(actual: &str, expected: &str) -> bool {
    let actual_words: Vec<&str> = actual.split_whitespace().collect();
    let expected_words: Vec<&str> = expected.split_whitespace().collect();
    actual_words == expected_words
}

// Runs user command and compares the result with the expected value.
fn run_user_execute_command(
    user_execute_command: &str,
    input_path: &Path,
    timeout: Duration,
) -> Result<ExecResult> {
    let input = std::fs::read(input_path)
        .with_context(|| format!("failed to read input file: {}", input_path.display()))?;

    let start = Instant::now();
    let mut child = Command::new("sh")
        .args(["-c", user_execute_command])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn command")?;

    let mut stdin_handle = child.stdin.take();
    let writer = std::thread::spawn(move || {
        if let Some(ref mut stdin) = stdin_handle {
            stdin.write_all(&input).ok();
        }
    });

    let mut stdout_handle = child.stdout.take();
    let reader = std::thread::spawn(move || -> Vec<u8> {
        let mut buf = Vec::new();
        if let Some(ref mut stdout) = stdout_handle {
            std::io::Read::read_to_end(stdout, &mut buf).ok();
        }
        buf
    });

    let wait_result = child
        .wait_timeout(timeout)
        .context("failed to wait for command")?;
    let elapsed = start.elapsed();

    let status = match wait_result {
        Some(status) => status,
        None => {
            child.kill().ok();
            child.wait().ok();
            writer.join().ok();
            let stdout_bytes = reader.join().unwrap_or_default();
            return Ok(ExecResult::TimedOut {
                stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
                elapsed,
            });
        }
    };
    writer.join().ok();
    let stdout_bytes = reader.join().unwrap_or_default();

    Ok(ExecResult::Completed {
        stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
        exitcode: status.code().unwrap_or(-1),
        elapsed,
    })
}

/// Runs one problem.
pub fn run_test_suite(
    user_execute_command: &str,
    cases: &[TestCase],
    tle: Duration,
    problem_id: &ProblemId,
) -> Result<TestSummary> {
    let total_start = Instant::now();
    let mut results = Vec::new();

    for case in cases {
        let exec = run_user_execute_command(user_execute_command, &case.input_path, tle)?;

        let expected = std::fs::read_to_string(&case.output_path).with_context(|| {
            format!(
                "failed to read expected output: {}",
                case.output_path.display()
            )
        })?;
        let status = exec.judge_status(&expected);

        // print one test case result
        let elapsed = exec.elapsed();
        eprintln!(
            "  {} ... {} ({:.3}s)",
            case.name,
            status,
            elapsed.as_secs_f64()
        );

        if status != JudgeStatus::AC {
            let stdout = exec.stdout();
            if !stdout.is_empty() {
                eprint!("{stdout}");
                if !stdout.ends_with('\n') {
                    eprintln!();
                }
            }
        }

        results.push(TestCaseResult {
            name: case.name.clone(),
            status,
            elapsed,
        });
    }

    let total_elapsed = total_start.elapsed();
    let exec_elapsed: Duration = results.iter().map(|r| r.elapsed).sum();
    let all_ac = results.iter().all(|r| r.status == JudgeStatus::AC);

    // print the test suite result
    let ac_count = results
        .iter()
        .filter(|r| r.status == JudgeStatus::AC)
        .count();
    eprintln!(
        "\n{}/{} tests passed for {} (exec {:.3}s / total {:.3}s)",
        ac_count,
        results.len(),
        problem_id,
        exec_elapsed.as_secs_f64(),
        total_elapsed.as_secs_f64()
    );

    Ok(TestSummary {
        success: all_ac,
        results,
        elapsed: total_elapsed,
    })
}
