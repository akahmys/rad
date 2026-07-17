#![deny(clippy::pedantic)]
#![allow(
    unsafe_op_in_unsafe_fn,
    clippy::too_many_lines,
    clippy::collapsible_if,
    clippy::uninlined_format_args,
    clippy::single_match_else,
    clippy::manual_assert
)]

wit_bindgen::generate!({
    path: "../../wit/rad.wit",
    world: "rad-orchestrator",
});

use rad_models::{RasCoreEvent as CoreCoreEvent, RasRpcCommand as CoreRpcCommand};

use self::radcomp::extension::types as wit;

mod conv;
mod llm;
mod orchestrator;
mod tool;
mod types;

struct ExtensionImpl;

impl Guest for ExtensionImpl {
    fn on_event(event: wit::RasCoreEvent) -> Result<(), String> {
        let core_event = CoreCoreEvent::from(event);
        orchestrator::handle_event(core_event)
    }
}

export!(ExtensionImpl);

pub(crate) fn call_host(command: CoreRpcCommand) -> Result<serde_json::Value, String> {
    let wit_cmd = wit::RasRpcCommand::from(command);
    match host_rpc(&wit_cmd) {
        Ok(json_str) => {
            if json_str.is_empty() || json_str == "null" {
                Ok(serde_json::Value::Null)
            } else {
                serde_json::from_str(&json_str)
                    .map_err(|e| format!("JSON parse error from host: {e}"))
            }
        }
        Err(err_msg) => Err(err_msg),
    }
}
