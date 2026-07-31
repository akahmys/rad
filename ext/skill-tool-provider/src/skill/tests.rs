use super::{SkillMode, parse_skill_md, skill_dirs, substitute_args};

#[test]
fn test_skill_dirs_project_local_before_user_global() {
    let dirs = skill_dirs();
    assert_eq!(dirs[0], ".agents/skills");
    assert_eq!(dirs[1], "~/.rad/skills");
}

#[test]
fn test_parse_skill_md_extracts_description_and_body() {
    let content = "---\ndescription: Runs the checklist.\n---\n\nDo the thing.";
    let skill = parse_skill_md("review", content).unwrap();
    assert_eq!(skill.name, "review");
    assert_eq!(skill.description, "Runs the checklist.");
    assert_eq!(skill.mode, SkillMode::Inline);
    assert_eq!(skill.body, "Do the thing.");
}

#[test]
fn test_parse_skill_md_returns_none_without_frontmatter_delimiters() {
    assert!(parse_skill_md("review", "no frontmatter here").is_none());
}

#[test]
fn test_parse_skill_md_returns_none_without_description() {
    let content = "---\nmode: inline\n---\n\nBody only.";
    assert!(parse_skill_md("review", content).is_none());
}

#[test]
fn test_parse_skill_md_defaults_to_inline_mode() {
    let content = "---\ndescription: Something.\n---\n\nBody.";
    assert_eq!(parse_skill_md("s", content).unwrap().mode, SkillMode::Inline);
}

#[test]
fn test_parse_skill_md_recognizes_subagent_mode() {
    let content = "---\ndescription: Something.\nmode: subagent\n---\n\nBody.";
    assert_eq!(parse_skill_md("s", content).unwrap().mode, SkillMode::Subagent);
}

#[test]
fn test_parse_skill_md_ignores_unknown_frontmatter_fields() {
    let content = "---\ndescription: Something.\nallowed_tools: read, write\nunknown_field: whatever\n---\n\nBody.";
    let skill = parse_skill_md("s", content).unwrap();
    assert_eq!(skill.description, "Something.");
}

#[test]
fn test_substitute_args_replaces_placeholder() {
    let body = "Investigate $ARGUMENTS thoroughly.".to_string();
    assert_eq!(
        substitute_args(body, "the login bug"),
        "Investigate the login bug thoroughly."
    );
}

#[test]
fn test_substitute_args_appends_when_no_placeholder_and_args_present() {
    let body = "Run the checklist.".to_string();
    assert_eq!(
        substitute_args(body, "for PR #42"),
        "Run the checklist.\n\nfor PR #42"
    );
}

#[test]
fn test_substitute_args_unchanged_when_no_placeholder_and_no_args() {
    let body = "Run the checklist.".to_string();
    assert_eq!(substitute_args(body.clone(), ""), body);
}
