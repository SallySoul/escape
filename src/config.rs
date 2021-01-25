use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct CutoffColor {
    pub cutoff: usize,
    pub color: [f64; 3],
}

#[derive(Serialize, Deserialize)]
pub struct RenderConfig {
    pub cutoffs: Vec<CutoffColor>,

    pub norm_cutoff: f64,

    pub sampling_instances: usize,
    pub initial_sample_attempts: usize,

    pub output_path: PathBuf,
}
