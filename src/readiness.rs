use nix::sys::statvfs::statvfs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::{GatewayGfg, HealthCfg};

pub struct Readiness {
    pub accepting: AtomicBool,
    pub disk_ok: AtomicBool,
    pub mqtt_ok: AtomicBool,
}

impl Readiness {
    pub fn new() -> Self {
        Self {
            accepting: AtomicBool::new(false),
            disk_ok: AtomicBool::new(false),
            mqtt_ok: AtomicBool::new(false),
        }
    }
    pub fn set_accepting(&self, v: bool) {
        self.accepting.store(v, Ordering::SeqCst);
    }

    pub fn is_ready(&self, gates: &HealthCfg) -> bool {
        let accepting = self.accepting.load(Ordering::SeqCst);
        if !accepting {
            return false;
        }
        if gates.require_disk && !self.disk_ok.load(Ordering::Relaxed) {
            return false;
        }
        if gates.require_mqtt && !self.mqtt_ok.load(Ordering::Relaxed) {
            return false;
        }
        true
    }
}

pub fn start_readisness_probes(cfg: Arc<GatewayGfg>, ready: Arc<Readiness>) {
    let interval = std::time::Duration::from_millis(cfg.health.probe_interval_ms.unwrap_or(1000));

    // Disk probe (disabled when min_free_bytes == 0)
    {
        let ready = ready.clone();
        let path = cfg.storage.db_path.clone();
        let min = cfg.storage.min_free_bytes.clone();
        if min == 0 {
            ready.disk_ok.store(true, Ordering::Relaxed);
        } else {
            tokio::spawn(async move {
                let mut tick = tokio::time::interval(interval);
                let check =
                    |p: &std::path::Path, min| free_bytes_for(p).map(|b| b >= min).unwrap_or(false);
                ready.disk_ok.store(check(&path, min), Ordering::Relaxed);
                loop {
                    tick.tick().await;
                    ready.disk_ok.store(check(&path, min), Ordering::Relaxed);
                }
            });
        }
    }

    // MQTT probe (disabled when host=="" or port==0)
    {
        let ready = ready.clone();
        let host = cfg.mqtt.host.clone();
        let port = cfg.mqtt.port;
        if host.is_empty() || port == 0 {
            ready.mqtt_ok.store(true, Ordering::Relaxed);
        } else {
            tokio::spawn(async move {
                use tokio::net::TcpStream;
                let mut tick = tokio::time::interval(interval);
                // immediate
                ready
                    .mqtt_ok
                    .store(try_connect(&host, port, interval).await, Ordering::Relaxed);
                loop {
                    tick.tick().await;
                    ready
                        .mqtt_ok
                        .store(try_connect(&host, port, interval).await, Ordering::Relaxed);
                }
                async fn try_connect(h: &str, p: u16, t: std::time::Duration) -> bool {
                    match tokio::time::timeout(t, TcpStream::connect((h, p))).await {
                        Ok(Ok(_)) => true,
                        _ => false,
                    }
                }
            });
        }
    }
}

fn free_bytes_for(path: &std::path::Path) -> anyhow::Result<u64> {
    let p = path.parent().unwrap_or(path);
    let stats = statvfs(p)?;
    Ok((stats.blocks_available() as u64) * stats.block_size())
}
