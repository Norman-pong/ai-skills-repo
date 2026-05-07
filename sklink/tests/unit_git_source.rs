use sklink::git_source::looks_like_git_url;

#[test]
fn looks_like_git_url_matches_common_inputs() {
    assert!(looks_like_git_url("https://github.com/org/repo"));
    assert!(looks_like_git_url("git@github.com:org/repo.git"));
    assert!(looks_like_git_url("/tmp/something.git"));
    assert!(!looks_like_git_url("./skills/software-engineer"));
}
