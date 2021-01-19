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
    pub width: usize,
    pub samples: usize,
    pub norm_cutoff: f64,
    pub output_path: PathBuf,
}
