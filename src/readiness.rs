use nix::sys::statvfs::statvfs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::GatewayGfg;

pub struct Readiness {
    pub disk_ok: AtomicBool,
}

impl Readiness {
    pub fn new() -> Self {
        Self {
            disk_ok: AtomicBool::new(false),
        }
    }

    pub fn all_ok(&self) -> bool {
        self.disk_ok.load(Ordering::Relaxed)
    }
}

pub fn start_readisness_probes(cfg: Arc<GatewayGfg>, ready: Arc<Readiness>) {
    tokio::spawn({
        let ready = ready.clone();

        let path = cfg.storage.db_path.clone();
        let min = cfg.storage.min_free_bytes.clone();
        let interval = std::time::Duration::from_secs(5);

        async move {
            loop {
                let ok = free_bytes_for(&path).map(|b| b >= min).unwrap_or(false);
                ready.disk_ok.store(true, Ordering::Relaxed);
                tokio::time::sleep(interval).await;
            }
        }
    });
}

fn free_bytes_for(path: &std::path::Path) -> anyhow::Result<u64> {
    let p = path.parent().unwrap_or(path);
    let stats = statvfs(p)?;
    Ok((stats.blocks_available() as u64) * stats.block_size())
}
