#![deny(clippy::pedantic)]

use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[cfg(not(test))]
unsafe extern "C" {
    fn rad_host_rpc(ptr: *const u8, len: usize) -> u64;
}

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
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

struct OrchestratorState {
    messages: Vec<Message>,
    stream_buffer: String,
}

static STATE: Mutex<Option<OrchestratorState>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    let size = match usize::try_from(size) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: i32) {
    let size = match usize::try_from(size) {
        Ok(s) => s,
        Err(_) => return,
    };
    if !ptr.is_null() && size > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, size, size);
        }
    }
}

#[cfg(test)]
fn call_host(_command: RasRpcCommand) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!("http_stream_mock_id"))
}

#[cfg(not(test))]
fn call_host(command: RasRpcCommand) -> Result<serde_json::Value, String> {
    let request = RasRpcRequest {
        id: Some("wasm_call".to_string()),
        command,
    };
    let req_bytes = serde_json::to_vec(&request).map_err(|e| format!("JSON serialize error: {e}"))?;
    
    unsafe {
        let ret = rad_host_rpc(req_bytes.as_ptr(), req_bytes.len());
        let ptr = (ret >> 32) as *mut u8;
        let len = (ret & 0xFFFF_FFFF) as usize;
        if ptr.is_null() || len == 0 {
            return Err("Host RPC returned null or empty".to_string());
        }
        let resp_bytes = Vec::from_raw_parts(ptr, len, len);
        let resp: RasRpcResponse = serde_json::from_slice(&resp_bytes).map_err(|e| format!("JSON deserialize error: {e}"))?;
        resp.result
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rad_on_event(ptr: *const u8, len: i32) -> u64 {
    let len = match usize::try_from(len) {
        Ok(l) => l,
        Err(_) => return 1,
    };
    if ptr.is_null() || len == 0 {
        return 1;
    }
    
    let event_bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let event: RasCoreEvent = match serde_json::from_slice(event_bytes) {
        Ok(e) => e,
        Err(_) => return 2,
    };
    
    if let Err(e) = handle_event(event) {
        eprintln!("Error in handle_event: {e}");
        return 3;
    }
    
    0
}

fn handle_event(event: RasCoreEvent) -> Result<(), String> {
    match event {
        RasCoreEvent::HumanInputReceived { text } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            let state = state_guard.get_or_insert_with(|| OrchestratorState {
                messages: Vec::new(),
                stream_buffer: String::new(),
            });
            
            state.messages.push(Message {
                role: "user".to_string(),
                content: text,
            });
            
            trigger_llm_stream(state)?;
        }
        RasCoreEvent::HttpChunkReceived { chunk } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            if let Some(state) = state_guard.as_mut() {
                state.stream_buffer.push_str(&chunk);
                process_sse_buffer(state)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn trigger_llm_stream(state: &OrchestratorState) -> Result<(), String> {
    let req = ChatCompletionsRequest {
        model: "qwen".to_string(),
        messages: state.messages.clone(),
        stream: true,
    };
    let body = serde_json::to_string(&req).map_err(|e| format!("JSON serialize error: {e}"))?;
    
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    
    let url = "http://127.0.0.1:8080/v1/chat/completions".to_string();
    
    call_host(RasRpcCommand::OpenHttpStream {
        url,
        headers,
        body,
    })?;
    
    Ok(())
}

fn process_sse_buffer(state: &mut OrchestratorState) -> Result<(), String> {
    while let Some(pos) = state.stream_buffer.find('\n') {
        let line = state.stream_buffer[..pos].trim().to_string();
        state.stream_buffer = state.stream_buffer[pos + 1..].to_string();
        
        if line.is_empty() {
            continue;
        }
        
        if line.starts_with("data:") {
            let data_str = line["data:".len()..].trim();
            if data_str == "[DONE]" {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: "\n".to_string(),
                })?;
                let _ = call_host(RasRpcCommand::CompleteTask)?;
                break;
            }
            
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                if let Some(content) = val.pointer("/choices/0/delta/content").and_then(|v| v.as_str()) {
                    let _ = call_host(RasRpcCommand::WriteStdout {
                        text: content.to_string(),
                    })?;
                }
            }
        }
    }
    Ok(())
}
