use anyhow::Result;

pub fn handle_git(commits: bool, log: bool, _hook: bool) -> Result<()> {
    use mnem_core::env::get_base_dir;
    use mnem_core::storage::Repository;

    let base_dir = get_base_dir()?;
    let cwd = std::env::current_dir()?;
    let repo = Repository::open(base_dir, cwd)?;

    if commits {
        let git_commits = repo.list_commits()?;
        println!("Git Commits:");
        println!("─");

        for (hash, msg, author, ts, files) in git_commits {
            println!("{}  {}  {}", &hash[..8], ts, msg);
            println!("  Author: {}", author);
            println!("  Files: {}", files);
            println!();
        }
        return Ok(());
    }

    if log {
        let git_commits = repo.list_commits()?;
        println!("Git Log:");
        println!("─");

        for (hash, msg, _author, ts, _files) in git_commits {
            println!("{}  {}  {}", &hash[..8], ts, msg);
        }
        return Ok(());
    }

    println!("Usage:");
    println!("  mnem git --commits   # list commits");
    println!("  mnem git --log      # git log");
    println!("  mnem git --hook     # install hook");

    Ok(())
}
