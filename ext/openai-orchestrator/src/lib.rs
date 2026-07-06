#![deny(clippy::pedantic)]
#![allow(
    clippy::manual_let_else,
    clippy::same_length_and_capacity,
    clippy::not_unsafe_ptr_arg_deref,
    clippy::unnecessary_wraps,
    clippy::missing_safety_doc,
    clippy::manual_strip,
    clippy::collapsible_if
)]

mod types;
use types::{RasRpcCommand, RasCoreEvent};

#[cfg(not(test))]
use types::{RasRpcRequest, RasRpcResponse};

#[cfg(test)]
use types::Dag;

#[cfg(test)]
use std::collections::HashMap;

#[cfg(not(test))]
unsafe extern "C" {
    fn rad_host_rpc(ptr: *const u8, len: usize) -> u64;
}

#[cfg(test)]
mod tests;

mod orchestrator;
mod tool;

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
pub(crate) fn call_host(command: RasRpcCommand) -> Result<serde_json::Value, String> {
    let cmd = command;
    match cmd {
        RasRpcCommand::GetDag => {
            let dag = Dag {
                nodes: HashMap::new(),
                current_node_id: None,
                next_node_index: 0,
            };
            serde_json::to_value(&dag).map_err(|e| e.to_string())
        }
        RasRpcCommand::CreateNode { .. } => {
            Ok(serde_json::json!("node_0"))
        }
        RasRpcCommand::SetNodeText { .. } => {
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::OpenHttpStream { .. } => {
            Ok(serde_json::json!("http_stream_mock_id"))
        }
        _ => Ok(serde_json::Value::Null),
    }
}

#[cfg(not(test))]
pub(crate) fn call_host(command: RasRpcCommand) -> Result<serde_json::Value, String> {
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
    
    if let Err(e) = orchestrator::handle_event(event) {
        eprintln!("Error in handle_event: {e}");
        let err_str = e.to_string();
        let err_len = err_str.len();
        let err_ptr = alloc(err_len as i32);
        if !err_ptr.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(err_str.as_ptr(), err_ptr, err_len);
            }
            let ptr_u64 = err_ptr as u64;
            let len_u64 = err_len as u64;
            return (ptr_u64 << 32) | len_u64;
        }
        return 3;
    }
    
    0
}
