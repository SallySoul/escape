use rand::distributions::Distribution;
use std::sync::Arc;

use std::io::BufReader;

use crate::cli_options::SampleOptions;
use crate::comptroller::{ARComptroller, Comptroller};
use crate::config::{SampleConfig, ViewConfig};
use crate::error::EscapeResult;
use crate::histogram_result::HistogramResult;
use crate::types::{Complex, CountGrid, NormalizedGrid};

use tracing::{info, trace};

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

fn random_prob() -> f64 {
    let mut rng = rand::thread_rng();
    rand::distributions::Uniform::from(0.0..1.0).sample(&mut rng)
}

pub fn project_onto_view(view: &ViewConfig, c: &Complex) -> Option<(usize, usize)> {
    let x_fp = ((c.re - view.center.re) * view.zoom) + 0.5;
    let y_fp = ((c.im - view.center.im) * view.zoom) + 0.5;

    let x_signed = (x_fp * view.width as f64) as i32;
    let y_signed = (y_fp * view.height as f64) as i32;

    if x_signed >= 0
        && y_signed >= 0
        && x_signed < view.width as i32
        && y_signed < view.height as i32
    {
        Some((x_signed as usize, y_signed as usize))
    } else {
        None
    }
}

#[derive(Debug)]
pub struct WorkerState {
    sample_config: SampleConfig,
    pub grids: Vec<CountGrid>,
    norm_cutoff_sqr: f64,
    iteration_cutoff: usize,
    iteration_cutoff_f64: f64,
    orbit_buffer: Vec<Complex>,
    comptroller: ARComptroller,
}

impl WorkerState {
    pub fn new(sample_config: &SampleConfig, comptroller: ARComptroller) -> WorkerState {
        let cutoff = sample_config.cutoffs.last().unwrap().clone();
        let view = sample_config.view;
        WorkerState {
            sample_config: sample_config.clone(),
            grids: vec![CountGrid::new(view.width, view.height); sample_config.cutoffs.len()],
            norm_cutoff_sqr: sample_config.norm_cutoff * sample_config.norm_cutoff,
            iteration_cutoff: cutoff,
            iteration_cutoff_f64: cutoff as f64,
            orbit_buffer: Vec::with_capacity(cutoff),
            comptroller,
        }
    }

    pub fn project(&self, c: &Complex) -> Option<(usize, usize)> {
        project_onto_view(&self.sample_config.view, c)
    }

    #[tracing::instrument(skip(self))]
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
    #[tracing::instrument(skip(self))]
    fn orbit_intersections(&mut self) -> usize {
        let mut result = 0;
        for c in &self.orbit_buffer {
            if let Some(_) = self.project(&c) {
                result += 1;
            }
            if let Some(_) = self.project(&c.conj()) {
                result += 1;
            }
        }
        result
    }

