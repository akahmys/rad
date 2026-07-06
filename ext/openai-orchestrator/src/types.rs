use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum Target {
    Llm,
    Process(i32),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum TimeoutPolicy {
    Dynamic {
        heartbeat_timeout_ms: u64,
        max_silent_wait_ms: u64,
    },
    Infinite,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "params")]
pub enum RasRpcCommand {
    FileRead {
        path: std::path::PathBuf,
    },
    FileWrite {
        path: std::path::PathBuf,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    FileEditPatch {
        path: std::path::PathBuf,
        diff: String,
    },
    SpawnBashProcess {
        command: String,
    },
    CreateNode {
        parent_id: String,
        node_type: String,
    },
    SetNodeText {
        node_id: String,
        text: String,
    },
    MergeNodes {
        node_ids: Vec<String>,
        summary_text: String,
    },
    DeleteNode {
        node_id: String,
    },
    TakeSnapshot {
        node_id: String,
        target_paths: Vec<std::path::PathBuf>,
    },
    CheckoutSnapshot {
        node_id: String,
    },
    OpenHttpStream {
        url: String,
        headers: HashMap<String, String>,
        body: String,
    },
    SetStreamTimeoutPolicy {
        target: Target,
        policy: TimeoutPolicy,
    },
    WriteStdout {
        text: String,
    },
    CompleteTask,
    GetDag,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RasRpcRequest {
    pub id: Option<String>,
    #[serde(flatten)]
    pub command: RasRpcCommand,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RasRpcResponse {
    pub id: Option<String>,
    pub result: Result<serde_json::Value, String>,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RasCoreEvent {
    HttpChunkReceived {
        chunk: String,
    },
    HumanInputReceived {
        text: String,
    },
    ProcessSpawned {
        pgid: i32,
        pid: i32,
    },
    ProcessStdout {
        pgid: i32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    ProcessStderr {
        pgid: i32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    ProcessExited {
        pgid: i32,
        exit_code: Option<i32>,
    },
    FileChanged {
        path: std::path::PathBuf,
        change_type: String,
    },
    StreamTimeout {
        target: String,
        duration_ms: u64,
    },
    TaskCompleted,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DagNode {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub node_type: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Dag {
    pub nodes: HashMap<String, DagNode>,
    pub current_node_id: Option<String>,
    pub next_node_index: usize,
}
