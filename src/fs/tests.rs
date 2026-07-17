use super::*;
use std::fs;

fn setup_temp_dirs(test_name: &str) -> (PathBuf, PathBuf) {
    let rand_id = format!(
        "{}_{}",
        test_name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let workspace = PathBuf::from(".rad/test_scratch")
        .join(&rand_id)
        .join("workspace");
    let snapshots = PathBuf::from(".rad/test_scratch")
        .join(&rand_id)
        .join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();
    (workspace, snapshots)
}

fn cleanup_temp_dirs(workspace: &Path, _snapshots: &Path) {
    if let Some(parent) = workspace.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

#[test]
fn test_permissions() {
    let (workspace, snapshots) = setup_temp_dirs("test_permissions");
    let sandbox = FsSandbox::new(
        workspace.clone(),
        snapshots.clone(),
        vec!["allowed_read".to_string(), "shared/subdir".to_string()],
        vec!["allowed_write".to_string()],
    );

    let read_err1 = sandbox
        .file_read(Path::new("forbidden/file.txt"))
        .unwrap_err();
    assert!(read_err1.to_string().contains("Read permission denied"));

    let read_err2 = sandbox
        .file_read(Path::new("allowed_read/file.txt"))
        .unwrap_err();
    assert!(!read_err2.to_string().contains("Read permission denied"));

    let write_err1 = sandbox
        .file_write(Path::new("forbidden/file.txt"), b"data")
        .unwrap_err();
    assert!(write_err1.to_string().contains("Write permission denied"));

    let write_ok = sandbox.file_write(Path::new("allowed_write/file.txt"), b"data");
    assert!(write_ok.is_ok());

    cleanup_temp_dirs(&workspace, &snapshots);
}

#[test]
fn test_wildcard_permission() {
    let (workspace, snapshots) = setup_temp_dirs("test_wildcard");
    let sandbox = FsSandbox::new(
        workspace.clone(),
        snapshots.clone(),
        vec!["*".to_string()],
        vec!["*".to_string()],
    );

    let file_path = Path::new("any_dir/file.txt");
    assert!(sandbox.file_write(file_path, b"test").is_ok());
    assert_eq!(sandbox.file_read(file_path).unwrap(), b"test");

    cleanup_temp_dirs(&workspace, &snapshots);
}

#[test]
fn test_patch() {
    let (workspace, snapshots) = setup_temp_dirs("test_patch");
    let sandbox = FsSandbox::new(
        workspace.clone(),
        snapshots.clone(),
        vec!["*".to_string()],
        vec!["*".to_string()],
    );

    let file_path = Path::new("code.txt");
    sandbox
        .file_write(file_path, b"line 1\nline 2\nline 3\n")
        .unwrap();

    let diff = "--- code.txt\n+++ code.txt\n@@ -1,3 +1,3 @@\n line 1\n-line 2\n+line 2 modified\n line 3\n";
    assert!(sandbox.file_edit_patch(file_path, diff).is_ok());

    let content = String::from_utf8(sandbox.file_read(file_path).unwrap()).unwrap();
    assert_eq!(content, "line 1\nline 2 modified\nline 3\n");

    cleanup_temp_dirs(&workspace, &snapshots);
}

#[test]
fn test_snapshot_backup_restore() {
    let (workspace, snapshots) = setup_temp_dirs("test_snapshot");
    let sandbox = FsSandbox::new(
        workspace.clone(),
        snapshots.clone(),
        vec!["*".to_string()],
        vec!["*".to_string()],
    );

    let file1 = Path::new("file1.txt");
    let file2 = Path::new("dir/file2.txt");
    sandbox.file_write(file1, b"hello").unwrap();
    sandbox.file_write(file2, b"world").unwrap();

    sandbox
        .take_snapshot("node_1", &[file1.to_path_buf(), PathBuf::from("dir")])
        .unwrap();

    sandbox.file_write(file1, b"modified hello").unwrap();
    let abs_file2 = workspace.join(file2);
    fs::remove_file(abs_file2).unwrap();

    sandbox.checkout_snapshot("node_1").unwrap();

    assert_eq!(sandbox.file_read(file1).unwrap(), b"hello");
    assert_eq!(sandbox.file_read(file2).unwrap(), b"world");

    cleanup_temp_dirs(&workspace, &snapshots);
}
