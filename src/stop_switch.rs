use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{error, info};
use crate::types::EscapeResult;

pub async fn ctrl_c_handler(switch: ArcSwitch) -> EscapeResult {
    loop {
        tokio::signal::ctrl_c().await?;
        let mut s = switch.write();
        if s.stop {
            error!("Stop Switch already triggered");
        } else {
            info!("CTRL C pressed!");
            s.stop = true;
        }
    }
}

pub async fn duration_handler(switch: ArcSwitch, seconds: u64) -> EscapeResult {
    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
    let mut s = switch.write();
    if s.stop {
        error!("Stop Switch already triggered");
    } else {
        info!("Duration complete");
        s.stop = true;
    }
    Ok(())
}

/// Utility used to synchronize workers
#[derive(Debug)]
pub struct StopSwitch {
    stop: bool,
}
pub type ArcSwitch = Arc<RwLock<StopSwitch>>;

impl StopSwitch {
    pub async fn new(maybe_duration: &Option<u64>) -> ArcSwitch {
        let result = Arc::new(RwLock::new(StopSwitch { stop: false }));

        tokio::spawn(ctrl_c_handler(result.clone()));

        if let Some(seconds) = maybe_duration {
            tokio::spawn(duration_handler(result.clone(), *seconds));
        }

        result
    }

    pub fn stop(&self) -> bool {
        self.stop
    }
}
