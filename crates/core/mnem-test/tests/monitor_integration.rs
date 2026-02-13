use mnem_core::Repository;
use mnem_daemon::Monitor;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_monitor_integration() {
    let dir = TempDir::new().unwrap();
    let base_dir = dir.path().join("home_mnemosyne");
    fs::create_dir_all(&base_dir).unwrap();
    let project_dir = dir.path().join("project");
    let project_mnem_dir = project_dir.join(".mnemosyne");
    fs::create_dir_all(&project_mnem_dir).unwrap();
    fs::write(project_mnem_dir.join("tracked"), "project_id: monitor-test").unwrap();

    let repo = Arc::new(Repository::open(base_dir, project_dir.clone()).unwrap());
    let monitor = Monitor::new(project_dir.clone(), repo.clone());

    // Create a file
    let test_file = project_dir.join("test.rs");
    fs::write(&test_file, "fn main() {}").unwrap();

    // Run initial scan
    monitor.initial_scan().unwrap();

    // Verify snapshot created
    let history = repo.get_history(&test_file.to_string_lossy()).unwrap();
    assert!(!history.is_empty());
}
