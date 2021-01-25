use std::sync::Arc;
use rand::distributions::Distribution;

use crate::config::RenderConfig;
use crate::view_config::ViewConfig;
use crate::types::{Complex, CountGrid};

struct FullRandomSample {
    re_range: rand::distributions::Uniform<f64>,
    im_range: rand::distributions::Uniform<f64>,
}

impl FullRandomSample {
    fn new() -> FullRandomSample {
        FullRandomSample {
            re_range: rand::distributions::Uniform::from(-2.0..2.0),
            im_range: rand::distributions::Uniform::from(0.0..2.0),
        }
    }

    fn sample(&self) -> Complex {
        let mut rng = rand::thread_rng();
        let re_sample = self.re_range.sample(&mut rng);
        let im_sample = self.im_range.sample(&mut rng);
        Complex::new(re_sample, im_sample)
    }
}

#[cfg(test)]
mod full_random_sample_tests {
    use super::*;

    #[test]
    fn random_sampling() {
        let r = FullRandomSample::new();

        for i in 0..500 {
            let c = r.sample();
            assert!(c.re >= -2.0);
            assert!(c.re <= 2.0);
            assert!(c.im >= 0.0);
            assert!(c.im <= 2.0);
        }
    }
}

struct SamplingInstance {

}

struct WorkerState {
    render_config: RenderConfig,
    view_config: ViewConfig,
    grids: Vec<CountGrid>,
    sampling_instances: Vec<SamplingInstance>,
    full_random: FullRandomSample,
    norm_cutoff_sqr: f64,
    iteration_cutoff: usize,
    orbit_buffer: Vec<Complex>,
}

impl WorkerState {
    fn new(render_config: RenderConfig, view_config: ViewConfig) -> WorkerState {
        let cutoff = render_config.cutoffs.last().unwrap().cutoff;
        WorkerState {
            render_config,
            view_config,
            grids: vec![
                CountGrid::new(view_config.width, view_config.height);
                render_config.cutoffs.len()
            ],
            sampling_instances: Vec::new(),
            full_random: FullRandomSample::new(),
            norm_cutoff_sqr: render_config.norm_cutoff * render_config.norm_cutoff,
            iteration_cutoff: cutoff,
            orbit_buffer: Vec::with_capacity(cutoff),
        }
    }

    fn evaluate(&mut self, c: &Complex) -> bool {
        self.orbit_buffer.clear();
        let mut z = c.clone();
        let mut iteration = 0;
        while z.norm_sqr() <= self.norm_cutoff_sqr && iteration <= self.iteration_cutoff {
            self.orbit_buffer.push(z);
            z = z * z + c;
            iteration += 1;
        }

        // Did point escape?
        if z.norm_sqr() > self.norm_cutoff_sqr || iteration != self.iteration_cutoff {
            true
        } else {
            false
        }
    }

    /// The contribution of a proposed value c
    /// is the fraction of the orbit that intersects the view
    fn contrib(&self, intersection_count: usize) -> f64 {
        // Note that we have to account for symetry though!
        intersection_count as f64 / (2.0 * self.iteration_cutoff as f64)
    }

    /// Find the number of times the orbit buffer intersects the view
    /// without modifying the counts
    /// This is useful when finding samples or warming up the sampling routine
    fn orbit_intersection(&mut self) -> usize {
        let mut result = 0;
        for c in self.orbit_buffer {
            if let Some((x, y)) = self.view_config.project(&c) {
                result += 1;
            }
            if let Some((x, y)) = self.view_config.project(&c.conj()) {
                result += 1;
            }
        }
        result
    }

    /// Record the contents of the orbit buffer to the count grids
    /// Return the number of intersections (does include symetry)
    fn record_orbit(&mut self) -> usize {
        let mut result = 0;
        for (i, cutoff) in self.render_config.cutoffs.iter().enumerate() {
            if self.orbit_buffer.len() <= cutoff.cutoff {
                for c in self.orbit_buffer {
                    // Account for symetry by adding the point and its conjugate
                    if let Some((x, y)) = self.view_config.project(&c) {
                        self.grids[i].increment(x, y);
                        result += 1;
                    }

                    if let Some((x, y)) = self.view_config.project(&c.conj()) {
                        self.grids[i].increment(x, y);
                        result += 1;
                    }
                }
                return result;
            }
        }
        // TODO: maybe we should just crash here instead?
        // Shouldn't ever happen
        0
    }
}

/*

async fn run_worker(state: Arc<WorkerState>) -> EscapeResult {
    Ok(())
}
*/
