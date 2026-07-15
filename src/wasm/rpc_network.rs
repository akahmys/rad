use crate::ipc::{RasRpcCommand, Target, TimeoutPolicy};
use crate::wasm::rpc::RpcContext;

pub(crate) fn handle_network(
    cmd: &RasRpcCommand,
    ctx: &RpcContext<'_>,
) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::OpenHttpStream { url, headers, body } => {
            let final_url = if let Ok(test_port) = std::env::var("RAD_TEST_PORT") {
                url.replace("127.0.0.1:8080", &format!("127.0.0.1:{test_port}"))
            } else {
                url.clone()
            };
            crate::terminal::get_terminal().set_state(crate::terminal::TerminalState::Thinking);
            let stream_id = ctx.network.open_http_stream(
                &final_url,
                headers.clone(),
                body,
                ctx.event_tx.clone(),
                ctx.llm_timeout_policy.clone(),
            )?;
            serde_json::to_value(stream_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::SetStreamTimeoutPolicy { target, policy } => {
            handle_set_timeout(target, policy, ctx)
        }
        _ => Err("Unhandled RPC command in handle_network".to_string()),
    }
}

fn handle_set_timeout(
    target: &Target,
    policy: &TimeoutPolicy,
    ctx: &RpcContext<'_>,
) -> Result<serde_json::Value, String> {
    match target {
        Target::Llm => {
            let mut guard = ctx.llm_timeout_policy.lock();
            *guard = policy.clone();
        }
        Target::Process(pgid) => {
            let processes = ctx.active_processes.lock();
            if let Some(proc) = processes.get(pgid) {
                let mut guard = proc.timeout_policy.lock();
                *guard = policy.clone();
            } else {
                return Err(format!("Process with PGID {pgid} not found"));
            }
        }
    }
    Ok(serde_json::Value::Null)
}
