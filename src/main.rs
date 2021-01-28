#![allow(unused_imports)]
#![allow(dead_code)]
//#![feature(clamp)]
#![feature(get_mut_unchecked)]

use image::{Rgb, RgbImage};
use nalgebra::Complex;
use rand::distributions::Distribution;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;

mod config;
mod error;
mod grid;
mod types;
mod view_config;
mod worker;

use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

use crate::config::{CutoffColor, RenderConfig};
use crate::error::{EscapeError, EscapeResult};
use crate::types::CountGrid;
use crate::types::NormalizedGrid;
use crate::worker::WorkerState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct PartialResult {
    config: RenderConfig,
    grids: Vec<NormalizedGrid>,
}

impl PartialResult {
    fn to_file(
        config: &RenderConfig,
        grids: &Vec<NormalizedGrid>,
        path: &std::path::Path,
    ) -> EscapeResult {
        let writer = BufWriter::new(std::fs::File::create(path)?);
        serde_json::to_writer(
            writer,
            &PartialResult {
                config: config.clone(),
                grids: grids.clone(),
            },
        )?;
        Ok(())
    }

    fn from_file(
        path: &std::path::Path,
    ) -> Result<(Arc<RenderConfig>, Vec<NormalizedGrid>), EscapeError> {
        let reader = BufReader::new(std::fs::File::open(path)?);
        let partial_result: PartialResult = serde_json::from_reader(reader)?;
        Ok((Arc::from(partial_result.config), partial_result.grids))
    }
}

fn merge_grids(config: &RenderConfig, grids: Vec<CountGrid>) -> CountGrid {
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

async fn merge_results(
    config: Arc<RenderConfig>,
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

async fn run_worker(mut state_arc: Arc<WorkerState>) {
    unsafe {
        let state = Arc::get_mut_unchecked(&mut state_arc);
        state.run_worker();
    };
}

async fn async_main(
    config: Arc<RenderConfig>,
    workers: usize,
    store_result: &Option<PathBuf>,
    output: &Path,
) -> Result<(), EscapeError> {
    let mut worker_states = Vec::with_capacity(workers);
    let mut futures = Vec::with_capacity(workers);
    for i in 0..workers {
        worker_states.push(Arc::new(WorkerState::new(&config)));
        futures.push(tokio::spawn(run_worker(worker_states[i].clone())));
    }

    println!("Futures Started");

    let mut results = Vec::with_capacity(workers);
    for w in futures {
        results.push(w.await.unwrap());
    }

    println!("Futures done");

    let mut results = Vec::new();
    for w in &worker_states {
        results.push(&w.grids);
    }
    let merged_grids = merge_results(config.clone(), &results).await;
    if let Some(output) = store_result {
        PartialResult::to_file(&config, &merged_grids, output)?;
    }

    println!("Done Merging");
    color_grids(&config, &merged_grids).save(output)?;

    println!("Done writing file, {}", output.display());

    Ok(())
}

fn color_grids(config: &RenderConfig, grids: &[NormalizedGrid]) -> RgbImage {
    let mut result = RgbImage::new(config.view.width as u32, config.view.height as u32);
    for x in 0..config.view.width {
        for y in 0..config.view.height {
            let mut rgb_fp = [0.0, 0.0, 0.0];
            for (cutoff_index, config) in config.cutoffs.iter().enumerate() {
                for color in 0..2 {
                    rgb_fp[color] += grids[cutoff_index].value(x, y) * config.color[color];
                }
                for color in 2..3 {
                    rgb_fp[color] += grids[cutoff_index].value(x, y).sqrt() * config.color[color];
                }
            }

            let rgb = {
                let mut rgb = [0, 0, 0];
                for color in 0..3 {
                    rgb[color] = (rgb_fp[color].clamp(0.0, 1.0) * 255.0) as u8;
                }
                rgb
            };

            result.put_pixel(x as u32, y as u32, Rgb(rgb));
        }
    }

    result
}

#[derive(StructOpt, Debug)]
#[structopt(name = "escape")]
struct CliOptions {
    /// Path to the config file
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,

    /// Path to the partial result file
    #[structopt(short, long, parse(from_os_str))]
    partial_result: Option<PathBuf>,

    /// Path to store partial result
    #[structopt(short, long, parse(from_os_str))]
    store_result: Option<PathBuf>,

    /// Path to store partial image
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// The number of worker threads to spawn (more threads may be used)
    #[structopt(short, long)]
    workers: usize,
}

fn main() -> Result<(), EscapeError> {
    let cli_options = CliOptions::from_args();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(cli_options.workers)
        .build()
        .unwrap();

    if let Some(config_path) = &cli_options.config {
        let config: Arc<RenderConfig> =
            Arc::new(serde_json::from_reader(std::fs::File::open(config_path)?)?);
        rt.block_on(async_main(
            config,
            cli_options.workers,
            &cli_options.store_result,
            &cli_options.output,
        ))?;
    } else {
        println!("Loading stored Result");
        let (config, grids) = PartialResult::from_file(&cli_options.partial_result.unwrap())?;
        color_grids(&config, &grids).save(&cli_options.output)?;
        println!("Done writing file, {}", &cli_options.output.display());
    }

    println!("Done!");

    Ok(())
}
