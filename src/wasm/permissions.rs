use crate::config::PermissionConfig;
use crate::ipc::RasRpcCommand;
use std::path::{Path, PathBuf};

fn clean_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if let Some(last) = components.last() {
                    match last {
                        std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
                        _ => {
                            components.pop();
                        }
                    }
                }
            }
            std::path::Component::Normal(c) => {
                components.push(std::path::Component::Normal(c));
            }
            std::path::Component::CurDir => {}
            std::path::Component::Prefix(p) => {
                components.push(std::path::Component::Prefix(p));
            }
            std::path::Component::RootDir => {
                components.clear();
                components.push(std::path::Component::RootDir);
            }
        }
    }
    components.iter().collect::<PathBuf>()
}

fn has_path_permission(path: &Path, allowed_patterns: &[String], workspace: &Path) -> bool {
    let absolute_target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    };

    let cleaned_target = clean_path(&absolute_target);

    let canonical_target = match cleaned_target.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            let mut current = cleaned_target.as_path();
            while !current.exists() {
                if let Some(parent) = current.parent() {
                    current = parent;
                } else {
                    break;
                }
            }
            if current.exists() {
                if let Ok(canonical_parent) = current.canonicalize() {
                    if let Ok(relative) = cleaned_target.strip_prefix(current) {
                        canonical_parent.join(relative)
                    } else {
                        cleaned_target.clone()
                    }
                } else {
                    cleaned_target.clone()
                }
            } else {
                cleaned_target.clone()
            }
        }
    };

    let canonical_workspace = match workspace.canonicalize() {
        Ok(p) => p,
        Err(_) => clean_path(workspace),
    };

    for pattern in allowed_patterns {
        if pattern == "*" {
            return true;
        }
        let pattern_buf = PathBuf::from(pattern);
        let absolute_pattern = if pattern_buf.is_absolute() {
            pattern_buf
        } else {
            workspace.join(&pattern_buf)
        };
        let cleaned_pattern = clean_path(&absolute_pattern);
        let canonical_pattern = match cleaned_pattern.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                let mut current = cleaned_pattern.as_path();
                while !current.exists() {
                    if let Some(parent) = current.parent() {
                        current = parent;
                    } else {
                        break;
                    }
                }
                if current.exists() {
                    if let Ok(canonical_parent) = current.canonicalize() {
                        if let Ok(relative) = cleaned_pattern.strip_prefix(current) {
                            canonical_parent.join(relative)
                        } else {
                            cleaned_pattern.clone()
                        }
                    } else {
                        cleaned_pattern.clone()
                    }
                } else {
                    cleaned_pattern.clone()
                }
            }
        };

        if canonical_target.starts_with(&canonical_pattern)
            && canonical_target.starts_with(&canonical_workspace)
        {
            return true;
        }
    }
    false
}

/// Verifies whether the given command is allowed based on the `PermissionConfig`.
///
/// # Errors
///
/// Returns an error if the capability check fails or the action is denied.
pub fn check_permissions(
    cmd: &RasRpcCommand,
    perms: &PermissionConfig,
    workspace: &Path,
) -> Result<(), String> {
    match cmd {
        RasRpcCommand::SpawnBashProcess { command } => {
            let exec_config = perms
                .execution
                .as_ref()
                .ok_or("Execution permission denied: no execution config")?;
            if !exec_config.allow_bash {
                return Err("Execution permission denied: allow_bash is false".to_string());
            }

            for blocked in &exec_config.block_commands {
                if command.contains(blocked) {
                    return Err(format!(
                        "Execution permission denied: command contains blocked pattern '{blocked}'"
                    ));
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
                    return Err(
                        "Execution permission denied: command is not whitelisted".to_string()
                    );
                }
            }

            Ok(())
        }
        RasRpcCommand::OpenProcess { command } => check_permissions(
            &RasRpcCommand::SpawnBashProcess {
                command: command.clone(),
            },
            perms,
            workspace,
        ),
        RasRpcCommand::OpenFile { path, writeable } => {
            let allowed = if *writeable {
                has_path_permission(path, &perms.fs_write_allow, workspace)
            } else {
                has_path_permission(path, &perms.fs_read_allow, workspace)
            };
            if !allowed {
                return Err(format!(
                    "File permission denied: access to '{}' is not allowed",
                    path.display()
                ));
            }
            Ok(())
        }
        RasRpcCommand::FileRead { path } => {
            if !has_path_permission(path, &perms.fs_read_allow, workspace) {
                return Err(format!(
                    "File permission denied: read access to '{}' is not allowed",
                    path.display()
                ));
            }
            Ok(())
        }
        RasRpcCommand::FileWrite { path, .. } | RasRpcCommand::FileEditPatch { path, .. } => {
            if !has_path_permission(path, &perms.fs_write_allow, workspace) {
                return Err(format!(
                    "File permission denied: write access to '{}' is not allowed",
                    path.display()
                ));
            }
            Ok(())
        }
        RasRpcCommand::OpenHttpStream { url, .. } => {
            let net_config = perms
                .network
                .as_ref()
                .ok_or("Network permission denied: no network config")?;
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
                    if allowed_domain == "*"
                        || domain == allowed_domain
                        || domain.ends_with(&format!(".{allowed_domain}"))
                    {
                        allowed = true;
                        break;
                    }
                }
                if !allowed {
                    return Err(format!(
                        "Network permission denied: domain '{domain}' is not whitelisted"
                    ));
                }
            }

            Ok(())
        }
        RasRpcCommand::SpawnMcpServer { name, .. } | RasRpcCommand::SendMcpRequest { name, .. } => {
            if !perms.allowed_mcp_servers.contains(name) {
                return Err(format!(
                    "MCP permission denied: server '{name}' is not whitelisted"
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
