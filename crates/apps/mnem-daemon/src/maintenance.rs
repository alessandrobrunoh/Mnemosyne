use std::sync::Arc;
use log::{info, error};
use mnem_core::Repository;
use crate::state::DaemonState;

pub async fn run_background_maintenance(state: Arc<DaemonState>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Every hour
    loop {
        interval.tick().await;
        info!("Running background maintenance (GC & Migration)...");

        let repos: Vec<Arc<Repository>> = state.repos.iter().map(|r| r.value().clone()).collect();

        for repo in repos {
            match repo.run_gc() {
                Ok(pruned) => {
                    if pruned > 0 {
                        info!("GC pruned {} snapshots in {}", pruned, repo.project.path);
                    }
                }
                Err(e) => error!("GC failed for {}: {}", repo.project.path, e),
            }

            match repo.run_migration() {
                Ok(moved) => {
                    if moved > 0 {
                        info!("Migrated {} objects in {}", moved, repo.project.path);
                    }
                }
                Err(e) => error!("Migration failed for {}: {}", repo.project.path, e),
            }
        }
    }
}

