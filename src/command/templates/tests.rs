use super::{expand, list_names};
use std::fs;

#[test]
fn test_expand_substitutes_placeholder() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join(".agents/commands");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("greet.md"), "Hello, $ARGUMENTS!").unwrap();

    let expanded = expand(tmp.path().to_str().unwrap(), "greet", "world").unwrap();
    assert_eq!(expanded, "Hello, world!");
}

#[test]
fn test_expand_appends_args_when_no_placeholder() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join(".agents/commands");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("review.md"),
        "Review the current diff for correctness.",
    )
    .unwrap();

    let expanded = expand(tmp.path().to_str().unwrap(), "review", "focus on security").unwrap();
    assert_eq!(
        expanded,
        "Review the current diff for correctness.\n\nfocus on security"
    );
}

#[test]
fn test_expand_returns_template_unchanged_when_no_args_and_no_placeholder() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join(".agents/commands");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("review.md"), "Review the current diff.").unwrap();

    let expanded = expand(tmp.path().to_str().unwrap(), "review", "").unwrap();
    assert_eq!(expanded, "Review the current diff.");
}

#[test]
fn test_expand_returns_none_for_unknown_name() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(expand(tmp.path().to_str().unwrap(), "nonexistent", "").is_none());
}

#[test]
fn test_list_names_finds_md_files_only() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join(".agents/commands");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("alpha.md"), "a").unwrap();
    fs::write(dir.join("beta.md"), "b").unwrap();
    fs::write(dir.join("notes.txt"), "not a command").unwrap();

    let names = list_names(tmp.path().to_str().unwrap());
    assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
}

#[test]
fn test_list_names_empty_when_no_dir() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(list_names(tmp.path().to_str().unwrap()).is_empty());
}
