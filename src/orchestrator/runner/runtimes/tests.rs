use super::*;
use crate::config::{Config, CoreConfig, ExtensionConfig};
use crate::dag::Dag;

/// `find_extension_arc_by_role` resolves by declared `role`, not the
/// extension's registered name — so a user-supplied replacement extension
/// under any name still gets found, as long as it declares the right role.
#[test]
fn test_find_extension_arc_by_role_resolves_by_role_not_name() {
    let wasm_path = std::path::Path::new("target/wasm32-wasip2/debug/context_tools.wasm");
    if !wasm_path.exists() {
        // Unit test, not meant to require the full build pipeline; the
        // real behavior is also covered end-to-end by
        // `tests/context_tools_tests.rs` and `tests/multi_extension_tests.rs`.
        return;
    }

    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let snapshots = temp.path().join("snapshots");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&snapshots).unwrap();

    let config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: temp.path().join("logs").to_string_lossy().to_string(),
            ..Default::default()
        },
        extensions: vec![ExtensionConfig {
            name: "my-custom-compactor".to_string(),
            source: wasm_path.to_string_lossy().to_string(),
            enabled: true,
            role: "context-tools".to_string(),
            permissions: None,
            config: HashMap::new(),
        }],
        ..Default::default()
    };

    let dag = Arc::new(Mutex::new(Dag::new()));
    let orch = Arc::new(Orchestrator::new(
        config,
        "test_session".to_string(),
        dag,
        None,
    ));

    let (tx, _rx) = std::sync::mpsc::channel();
    orch.get_or_init_runtimes(&tx).unwrap();

    assert!(
        orch.find_extension_arc_by_role("context-tools").is_some(),
        "should resolve the extension named 'my-custom-compactor' by its declared role"
    );
    assert!(
        orch.find_extension_arc_by_role("nonexistent-role")
            .is_none()
    );
}
