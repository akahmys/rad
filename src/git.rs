use std::path::Path;
use std::process::Command;

fn run_git_cmd(workspace: &Path, args: &[&str]) -> Result<String, crate::error::UnifiedError> {
    let output = Command::new("git")
        .current_dir(workspace)
        .args(args)
        .output()
        .map_err(|e| crate::error::UnifiedError::l1(format!("Failed to execute git command: {e}"), "Git"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(crate::error::UnifiedError::l1(String::from_utf8_lossy(&output.stderr).trim().to_string(), "Git"))
    }
}

pub fn create_autopilot_branch(workspace: &Path, task_id: &str) -> Result<String, crate::error::UnifiedError> {
    let branch_name = format!("rad-autopilot-{}", task_id);
    // Check if branch already exists
    let exists = run_git_cmd(
        workspace,
        &[
            "show-ref",
            "--verify",
            &format!("refs/heads/{}", branch_name),
        ],
    )
    .is_ok();
    if exists {
        run_git_cmd(workspace, &["checkout", &branch_name])?;
    } else {
        run_git_cmd(workspace, &["checkout", "-b", &branch_name])?;
    }
    Ok(branch_name)
}

pub fn create_checkpoint(workspace: &Path, message: &str) -> Result<String, crate::error::UnifiedError> {
    // 1. Stage all changes
    run_git_cmd(workspace, &["add", "."])?;
    // 2. Commit with checkpoint message
    let commit_msg = format!("rad-autopilot: checkpoint {}", message);
    run_git_cmd(workspace, &["commit", "-m", &commit_msg, "--allow-empty"])?;
    // 3. Return the commit SHA
    get_head_sha(workspace)
}

pub fn rollback_to_checkpoint(workspace: &Path, target_commit: &str) -> Result<(), crate::error::UnifiedError> {
    // 1. Reset HEAD and index
    run_git_cmd(workspace, &["reset", "--hard", target_commit])?;
    // 2. Clean untracked files
    run_git_cmd(workspace, &["clean", "-fd"])?;
    Ok(())
}

pub fn get_head_sha(workspace: &Path) -> Result<String, crate::error::UnifiedError> {
    run_git_cmd(workspace, &["rev-parse", "HEAD"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn init_git_repo(path: &Path) -> Result<(), String> {
        run_git_cmd(path, &["init"])?;
        run_git_cmd(path, &["config", "user.name", "Test User"])?;
        run_git_cmd(path, &["config", "user.email", "test@example.com"])?;
        fs::write(path.join("initial.txt"), "hello").map_err(|e| e.to_string())?;
        run_git_cmd(path, &["add", "."])?;
        run_git_cmd(path, &["commit", "-m", "initial commit", "--allow-empty"])?;
        Ok(())
    }

    #[test]
    fn test_git_autopilot_flow() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let repo_path = temp_dir.path();
        init_git_repo(repo_path)?;

        let initial_sha = get_head_sha(repo_path)?;

        // 1. Create autopilot branch
        let branch = create_autopilot_branch(repo_path, "123")?;
        assert_eq!(branch, "rad-autopilot-123");

        // 2. Modify files and commit checkpoint
        fs::write(repo_path.join("initial.txt"), "hello modified")?;
        fs::write(repo_path.join("new.txt"), "new file")?;

        let checkpoint_sha = create_checkpoint(repo_path, "step_1")?;
        assert_ne!(initial_sha, checkpoint_sha);

        // Verify changes are in git history
        let log = run_git_cmd(repo_path, &["log", "-1", "--pretty=%s"])?;
        assert_eq!(log, "rad-autopilot: checkpoint step_1");

        // 3. Make breaking changes
        fs::write(repo_path.join("initial.txt"), "breaking change")?;
        fs::write(repo_path.join("broken.txt"), "broken file")?;

        // 4. Rollback to checkpoint
        rollback_to_checkpoint(repo_path, &checkpoint_sha)?;

        // Verify state is restored to checkpoint
        assert_eq!(
            fs::read_to_string(repo_path.join("initial.txt"))?,
            "hello modified"
        );
        assert_eq!(fs::read_to_string(repo_path.join("new.txt"))?, "new file");
        assert!(!repo_path.join("broken.txt").exists());
        assert_eq!(get_head_sha(repo_path)?, checkpoint_sha);

        Ok(())
    }
}
