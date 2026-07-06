use std::path::Path;
use std::process::Command;

fn run_git_cmd(workspace: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(workspace)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute git command: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn create_autopilot_branch(workspace: &Path, task_id: &str) -> Result<String, String> {
    let branch_name = format!("rad-autopilot-{}", task_id);
    // Check if branch already exists
    let exists = run_git_cmd(workspace, &["show-ref", "--verify", &format!("refs/heads/{}", branch_name)]).is_ok();
    if exists {
        run_git_cmd(workspace, &["checkout", &branch_name])?;
    } else {
        run_git_cmd(workspace, &["checkout", "-b", &branch_name])?;
    }
    Ok(branch_name)
}

pub fn create_checkpoint(workspace: &Path, message: &str) -> Result<String, String> {
    // 1. Stage all changes
    run_git_cmd(workspace, &["add", "."])?;
    // 2. Commit with checkpoint message
    let commit_msg = format!("rad-autopilot: checkpoint {}", message);
    run_git_cmd(workspace, &["commit", "-m", &commit_msg, "--allow-empty"])?;
    // 3. Return the commit SHA
    get_head_sha(workspace)
}

pub fn rollback_to_checkpoint(workspace: &Path, target_commit: &str) -> Result<(), String> {
    // 1. Reset HEAD and index
    run_git_cmd(workspace, &["reset", "--hard", target_commit])?;
    // 2. Clean untracked files
    run_git_cmd(workspace, &["clean", "-fd"])?;
    Ok(())
}

pub fn get_head_sha(workspace: &Path) -> Result<String, String> {
    run_git_cmd(workspace, &["rev-parse", "HEAD"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn init_git_repo(path: &Path) {
        run_git_cmd(path, &["init"]).unwrap();
        run_git_cmd(path, &["config", "user.name", "Test User"]).unwrap();
        run_git_cmd(path, &["config", "user.email", "test@example.com"]).unwrap();
        fs::write(path.join("initial.txt"), "hello").unwrap();
        run_git_cmd(path, &["add", "."]).unwrap();
        run_git_cmd(path, &["commit", "-m", "initial commit"]).unwrap();
    }

    #[test]
    fn test_git_autopilot_flow() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);

        let initial_sha = get_head_sha(repo_path).unwrap();

        // 1. Create autopilot branch
        let branch = create_autopilot_branch(repo_path, "123").unwrap();
        assert_eq!(branch, "rad-autopilot-123");

        // 2. Modify files and commit checkpoint
        fs::write(repo_path.join("initial.txt"), "hello modified").unwrap();
        fs::write(repo_path.join("new.txt"), "new file").unwrap();

        let checkpoint_sha = create_checkpoint(repo_path, "step_1").unwrap();
        assert_ne!(initial_sha, checkpoint_sha);

        // Verify changes are in git history
        let log = run_git_cmd(repo_path, &["log", "-1", "--pretty=%s"]).unwrap();
        assert_eq!(log, "rad-autopilot: checkpoint step_1");

        // 3. Make breaking changes
        fs::write(repo_path.join("initial.txt"), "breaking change").unwrap();
        fs::write(repo_path.join("broken.txt"), "broken file").unwrap();

        // 4. Rollback to checkpoint
        rollback_to_checkpoint(repo_path, &checkpoint_sha).unwrap();

        // Verify state is restored to checkpoint
        assert_eq!(fs::read_to_string(repo_path.join("initial.txt")).unwrap(), "hello modified");
        assert_eq!(fs::read_to_string(repo_path.join("new.txt")).unwrap(), "new file");
        assert!(!repo_path.join("broken.txt").exists());
        assert_eq!(get_head_sha(repo_path).unwrap(), checkpoint_sha);
    }
}
