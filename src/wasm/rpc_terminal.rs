use crate::ipc::RasRpcCommand;
use crate::wasm::rpc::RpcContext;

pub(crate) fn handle_terminal(
    cmd: &RasRpcCommand,
    _ctx: &RpcContext<'_>,
) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::WriteStdout { text } => {
            crate::terminal::get_terminal().write_llm_token(text);
            Ok(serde_json::Value::Null)
        }
        _ => Err("Unhandled RPC command in handle_terminal".to_string()),
    }
}
