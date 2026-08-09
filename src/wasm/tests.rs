//! What is left after AWU 973 took `verify_rpc` and the component harness that
//! existed to drive it.
//!
//! The three tests that lived here called `WasmRuntime::verify_rpc` against a
//! real `security-guard` component. Their subjects moved before the extension
//! did, per the rule stage 5 set:
//!
//! - `test_verify_rpc_blocked_command` → `tests/policy_module_tests.rs`'s
//!   `patterns_from_kernel_config_are_applied`, and end to end in
//!   `tests/policy_gate_tests.rs`'s `a_blocked_command_never_reaches_proc_spawn`.
//! - `test_verify_rpc_allowed` → `an_unconfigured_policy_allows_everything` and
//!   `a_configured_policy_allows_an_unmatched_call`.
//! - `test_verify_rpc_blocked_file` has **no equivalent, deliberately.** It
//!   drove `block_path_patterns` against `FileWrite`, and AWU 971 measured that
//!   list as changing no outcome on any end-to-end path before dropping it. It
//!   was the only thing still executing that branch, which is the point: a test
//!   whose subject exists only because the test drives it directly.
use std::fs;

#[test]
fn test_resolve_and_verify_path_helper() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    let workspace = workspace.canonicalize().unwrap();

    let safe_file = "safe.txt";
    let res = super::imports_rpc::resolve_and_verify_path(&workspace, safe_file);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), workspace.join("safe.txt"));

    let unsafe_traversal = "../unsafe.txt";
    let res = super::imports_rpc::resolve_and_verify_path(&workspace, unsafe_traversal);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Access denied"));
}
