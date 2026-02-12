use mnem_core::Repository;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_repository_lifecycle() {
    let dir = TempDir::new().unwrap();
    let base_dir = dir.path().join(".mnemosyne");
    let project_dir = dir.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let repo = Repository::open(base_dir, project_dir).unwrap();

    let file_path = repo.project.path.clone() + "/test.txt";
    let path = Path::new(&file_path);
    fs::write(path, "version 1").unwrap();

    // 1. Save snapshot
    let hash1 = repo.save_snapshot_from_file(path).unwrap();

    // 2. Modify and save again
    fs::write(path, "version 2").unwrap();
    let hash2 = repo.save_snapshot_from_file(path).unwrap();
    assert_ne!(hash1, hash2);

    // 3. Save same content (dedup check)
    let hash3 = repo.save_snapshot_from_file(path).unwrap();
    assert_eq!(hash2, hash3);

    // 4. Verify history
    let history = repo.get_history(&file_path).unwrap();
    assert_eq!(history.len(), 2);

    // 5. Restore
    repo.restore_file(&hash1, &file_path).unwrap();
    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content, "version 1");
}
