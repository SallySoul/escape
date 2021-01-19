#![allow(unused_imports)]
#![allow(dead_code)]

use image::{Rgb, RgbImage};
use nalgebra::Complex;
use rand::distributions::Distribution;
use std::path::PathBuf;

mod config;
mod error;
mod grid;

use crate::config::{CutoffColor, RenderConfig};
use crate::error::EscapeError;
use crate::grid::{CountGrid, NormalizedGrid};

fn run_config(config: &RenderConfig, worker_samples: usize) -> Vec<NormalizedGrid> {
    let min = Complex::new(-2.0, -2.0);
    let mut result = vec![
        CountGrid::new(min, 4.0 / config.width as f64, config.width, config.width);
        config.cutoffs.len()
    ];

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
        .iter()
        .map(|grid| grid.to_normalized_grid())
        .collect()
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

fn main() -> Result<(), EscapeError> {
    let args: Vec<String> = std::env::args().collect();

    let config = serde_json::from_reader(std::fs::File::open(&args[1])?)?;
    let grids = run_config(&config, config.samples);
    color_grids(&config, &grids).save(config.output_path)?;

    Ok(())
}

/*
#[tokio::main]
async fn main() -> Result<(), EscapeError> {
    println!("Hello World");
    error()?;
    Ok(())
}
*/
