use super::*;

#[test]
fn test_from_url_valid() {
    assert_eq!(
        from_url("https://judge.yosupo.jp/problem/aplusb"),
        Some(ProblemId::LibraryChecker("aplusb".to_string()))
    );
    assert_eq!(
        from_url("https://judge.yosupo.jp/problem/shortest_path"),
        Some(ProblemId::LibraryChecker("shortest_path".to_string()))
    );
}

#[test]
fn test_from_url_invalid() {
    assert_eq!(from_url("https://example.com/problem/aplusb"), None);
    assert_eq!(from_url("https://judge.yosupo.jp/aplusb"), None);
    assert_eq!(from_url("not a url"), None);
}

#[test]
fn test_repo_path() {
    let p = repo_path(Path::new("/tmp/cache"));
    assert_eq!(p, PathBuf::from("/tmp/cache/library-checker-problems"));
}