    /// Record the contents of the orbit buffer to the count grids
    /// Return the number of intersections (does include symetry)
    #[tracing::instrument(skip(self))]
    fn record_orbit(&mut self) -> usize {
        let mut result = 0;
        for (i, cutoff) in self.sample_config.cutoffs.iter().enumerate() {
            if self.orbit_buffer.len() <= cutoff.clone() {
                for c in &self.orbit_buffer {
                    // Account for symetry by adding the point and its conjugate
                    if let Some((x, y)) = self.project(&c) {
                        self.grids[i].increment(x, y);
                        result += 1;
                    }

                    if let Some((x, y)) = self.project(&c.conj()) {
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
    #[tracing::instrument(skip(self))]
    fn find_initial_sample(&mut self) -> Option<Complex> {
        let (result, depth) = self.find_initial_sample_r(&Complex::new(0.0, 0.0), 2.0, 0);
        trace!(depth, "find initial sample recursion completed");
        result
    }

    /// Here's the gist
    /// as this function recurses, it also narrows the spatial scope of its search
    /// In general, we are looking for a point whose orbit intersects the view
    /// Failing that, we keep track of sample whose orbit has gotten the closest
    /// For each search scope / radius / recursion we try for n times to generate a random perturbation of the seed point that either intersects the view of see if it gets closer
    fn find_initial_sample_r(
        &mut self,
        seed_r: &Complex,
        radius: f64,
        depth: usize,
    ) -> (Option<Complex>, usize) {
        if depth > self.sample_config.initial_search_depth {
            return (None, depth);
        }

        let view = self.sample_config.view;
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
                return (Some(sample), depth);
            }

            // Otherwise, lets keep track of the sample that produced an orbit with an
            // element that was closest to the view
            for c in &self.orbit_buffer {
                let distance = (c - view.center).norm_sqr();
                if distance < closest_distance {
                    closest_sample = sample;
                    closest_distance = distance;
                }
            }
        }

        return self.find_initial_sample_r(&closest_sample, radius / 2.0, depth + 1);
    }

    /// Sampling with the Metropolis-Hastings algorithm is based on mutating a "good" sample
    /// Some of the time we want to perturb the last good sample
    /// Other times we want to try a complelety new point
    #[tracing::instrument(skip(self))]
    fn mutate(&self, c: &Complex) -> Complex {
        let view = self.sample_config.view;
        if random_prob() < self.sample_config.random_sample_prob {
            radius_sample(self.sample_config.norm_cutoff)
        } else {
            let mut result = c.clone();
            let r1 = 1.0 / view.zoom * 0.0001;
            let r2 = 1.0 / view.zoom * 0.1;
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

    #[tracing::instrument(skip(self))]
    fn run_metro_instance(&mut self) {
        //println!("*** Starting Run");
        // TODO these need to be setup properly
        let mut z = match self.find_initial_sample() {
            Some(z) => z,
            None => {
                println!("Failed to find initial sample");
                return;
            }
        };

        //println!("*** Found Initial Sample");
        let mut z_orbit_len = self.orbit_buffer.len();
        let z_orbit_intersections = self.orbit_intersections();
        let mut z_contrib = self.contribution(z_orbit_intersections);

        let mut accepted_samples = 0;
        let mut rejected_samples = 0;
        let mut outside_samples = 0;

        //println!("*** Starting WarmUp");
        for s in 0..self.sample_config.warm_up_samples {
            if s % 1000 == 0 {
                if self.stop() {
                    println!("In sampling stop");
                    break;
                }
            }

            let mutation = self.mutate(&z);
            self.evaluate(&mutation);
            let mutation_orbit_len = self.orbit_buffer.len();
            let intersection_count = self.orbit_intersections();
            // If the mutation doesn't intersect at all, it's a dud
            if intersection_count == 0 {
                outside_samples += 1;
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

        trace!(
            accepted_samples,
            rejected_samples,
            outside_samples,
            "Warm up complete"
        );

        accepted_samples = 0;
        rejected_samples = 0;
        outside_samples = 0;
        for s in 0..self.sample_config.samples {
            if s % 5000 == 0 {
                if self.stop() {
                    println!("In sampling stop");
                    break;
                }
            }

            let mutation = self.mutate(&z);
            self.evaluate(&mutation);
            let mutation_orbit_len = self.orbit_buffer.len();
            let intersection_count = self.record_orbit();

            // If the mutation doesn't intersect at all, it's a dud
            if intersection_count == 0 {
                outside_samples += 1;
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

        trace!(
            accepted_samples,
            rejected_samples,
            outside_samples,
            "Sampling complete"
        );
    }

    fn stop(&self) -> bool {
        self.comptroller.read().unwrap().stop()
    }

    #[tracing::instrument(skip(self))]
    pub fn run_worker(&mut self) {
        let mut metro_instances = 0;
        while !self.stop() {
            metro_instances += 1;
            let thread_id = std::thread::current().id().as_u64();
            trace!(thread_id, metro_instances, "Starting metro instance");
            self.run_metro_instance();
        }
        println!("Ran {} metro instances", metro_instances);
    }
}

#[tracing::instrument(skip(config, grids))]
fn merge_grids(config: &SampleConfig, grids: Vec<CountGrid>) -> CountGrid {
    let mut result = CountGrid::new(config.view.width, config.view.height);
    for x in 0..config.view.width {
        for y in 0..config.view.height {
            let mut sum = 0;
            for grid in &grids {
                sum += grid.value(x, y);
            }
            result.set_value(sum, x, y);
        }
    }

    result
}

#[tracing::instrument(skip(config, results))]
async fn merge_results(
    config: Arc<SampleConfig>,
    results: &Vec<&Vec<CountGrid>>,
) -> Vec<NormalizedGrid> {
    let cutoff_count = config.cutoffs.len();
    let mut tasks = Vec::with_capacity(cutoff_count);
    for cutoff_index in 0..cutoff_count {
        let mut count_grids = Vec::with_capacity(results.len());
        for w in 0..results.len() {
            count_grids.push(results[w][cutoff_index].clone());
        }
        let c = config.clone();
        tasks.push(tokio::spawn(async move {
            merge_grids(&c, count_grids).to_normalized_grid()
        }));
    }

    let mut result = Vec::with_capacity(cutoff_count);
    for task in tasks {
        result.push(task.await.unwrap());
    }

    result
}

#[tracing::instrument(skip(state_arc))]
async fn run_worker(mut state_arc: Arc<WorkerState>) {
    unsafe {
        let state = Arc::get_mut_unchecked(&mut state_arc);
        state.run_worker();
    };
}

async fn async_main(config: Arc<SampleConfig>, cli_options: &SampleOptions) -> EscapeResult {
    tracing_subscriber::fmt::init();

    let c = Comptroller::new(&cli_options.duration).await;

    let mut worker_states = Vec::with_capacity(cli_options.workers);
    let mut futures = Vec::with_capacity(cli_options.workers);
    for i in 0..cli_options.workers {
        worker_states.push(Arc::new(WorkerState::new(&config, c.clone())));
        futures.push(tokio::spawn(run_worker(worker_states[i].clone())));
    }
    info!(cli_options.workers, "Started sampling workers");

    let mut results = Vec::with_capacity(cli_options.workers);
    for w in futures {
        results.push(w.await.unwrap());
    }
    info!("Sampling workers have completed");

    let mut results = Vec::new();
    for w in &worker_states {
        results.push(&w.grids);
    }
    let merged_grids = merge_results(config.clone(), &results).await;
    info!("Worker results have been merged");

    HistogramResult::to_file(&config, &merged_grids, &cli_options.output)?;
    info!(
        "Result has been written to {}",
        &cli_options.output.display()
    );

    Ok(())
}

pub fn run_sampling(sample_options: &SampleOptions) -> EscapeResult {
    // Note that we add one extra thread for timers / signal handlers
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(sample_options.workers + 1)
        .enable_all()
        .build()
        .unwrap();

    let mut config_reader = BufReader::new(std::fs::File::open(&sample_options.config)?);
    let config: Arc<SampleConfig> = Arc::new(serde_json::from_reader(&mut config_reader)?);

    rt.block_on(async_main(config, &sample_options))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radius_sampling() {
        for _ in 0..500 {
            let c = radius_sample(2.0);
            assert!(c.re >= -2.0);
            assert!(c.re <= 2.0);
            assert!(c.im >= 0.0);
            assert!(c.im <= 2.0);
            assert!(c.norm_sqr() <= 4.0);
        }
    }

    #[test]
    fn prob_sampling() {
        for _ in 0..500 {
            let p = random_prob();
            assert!(p >= 0.0);
            assert!(p <= 1.0);
        }
    }

    #[test]
    fn test_project_1() {
        let config = ViewConfig::new(Complex::new(0.0, 0.0), 1.0, 500, 400);

        let pixel_eps = 0.00001;

        assert_eq!(
            project_onto_view(&config, &Complex::new(0.0, 0.0)),
            Some((250, 200))
        );
        assert_eq!(
            project_onto_view(&config, &Complex::new(-0.5, -0.5)),
            Some((0, 0))
        );
        assert_eq!(
            project_onto_view(&config, &Complex::new(-0.5, 0.5 - pixel_eps)),
            Some((0, 399))
        );
        assert_eq!(
            project_onto_view(&config, &Complex::new(0.5 - pixel_eps, -0.5)),
            Some((499, 0))
        );
        assert_eq!(
            project_onto_view(&config, &Complex::new(0.5 - pixel_eps, 0.5 - pixel_eps)),
            Some((499, 399))
        );

        assert_eq!(project_onto_view(&config, &Complex::new(100.0, 0.5)), None);
        assert_eq!(project_onto_view(&config, &Complex::new(-0.6, 0.8)), None);
        assert_eq!(
            project_onto_view(&config, &Complex::new(std::f64::NAN, 0.8)),
            None
        );
    }

    #[test]
    fn test_project_2() {
        let config = ViewConfig::new(Complex::new(-1.0, 2.0), 2.0, 500, 400);

        let pixel_eps = 0.00001;

        assert_eq!(
            project_onto_view(&config, &Complex::new(-1.0, 2.0)),
            Some((250, 200))
        );
        assert_eq!(
            project_onto_view(&config, &Complex::new(-1.25, 1.75)),
            Some((0, 0))
        );
        assert_eq!(
            project_onto_view(&config, &Complex::new(-1.25, 2.25 - pixel_eps)),
            Some((0, 399))
        );
        assert_eq!(
            project_onto_view(&config, &Complex::new(-0.75 - pixel_eps, 1.75)),
            Some((499, 0))
        );
        assert_eq!(
            project_onto_view(&config, &Complex::new(-0.75 - pixel_eps, 2.25 - pixel_eps)),
            Some((499, 399))
        );

        assert_eq!(project_onto_view(&config, &Complex::new(100.0, 0.5)), None);
        assert_eq!(project_onto_view(&config, &Complex::new(-0.6, 0.8)), None);
        assert_eq!(
            project_onto_view(&config, &Complex::new(std::f64::NAN, 0.8)),
            None
        );
    }
}
