#![deny(clippy::pedantic)]
#![allow(unsafe_op_in_unsafe_fn)]


wit_bindgen::generate!({
    path: "../../wit/rad.wit",
    world: "rad-extension",
});

use rad_models::{RasRpcCommand as CoreRpcCommand, RasCoreEvent as CoreCoreEvent};

#[cfg(test)]
use rad_models::Dag;
#[cfg(test)]
use std::collections::HashMap;

use self::radcomp::extension::types as wit;

mod types;
mod orchestrator;
mod tool;
mod sse;
mod llm;
pub mod mcp_client;
pub mod tool_runner;
mod conv;
mod security;
#[cfg(test)]
mod tests;


struct ExtensionImpl;


impl Guest for ExtensionImpl {
    fn on_event(event: wit::RasCoreEvent) -> Result<(), String> {
        let core_event = CoreCoreEvent::from(event);
        orchestrator::handle_event(core_event)
    }

    fn verify_rpc(command: wit::RasRpcCommand) -> bool {
        let rpc_cmd = CoreRpcCommand::from(command);
        security::verify_rpc(&rpc_cmd)
    }
}


export!(ExtensionImpl);

#[cfg(test)]
pub(crate) fn call_host(command: CoreRpcCommand) -> Result<serde_json::Value, String> {
    let cmd = command;
    match cmd {
        CoreRpcCommand::GetDag => {
            let dag = Dag {
                nodes: HashMap::new(),
                current_node_id: None,
                next_node_index: 0,
            };
            serde_json::to_value(&dag).map_err(|e| e.to_string())
        }
        CoreRpcCommand::CreateNode { .. } => {
            Ok(serde_json::json!("node_0"))
        }
        CoreRpcCommand::SetNodeText { .. } => {
            Ok(serde_json::Value::Null)
        }
        CoreRpcCommand::OpenHttpStream { .. } => {
            Ok(serde_json::json!("http_stream_mock_id"))
        }
        _ => Ok(serde_json::Value::Null),
    }
}

#[cfg(not(test))]
pub(crate) fn call_host(command: CoreRpcCommand) -> Result<serde_json::Value, String> {
    let wit_cmd = wit::RasRpcCommand::from(command);
    if !ExtensionImpl::verify_rpc(wit_cmd.clone()) {
        return Err("Operation rejected by security extension".to_string());
    }
    match host_rpc(&wit_cmd) {
        Ok(json_str) => {
            if json_str.is_empty() || json_str == "null" {
                Ok(serde_json::Value::Null)
            } else {
                serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error from host: {e}"))
            }
        }
        Err(err_msg) => Err(err_msg),
    }
}


