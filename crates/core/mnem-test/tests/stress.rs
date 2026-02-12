use mnem_core::Repository;
use mnem_daemon::Monitor;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
#[ignore]
fn stress_test_large_codebase() {
    let dir = TempDir::new().unwrap();
    let base_dir = dir.path().join(".mnemosyne");
    let project_dir = dir.path().join("huge_project");
    fs::create_dir_all(&project_dir).unwrap();

    // Generate 10,000 files
    for i in 0..10000 {
        let subdir = project_dir.join(format!("dir_{}", i % 50));
        fs::create_dir_all(&subdir).unwrap();
        let file_path = subdir.join(format!("file_{}.rs", i));
        fs::write(
            file_path,
            format!(
                "// Content for file {}\nfn main() {{ println!(\"hello from {}\"); }}",
                i, i
            ),
        )
        .unwrap();
    }

    let repo = Arc::new(Repository::open(base_dir, project_dir.clone()).unwrap());
    let monitor = Monitor::new(project_dir.clone(), repo.clone());

    let start = std::time::Instant::now();
    monitor.initial_scan().unwrap();
    let duration = start.elapsed();
    println!("Initial scan of 10,000 files took: {:?}", duration);

    // Stress test the grep (search) across all 10,000 unique content blobs
    let grep_start = std::time::Instant::now();
    let search_results = repo.grep_contents("hello from 9999", None).unwrap();
    let grep_duration = grep_start.elapsed();
    println!("Grep search took: {:?}", grep_duration);

    assert!(!search_results.is_empty());
}
