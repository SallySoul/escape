use crate::types::Complex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DrawConfig {
    pub colors: Vec<[f64; 3]>,
    pub background_color: [f64; 3],
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ViewConfig {
    pub center: Complex,
    pub zoom: f64,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SampleConfig {
    pub cutoffs: Vec<usize>,
    pub view: ViewConfig,

    /// Depth Limit for the initial samples search
    /// Default value is 500
    #[serde(default = "SampleConfig::default_initial_search_depth")]
    pub initial_search_depth: usize,

    /// Specifies how many samples the warm up will take.
    /// Default value is 10k
    #[serde(default = "SampleConfig::default_warm_up_samples")]
    pub warm_up_samples: usize,

    /// For each new sample, specify the probability that a random point is chosen
    /// as opposed to perturbing the previous sample.
    /// Default value is 0.2
    #[serde(default = "SampleConfig::default_random_sample_prob")]
    pub random_sample_prob: f64,

    /// We the norm at which we decide that an orbit has escaped.
    /// Default value is 2.0
    #[serde(default = "SampleConfig::default_norm_cutoff")]
    pub norm_cutoff: f64,

    /// How many samples to take from a metropolis hastings run
    /// Default value is 100000
    #[serde(default = "SampleConfig::default_samples")]
    pub samples: usize,

    /// End Metro-hastings run if this number of sampled orbits are sequencly outside the view
    /// Default value is 5
    #[serde(default = "SampleConfig::default_outside_limit")]
    pub outside_limit: usize,
}

impl SampleConfig {
    fn default_initial_search_depth() -> usize {
        500
    }

    fn default_samples() -> usize {
        100000
    }

    fn default_norm_cutoff() -> f64 {
        2.0
    }

    fn default_warm_up_samples() -> usize {
        10000
    }

    fn default_random_sample_prob() -> f64 {
        0.2
    }

    fn default_outside_limit() -> usize {
        5
    }

    pub fn compatible(&self, other: &Self) -> bool {
        self.cutoffs.len() == other.cutoffs.len()
            && self.view.width == other.view.width
            && self.view.height == other.view.height
    }
}
