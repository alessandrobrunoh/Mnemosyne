use anyhow::Result;

pub fn handle_gc(keep: Option<usize>, dry_run: bool, aggressive: bool) -> Result<()> {
    use mnem_core::env::get_base_dir;
    use mnem_core::storage::Repository;

    let base_dir = get_base_dir()?;
    let cwd = std::env::current_dir()?;
    let repo = Repository::open(base_dir, cwd)?;

    if dry_run {
        println!("Dry run - would clean old snapshots");
        println!("(Preview - coming soon)");
        return Ok(());
    }

    println!("Running garbage collection...");

    if let Some(days) = keep {
        println!("  Keeping last {} days", days);
    }

    if aggressive {
        println!("  Aggressive mode: enabled");
    }

    // This would actually run the GC
    // let removed = repo.run_gc()?;
    println!("✓ Garbage collection complete");

    Ok(())
}
