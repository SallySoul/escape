use rand::distributions::Distribution;
use std::sync::Arc;

use crate::config::RenderConfig;
use crate::types::{Complex, CountGrid};
use crate::view_config::ViewConfig;

fn radius_sample(radius: f64) -> Complex {
    let mut rng = rand::thread_rng();
    let range = rand::distributions::Uniform::from(-radius..radius);
    let rad_sqr = radius * radius;
    loop {
        let c = Complex::new(range.sample(&mut rng), range.sample(&mut rng));
        if c.norm_sqr() < rad_sqr {
            return c;
        }
    }
}

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
            let c = self.sample_square();
            if c.norm_sqr() <= 4.0 {
                return c;
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
    rand::distributions::Uniform::from(0.0..1.0).sample(&mut rng)
}

struct WorkerState {
    render_config: RenderConfig,
    view_config: ViewConfig,
    grids: Vec<CountGrid>,
    full_random: FullRandomSample,
    norm_cutoff_sqr: f64,
    iteration_cutoff: usize,
    iteration_cutoff_f64: f64,
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
            full_random: FullRandomSample::new(),
            norm_cutoff_sqr: render_config.norm_cutoff * render_config.norm_cutoff,
            iteration_cutoff: cutoff,
            iteration_cutoff_f64: cutoff as f64,
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
    fn find_initial_sample(&mut self) -> Complex {
        return self.find_initial_sample_r(&Complex::new(0.0, 0.0), 2.0);
    }

    /// Here's the gist
    /// as this function recurses, it also narrows the spatial scope of its search
    /// In general, we are looking for a point whose orbit intersects the view
    /// Failing that, we keep track of sample whose orbit has gotten the closest
    /// For each search scope / radius / recursion we try for n times to generate a random perturbation of the seed point that either intersects the view of see if it gets closer
    fn find_initial_sample_r(&mut self, seed_r: &Complex, radius: f64) -> Complex {
        let mut closest_distance = std::f64::MAX;
        let mut closest_sample = Complex::new(0.0, 0.0);

        for _ in 0..200 {
            // Generate sample for this iteration
            let sample = seed_r + radius_sample(radius);

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

        return self.find_initial_sample_r(&closest_sample, radius / 2.0);
    }

    /// Sampling with the Metropolis-Hastings algorithm is based on mutating a "good" sample
    /// Some of the time we want to perturb the last good sample
    /// Other times we want to try a complelety new point
    fn mutate(&self, c: &Complex) -> Complex {
        if random_prob() < self.render_config.random_sample_prob {
            self.full_random.sample()
        } else {
            let mut result = c.clone();
            let r1 = 1.0 / self.view_config.zoom * 0.0001;
            let r2 = 1.0 / self.view_config.zoom * 0.1;
            let phi = random_prob() * 2.0 * std::f64::consts::PI;
            let r = r2 * (-(r2 / r1).ln() * random_prob()).exp();

            result.re += r * phi.cos();
            result.im += r * phi.sin();

            result
        }
    }

    fn transition_probability(&self, orbit_len_1: usize, orbit_len_2: usize) -> f64 {
        let ol1 = orbit_len_1 as f64;
        let ol2 = orbit_len_2 as f64;

        let numerator = 1.0 - ((self.iteration_cutoff_f64 - ol1) / ol1);
        let denominator = 1.0 - ((self.iteration_cutoff_f64 - ol2) / ol2);

        numerator / denominator
    }

    fn run_metro_instance(&mut self) {
        println!("*** Starting Run");
        // TODO these need to be setup properly
        let mut z = self.find_initial_sample();
        println!("*** Found Initial Sample");
        let mut z_orbit_len = self.orbit_buffer.len();
        let z_orbit_intersections = self.orbit_intersections();
        let mut z_contrib = self.contribution(z_orbit_intersections);

        let mut accepted_samples_warmup = 0;
        let mut accepted_samples = 0;
        let mut rejected_samples_warmup = 0;
        let mut rejected_samples = 0;

        println!("Starting WarmUp");
        for _ in 0..self.render_config.warm_up_samples {
            let mutation = self.mutate(&z);
            self.evaluate(&mutation);
            let mutation_orbit_len = self.orbit_buffer.len();
            let intersection_count = self.orbit_intersections();

            // If the mutation doesn't intersect at all, it's a dud
            if intersection_count == 0 {
                continue;
            }

            let mutation_contrib = self.contribution(intersection_count);

            let t1 = self.transition_probability(mutation_orbit_len, z_orbit_len);
            let t2 = self.transition_probability(z_orbit_len, mutation_orbit_len);
            let alpha =
                (((mutation_contrib * t1).ln() - (z_contrib * t2).ln()).exp()).clamp(0.0, 1.0);

            if alpha > random_prob() {
                z = mutation;
                z_contrib = mutation_contrib;
                z_orbit_len = mutation_orbit_len;
                accepted_samples_warmup += 1;
            } else {
                rejected_samples_warmup += 1;
            }
        }

        println!(
            "*** Warm up done! accepted: {}, rejected: {}",
            accepted_samples_warmup, rejected_samples_warmup
        );
        for _ in 0..self.render_config.samples {
            let mutation = self.mutate(&z);
            self.evaluate(&mutation);
            let mutation_orbit_len = self.orbit_buffer.len();
            let intersection_count = self.record_orbit();

            // If the mutation doesn't intersect at all, it's a dud
            if intersection_count == 0 {
                continue;
            }

            let mutation_contrib = self.contribution(intersection_count);

            let t1 = self.transition_probability(mutation_orbit_len, z_orbit_len);
            let t2 = self.transition_probability(z_orbit_len, mutation_orbit_len);
            let alpha =
                (((mutation_contrib * t1).ln() - (z_contrib * t2).ln()).exp()).clamp(0.0, 1.0);

            if alpha > random_prob() {
                z = mutation;
                z_contrib = mutation_contrib;
                z_orbit_len = mutation_orbit_len;
                accepted_samples += 1;
            } else {
                rejected_samples += 1;
            }
        }
        println!(
            "*** Done! accepted: {}, rejected: {}",
            accepted_samples, rejected_samples
        );
    }

    fn run_worker(&self) {
        for i in 0..self.render_config.metro_instances {
            println!("Running metro instance {}", i);
            self.run_metro_instance();
        }
    }
}

/*

async fn run_worker(state: Arc<WorkerState>) -> EscapeResult {
    Ok(())
}
*/
