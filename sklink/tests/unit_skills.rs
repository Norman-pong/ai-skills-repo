use sklink::skills::discover_skills;

#[test]
fn discover_skills_returns_sorted_unique_directory_names() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("b-skill")).unwrap();
    std::fs::create_dir_all(dir.path().join("a-skill")).unwrap();
    std::fs::write(dir.path().join("not-a-dir.txt"), "x").unwrap();

    let skills = discover_skills(dir.path()).unwrap();
    assert_eq!(
        skills.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
        vec!["a-skill".to_string(), "b-skill".to_string()]
    );
}
