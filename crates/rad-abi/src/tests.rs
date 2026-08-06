use super::{ABI_VERSION, Manifest, ManifestError};

fn sample() -> Manifest {
    Manifest {
        name: "context".to_string(),
        version: "0.3.1".to_string(),
        abi: ABI_VERSION.to_string(),
        provides: vec!["context.optimize".to_string(), "context.digest".to_string()],
    }
}

#[test]
fn round_trips_through_json() {
    let original = sample();
    let parsed = Manifest::parse(&original.to_json().unwrap()).unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn rejects_a_mismatched_abi() {
    let mut m = sample();
    m.abi = "0.9".to_string();
    let json = serde_json::to_string(&m).unwrap();
    assert_eq!(
        Manifest::parse(&json),
        Err(ManifestError::AbiMismatch {
            expected: ABI_VERSION.to_string(),
            found: "0.9".to_string(),
        })
    );
}

#[test]
fn rejects_empty_identifying_fields() {
    for (field, mutate) in [
        (
            "name",
            (|m: &mut Manifest| m.name = String::new()) as fn(&mut Manifest),
        ),
        ("version", |m: &mut Manifest| m.version = "   ".to_string()),
    ] {
        let mut m = sample();
        mutate(&mut m);
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(Manifest::parse(&json), Err(ManifestError::Empty(field)));
    }
}

#[test]
fn rejects_malformed_json_with_the_reason_attached() {
    let err = Manifest::parse("{not json").unwrap_err();
    let ManifestError::Malformed(detail) = err else {
        panic!("expected Malformed, got {err:?}");
    };
    // The serde message must survive: a module author debugging a hand-written
    // manifest needs to know *what* failed to parse, not merely that it did.
    assert!(!detail.is_empty(), "parse error detail was discarded");
}

#[test]
fn a_module_may_provide_nothing() {
    // Valid: a module that only receives events (`handle("event.…")`) routes no
    // methods of its own. `provides` defaults rather than being required.
    let json = format!(r#"{{"name":"ui","version":"0.1.0","abi":"{ABI_VERSION}"}}"#);
    assert_eq!(
        Manifest::parse(&json).unwrap().provides,
        Vec::<String>::new()
    );
}

#[test]
fn abi_version_matches_the_wit_package_version() {
    // `wit/kernel/kernel.wit` declares `package rad:kernel@1.0.0`. These are
    // separate strings in separate files and nothing links them, so a change to
    // one silently diverging from the other is exactly what this pins.
    assert_eq!(ABI_VERSION, "1.0");
}
