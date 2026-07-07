use rad_models::RasRpcCommand as CoreRpcCommand;

/// Verifies whether the requested RPC command is allowed under the current security policy.
pub fn verify_rpc(command: &CoreRpcCommand) -> bool {
    match command {
        CoreRpcCommand::FileWrite { path, .. } => {
            if path.to_string_lossy().contains("blocked.txt") {
                return false;
            }
        }
        CoreRpcCommand::SpawnBashProcess { command } => {
            if command.contains("blocked_command") {
                return false;
            }
        }
        _ => {}
    }
    true
}
