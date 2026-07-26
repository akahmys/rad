#![deny(clippy::pedantic)]
#![allow(
    unsafe_op_in_unsafe_fn,
    clippy::needless_pass_by_value,
    clippy::same_length_and_capacity,
    clippy::collapsible_if,
    clippy::uninlined_format_args,
    clippy::cast_possible_truncation,
    clippy::manual_strip
)]

wit_bindgen::generate!({
    path: "../../wit/llm-connector.wit",
    world: "llm-connector",
});

// Request wire types, SSE event-stream parsing, and the Guest/ConnectorImpl
// implementation live in sibling files to keep this one under the 300-line
// limit.
mod connector;
mod event_stream;
mod serialize_types;

// export! only accepts a plain identifier, not a path, so bring the type
// into scope here first.
use connector::ConnectorImpl;

export!(ConnectorImpl);
