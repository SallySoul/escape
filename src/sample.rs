use parking_lot::RwLock;
use rand::distributions::Distribution;
use std::io::BufReader;
use std::sync::Arc;
use tracing::{error, info, trace, warn};

use crate::cli_options::{MergeOptions, ReportOptions, SampleOptions};
use crate::config::{SampleConfig, ViewConfig};
use crate::histogram_result::HistogramResult;
use crate::types::{Complex, CountGrid, EscapeError, EscapeResult};

/// Randomly sample a complex number with a norm less than radius
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

/// Random sample fro [0..1)
fn random_prob() -> f64 {
    let mut rng = rand::thread_rng();
    rand::distributions::Uniform::from(0.0..1.0).sample(&mut rng)
}

/// Find the grid coords for a given complex number and view config
fn project_onto_view(view: &ViewConfig, c: &Complex) -> Option<(usize, usize)> {
    let v_w = view.width as f64;
    let v_h = view.height as f64;
    let x_fp = ((c.re - view.center.re) * view.zoom) * v_h + v_w / 2.0;
    let y_fp = ((c.im - view.center.im) * view.zoom) * v_h + v_h / 2.0;
    let x_signed = x_fp as i32;
    let y_signed = y_fp as i32;

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

/// Utility used to synchronize workers
#[derive(Debug)]
struct StopSwitch {
    stop: bool,
}
type ARStopSwitch = Arc<RwLock<StopSwitch>>;

impl StopSwitch {
    async fn new(maybe_duration: &Option<u64>) -> ARStopSwitch {
        let result = Arc::new(RwLock::new(StopSwitch { stop: false }));

        tokio::spawn(ctrl_c_handler(result.clone()));

        if let Some(seconds) = maybe_duration {
            tokio::spawn(duration_handler(result.clone(), *seconds));
        }

        result
    }

    fn stop(&self) -> bool {
        self.stop
    }
}

async fn ctrl_c_handler(switch: ARStopSwitch) -> EscapeResult {
    loop {
        tokio::signal::ctrl_c().await?;
        let mut s = switch.write();
        if s.stop {
            error!("Stop Switch already triggered");
        } else {
            info!("CTRL C pressed!");
            s.stop = true;
        }
    }
}

async fn duration_handler(switch: ARStopSwitch, seconds: u64) -> EscapeResult {
    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
    let mut s = switch.write();
    if s.stop {
        error!("Stop Switch already triggered");
    } else {
        info!("Duration complete");
        s.stop = true;
    }
    Ok(())
}

#[derive(Debug)]
struct WorkerState {
    sample_config: SampleConfig,
    grids: Vec<CountGrid>,
    norm_cutoff_sqr: f64,
    iteration_cutoff: usize,
    iteration_cutoff_f64: f64,
    orbit_buffer: Vec<Complex>,
    stop_switch: ARStopSwitch,
}

impl WorkerState {
    fn new(sample_config: &SampleConfig, stop_switch: ARStopSwitch) -> WorkerState {
        let cutoff = *sample_config.cutoffs.last().unwrap();
        let view = sample_config.view;
        WorkerState {
            sample_config: sample_config.clone(),
            grids: vec![CountGrid::zero(view.width, view.height); sample_config.cutoffs.len()],
            norm_cutoff_sqr: sample_config.norm_cutoff * sample_config.norm_cutoff,
            iteration_cutoff: cutoff,
            iteration_cutoff_f64: cutoff as f64,
            orbit_buffer: Vec::with_capacity(cutoff),
            stop_switch,
        }
    }

    fn project(&self, c: &Complex) -> Option<(usize, usize)> {
        project_onto_view(&self.sample_config.view, c)
    }

    /// Typical mandelbrot code, though we do fill the orbit buffer with the iterations
    /// Returns true if the sample escaped
    fn evaluate(&mut self, sample: &Complex) -> bool {
        self.orbit_buffer.clear();
        let jc = self.sample_config.julia_set_param;
        let m = self.sample_config.mandelbrot_param;
        let c = jc + m * sample;
        let mut z = *sample;
        let mut iteration = 0;
        while z.norm_sqr() <= self.norm_cutoff_sqr && iteration < self.iteration_cutoff {
            self.orbit_buffer.push(z);
            z = z * z + c;
            iteration += 1;
        }

        // Did point escape?
        z.norm_sqr() > self.norm_cutoff_sqr || iteration != self.iteration_cutoff
    }

    /// The contribution of a proposed value c
    /// is the fraction of the orbit that intersects the view
    fn contribution(&self, intersection_count: usize) -> f64 {
        intersection_count as f64 / self.iteration_cutoff as f64
    }

    /// Find the number of times the orbit buffer intersects the view
    /// without modifying the counts
    /// This is useful when finding samples or warming up the sampling routine
    #[tracing::instrument(skip(self))]
    fn orbit_intersections(&mut self) -> usize {
        let mut result = 0;
        for c in &self.orbit_buffer {
            if self.project(&c).is_some() {
                result += 1;
            }
        }
        result
    }

    /// Record the contents of the orbit buffer to the count grids
    /// Return the number of intersections
    #[tracing::instrument(skip(self))]
    fn record_orbit(&mut self) -> usize {
        let mut result = 0;
        for (i, cutoff) in self.sample_config.cutoffs.iter().enumerate() {
            if self.orbit_buffer.len() <= *cutoff {
                for c in &self.orbit_buffer {
                    if let Some((x, y)) = self.project(&c) {
                        self.grids[i].increment(x, y);
                        result += 1;
                    }
                }
                return result;
            }
        }
        panic!(
            "Did not record orbit to cutoff, ob len() {}, cutoff {}",
            self.orbit_buffer.len(),
            self.sample_config.cutoffs.last().unwrap()
        );
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
            if self.stop() {
                info!("In warmup stop");
                return (None, depth);
            }

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

        self.find_initial_sample_r(&closest_sample, radius / 2.0, depth + 1)
    }

    /// Sampling with the Metropolis-Hastings algorithm is based on mutating a "good" sample
    /// Some of the time we want to perturb the last good sample
    /// Other times we want to try a complelety new point
    fn mutate(&self, c: &Complex) -> Complex {
        let view = self.sample_config.view;
        if random_prob() < self.sample_config.random_sample_prob {
            radius_sample(self.sample_config.norm_cutoff)
        } else {
            let mut result = *c;
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
        // TODO these need to be setup properly
        let mut z = match self.find_initial_sample() {
            Some(z) => z,
            None => {
                warn!("Failed to find initial sample");
                return;
            }
        };

        let mut z_orbit_len = self.orbit_buffer.len();
        let z_orbit_intersections = self.orbit_intersections();
        let mut z_contrib = self.contribution(z_orbit_intersections);

        let mut accepted_samples = 0;
        let mut rejected_samples = 0;
        let mut outside_samples = 0;

        let mut outside_streak = 0;
        for warm_up_sample in 0..self.sample_config.warm_up_samples {
            if self.stop() {
                info!("In warmup stop");
                break;
            }

            let mutation = self.mutate(&z);
            self.evaluate(&mutation);
            let mutation_orbit_len = self.orbit_buffer.len();
            let intersection_count = self.orbit_intersections();
            // If the mutation doesn't intersect at all, it's a dud
            if intersection_count == 0 {
                outside_samples += 1;
                outside_streak += 1;
                if outside_streak > self.sample_config.outside_limit {
                    warn!(
                        warm_up_sample,
                        accepted_samples,
                        rejected_samples,
                        outside_samples,
                        "Outside streak exceeded in warm up"
                    );
                    return;
                }
                continue;
            } else {
                outside_streak = 0;
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
        for sample in 0..self.sample_config.samples {
            if self.stop() {
                info!("In sampling stop");
                break;
            }

            let mutation = self.mutate(&z);
            self.evaluate(&mutation);
            let mutation_orbit_len = self.orbit_buffer.len();
            let intersection_count = self.record_orbit();

            // If the mutation doesn't intersect at all, it's a dud
            if intersection_count == 0 {
                outside_samples += 1;
                outside_streak += 1;

                if outside_streak > self.sample_config.outside_limit {
                    trace!(
                        sample,
                        accepted_samples,
                        rejected_samples,
                        outside_samples,
                        "Outside streak exceeded in sampling"
                    );
                    return;
                }
                continue;
            } else {
                outside_streak = 0;
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

        info!(
            accepted_samples,
            rejected_samples, outside_samples, "Sampling complete"
        );
    }

    fn stop(&self) -> bool {
        self.stop_switch.read().stop()
    }

    #[tracing::instrument(skip(self))]
    fn run_worker(mut self) -> Self {
        let mut metro_instances = 0;
        while !self.stop() {
            metro_instances += 1;
            trace!(metro_instances, "Starting metro instance");
            self.run_metro_instance();
        }
        info!("Ran {} metro instances", metro_instances);
        self
    }
}

#[tracing::instrument(skip(config, grids))]
fn merge_grids(config: &SampleConfig, grids: Vec<&CountGrid>) -> CountGrid {
    let mut result = CountGrid::zero(config.view.width, config.view.height);
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

//#[tracing::instrument(skip(config, results))]
async fn merge_results<
    E: 'static + Send + Sync,
    F: 'static + Fn(&E, usize) -> &CountGrid + Send + Sync + Copy,
>(
    config: Arc<SampleConfig>,
    results: Arc<Vec<E>>,
    extract: F,
) -> Result<Vec<CountGrid>, EscapeError> {
    let cutoff_count = config.cutoffs.len();
    let mut tasks = Vec::with_capacity(cutoff_count);
    for cutoff_index in 0..cutoff_count {
        let c = config.clone();
        let task_results = results.clone();
        tasks.push(tokio::spawn(async move {
            let mut count_grids = Vec::with_capacity(task_results.len());
            for result in &*task_results {
                count_grids.push(extract(result, cutoff_index));
            }
            merge_grids(&c, count_grids)
        }));
    }

    let mut result = Vec::with_capacity(cutoff_count);
    for task in tasks {
        result.push(task.await?);
    }

    Ok(result)
}

async fn async_sampling(cli_options: &SampleOptions) -> EscapeResult {
    let logger_builder = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_thread_ids(true)
        .with_max_level(&cli_options.verbosity);

    if cli_options.pretty_logging {
        logger_builder.pretty().init()
    } else {
        logger_builder.init();
    }

    let mut config_reader = BufReader::new(std::fs::File::open(&cli_options.config)?);
    let config: Arc<SampleConfig> = Arc::new(serde_json::from_reader(&mut config_reader)?);
    trace!("Sample config loaded: {}", &cli_options.config.display());

    let stop_switch = StopSwitch::new(&cli_options.duration).await;
    let mut futures = Vec::with_capacity(cli_options.workers);
    for worker in 0..cli_options.workers {
        let s = stop_switch.clone();
        let c = config.clone();
        futures.push(tokio::spawn(async move {
            let state = WorkerState::new(&c, s);
            state.run_worker()
        }));
        trace!(worker, "Created worker future");
    }
    info!(cli_options.workers, "Started sampling workers");

    let mut results = Vec::with_capacity(cli_options.workers);
    for w in futures {
        results.push(w.await?.grids);
    }
    let arc_results = Arc::new(results);
    info!("Sampling workers have completed");

    let merged_grids = merge_results(config.clone(), arc_results, |x, i| &x[i]).await?;
    info!("Worker results have been merged");

    HistogramResult::save(&config, &merged_grids, &cli_options.output)?;
    info!(
        "Result has been written to {}",
        &cli_options.output.display()
    );

    Ok(())
}

pub fn run_sampling(sample_options: &SampleOptions) -> EscapeResult {
    // Note that we add one extra thread for timers / signal handlers
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(sample_options.workers + 1)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async_sampling(&sample_options))?;

    Ok(())
}

async fn async_merge(cli_options: &MergeOptions) -> EscapeResult {
    let logger_builder = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_thread_ids(true)
        .with_max_level(&cli_options.verbosity);

    if cli_options.pretty_logging {
        logger_builder.pretty().init()
    } else {
        logger_builder.init();
    }

    let histogram_count = cli_options.histograms.len();
    if histogram_count == 0 {
        warn!("No histograms");
        return Ok(());
    }

    // Open all the files / HistogramResults
    let mut result_futures = Vec::with_capacity(histogram_count);
    for path in cli_options.histograms.clone() {
        result_futures.push(tokio::spawn(
            async move { HistogramResult::from_file(&path) },
        ));
    }

    let mut configs = Vec::with_capacity(histogram_count);
    let mut results = Vec::with_capacity(histogram_count);
    for r in result_futures {
        let histogram = r.await??;
        configs.push(histogram.0);
        results.push(histogram.1)
    }
    let arc_results = Arc::new(results);
    info!("Files loaded");

    // Check compatability
    let mut check = true;
    for i in 1..configs.len() {
        check |= configs[i - 1].compatible(&configs[i]);
    }
    if !check {
        return Err(EscapeError::IncompatibleHistograms);
    }
    info!("Configs are compatible");

    let result = merge_results(configs[0].clone(), arc_results, |x, i| &x[i]).await?;
    info!("Results have been merged");

    HistogramResult::save(&configs[0], &result, &cli_options.output)?;
    info!("Result saved to {}", cli_options.output.display());

    Ok(())
}

pub fn run_merge(merge_options: &MergeOptions) -> EscapeResult {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(merge_options.workers)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async_merge(&merge_options))?;

    Ok(())
}

pub fn run_report(report_options: &ReportOptions) -> EscapeResult {
    let (_, histogram) = HistogramResult::from_file(&report_options.histogram)?;
    for (i, grid) in histogram.iter().enumerate() {
        let data: Vec<f64> = grid.data().iter().map(|c| *c as f64).collect();
        println!("Grid {}:", i);
        let min = grid.data().iter().min().unwrap();
        println!("  min: {}", min);
        let max = grid.data().iter().max().unwrap();
        println!("  max: {}", max);
        let mean = statistical::mean(&data);
        println!("  mean: {}", mean);
        //let median = statistical::median(&data);
        //println!("  median: {}", median);
        let sd = statistical::standard_deviation(&data, None);
        println!("  stdev: {}", sd);
        println!();
    }
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
            assert!(c.im >= -2.0);
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
        let config = ViewConfig {
            center: Complex::new(0.0, 0.0),
            zoom: 1.0,
            width: 500,
            height: 400,
        };

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
        let config = ViewConfig {
            center: Complex::new(-1.0, 2.0),
            zoom: 2.0,
            width: 500,
            height: 400,
        };

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
