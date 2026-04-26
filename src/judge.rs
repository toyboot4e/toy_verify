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

fn run_command(command: &str, input_path: &Path, timeout: Option<Duration>) -> Result<ExecResult> {
    let input = std::fs::read(input_path)
        .with_context(|| format!("failed to read input file: {}", input_path.display()))?;

    let start = Instant::now();
    let mut child = Command::new("sh")
        .args(["-c", command])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn command")?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(&input).ok();
    }
    drop(child.stdin.take());

    if let Some(tle) = timeout {
        match child.try_wait() {
            Ok(Some(_)) => {}
            _ => {
                std::thread::sleep(Duration::from_millis(10));
                let deadline = start + tle;
                loop {
                    match child.try_wait() {
                        Ok(Some(_)) => break,
                        Ok(None) => {
                            if Instant::now() >= deadline {
                                child.kill().ok();
                                child.wait().ok();
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
        }
    }

    let output = child.wait_with_output().context("failed to wait for command")?;
    let elapsed = start.elapsed();

    Ok(ExecResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        exitcode: output.status.code(),
        elapsed,
        timed_out: false,
    })
}

pub fn compare_output(actual: &str, expected: &str) -> bool {
    let actual_words: Vec<&str> = actual.split_whitespace().collect();
    let expected_words: Vec<&str> = expected.split_whitespace().collect();
    actual_words == expected_words
}

pub fn special_judge(
    checker: &Path,
    input: &Path,
    actual_output: &str,
    expected: &Path,
) -> Result<bool> {
    let mut tmp = std::env::temp_dir();
    tmp.push("toy_verify_actual.out");
    std::fs::write(&tmp, actual_output)?;

    let status = Command::new(checker)
        .args([
            input.to_str().unwrap(),
            expected.to_str().unwrap(),
            tmp.to_str().unwrap(),
        ])
        .status()
        .context("failed to run checker")?;

    Ok(status.success())
}

pub fn determine_status(exitcode: Option<i32>, matched: bool, timed_out: bool) -> JudgeStatus {
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

pub fn run_test_suite(
    command: &str,
    cases: &[TestCase],
    checker: Option<&Path>,
    tle: Option<Duration>,
) -> Result<TestSummary> {
    let total_start = Instant::now();
    let mut results = Vec::new();
    let mut all_ac = true;

    for case in cases {
        let exec = run_command(command, &case.input_path, tle)?;

        let matched = if exec.timed_out {
            false
        } else if let Some(checker_path) = checker {
            special_judge(checker_path, &case.input_path, &exec.stdout, &case.output_path)?
        } else {
            let expected = std::fs::read_to_string(&case.output_path)
                .with_context(|| format!("failed to read expected output: {}", case.output_path.display()))?;
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

    let ac_count = results.iter().filter(|r| r.status == JudgeStatus::AC).count();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_output_exact() {
        assert!(compare_output("42\n", "42\n"));
        assert!(compare_output("hello world\n", "hello world\n"));
    }

    #[test]
    fn test_compare_output_whitespace_tolerant() {
        assert!(compare_output("42\n", "42"));
        assert!(compare_output("  42  \n", "42\n"));
        assert!(compare_output("1 2 3\n", "1  2  3\n"));
        assert!(compare_output("1\n2\n3\n", "1 2 3"));
    }

    #[test]
    fn test_compare_output_mismatch() {
        assert!(!compare_output("42\n", "43\n"));
        assert!(!compare_output("1 2\n", "1 2 3\n"));
    }

    #[test]
    fn test_determine_status() {
        assert_eq!(determine_status(Some(0), true, false), JudgeStatus::AC);
        assert_eq!(determine_status(Some(0), false, false), JudgeStatus::WA);
        assert_eq!(determine_status(Some(1), true, false), JudgeStatus::RE);
        assert_eq!(determine_status(None, false, true), JudgeStatus::TLE);
    }
}
