use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum Target {
    Llm,
    Process(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum TimeoutPolicy {
    Dynamic {
        heartbeat_timeout_ms: u64,
        max_silent_wait_ms: u64,
    },
    Infinite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum RasCoreEvent {
    HttpChunkReceived {
        chunk: String,
    },
    ToolCallRequested {
        call_id: String,
        name: String,
        args: serde_json::Value,
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
        path: PathBuf,
        change_type: String, // "create" | "modify" | "remove"
    },
    StreamTimeout {
        target: String, // "llm" | "process_<pgid>"
        duration_ms: u64,
    },
    HumanInputReceived {
        text: String,
    },
    TaskCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "method", content = "params")]
pub enum RasRpcCommand {
    FileRead {
        path: PathBuf,
    },
    FileWrite {
        path: PathBuf,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    FileEditPatch {
        path: PathBuf,
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
        target_paths: Vec<PathBuf>,
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

pub struct IpcBridge<R, W> {
    reader: R,
    writer: W,
}

impl<R: BufRead, W: Write> IpcBridge<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    /// Read next RPC request from stream (1 JSON per line)
    ///
    /// # Errors
    ///
    /// Returns error if reading fails or JSON is invalid.
    pub fn read_request(&mut self) -> Result<Option<RasRpcRequest>, String> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).map_err(|e| format!("Failed to read line: {e}"))?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let req = serde_json::from_str(&line).map_err(|e| format!("Invalid JSON: {e}"))?;
        Ok(Some(req))
    }

    /// Write RPC response to stream
    ///
    /// # Errors
    ///
    /// Returns error if writing or flushing fails.
    pub fn write_response(&mut self, resp: &RasRpcResponse) -> Result<(), String> {
        let mut json = serde_json::to_vec(resp).map_err(|e| format!("Serialization error: {e}"))?;
        json.push(b'\n');
        self.writer.write_all(&json).map_err(|e| format!("Write error: {e}"))?;
        self.writer.flush().map_err(|e| format!("Flush error: {e}"))?;
        Ok(())
    }

    /// Write Core Event to stream
    ///
    /// # Errors
    ///
    /// Returns error if writing or flushing fails.
    pub fn write_event(&mut self, event: &RasCoreEvent) -> Result<(), String> {
        let mut json = serde_json::to_vec(event).map_err(|e| format!("Serialization error: {e}"))?;
        json.push(b'\n');
        self.writer.write_all(&json).map_err(|e| format!("Write error: {e}"))?;
        self.writer.flush().map_err(|e| format!("Flush error: {e}"))?;
        Ok(())
    }
}

/// Route specific physical events directly to Stdout/Stderr in real-time
///
/// # Errors
///
/// Returns error if standard stream writing or flushing fails.
pub fn route_event_to_terminal(event: &RasCoreEvent) -> Result<(), String> {
    match event {
        RasCoreEvent::ProcessStdout { data, .. } => {
            std::io::stdout().write_all(data).map_err(|e| format!("Stdout write error: {e}"))?;
            std::io::stdout().flush().map_err(|e| format!("Stdout flush error: {e}"))?;
        }
        RasCoreEvent::ProcessStderr { data, .. } => {
            std::io::stderr().write_all(data).map_err(|e| format!("Stderr write error: {e}"))?;
            std::io::stderr().flush().map_err(|e| format!("Stderr flush error: {e}"))?;
        }
        _ => {}
    }
    Ok(())
}

