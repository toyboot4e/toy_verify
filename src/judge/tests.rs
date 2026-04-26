use std::time::Duration;

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
    let elapsed = Duration::from_millis(100);

    let ac = ExecResult::Completed {
        stdout: "42\n".to_string(),
        exitcode: 0,
        elapsed,
    };
    assert_eq!(ac.judge_status("42"), JudgeStatus::AC);

    let wa = ExecResult::Completed {
        stdout: "43\n".to_string(),
        exitcode: 0,
        elapsed,
    };
    assert_eq!(wa.judge_status("42"), JudgeStatus::WA);

    let re = ExecResult::Completed {
        stdout: "42\n".to_string(),
        exitcode: 1,
        elapsed,
    };
    assert_eq!(re.judge_status("42"), JudgeStatus::RE);

    let tle = ExecResult::TimedOut {
        stdout: String::new(),
        elapsed,
    };
    assert_eq!(tle.judge_status("42"), JudgeStatus::TLE);
}
