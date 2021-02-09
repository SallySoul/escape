use crate::types::{Complex, EscapeError, EscapeResult};
use serde::{Deserialize, Serialize};

/// ViewConfig describes what region of the buddhabrot to render
/// as well as the grid to use when creating histograms
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ViewConfig {
    pub center: Complex,
    pub zoom: f64,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SampleConfig {
    /// When sampling, we record orbits in different histograms depending
    /// on the length of the orbit before it escapes.
    pub cutoffs: Vec<usize>,

    /// The region of the buddhabrot to render
    pub view: ViewConfig,

    /// Julia set parameter.
    /// Default value is 0+0i
    /// Combined with default mandelbrot param, this produces
    /// producing traditional mandelbrot itreration.
    /// coord = complex coord
    /// jc = julia set param
    /// m = mandelbrot param
    /// f(z) = z^2 + (jc + m * coord)
    #[serde(default = "SampleConfig::default_julia_set_param")]
    pub julia_set_param: Complex,

    /// A scaling param to transform from julia to mandelbrot iterations
    /// Default value is 1.0+0i
    #[serde(default = "SampleConfig::default_mandelbrot_param")]
    pub mandelbrot_param: Complex,

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
        100
    }

    fn default_julia_set_param() -> Complex {
        Complex::new(0.0, 0.0)
    }

    fn default_mandelbrot_param() -> Complex {
        Complex::new(1.0, 0.0)
    }

    pub fn compatible(&self, other: &Self) -> bool {
        self.cutoffs.len() == other.cutoffs.len()
            && self.view.width == other.view.width
            && self.view.height == other.view.height
    }
}

/// StlConfig is used to color histogram results
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StlConfig {
    pub contributions: Vec<f32>,
    pub powers: Vec<f64>,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub relief_height: f32,
}

impl StlConfig {
    pub fn compatible(&self, sample_config: &SampleConfig) -> EscapeResult {
        let cutoff_count = sample_config.cutoffs.len();
        let contribution_count = self.contributions.len();
        let powers_count = self.powers.len();

        if contribution_count != cutoff_count {
            let msg = format!(
                "Sample config had {} cutoffs, stl config had {} contributions",
                cutoff_count, contribution_count
            );
            return Err(EscapeError::IncompatibleStlConfig(msg));
        }

        if powers_count != cutoff_count {
            let msg = format!(
                "Sample config had {} cutoffs, stl config had {} powers",
                cutoff_count, powers_count
            );
            return Err(EscapeError::IncompatibleStlConfig(msg));
        }

        Ok(())
    }
}

/// DrawConfig is used to color histogram results
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DrawConfig {
    pub colors: Vec<[i32; 3]>,
    pub powers: Vec<f64>,
    pub background_color: [f64; 3],
}

impl DrawConfig {
    pub fn compatible(&self, sample_config: &SampleConfig) -> EscapeResult {
        let cutoff_count = sample_config.cutoffs.len();
        let colors_count = self.colors.len();
        let powers_count = self.powers.len();

        if colors_count != cutoff_count {
            let msg = format!(
                "Sample config had {} cutoffs, draw config had {} colors",
                cutoff_count, colors_count
            );
            return Err(EscapeError::IncompatibleDrawConfig(msg));
        }

        if powers_count != cutoff_count {
            let msg = format!(
                "Sample config had {} cutoffs, draw config had {} powers",
                cutoff_count, powers_count
            );
            return Err(EscapeError::IncompatibleDrawConfig(msg));
        }

        Ok(())
    }
}
