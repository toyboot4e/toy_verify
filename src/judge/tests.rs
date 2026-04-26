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
