use super::STATE;
use crate::types::RasRpcCommand;

// `handle_done` (the Done-event finalizer) and the inline-tool-call fallback
// parser live in sibling files to keep this one under the 300-line limit.
mod done;
mod inline_tool_calls;
pub(crate) use done::handle_done;

pub(crate) fn trim_large_output(text: &str) -> String {
    let max_chars = STATE
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|s| s.max_tool_output_chars))
        .unwrap_or(2000);

    if text.len() <= max_chars {
        return text.to_string();
    }

    let head_len = max_chars / 4;
    let tail_len = max_chars - head_len;

    let head: String = text.chars().take(head_len).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(tail_len)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    format!(
        "{head}\n\n[ERROR: THIS PART IS TRUNCATED. YOU MUST READ THIS RANGE SEPARATELY BEFORE EDITING ({} characters saved)]\n\n{tail}",
        text.len() - max_chars
    )
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ExtUnifiedError {
    pub level: String,
    pub payload: serde_json::Value,
}

pub(crate) fn call_host(command: RasRpcCommand) -> Result<serde_json::Value, String> {
    let wit_cmd = crate::radcomp::extension::types::RasRpcCommand::from(command);
    match crate::host_rpc(&wit_cmd) {
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
