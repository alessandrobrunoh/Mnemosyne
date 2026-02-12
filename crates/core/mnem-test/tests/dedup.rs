use mnem_core::Repository;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_multiple_modifications_and_dedup() {
    let dir = TempDir::new().unwrap();
    let base_dir = dir.path().join(".mnemosyne");
    let project_dir = dir.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();
    
    let repo = Repository::open(base_dir, project_dir).unwrap();
    let file_path = repo.project.path.clone() + "/work.txt";
    let path = Path::new(&file_path);
    
    // 1. Initial version
    fs::write(path, "Initial Content").unwrap();
    let hash_a = repo.save_snapshot_from_file(path).unwrap();
    
    // 2. Change to B
    fs::write(path, "Modified Content").unwrap();
    let hash_b = repo.save_snapshot_from_file(path).unwrap();
    assert_ne!(hash_a, hash_b);
    
    // 3. Change back to A
    fs::write(path, "Initial Content").unwrap();
    let hash_a_again = repo.save_snapshot_from_file(path).unwrap();
    
    // IMPORTANT: Content-addressed storage must reuse the same hash
    assert_eq!(hash_a, hash_a_again);
    
    // 4. Check history sequence
    let history = repo.get_history(&file_path).unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].content_hash, hash_a);
    assert_eq!(history[1].content_hash, hash_b);
    assert_eq!(history[2].content_hash, hash_a);
}
