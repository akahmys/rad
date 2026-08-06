use super::Error;

#[test]
fn display_keeps_the_variant_so_it_survives_dispatch() {
    // Dispatch flattens this to a string, so the kind must be in the text or it
    // is lost at the boundary.
    assert_eq!(
        Error::invalid("no such node").to_string(),
        "Invalid: no such node"
    );
    assert_eq!(Error::io("stream closed").to_string(), "Io: stream closed");
    assert_eq!(
        Error::unsupported("subagent mode").to_string(),
        "Unsupported: subagent mode"
    );
}

#[test]
fn constructors_accept_both_str_and_string() {
    assert_eq!(Error::invalid("a"), Error::Invalid("a".to_string()));
    assert_eq!(Error::io(String::from("b")), Error::Io("b".to_string()));
}
