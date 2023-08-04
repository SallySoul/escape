use crate::cli_options::OrbitStudyOptions;
use crate::types::{Complex, EscapeResult};
use tracing::*;
use crate::stop_switch::*;
use crate::config::*;
use nalgebra::identities::{One, Zero};

#[derive(Debug)]
struct OrbitWorker {
    sample_config: SampleConfig,
    orbit_buffer: Vec<Complex>,
    stop_switch: ArcSwitch,
}

impl OrbitWorker {
    fn new(stop_switch: ArcSwitch) -> OrbitWorker {
        // Whole View
        let sample_config = SampleConfig {
            cutoffs: Vec::new(),
            view: ViewConfig {
                center: nalgebra::zero(),
                zoom: 0.25,
                width: 800,
                height: 800,
            },
            julia_set_param: nalgebra::zero(),
            mandlebrot_param: Complex::new(1.0, 0.0),
            initial_search_depth: 500,
            warm_up_samples: 10000,
            norm_cutoff: 2.0,
            samples: 10000,
            outside_limit: 5,
        };

        OrbitWorker {
            sample_config,
            orbit_buffer: Vec::new(),
            stop_switch,
        }
    }

    fn stop(&self) -> bool {
        self.stop_switch.read().stop()
    }


}

async fn async_orbit_study(orbit_options: &OrbitStudyOptions, stop_switch: ArcSwitch) -> EscapeResult {
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

            // Only orbits that escape should not be counted
            if !self.evaluate(&mutation) {
                outside_samples += 1;
                outside_streak += 1;
                if outside_streak > self.sample_config.outside_limit {
                    warn!(
                        warm_up_sample,
                        accepted_samples,
                        rejected_samples,
                        outside_samples,
                        "Outside streak exceeded in warm up, evaluate"
                    );
                    return;
                }
            }

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
            if !self.evaluate(&mutation) {
                outside_samples += 1;
                outside_streak += 1;
                if outside_streak > self.sample_config.outside_limit {
                    warn!(
                        sample,
                        accepted_samples,
                        rejected_samples,
                        outside_samples,
                        "Outside streak exceeded, evaluate"
                    );
                    return;
                }
                continue;
            }

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
    Ok(())
}

pub fn run_orbit_study(cli_options: &OrbitStudyOptions) -> EscapeResult
{
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    

    let stop_switch = runtime.block_on(StopSwitch::new(&cli_options.duration));
    // Make  Stop Switch
    // Run Samplgin
    //runtime.block_on(async_orbit_study(cli_options, stop_switch))?;

    Ok(())
}

