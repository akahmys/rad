//! Rad Core/Extension Shared IPC Models and Schemas
//!
//! This crate defines the frozen JSON-RPC API commands, request/response models,
//! and system-level events used for communication between the Rad Core (host)
//! and WebAssembly Extensions (guests).
//!
//! Any modifications here affect the ABI compatibility.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Timeout targets for connection/read timeouts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum Target {
    /// Applies timeout to LLM connections.
    Llm,
    /// Applies timeout to process execution with specified pgid.
    Process(i32),
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
    HttpChunkReceived {
        chunk: String,
    },
    /// Received an HTTP/connection error during LLM streaming.
    HttpErrorReceived {
        message: String,
    },
    /// An extension requested a tool call.
    ToolCallRequested {
        call_id: String,
        name: String,
        args: serde_json::Value,
    },
    /// A bash process was successfully spawned.
    ProcessSpawned {
        pgid: i32,
        pid: i32,
    },
    /// Standard output data received from spawned process.
    ProcessStdout {
        pgid: i32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    /// Standard error data received from spawned process.
    ProcessStderr {
        pgid: i32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    /// Spawend process group has exited.
    ProcessExited {
        pgid: i32,
        exit_code: Option<i32>,
    },
    /// File changes detected in the sandbox workspace.
    FileChanged {
        path: PathBuf,
        change_type: String,
    },
    /// Connection/stream read timed out.
    StreamTimeout {
        target: String,
        duration_ms: u64,
    },
    /// Received human user prompt input.
    HumanInputReceived {
        text: String,
    },
    /// The autonomous execution loop has completed.
    TaskCompleted,
    /// Recovery event containing active processes to rehydrate Wasm guest state.
    Rehydrate {
        active_calls: Vec<PendingToolCallInfo>,
    },
}

/// Recovery metadata for a pending process/tool execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub pgid: Option<i32>,
}

/// JSON-RPC Command list dispatched from Wasm Extension to Rad Core.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "method", content = "params")]
pub enum RasRpcCommand {
    /// Read binary data from a file in the workspace.
    FileRead {
        path: PathBuf,
    },
    /// Write binary data to a file in the workspace.
    FileWrite {
        path: PathBuf,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    /// Apply unified diff patch to a file in the workspace.
    FileEditPatch {
        path: PathBuf,
        diff: String,
    },
    /// Spawn an isolated background bash shell process.
    SpawnBashProcess {
        command: String,
    },
    /// Create a new node in the execution history DAG.
    CreateNode {
        parent_id: String,
        node_type: String,
    },
    /// Update the text content of a DAG node.
    SetNodeText {
        node_id: String,
        text: String,
    },
    /// Merge multiple DAG nodes into a single summary node.
    MergeNodes {
        node_ids: Vec<String>,
        summary_text: String,
    },
    /// Delete a node from the history DAG.
    DeleteNode {
        node_id: String,
    },
    /// Snapshot the workspace files for recovery/rollback.
    TakeSnapshot {
        node_id: String,
        target_paths: Vec<PathBuf>,
    },
    /// Revert workspace files back to a snapshot node state.
    CheckoutSnapshot {
        node_id: String,
    },
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
    WriteStdout {
        text: String,
    },
    /// Conclude the current task and await new instructions.
    CompleteTask,
    /// Fetch the current workspace history DAG.
    GetDag,
    /// Interactively prompt the human user for approval.
    AskHumanApproval {
        prompt: String,
    },
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DagNode {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub node_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dag {
    pub nodes: HashMap<String, DagNode>,
    pub current_node_id: Option<String>,
    pub next_node_index: usize,
}

impl Dag {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_node(&mut self, parent_id: &str, node_type: &str) -> Result<String, String> {
        let mut parent_ids = Vec::new();
        if !parent_id.is_empty() {
            if !self.nodes.contains_key(parent_id) {
                return Err(format!("Parent node '{parent_id}' not found"));
            }
            parent_ids.push(parent_id.to_string());
        }

        let new_id = format!("node_{}", self.next_node_index);
        self.next_node_index += 1;

        let node = DagNode {
            id: new_id.clone(),
            parent_ids,
            node_type: node_type.to_string(),
            text: String::new(),
        };

        self.nodes.insert(new_id.clone(), node);
        self.current_node_id = Some(new_id.clone());

        Ok(new_id)
    }

    pub fn set_node_text(&mut self, node_id: &str, text: &str) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or_else(|| format!("Node '{node_id}' not found"))?;
        node.text = text.to_string();
        Ok(())
    }

    pub fn merge_nodes(&mut self, node_ids: &[String], summary_text: &str) -> Result<String, String> {
        if node_ids.is_empty() {
            return Err("Cannot merge empty list of nodes".to_string());
        }

        let mut collected_parents = HashSet::new();
        let target_set: HashSet<&String> = node_ids.iter().collect();

        for id in node_ids {
            let node = self.nodes.get(id).ok_or_else(|| format!("Node '{id}' not found"))?;
            for parent in &node.parent_ids {
                if !target_set.contains(parent) {
                    collected_parents.insert(parent.clone());
                }
            }
        }

        let new_id = format!("node_{}", self.next_node_index);
        self.next_node_index += 1;

        let merge_node = DagNode {
            id: new_id.clone(),
            parent_ids: collected_parents.into_iter().collect(),
            node_type: "merge".to_string(),
            text: summary_text.to_string(),
        };

        self.redirect_children(node_ids, &new_id);

        for id in node_ids {
            self.nodes.remove(id);
        }

        self.nodes.insert(new_id.clone(), merge_node);
        self.current_node_id = Some(new_id.clone());

        Ok(new_id)
    }

    fn redirect_children(&mut self, merged_ids: &[String], new_parent_id: &str) {
        let target_set: HashSet<&String> = merged_ids.iter().collect();
        for node in self.nodes.values_mut() {
            if target_set.contains(&node.id) {
                continue;
            }
            for parent in &mut node.parent_ids {
                if target_set.contains(parent) {
                    *parent = new_parent_id.to_string();
                }
            }
            node.parent_ids.sort();
            node.parent_ids.dedup();
        }
    }

    pub fn delete_node(&mut self, node_id: &str) -> Result<(), String> {
        if !self.nodes.contains_key(node_id) {
            return Err(format!("Node '{node_id}' not found"));
        }

        self.nodes.remove(node_id);

        for node in self.nodes.values_mut() {
            node.parent_ids.retain(|x| x != node_id);
        }

        if self.current_node_id.as_deref() == Some(node_id) {
            self.current_node_id = None;
        }

        Ok(())
    }
}
