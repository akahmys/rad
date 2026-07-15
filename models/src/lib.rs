//! Rad Core/Extension Shared IPC Models and Schemas
//!
//! This crate defines the frozen JSON-RPC API commands, request/response models,
//! and system-level events used for communication between the Rad Core (host)
//! and WebAssembly Extensions (guests).
//!
//! Any modifications here affect the ABI compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Timeout targets for connection/read timeouts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum Target {
    /// Applies timeout to LLM connections.
    Llm,
    /// Applies timeout to process execution with specified pgid.
    Process(String),
}

/// Dynamic or Infinite timeout configuration policies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum TimeoutPolicy {
    /// Set a custom connection/read timeout policy.
    Dynamic {
        heartbeat_timeout_ms: u64,
        max_silent_wait_ms: u64,
    },
    /// Disable timeout checks completely.
    Infinite,
}

/// Asynchronous events streamed from Rad Core to Wasm Extensions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum RasCoreEvent {
    /// Received a chunk of text from LLM SSE stream.
    HttpChunkReceived { chunk: String },
    /// Received an HTTP/connection error during LLM streaming.
    HttpErrorReceived { message: String },
    /// An extension requested a tool call.
    ToolCallRequested {
        call_id: String,
        name: String,
        args: serde_json::Value,
    },
    /// A bash process was successfully spawned.
    ProcessSpawned { pgid: String, pid: i32 },
    /// Standard output data received from spawned process.
    ProcessStdout {
        pgid: String,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    /// Standard error data received from spawned process.
    ProcessStderr {
        pgid: String,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    /// Spawend process group has exited.
    ProcessExited {
        pgid: String,
        exit_code: Option<i32>,
    },
    /// File changes detected in the sandbox workspace.
    FileChanged { path: PathBuf, change_type: String },
    /// Connection/stream read timed out.
    StreamTimeout { target: String, duration_ms: u64 },
    /// Received human user prompt input.
    HumanInputReceived { text: String },
    /// The autonomous execution loop has completed.
    TaskCompleted,
    /// Recovery event containing active processes to rehydrate Wasm guest state.
    Rehydrate {
        active_calls: Vec<PendingToolCallInfo>,
    },
    /// Message response received from external MCP server.
    McpResponse {
        call_id: String,
        name: String,
        message: String,
    },
}

/// Recovery metadata for a pending process/tool execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub pgid: Option<String>,
}

/// JSON-RPC Command list dispatched from Wasm Extension to Rad Core.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "method", content = "params")]
pub enum RasRpcCommand {
    /// Read binary data from a file in the workspace.
    FileRead { path: PathBuf },
    /// Write binary data to a file in the workspace.
    FileWrite {
        path: PathBuf,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    /// Apply unified diff patch to a file in the workspace.
    FileEditPatch { path: PathBuf, diff: String },
    /// Spawn an isolated background bash shell process.
    SpawnBashProcess { command: String },
    /// Create a new node in the execution history DAG.
    CreateNode {
        parent_id: String,
        node_type: String,
    },
    /// Update the text content of a DAG node.
    SetNodeText { node_id: String, text: String },
    /// Merge multiple DAG nodes into a single summary node.
    MergeNodes {
        node_ids: Vec<String>,
        summary_text: String,
    },
    /// Delete a node from the history DAG.
    DeleteNode { node_id: String },
    /// Snapshot the workspace files for recovery/rollback.
    TakeSnapshot {
        node_id: String,
        target_paths: Vec<PathBuf>,
    },
    /// Revert workspace files back to a snapshot node state.
    CheckoutSnapshot { node_id: String },
    /// Establish a streaming outbound HTTP connection (SSE).
    OpenHttpStream {
        url: String,
        headers: HashMap<String, String>,
        body: String,
    },
    /// Dynamically update timeout policies for targets.
    SetStreamTimeoutPolicy {
        target: Target,
        policy: TimeoutPolicy,
    },
    /// Print a message to the human terminal output.
    WriteStdout { text: String },
    /// Conclude the current task and await new instructions.
    CompleteTask,
    /// Fetch the semantic repository map of the workspace.
    GetRepoMap,
    /// Fetch combined tool definitions from the Tool Provider extension.
    GetTools,
    /// Delegate tool execution to the Tool Provider extension.
    ExecuteTool {
        call_id: String,
        name: String,
        arguments: String,
    },
    /// Open a file inside the workspace sandboxed environment.
    OpenFile { path: PathBuf, writeable: bool },
    /// Spawn a bash process inside the sandbox environment.
    OpenProcess { command: String },
    /// Retrieve the current execution DAG.
    GetDag,
    /// Ask for human approval via the terminal.
    AskHumanApproval { prompt: String },
    /// Report token usage for the LLM.
    ReportTokenUsage {
        prompt_tokens: u32,
        completion_tokens: u32,
    },
    /// Spawn an MCP server process.
    SpawnMcpServer {
        name: String,
        command: String,
        args: Vec<String>,
    },
    /// Send a request to an MCP server.
    SendMcpRequest { name: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RasRpcRequest {
    pub id: Option<String>,
    #[serde(flatten)]
    pub command: RasRpcCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RasRpcResponse {
    pub id: Option<String>,
    pub result: Result<serde_json::Value, String>,
}
pub mod dag;
pub use dag::{Dag, DagNode};
