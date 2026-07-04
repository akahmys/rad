use crate::config::PermissionConfig;
use crate::ipc::RasRpcCommand;

/// Verifies whether the given command is allowed based on the `PermissionConfig`.
///
/// # Errors
///
/// Returns an error if the capability check fails or the action is denied.
pub fn check_permissions(cmd: &RasRpcCommand, perms: &PermissionConfig) -> Result<(), String> {
    match cmd {

        RasRpcCommand::SpawnBashProcess { command } => {
            let exec_config = perms.execution.as_ref().ok_or("Execution permission denied: no execution config")?;
            if !exec_config.allow_bash {
                return Err("Execution permission denied: allow_bash is false".to_string());
            }

            for blocked in &exec_config.block_commands {
                if command.contains(blocked) {
                    return Err(format!("Execution permission denied: command contains blocked pattern '{blocked}'"));
                }
            }

            if !exec_config.allow_commands.is_empty() {
                let mut allowed = false;
                for allowed_cmd in &exec_config.allow_commands {
                    if command.starts_with(allowed_cmd) {
                        allowed = true;
                        break;
                    }
                }
                if !allowed {
                    return Err("Execution permission denied: command is not whitelisted".to_string());
                }
            }

            Ok(())
        }
        RasRpcCommand::OpenHttpStream { url, .. } => {
            let net_config = perms.network.as_ref().ok_or("Network permission denied: no network config")?;
            if !net_config.allow_network {
                return Err("Network permission denied: allow_network is false".to_string());
            }

            let mut domain = url.as_str();
            if let Some(stripped) = url.strip_prefix("https://") {
                domain = stripped;
            } else if let Some(stripped) = url.strip_prefix("http://") {
                domain = stripped;
            }
            if let Some(pos) = domain.find('/') {
                domain = &domain[..pos];
            }
            if let Some(pos) = domain.find(':') {
                domain = &domain[..pos];
            }

            if !net_config.allow_domains.is_empty() {
                let mut allowed = false;
                for allowed_domain in &net_config.allow_domains {
                    if domain == allowed_domain || domain.ends_with(&format!(".{allowed_domain}")) {
                        allowed = true;
                        break;
                    }
                }
                if !allowed {
                    return Err(format!("Network permission denied: domain '{domain}' is not whitelisted"));
                }
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
