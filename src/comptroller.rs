use crate::error::EscapeResult;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct Comptroller {
    stop: bool,
}

pub type ARComptroller = Arc<RwLock<Comptroller>>;

async fn ctrl_c_handler(comptroller: ARComptroller) -> EscapeResult {
    loop {
        tokio::signal::ctrl_c().await?;
        println!("CTRL C pressed!");
        let mut c = comptroller.write().unwrap();
        c.stop = true;
    }
}

async fn duration_handler(comptroller: ARComptroller, seconds: u64) -> EscapeResult {
    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
    println!("Duration up");
    let mut c = comptroller.write().unwrap();
    c.stop = true;
    Ok(())
}

impl Comptroller {
    pub async fn new(maybe_duration: &Option<u64>) -> ARComptroller {
        let result = Arc::new(RwLock::new(Comptroller { stop: false }));

        tokio::spawn(ctrl_c_handler(result.clone()));

        if let Some(seconds) = maybe_duration {
            tokio::spawn(duration_handler(result.clone(), seconds.clone()));
        }

        result
    }

    pub fn stop(&self) -> bool {
        self.stop
    }
}
