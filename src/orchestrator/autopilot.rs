use crate::git;
use std::path::Path;

pub(crate) fn run_verification_cmd(workspace: &Path, command_str: &str) -> bool {
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .current_dir(workspace)
            .args(["/C", command_str])
            .output()
    } else {
        std::process::Command::new("sh")
            .current_dir(workspace)
            .args(["-c", command_str])
            .output()
    };

    match output {
        Ok(out) => {
            let success = out.status.success();
            if !success {
                eprintln!("Verification FAILED: exit code = {:?}", out.status.code());
                eprintln!("stderr: {}", String::from_utf8_lossy(&out.stderr));
            }
            success
        }
        Err(e) => {
            eprintln!("Failed to execute verification command: {e}");
            false
        }
    }
}

pub(crate) fn setup_git_autopilot(
    workspace_path: &Path,
    session_id: &str,
) -> (bool, Option<String>) {
    let has_git = workspace_path.join(".git").exists();
    let mut initial_sha = None;
    if has_git {
        match git::create_autopilot_branch(workspace_path, session_id) {
            Ok(br) => {
                println!("Autopilot branch created/checked out: {br}");
                if let Ok(sha) = git::get_head_sha(workspace_path) {
                    initial_sha = Some(sha);
                }
            }
            Err(e) => {
                eprintln!("Failed to setup autopilot branch: {e}");
            }
        }
    }
    (has_git, initial_sha)
}
