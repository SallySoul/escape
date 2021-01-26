use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CutoffColor {
    pub cutoff: usize,
    pub color: [f64; 3],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub cutoffs: Vec<CutoffColor>,

    pub norm_cutoff: f64,

    pub sampling_instances: usize,

    pub initial_sample_attempts: usize,

    /// Specifies how many samples the warm up will take
    #[serde(default = "RenderConfig::default_warm_up_samples")]
    pub warm_up_samples: usize,

    pub 

    pub output_path: PathBuf,
}

impl RenderConfig {
    fn default_warm_up_samples() -> usize {
        10000
    }
}
