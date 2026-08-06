use super::{Registry, RegistryError};
use rad_abi::{ABI_VERSION, Manifest};

fn manifest(name: &str, provides: &[&str]) -> Manifest {
    Manifest {
        name: name.to_string(),
        version: "0.1.0".to_string(),
        abi: ABI_VERSION.to_string(),
        provides: provides.iter().map(ToString::to_string).collect(),
    }
}

#[test]
fn routes_methods_to_the_module_that_provides_them() {
    let mut r = Registry::new();
    r.register(manifest("context", &["context.optimize"]))
        .unwrap();
    r.register(manifest("skills", &["skills.list"])).unwrap();
    assert_eq!(r.route("context.optimize"), Some("context"));
    assert_eq!(r.route("skills.list"), Some("skills"));
    assert_eq!(r.route("nobody.provides.this"), None);
}

#[test]
fn a_method_claimed_twice_is_a_startup_error_naming_both() {
    // §3.6.8: never an implicit first-wins. Which module serves a call must not
    // depend on the order entries happen to sit in a config file.
    let mut r = Registry::new();
    r.register(manifest("a", &["shared.method"])).unwrap();
    let err = r.register(manifest("b", &["shared.method"])).unwrap_err();
    assert_eq!(
        err,
        RegistryError::DuplicateMethod {
            method: "shared.method".to_string(),
            existing: "a".to_string(),
            incoming: "b".to_string(),
        }
    );
    let text = err.to_string();
    assert!(text.contains("shared.method") && text.contains("'a'") && text.contains("'b'"));
}

#[test]
fn a_rejected_module_leaves_no_partial_routes() {
    // The conflict is on the *second* method, so a naive implementation would
    // already have routed the first one to a module it then refused.
    let mut r = Registry::new();
    r.register(manifest("a", &["a.one"])).unwrap();
    r.register(manifest("b", &["b.fresh", "a.one"]))
        .unwrap_err();
    assert_eq!(
        r.route("b.fresh"),
        None,
        "half-registered a rejected module"
    );
    assert_eq!(r.route("a.one"), Some("a"));
    assert_eq!(r.module_names(), vec!["a"]);
}

#[test]
fn duplicate_module_names_are_rejected() {
    let mut r = Registry::new();
    r.register(manifest("dup", &["x.one"])).unwrap();
    assert_eq!(
        r.register(manifest("dup", &["x.two"])).unwrap_err(),
        RegistryError::DuplicateModule("dup".to_string())
    );
}

#[test]
fn a_module_providing_nothing_still_registers() {
    // Event-only modules route no methods but must still be addressable, since
    // the kernel posts to them by name.
    let mut r = Registry::new();
    r.register(manifest("ui", &[])).unwrap();
    assert_eq!(r.module_names(), vec!["ui"]);
    assert!(r.manifest("ui").is_some());
}

#[test]
fn empty_registry_reports_itself_empty() {
    assert!(Registry::new().is_empty());
}
