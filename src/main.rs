#![allow(unused_imports)]
#![allow(dead_code)]

use image::{Rgb, RgbImage};
use nalgebra::Complex;
use rand::distributions::Distribution;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

mod config;
mod error;
mod grid;

use crate::config::{CutoffColor, RenderConfig};
use crate::error::EscapeError;
use crate::grid::{CountGrid, NormalizedGrid};

async fn run_config_async(config: Arc<RenderConfig>, worker_samples: usize) -> Vec<Arc<CountGrid>> {
    run_config(&config, worker_samples)
}

fn run_config(config: &RenderConfig, worker_samples: usize) -> Vec<Arc<CountGrid>> {
    let min = Complex::new(-2.0, -2.0);
    let result_grids = vec![
        CountGrid::new(min, 4.0 / config.width as f64, config.width, config.width);
        config.cutoffs.len()
    ];
    let mut result: Vec<Arc<CountGrid>> = result_grids.drain(0..).map(|grid| Arc::new(grid)).collect();


    let sample_range = rand::distributions::Uniform::from(-2.0..2.0);
    let mut rng = rand::thread_rng();

    let max_iteration = config.cutoffs.last().unwrap().cutoff;
    let norm_cutoff_sqr = config.norm_cutoff * config.norm_cutoff;
    let mut sequence_buffer = Vec::with_capacity(max_iteration);
    for _ in 0..worker_samples {
        let c = Complex::new(sample_range.sample(&mut rng), sample_range.sample(&mut rng));
        let mut z = c;

        let mut iteration = 0;
        while z.norm_sqr() <= norm_cutoff_sqr && iteration <= max_iteration {
            sequence_buffer.push(z);
            z = z * z + c;
            iteration += 1;
        }

        for (i, cutoff) in config.cutoffs.iter().enumerate() {
            if iteration <= cutoff.cutoff {
                result[i].increment_samples(&sequence_buffer);
                break;
            }
        }

        sequence_buffer.clear();
    }

    result
}

fn color_grids(config: &RenderConfig, grids: &[NormalizedGrid]) -> RgbImage {
    let mut result = RgbImage::new(config.width as u32, config.width as u32);
    for x in 0..config.width {
        for y in 0..config.width {
            let mut rgb_fp = [0.0, 0.0, 0.0];
            for (cutoff_index, config) in config.cutoffs.iter().enumerate() {
                for color in 0..3 {
                    rgb_fp[color] += grids[cutoff_index].value(x, y) * config.color[color];
                }
            }

            let rgb = {
                let mut rgb = [0, 0, 0];
                for color in 0..3 {
                    rgb[color] = (rgb_fp[color] * 255.0) as u8;
                }
                rgb
            };

            result.put_pixel(x as u32, y as u32, Rgb(rgb));
        }
    }

    result
}

fn merge_grids(config: &RenderConfig, grids: Vec<CountGrid>) -> CountGrid {
    let min = Complex::new(-2.0, -2.0);
    let mut result = CountGrid::new(min, 4.0 / config.width as f64, config.width, config.width);
    for x in 0..config.width {
        for y in 0..config.width {
            let mut sum = 0;
            for grid in grids {
                sum += grid.value(x, y);
            }
            result.set_value(sum, x, y);
        }
    }

    result
}

#[derive(StructOpt, Debug)]
#[structopt(name = "escape")]
struct CliOptions {
    /// Path to the config file
    #[structopt(short, long, parse(from_os_str))]
    config: PathBuf,

    /// The number of worker threads to spawn (more threads may be used)
    #[structopt(short, long)]
    workers: usize,
}

async fn async_main(config: Arc<RenderConfig>, workers: usize) -> Result<(), EscapeError>{
    let mut futures = Vec::with_capacity(workers);
    let x = config.samples / workers;
    for _ in 0..workers {
        futures.push(tokio::spawn(run_config_async(config.clone(), x)));
    }

    println!("Futures Started");

    let mut results = Vec::with_capacity(workers);
    for w in futures {
        results.push(w.await.unwrap());
    }

    println!("Futures done");
/*
    let mut merged_grids = Vec::with_capacity(config.cutoffs.len());
    let mut grid_buffer = Vec::with_capacity(workers);
    for i in 0..config.cutoffs.len() {
        for w in 0..workers {
            grid_buffer.push(&results[w][i]);
        }
        merged_grids.push(merge_grids(&config, grid_buffer).to_normalized_grid());
        grid_buffer.clear();
    }

    println!("Done Merging");

    color_grids(&config, &merged_grids).save(&config.output_path)?;
*/
    println!("Done writing file");

    Ok(())
}

async fn merge_results(config: Arc<RenderConfig>, results: Vec<Vec<CountGrid>>) -> Vec<NormalizedGrid> {
    let cutoff_count = config.cutoffs.len();
    let mut tasks = Vec::with_capacity(cutoff_count);
    for cutoff_index in 0..cutoff_count {
        let mut count_grids = Vec::with_capacity(results.len());
        for w in 0..results.len() {
            count_grids.push(results[w][cutoff_index]);
        }
        tasks.push(tokio::spawn(async move {
            let c = config.clone();
            merge_grids(&c, count_grids).to_normalized_grid()
        }));
    }

    let mut result = Vec::with_capacity(cutoff_count);
    for task in tasks {
        result.push(task.await.unwrap());
    }

    result
}



fn main() -> Result<(), EscapeError> {
    let cli_options = CliOptions::from_args();
    let config: Arc<RenderConfig> = Arc::new(serde_json::from_reader(std::fs::File::open(&cli_options.config)?)?);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(cli_options.workers)
        .build()
        .unwrap();


    rt.block_on(async_main(config, cli_options.workers))?;

    println!("Done!");

    Ok(())
}
