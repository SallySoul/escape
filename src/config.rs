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

    pub metro_instances: usize,

    /// Specifies how many samples the warm up will take.
    /// Default value is 10k
    #[serde(default = "RenderConfig::default_warm_up_samples")]
    pub warm_up_samples: usize,

    /// For each new sample, specify the probability that a random point is chosen
    /// as opposed to perturbing the previous sample.
    /// Default value is 0.2
    #[serde(default = "RenderConfig::default_random_sample_prob")]
    pub random_sample_prob: f64,

    /// How many samples to take from a metropolis hastings run
    pub samples: usize,

    pub output_path: PathBuf,
}

impl RenderConfig {
    fn default_warm_up_samples() -> usize {
        10000
    }

    fn default_random_sample_prob() -> f64 {
        0.2
    }
}
