use super::*;
use crate::config::{ExtensionConfig, PermissionConfig};

/// `ExtensionConfig` has no `Default` on purpose — `enabled` and `role` carry
/// non-empty serde defaults, so a derived one would silently mean
/// `enabled: false`. Spelled out here rather than added to the production type
/// for a test's convenience.
fn ext(name: &str, permissions: Option<PermissionConfig>) -> ExtensionConfig {
    ExtensionConfig {
        name: name.to_string(),
        source: String::new(),
        enabled: true,
        role: "orchestrator".to_string(),
        permissions,
        config: std::collections::HashMap::new(),
    }
}

fn ext_with(name: &str, read: &[&str], write: &[&str]) -> ExtensionConfig {
    ext(
        name,
        Some(PermissionConfig {
            fs_read_allow: read.iter().map(ToString::to_string).collect(),
            fs_write_allow: write.iter().map(ToString::to_string).collect(),
            ..Default::default()
        }),
    )
}

/// The sandbox is built from the *union* across extensions, and `reload`
/// rebuilds it from the same rule. Nothing above this level notices if an
/// allow-list is dropped — the sandbox simply denies more than it should, and
/// the failure surfaces much later as an extension that cannot read a file it
/// was granted.
#[test]
fn fs_allow_lists_unions_every_extension() {
    let extensions = vec![
        ext_with("a", &["/read/a"], &["/write/a"]),
        // No permissions block at all: contributes nothing, rather than
        // shadowing what its neighbours granted.
        ext("no-permissions", None),
        ext_with("c", &["/read/c"], &["/write/c1", "/write/c2"]),
    ];

    let (read, write) = fs_allow_lists(&extensions);
    assert_eq!(read, vec!["/read/a", "/read/c"]);
    assert_eq!(write, vec!["/write/a", "/write/c1", "/write/c2"]);
}

#[test]
fn fs_allow_lists_of_nothing_is_empty() {
    let (read, write) = fs_allow_lists(&[]);
    assert!(read.is_empty() && write.is_empty());
}

#[test]
fn test_orchestrator_creation() {
    let config = Config::default();
    let dag = Arc::new(Mutex::new(Dag::new()));
    let orch = Orchestrator::new(config, "test_session".to_string(), dag, None);
    assert_eq!(*orch.session_id.lock(), "test_session");
}
