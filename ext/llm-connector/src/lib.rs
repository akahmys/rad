#![deny(clippy::pedantic)]

#[allow(
    unsafe_op_in_unsafe_fn,
    clippy::same_length_and_capacity,
    clippy::pedantic
)]
mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/connector/llm-connector.wit",
        world: "llm-connector",
    });

    use super::connector::ConnectorImpl;
    export!(ConnectorImpl);
}

pub use bindings::*;

mod connector;
mod dialect;
mod event_stream;
mod serialize_types;
