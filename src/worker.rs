use rand::distributions::Distribution;
use std::sync::Arc;

use crate::config::RenderConfig;
use crate::types::{Complex, CountGrid};
use crate::view_config::ViewConfig;

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

    fn sample_square(&self) -> Complex {
        let mut rng = rand::thread_rng();
        let re_sample = self.re_range.sample(&mut rng);
        let im_sample = self.im_range.sample(&mut rng);
        Complex::new(re_sample, im_sample)
    }

    fn sample(&self) -> Complex {
        loop {
            let c = self.sample();
            if c.norm_sqr() <= 4.0 {
                return c
            }
        }
    }
}

#[cfg(test)]
mod full_random_sample_tests {
    use super::*;

    #[test]
    fn random_sampling() {
        let r = FullRandomSample::new();

        for _ in 0..500 {
            let c = r.sample();
            assert!(c.re >= -2.0);
            assert!(c.re <= 2.0);
            assert!(c.im >= 0.0);
            assert!(c.im <= 2.0);
            assert!(c.norm_sqr() <= 4.0);
        }
    }
}

fn random_prob() -> f64 {
    let mut rng = rand::thread_rng();
    rand::distributions::Uniform::from(0..1.0).sample(&mut rng)
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
    fn new(render_config: &RenderConfig, view_config: &ViewConfig) -> WorkerState {
        let cutoff = render_config.cutoffs.last().unwrap().cutoff;
        WorkerState {
            render_config: render_config.clone(),
            view_config: view_config.clone(),
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
    fn contribution(&self, intersection_count: usize) -> f64 {
        // Note that we have to account for symetry though!
        intersection_count as f64 / (2.0 * self.iteration_cutoff as f64)
    }

    /// Find the number of times the orbit buffer intersects the view
    /// without modifying the counts
    /// This is useful when finding samples or warming up the sampling routine
    fn orbit_intersections(&mut self) -> usize {
        let mut result = 0;
        for c in &self.orbit_buffer {
            if let Some(_) = self.view_config.project(&c) {
                result += 1;
            }
            if let Some(_) = self.view_config.project(&c.conj()) {
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
                for c in &self.orbit_buffer {
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

    /// Find a point whose orbit passes through the view
    ///
    /// This is a port of Alexander Boswell's FindInitialSample function.
    /// Per the comment in his code, better than random sampling for higher zooms
    ///
    fn find_initial_sample(&mut self) -> Complex {
        return self.find_initial_sample_r(&Complex::new(0.0, 0.0), 2.0);
    }

    fn find_initial_sample_r(&mut self, seed_r: &Complex, rad: f64) -> Complex {
        let mut closest_distance = std::f64::MAX;
        let mut closest_sample = Complex::new(0.0, 0.0);

        for _ in 0..200 {
            // Generate sample for this iteration
            let sample = self.full_random.sample() + seed_r;

            // If sample doesn't escape than its a dud
            let sample_escapes = self.evaluate(&sample);
            if !sample_escapes {
                continue;
            }

            // If sample's orbit intersects view then we're done
            let intersection_count = self.orbit_intersections();
            if intersection_count > 0 {
                return sample;
            }

            // Otherwise, lets keep track of the sample that produced an orbit with an
            // element that was closest to the view
            for c in &self.orbit_buffer {
                let distance = (c - self.view_config.center).norm_sqr();
                if distance < closest_distance {
                    closest_sample = sample;
                    closest_distance = distance;
                }
            }
        }

        return self.find_initial_sample_r(&closest_sample, rad / 2.0);
    }

    fn mutate(&self, c: &Complex) -> Complex {


        Complex::new(0.0, 0.0)
    }

    fn buddhabrot(&mut self) {
        let mut z = self.find_initial_sample();
        let mut _z_contrib = 0.0;

        for _ in 0..self.render_config.warm_up_samples {
            let mutation = self.mutate(&z);
            self.evaluate(&mutation);
            let intersection_count = self.orbit_intersections();

            // If the mutation doesn't intersect at all, it's a dud
            if intersection_count == 0 {
                continue
            }

            let mutation_contribution = self.contribution(intersection_count);

            let t1 = self.transition_prob();
            let t2 = self.transition_prob();
            let alpha = std::min(1.0,  )

        }
    }
}

/*

async fn run_worker(state: Arc<WorkerState>) -> EscapeResult {
    Ok(())
}
*/
