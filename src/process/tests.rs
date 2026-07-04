use super::*;
use nix::unistd::getpgid;
use std::time::Duration;

#[test]
fn test_spawn_and_pgid_isolation() {
    let manager = ProcessManager::new();
    let mut proc = manager.spawn_bash_process("sleep 10", None).unwrap();

    let expected_pgid = proc.pgid();
    let actual_pgid = getpgid(Some(expected_pgid)).unwrap();

    assert_eq!(expected_pgid, actual_pgid);
    proc.kill_group();
}

#[test]
fn test_stdout_stderr_capture() {
    let manager = ProcessManager::new();
    let mut proc = manager
        .spawn_bash_process("echo 'hello stdout'; echo 'hello stderr' >&2", None)
        .unwrap();

    let status = proc.wait_with_timeout(Duration::from_secs(5)).unwrap();
    assert!(status.success());

    let (stdout, _stderr) = proc.read_available();
    let stdout_str = String::from_utf8(stdout).unwrap();

    assert!(stdout_str.contains("hello stdout"));
    assert!(stdout_str.contains("hello stderr"));
}

#[test]
fn test_dynamic_timeout() {
    let manager = ProcessManager::new();
    let mut proc = manager.spawn_bash_process("sleep 5", None).unwrap();

    let res = proc.wait_with_timeout(Duration::from_millis(200));
    assert!(res.is_err());
    let err_msg = res.unwrap_err();
    assert!(err_msg.contains("timed out"));
}

#[test]
fn test_manager_drop_kills_descendants() {
    let proc_pid;

    {
        let manager = ProcessManager::new();
        let proc = manager.spawn_bash_process("sleep 100", None).unwrap();
        proc_pid = proc.pgid();

        let pgid = getpgid(Some(proc_pid));
        assert!(pgid.is_ok());
    }

    thread::sleep(Duration::from_millis(200));

    let pgid = getpgid(Some(proc_pid));
    assert!(pgid.is_err());
}
