//! The judge.

#[cfg(test)]
mod tests;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::types::{JudgeStatus, TestCase, TestCaseResult, TestSummary};

pub(crate) struct ExecResult {
    stdout: String,
    exitcode: Option<i32>,
    elapsed: Duration,
    timed_out: bool,
}

// Runs user command and compares the result with the expected value.
fn run_user_execute_command(
    user_execute_command: &str,
    input_path: &Path,
    timeout: Option<Duration>,
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

    if let Some(tle) = timeout {
        let deadline = start + tle;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        child.kill().ok();
                        child.wait().ok();
                        writer.join().ok();
                        reader.join().ok();
                        let elapsed = start.elapsed();
                        return Ok(ExecResult {
                            stdout: String::new(),
                            exitcode: None,
                            elapsed,
                            timed_out: true,
                        });
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    let status = child.wait().context("failed to wait for command")?;
    writer.join().ok();
    let stdout_bytes = reader.join().unwrap_or_default();
    let elapsed = start.elapsed();

    Ok(ExecResult {
        stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
        exitcode: status.code(),
        elapsed,
        timed_out: false,
    })
}

/// How we compare the result and the expected code. Currently, it's word_based.
fn compare_output(actual: &str, expected: &str) -> bool {
    let actual_words: Vec<&str> = actual.split_whitespace().collect();
    let expected_words: Vec<&str> = expected.split_whitespace().collect();
    actual_words == expected_words
}

/// Convers the execution result to [`JudgeStatus`] (AC, WA, RE, TLE).
fn determine_status(exitcode: Option<i32>, matched: bool, timed_out: bool) -> JudgeStatus {
    if timed_out {
        return JudgeStatus::TLE;
    }
    match exitcode {
        Some(0) => {
            if matched {
                JudgeStatus::AC
            } else {
                JudgeStatus::WA
            }
        }
        _ => JudgeStatus::RE,
    }
}

/// Runs one problem.
pub fn run_test_suite(
    user_execute_command: &str,
    cases: &[TestCase],
    tle: Option<Duration>,
) -> Result<TestSummary> {
    let total_start = Instant::now();
    let mut results = Vec::new();
    let mut all_ac = true;

    for case in cases {
        let exec = run_user_execute_command(user_execute_command, &case.input_path, tle)?;

        let matched = if exec.timed_out {
            false
        } else {
            let expected = std::fs::read_to_string(&case.output_path).with_context(|| {
                format!(
                    "failed to read expected output: {}",
                    case.output_path.display()
                )
            })?;
            compare_output(&exec.stdout, &expected)
        };

        let status = determine_status(exec.exitcode, matched, exec.timed_out);

        if status != JudgeStatus::AC {
            all_ac = false;
        }

        eprintln!(
            "  {} ... {} ({:.3}s)",
            case.name,
            status,
            exec.elapsed.as_secs_f64()
        );

        results.push(TestCaseResult {
            name: case.name.clone(),
            status,
            elapsed: exec.elapsed,
        });
    }

    let total_elapsed = total_start.elapsed();

    let ac_count = results
        .iter()
        .filter(|r| r.status == JudgeStatus::AC)
        .count();
    eprintln!(
        "\n{}/{} tests passed ({:.3}s)",
        ac_count,
        results.len(),
        total_elapsed.as_secs_f64()
    );

    Ok(TestSummary {
        success: all_ac,
        results,
        elapsed: total_elapsed,
    })
}
