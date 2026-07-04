use super::*;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[test]
fn test_watcher_detects_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let watcher = FsWatcher::new(temp_dir.path()).unwrap();

    // Give the watcher a moment to register with the OS
    thread::sleep(Duration::from_millis(100));

    let file_path = temp_dir.path().join("test_file.txt");

    // 1. Create file
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello watcher").unwrap();
    }

    let mut created = false;
    let mut modified = false;

    // Retry checking for events to account for OS file event dispatch delay
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        while let Ok(event) = watcher.try_recv() {
            if let RasCoreEvent::FileChanged { path, change_type } = event {
                let is_target = path.file_name() == file_path.file_name();
                if is_target && change_type == "create" {
                    created = true;
                } else if is_target && change_type == "modify" {
                    modified = true;
                }
            }
        }
        if created || modified {
            break;
        }
    }

    assert!(created || modified);

    // 2. Remove file
    std::fs::remove_file(&file_path).unwrap();

    let mut removed = false;
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        while let Ok(event) = watcher.try_recv() {
            if let RasCoreEvent::FileChanged { path, change_type } = event {
                let is_target = path.file_name() == file_path.file_name();
                if is_target && change_type == "remove" {
                    removed = true;
                }
            }
        }
        if removed {
            break;
        }
    }

    assert!(removed);
}
