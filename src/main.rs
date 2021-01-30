#![allow(unused_imports)]
#![allow(dead_code)]
//#![feature(clamp)]
#![feature(get_mut_unchecked)]
#![feature(thread_id_value)]

use image::{Rgb, RgbImage};
use nalgebra::Complex;
use rand::distributions::Distribution;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;

mod cli_options;
mod comptroller;
mod config;
mod error;
mod grid;
mod histogram_result;
mod types;
mod view_config;
mod worker;

use crate::cli_options::{CliOptions, DrawOptions, SampleOptions};
use crate::config::{DrawConfig, SampleConfig};
use crate::error::{EscapeError, EscapeResult};

/*
fn color_grids(config: &RenderConfig, grids: &[NormalizedGrid]) -> RgbImage {
    let mut result = RgbImage::new(config.view.width as u32, config.view.height as u32);
    for x in 0..config.view.width {
        for y in 0..config.view.height {
            let mut rgb_fp = [0.0, 0.0, 0.0];
            for (cutoff_index, config) in config.cutoffs.iter().enumerate() {
                for color in 0..3 {
                    rgb_fp[color] += grids[cutoff_index].value(x, y) * config.color[color];
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
*/
fn main() -> Result<(), EscapeError> {
    let cli_options = CliOptions::from_args();

    match &cli_options {
        CliOptions::Sample(sample_options) => {
            worker::run_sampling(&sample_options)?;
        }
        CliOptions::Draw(_draw_options) => {}
    }
    Ok(())
}
