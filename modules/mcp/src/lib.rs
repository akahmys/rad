//! MCP servers as a kernel module.
//!
//! Ported from `ext/mcp-tool-provider`. Three things shrink rather than move,
//! and all three were artefacts of the old boundary — see `client.rs` for the
//! configuration and spawning cases, and `execute` below for the third: the
//! extension returned every tool result by shelling out to `echo -n '<text>'`,
//! escaping quotes on the way, because its WIT export had to return an
//! `execution-handle`. A module returns the string.
#![deny(clippy::pedantic)]

mod client;
mod gate;
mod testmode;
mod tool;
mod transport;

use rad_sdk::Error;

#[derive(serde::Deserialize)]
pub struct ListReq {}

#[derive(serde::Serialize)]
pub struct ListRes {
    pub tools: Vec<serde_json::Value>,
}

#[derive(serde::Deserialize)]
pub struct CallReq {
    pub name: String,
    #[serde(default)]
    pub arguments: String,
}

#[derive(serde::Serialize)]
pub struct CallRes {
    pub content: String,
}

fn list(_req: ListReq) -> Result<ListRes, Error> {
    if let Some(tools) = testmode::tools() {
        return Ok(ListRes { tools });
    }
    let tools = tool::list().map_err(Error::invalid)?;
    Ok(ListRes {
        tools: tools
            .into_iter()
            .map(|t| serde_json::to_value(t).unwrap_or(serde_json::Value::Null))
            .collect(),
    })
}

/// Asks `policy` first, and before `testmode` — the synthetic tools run real
/// commands through `bash`, so a gate that sat below them would leave the one
/// path where the model's text reaches `argv` unguarded.
fn call(req: CallReq) -> Result<CallRes, Error> {
    let CallReq { name, arguments } = req;
    gate::check(&name, &arguments).map_err(Error::invalid)?;
    if let Some(content) = testmode::call(&name, &arguments) {
        return Ok(CallRes { content });
    }
    tool::call(&name, &arguments)
        .map(|content| CallRes { content })
        .map_err(Error::invalid)
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "mcp",
    version: "0.1.0",
    methods: {
        "mcp.tools.list" => list,
        "mcp.tools.call" => call,
    }
}
