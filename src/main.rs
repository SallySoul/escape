#![feature(get_mut_unchecked)]
#![feature(thread_id_value)]

/// Structs for the CLI interface
mod cli_options;

/// Describes config files for sampling and drawing
mod config;

/// Grid type used to store histograms and normalized results
mod grid;

/// Result type used to save sampling
mod histogram_result;

/// Common types found in the application, including the error type
mod types;

/// Buddhabrot sampling implementation
mod worker;

/// Implementation to draw buddhabrot samples
mod draw;

use crate::cli_options::CliOptions;
use crate::types::EscapeResult;
use structopt::StructOpt;

fn main() -> EscapeResult {
    let cli_options = CliOptions::from_args();

    match &cli_options {
        CliOptions::Sample(sample_options) => {
            worker::run_sampling(&sample_options)?;
        }
        CliOptions::Draw(draw_options) => {
            draw::run_draw(&draw_options)?;
        }
    }
    Ok(())
}
